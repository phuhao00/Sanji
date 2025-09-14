//! 阴影渲染系统

use crate::math::{Vec2, Vec3, Vec4, Mat4, Quat};
use crate::render::{Camera, Light, LightType, Mesh, Material};
use crate::ecs::Transform;
use wgpu::*;
use std::collections::HashMap;

/// 阴影映射类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShadowMapType {
    Hard,           // 硬阴影
    PCF,            // 百分比滤波
    PCSS,           // 百分比软阴影
    CSM,            // 级联阴影映射
    VSM,            // 方差阴影映射
}

/// 阴影质量设置
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShadowQuality {
    Low,        // 低质量 - 512x512
    Medium,     // 中等质量 - 1024x1024
    High,       // 高质量 - 2048x2048
    Ultra,      // 超高质量 - 4096x4096
}

impl ShadowQuality {
    pub fn resolution(&self) -> u32 {
        match self {
            ShadowQuality::Low => 512,
            ShadowQuality::Medium => 1024,
            ShadowQuality::High => 2048,
            ShadowQuality::Ultra => 4096,
        }
    }
}

/// 阴影配置
#[derive(Debug, Clone)]
pub struct ShadowConfig {
    pub enabled: bool,
    pub map_type: ShadowMapType,
    pub quality: ShadowQuality,
    pub bias: f32,              // 阴影偏移，防止阴影粉刺
    pub normal_bias: f32,       // 法线偏移
    pub max_distance: f32,      // 最大阴影距离
    pub cascade_count: u32,     // 级联数量（用于CSM）
    pub cascade_splits: Vec<f32>, // 级联分割距离
    pub soft_shadow_radius: f32, // 软阴影半径
    pub pcf_samples: u32,       // PCF采样数量
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            map_type: ShadowMapType::PCF,
            quality: ShadowQuality::Medium,
            bias: 0.005,
            normal_bias: 0.02,
            max_distance: 100.0,
            cascade_count: 4,
            cascade_splits: vec![0.1, 0.3, 0.6, 1.0],
            soft_shadow_radius: 2.0,
            pcf_samples: 16,
        }
    }
}

/// 阴影贴图数据
pub struct ShadowMap {
    pub texture: Texture,
    pub view: TextureView,
    pub sampler: Sampler,
    pub depth_texture: Texture,
    pub depth_view: TextureView,
    pub framebuffer: Option<RenderPass<'static>>,
    pub resolution: u32,
    pub light_view_matrix: Mat4,
    pub light_projection_matrix: Mat4,
}

impl ShadowMap {
    pub fn new(device: &Device, resolution: u32) -> Self {
        // 创建阴影贴图纹理
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Shadow Map Texture"),
            size: Extent3d {
                width: resolution,
                height: resolution,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        // 创建深度纹理
        let depth_texture = device.create_texture(&TextureDescriptor {
            label: Some("Shadow Map Depth Texture"),
            size: Extent3d {
                width: resolution,
                height: resolution,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&TextureViewDescriptor::default());

        // 创建采样器
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Shadow Map Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: Some(CompareFunction::LessEqual),
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            depth_texture,
            depth_view,
            framebuffer: None,
            resolution,
            light_view_matrix: Mat4::IDENTITY,
            light_projection_matrix: Mat4::IDENTITY,
        }
    }

    /// 更新光源矩阵
    pub fn update_light_matrices(&mut self, light: &Light, transform: &Transform, scene_bounds: &crate::math::bounds::AABB) {
        match light.light_type {
            LightType::Directional => {
                self.update_directional_light_matrices(light, transform, scene_bounds);
            }
            LightType::Point => {
                self.update_point_light_matrices(light, transform);
            }
            LightType::Spot => {
                self.update_spot_light_matrices(light, transform);
            }
        }
    }

    /// 更新方向光矩阵
    fn update_directional_light_matrices(&mut self, light: &Light, transform: &Transform, scene_bounds: &crate::math::bounds::AABB) {
        let light_direction = transform.forward().normalize();
        let light_position = scene_bounds.center() - light_direction * scene_bounds.size().length();

        // 构建光源视图矩阵
        self.light_view_matrix = Mat4::look_at_rh(
            light_position,
            light_position + light_direction,
            Vec3::Y,
        );

        // 计算正交投影矩阵
        let size = scene_bounds.size().length() * 0.5;
        self.light_projection_matrix = Mat4::orthographic_rh(
            -size, size,
            -size, size,
            -size * 2.0, size * 2.0,
        );
    }

    /// 更新点光源矩阵
    fn update_point_light_matrices(&mut self, light: &Light, transform: &Transform) {
        // 点光源需要6个面的阴影贴图（立方体贴图）
        // 这里简化为单一方向
        self.light_view_matrix = Mat4::look_at_rh(
            transform.position,
            transform.position + Vec3::new(0.0, 0.0, 1.0),
            Vec3::Y,
        );

        self.light_projection_matrix = Mat4::perspective_rh(
            90.0_f32.to_radians(),
            1.0,
            0.1,
            light.range,
        );
    }

    /// 更新聚光灯矩阵
    fn update_spot_light_matrices(&mut self, light: &Light, transform: &Transform) {
        let light_direction = transform.forward().normalize();
        
        self.light_view_matrix = Mat4::look_at_rh(
            transform.position,
            transform.position + light_direction,
            Vec3::Y,
        );

        self.light_projection_matrix = Mat4::perspective_rh(
            light.spot_angle * 2.0,
            1.0,
            0.1,
            light.range,
        );
    }

    /// 获取光源空间变换矩阵
    pub fn get_light_space_matrix(&self) -> Mat4 {
        self.light_projection_matrix * self.light_view_matrix
    }
}

/// 级联阴影贴图
pub struct CascadedShadowMap {
    pub cascades: Vec<ShadowMap>,
    pub cascade_distances: Vec<f32>,
    pub cascade_matrices: Vec<Mat4>,
}

impl CascadedShadowMap {
    pub fn new(device: &Device, cascade_count: u32, resolution: u32) -> Self {
        let mut cascades = Vec::new();
        for i in 0..cascade_count {
            let cascade_resolution = resolution >> (i / 2); // 远距离级联使用较低分辨率
            cascades.push(ShadowMap::new(device, cascade_resolution));
        }

        Self {
            cascades,
            cascade_distances: vec![0.0; cascade_count as usize],
            cascade_matrices: vec![Mat4::IDENTITY; cascade_count as usize],
        }
    }

    /// 更新级联阴影贴图
    pub fn update_cascades(
        &mut self,
        camera: &Camera,
        light: &Light,
        light_transform: &Transform,
        config: &ShadowConfig,
        scene_bounds: &crate::math::bounds::AABB,
    ) {
        let camera_near = camera.near_plane;
        let camera_far = config.max_distance.min(camera.far_plane);
        
        // 计算级联分割距离
        for (i, split) in config.cascade_splits.iter().enumerate() {
            if i < self.cascade_distances.len() {
                self.cascade_distances[i] = camera_near + (camera_far - camera_near) * split;
            }
        }

        // 预先计算所有级联的数据，避免借用冲突
        let mut cascade_data = Vec::new();
        for i in 0..config.cascade_splits.len().min(self.cascades.len()) {
            let near = if i == 0 { camera_near } else { self.cascade_distances[i - 1] };
            let far = self.cascade_distances[i];

            // 计算该级联的视锥体
            let frustum_corners = Self::calculate_frustum_corners_static(camera, near, far);
            
            // 计算包围盒
            let frustum_bounds = Self::calculate_frustum_bounds_static(&frustum_corners);
            
            cascade_data.push((i, frustum_bounds));
        }

        // 更新级联矩阵
        for (i, frustum_bounds) in cascade_data {
            self.cascades[i].update_directional_light_matrices(light, light_transform, &frustum_bounds);
            self.cascade_matrices[i] = self.cascades[i].get_light_space_matrix();
        }
    }

    /// 计算视锥体角点（静态版本）
    fn calculate_frustum_corners_static(camera: &Camera, near: f32, far: f32) -> [Vec3; 8] {
        let inv_view_proj = (camera.projection_matrix() * camera.view_matrix()).inverse();
        
        // NDC空间中的视锥体角点
        let ndc_corners = [
            Vec4::new(-1.0, -1.0, 0.0, 1.0), // near bottom left
            Vec4::new( 1.0, -1.0, 0.0, 1.0), // near bottom right
            Vec4::new(-1.0,  1.0, 0.0, 1.0), // near top left
            Vec4::new( 1.0,  1.0, 0.0, 1.0), // near top right
            Vec4::new(-1.0, -1.0, 1.0, 1.0), // far bottom left
            Vec4::new( 1.0, -1.0, 1.0, 1.0), // far bottom right
            Vec4::new(-1.0,  1.0, 1.0, 1.0), // far top left
            Vec4::new( 1.0,  1.0, 1.0, 1.0), // far top right
        ];

        let mut world_corners = [Vec3::ZERO; 8];
        for (i, &ndc_corner) in ndc_corners.iter().enumerate() {
            let world_corner = inv_view_proj * ndc_corner;
            world_corners[i] = Vec3::new(
                world_corner.x / world_corner.w,
                world_corner.y / world_corner.w,
                world_corner.z / world_corner.w,
            );
        }

        world_corners
    }

    /// 计算视锥体包围盒
    fn calculate_frustum_bounds(&self, corners: &[Vec3; 8]) -> crate::math::bounds::AABB {
        Self::calculate_frustum_bounds_static(corners)
    }

    fn calculate_frustum_bounds_static(corners: &[Vec3; 8]) -> crate::math::bounds::AABB {
        let mut min = corners[0];
        let mut max = corners[0];

        for &corner in corners.iter().skip(1) {
            min = min.min(corner);
            max = max.max(corner);
        }

        crate::math::bounds::AABB::new(min, max)
    }
}

/// 阴影渲染器
pub struct ShadowRenderer {
    pub config: ShadowConfig,
    shadow_maps: HashMap<u32, ShadowMap>, // 光源ID -> 阴影贴图
    cascaded_shadow_map: Option<CascadedShadowMap>,
    shadow_pass_pipeline: Option<RenderPipeline>,
    bind_group_layout: BindGroupLayout,
    uniform_buffer: Buffer,
}

impl ShadowRenderer {
    pub fn new(device: &Device, config: ShadowConfig) -> Self {
        // 创建绑定组布局
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Shadow Bind Group Layout"),
            entries: &[
                // 光源空间变换矩阵
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 阴影贴图
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Depth,
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // 阴影采样器
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });

        // 创建uniform缓冲区
        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Shadow Uniform Buffer"),
            size: std::mem::size_of::<ShadowUniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 创建级联阴影贴图（如果启用）
        let cascaded_shadow_map = if config.map_type == ShadowMapType::CSM {
            Some(CascadedShadowMap::new(
                device,
                config.cascade_count,
                config.quality.resolution(),
            ))
        } else {
            None
        };

        Self {
            config,
            shadow_maps: HashMap::new(),
            cascaded_shadow_map,
            shadow_pass_pipeline: None,
            bind_group_layout,
            uniform_buffer,
        }
    }

    /// 为光源创建阴影贴图
    pub fn create_shadow_map_for_light(&mut self, device: &Device, light_id: u32) {
        if !self.shadow_maps.contains_key(&light_id) {
            let shadow_map = ShadowMap::new(device, self.config.quality.resolution());
            self.shadow_maps.insert(light_id, shadow_map);
        }
    }

    /// 渲染阴影贴图
    pub fn render_shadow_map(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        light_id: u32,
        light: &Light,
        light_transform: &Transform,
        meshes: &[(&Mesh, &Mat4)], // (网格, 世界变换矩阵)
        scene_bounds: &crate::math::bounds::AABB,
    ) {
        if !self.config.enabled {
            return;
        }

        // 获取或创建阴影贴图
        if !self.shadow_maps.contains_key(&light_id) {
            self.create_shadow_map_for_light(device, light_id);
        }

        let shadow_map = self.shadow_maps.get_mut(&light_id).unwrap();
        shadow_map.update_light_matrices(light, light_transform, scene_bounds);

        // 创建渲染通道
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Shadow Map Pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &shadow_map.depth_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        // 更新uniform数据
        let uniforms = ShadowUniforms {
            light_space_matrix: shadow_map.get_light_space_matrix().to_cols_array_2d(),
            light_position: light_transform.position.extend(1.0).to_array(),
            shadow_bias: self.config.bias,
            normal_bias: self.config.normal_bias,
            cascade_count: self.config.cascade_count,
            _padding: 0,
            cascade_distances: [0.0; 4], // 暂时填充，实际使用时会更新
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // 渲染网格到阴影贴图
        for (mesh, world_matrix) in meshes {
            // TODO: 设置渲染管线和绘制网格
            // 这里需要使用专门的阴影渲染着色器
        }
    }

    /// 渲染级联阴影贴图
    pub fn render_cascaded_shadow_map(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        camera: &Camera,
        light: &Light,
        light_transform: &Transform,
        meshes: &[(&Mesh, &Mat4)],
        scene_bounds: &crate::math::bounds::AABB,
    ) {
        if !self.config.enabled || self.config.map_type != ShadowMapType::CSM {
            return;
        }

        let csm = self.cascaded_shadow_map.as_mut().unwrap();
        csm.update_cascades(camera, light, light_transform, &self.config, scene_bounds);

        // 渲染每个级联
        for (i, cascade) in csm.cascades.iter().enumerate() {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some(&format!("Cascade Shadow Map Pass {}", i)),
                color_attachments: &[],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &cascade.depth_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // 更新该级联的uniform数据
            let uniforms = ShadowUniforms {
                light_space_matrix: csm.cascade_matrices[i].to_cols_array_2d(),
                light_position: light_transform.position.extend(1.0).to_array(),
                shadow_bias: self.config.bias,
                normal_bias: self.config.normal_bias,
                cascade_count: self.config.cascade_count,
                _padding: 0,
                cascade_distances: [
                    csm.cascade_distances.get(0).copied().unwrap_or(0.0),
                    csm.cascade_distances.get(1).copied().unwrap_or(0.0),
                    csm.cascade_distances.get(2).copied().unwrap_or(0.0),
                    csm.cascade_distances.get(3).copied().unwrap_or(0.0),
                ],
            };

            queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

            // 渲染网格
            for (mesh, world_matrix) in meshes {
                // TODO: 渲染网格到级联阴影贴图
            }
        }
    }

    /// 获取阴影贴图
    pub fn get_shadow_map(&self, light_id: u32) -> Option<&ShadowMap> {
        self.shadow_maps.get(&light_id)
    }

    /// 获取级联阴影贴图
    pub fn get_cascaded_shadow_map(&self) -> Option<&CascadedShadowMap> {
        self.cascaded_shadow_map.as_ref()
    }

    /// 更新配置
    pub fn update_config(&mut self, device: &Device, new_config: ShadowConfig) {
        let resolution_changed = self.config.quality != new_config.quality;
        let cascade_changed = self.config.cascade_count != new_config.cascade_count ||
                             self.config.map_type != new_config.map_type;

        self.config = new_config;

        // 重新创建资源（如果需要）
        if resolution_changed {
            // 重新创建所有阴影贴图
            self.shadow_maps.clear();
        }

        if cascade_changed && self.config.map_type == ShadowMapType::CSM {
            self.cascaded_shadow_map = Some(CascadedShadowMap::new(
                device,
                self.config.cascade_count,
                self.config.quality.resolution(),
            ));
        }
    }

    /// 清理资源
    pub fn cleanup(&mut self) {
        self.shadow_maps.clear();
        self.cascaded_shadow_map = None;
    }
}

/// 阴影uniform数据
#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ShadowUniforms {
    pub light_space_matrix: [[f32; 4]; 4], // Mat4 as array
    pub light_position: [f32; 4],          // Vec4 as array
    pub shadow_bias: f32,
    pub normal_bias: f32,
    pub cascade_count: u32,
    pub _padding: u32,
    pub cascade_distances: [f32; 4],
}

// Manual implementation of bytemuck traits for ShadowUniforms
unsafe impl bytemuck::Pod for ShadowUniforms {}
unsafe impl bytemuck::Zeroable for ShadowUniforms {}

/// 阴影计算工具
pub struct ShadowUtils;

impl ShadowUtils {
    /// 计算PCF阴影
    pub fn sample_shadow_pcf(
        shadow_map: &TextureView,
        sampler: &Sampler,
        shadow_coord: Vec3,
        texel_size: f32,
        sample_count: u32,
    ) -> f32 {
        // 这是一个概念性的函数，实际实现需要在着色器中完成
        // 这里只是展示PCF的基本思路
        
        let mut shadow = 0.0;
        let half_samples = sample_count as f32 * 0.5;
        
        for x in 0..sample_count {
            for y in 0..sample_count {
                let offset_x = (x as f32 - half_samples) * texel_size;
                let offset_y = (y as f32 - half_samples) * texel_size;
                
                let sample_coord = Vec3::new(
                    shadow_coord.x + offset_x,
                    shadow_coord.y + offset_y,
                    shadow_coord.z,
                );
                
                // 在着色器中，这里会是纹理采样
                // shadow += texture(shadow_map, sample_coord);
            }
        }
        
        shadow / (sample_count * sample_count) as f32
    }

    /// 计算泊松圆盘采样偏移
    pub fn poisson_disk_samples(sample_count: u32) -> Vec<Vec2> {
        // 预计算的泊松圆盘采样点
        let samples = [
            Vec2::new(-0.94201624, -0.39906216),
            Vec2::new(0.94558609, -0.76890725),
            Vec2::new(-0.094184101, -0.92938870),
            Vec2::new(0.34495938, 0.29387760),
            Vec2::new(-0.91588581, 0.45771432),
            Vec2::new(-0.81544232, -0.87912464),
            Vec2::new(-0.38277543, 0.27676845),
            Vec2::new(0.97484398, 0.75648379),
            Vec2::new(0.44323325, -0.97511554),
            Vec2::new(0.53742981, -0.47373420),
            Vec2::new(-0.26496911, -0.41893023),
            Vec2::new(0.79197514, 0.19090188),
            Vec2::new(-0.24188840, 0.99706507),
            Vec2::new(-0.81409955, 0.91437590),
            Vec2::new(0.19984126, 0.78641367),
            Vec2::new(0.14383161, -0.14100790),
        ];

        samples.iter()
            .take(sample_count.min(16) as usize)
            .copied()
            .collect()
    }

    /// 世界坐标转换到光源空间
    pub fn world_to_light_space(world_pos: Vec3, light_space_matrix: &Mat4) -> Vec4 {
        *light_space_matrix * world_pos.extend(1.0)
    }

    /// 计算阴影坐标
    pub fn calculate_shadow_coords(light_space_pos: Vec4) -> Vec3 {
        // 透视除法
        let ndc = Vec3::new(
            light_space_pos.x / light_space_pos.w,
            light_space_pos.y / light_space_pos.w,
            light_space_pos.z / light_space_pos.w,
        );

        // 转换到[0,1]范围
        Vec3::new(
            ndc.x * 0.5 + 0.5,
            ndc.y * 0.5 + 0.5,
            ndc.z,
        )
    }
}
