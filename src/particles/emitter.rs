//! 粒子发射器

use crate::math::{Vec3, Vec2, Quat};
use crate::particles::{Particle, ParticleState};
use crate::render::RenderSystem;
use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};

/// 发射器ID类型
pub type EmitterId = u64;

/// 发射形状
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EmissionShape {
    Point,                              // 点发射
    Circle { radius: f32 },             // 圆形发射
    Sphere { radius: f32 },             // 球形发射
    Box { size: Vec3 },                 // 盒形发射
    Cone { angle: f32, radius: f32 },   // 锥形发射
    Line { start: Vec3, end: Vec3 },    // 线段发射
    Mesh { vertices: Vec<Vec3> },       // 网格表面发射
}

/// 混合模式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BlendMode {
    Alpha,      // 透明混合
    Additive,   // 加法混合
    Multiply,   // 乘法混合
    Screen,     // 屏幕混合
}

/// 模拟空间
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SimulationSpace {
    Local,  // 本地空间（相对于发射器）
    World,  // 世界空间
}

/// 粒子生命周期内的大小变化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeOverLifetime {
    pub curve: Vec<(f32, f32)>, // (生命周期百分比, 大小倍数)
}

impl SizeOverLifetime {
    pub fn new(curve: Vec<(f32, f32)>) -> Self {
        Self { curve }
    }

    pub fn evaluate(&self, lifetime_ratio: f32) -> f32 {
        if self.curve.is_empty() {
            return 1.0;
        }

        if self.curve.len() == 1 {
            return self.curve[0].1;
        }

        // 线性插值
        for i in 0..self.curve.len() - 1 {
            let (t1, v1) = self.curve[i];
            let (t2, v2) = self.curve[i + 1];

            if lifetime_ratio >= t1 && lifetime_ratio <= t2 {
                if t2 == t1 {
                    return v1;
                }
                let factor = (lifetime_ratio - t1) / (t2 - t1);
                return v1 + (v2 - v1) * factor;
            }
        }

        // 如果超出范围，返回最后一个值
        self.curve.last().unwrap().1
    }
}

/// 粒子生命周期内的速度变化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityOverLifetime {
    pub curve: Vec<(f32, Vec3)>, // (生命周期百分比, 速度)
}

impl VelocityOverLifetime {
    pub fn new(curve: Vec<(f32, Vec3)>) -> Self {
        Self { curve }
    }

    pub fn evaluate(&self, lifetime_ratio: f32) -> Vec3 {
        if self.curve.is_empty() {
            return Vec3::ZERO;
        }

        if self.curve.len() == 1 {
            return self.curve[0].1;
        }

        // 线性插值
        for i in 0..self.curve.len() - 1 {
            let (t1, v1) = self.curve[i];
            let (t2, v2) = self.curve[i + 1];

            if lifetime_ratio >= t1 && lifetime_ratio <= t2 {
                if t2 == t1 {
                    return v1;
                }
                let factor = (lifetime_ratio - t1) / (t2 - t1);
                return v1.lerp(v2, factor);
            }
        }

        // 如果超出范围，返回最后一个值
        self.curve.last().unwrap().1
    }
}

/// 粒子生命周期内的颜色变化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorOverLifetime {
    pub curve: Vec<(f32, [f32; 4])>, // (生命周期百分比, RGBA)
}

impl ColorOverLifetime {
    pub fn new(curve: Vec<(f32, [f32; 4])>) -> Self {
        Self { curve }
    }

    pub fn evaluate(&self, lifetime_ratio: f32) -> [f32; 4] {
        if self.curve.is_empty() {
            return [1.0, 1.0, 1.0, 1.0];
        }

        if self.curve.len() == 1 {
            return self.curve[0].1;
        }

        // 线性插值
        for i in 0..self.curve.len() - 1 {
            let (t1, c1) = self.curve[i];
            let (t2, c2) = self.curve[i + 1];

            if lifetime_ratio >= t1 && lifetime_ratio <= t2 {
                if t2 == t1 {
                    return c1;
                }
                let factor = (lifetime_ratio - t1) / (t2 - t1);
                return [
                    c1[0] + (c2[0] - c1[0]) * factor,
                    c1[1] + (c2[1] - c1[1]) * factor,
                    c1[2] + (c2[2] - c1[2]) * factor,
                    c1[3] + (c2[3] - c1[3]) * factor,
                ];
            }
        }

        // 如果超出范围，返回最后一个值
        self.curve.last().unwrap().1
    }
}

/// 发射器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitterConfig {
    /// 最大粒子数
    pub max_particles: usize,
    
    /// 发射速率（每秒发射粒子数）
    pub emission_rate: f32,
    
    /// 爆发发射数量（一次性发射）
    pub burst_count: usize,
    
    /// 发射器生命周期（秒，0表示无限）
    pub lifetime: f32,
    
    /// 粒子生命周期范围（秒）
    pub start_lifetime_range: (f32, f32),
    
    /// 粒子初始速度范围
    pub start_speed_range: (f32, f32),
    
    /// 粒子初始大小范围
    pub start_size_range: (f32, f32),
    
    /// 粒子初始颜色
    pub start_color: [f32; 4],
    
    /// 粒子结束颜色
    pub end_color: [f32; 4],
    
    /// 重力影响
    pub gravity: Vec3,
    
    /// 发射形状
    pub shape: EmissionShape,
    
    /// 纹理路径
    pub texture_path: Option<String>,
    
    /// 混合模式
    pub blend_mode: BlendMode,
    
    /// 大小随生命周期变化
    pub size_over_lifetime: Option<SizeOverLifetime>,
    
    /// 速度随生命周期变化
    pub velocity_over_lifetime: Option<VelocityOverLifetime>,
    
    /// 颜色随生命周期变化
    pub color_over_lifetime: Option<ColorOverLifetime>,
    
    /// 模拟空间
    pub simulation_space: SimulationSpace,
    
    /// 排序层
    pub sorting_layer: i32,
    
    /// 层内排序
    pub order_in_layer: i32,
}

impl Default for EmitterConfig {
    fn default() -> Self {
        Self {
            max_particles: 100,
            emission_rate: 10.0,
            burst_count: 0,
            lifetime: 0.0,
            start_lifetime_range: (1.0, 2.0),
            start_speed_range: (1.0, 3.0),
            start_size_range: (0.1, 0.2),
            start_color: [1.0, 1.0, 1.0, 1.0],
            end_color: [1.0, 1.0, 1.0, 0.0],
            gravity: Vec3::new(0.0, -9.81, 0.0),
            shape: EmissionShape::Point,
            texture_path: None,
            blend_mode: BlendMode::Alpha,
            size_over_lifetime: None,
            velocity_over_lifetime: None,
            color_over_lifetime: None,
            simulation_space: SimulationSpace::World,
            sorting_layer: 0,
            order_in_layer: 0,
        }
    }
}

/// 发射器状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmitterState {
    Stopped,    // 停止
    Playing,    // 播放
    Paused,     // 暂停
}

/// 粒子发射器
pub struct ParticleEmitter {
    pub id: EmitterId,
    pub config: EmitterConfig,
    pub particles: Vec<Particle>,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    
    state: EmitterState,
    emission_timer: f32,
    lifetime_timer: f32,
    burst_emitted: bool,
}

impl ParticleEmitter {
    pub fn new(id: EmitterId, config: EmitterConfig) -> Self {
        let max_particles = config.max_particles;
        Self {
            id,
            config,
            particles: Vec::with_capacity(max_particles),
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            state: EmitterState::Stopped,
            emission_timer: 0.0,
            lifetime_timer: 0.0,
            burst_emitted: false,
        }
    }

    /// 启动发射器
    pub fn start(&mut self) {
        self.state = EmitterState::Playing;
        self.emission_timer = 0.0;
        self.lifetime_timer = 0.0;
        self.burst_emitted = false;
    }

    /// 停止发射器
    pub fn stop(&mut self) {
        self.state = EmitterState::Stopped;
        self.particles.clear();
    }

    /// 暂停发射器
    pub fn pause(&mut self) {
        if self.state == EmitterState::Playing {
            self.state = EmitterState::Paused;
        }
    }

    /// 恢复发射器
    pub fn resume(&mut self) {
        if self.state == EmitterState::Paused {
            self.state = EmitterState::Playing;
        }
    }

    /// 检查发射器是否活跃
    pub fn is_active(&self) -> bool {
        self.state == EmitterState::Playing
    }

    /// 设置位置
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// 设置旋转
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
    }

    /// 设置缩放
    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
    }

    /// 设置配置
    pub fn set_config(&mut self, config: EmitterConfig) {
        self.particles.reserve(config.max_particles);
        self.config = config;
    }

    /// 立即发射爆发粒子
    pub fn emit_burst(&mut self) {
        if self.config.burst_count > 0 {
            self.emit_particles(self.config.burst_count);
        }
    }

    /// 更新发射器
    pub fn update(&mut self, delta_time: f32, available_particles: usize) {
        if self.state != EmitterState::Playing {
            return;
        }

        // 更新生命周期
        if self.config.lifetime > 0.0 {
            self.lifetime_timer += delta_time;
            if self.lifetime_timer >= self.config.lifetime {
                self.state = EmitterState::Stopped;
                return;
            }
        }

        // 发射爆发粒子（只发射一次）
        if !self.burst_emitted && self.config.burst_count > 0 {
            let burst_count = self.config.burst_count.min(available_particles);
            self.emit_particles(burst_count);
            self.burst_emitted = true;
        }

        // 发射持续粒子
        if self.config.emission_rate > 0.0 {
            self.emission_timer += delta_time;
            let emission_interval = 1.0 / self.config.emission_rate;
            
            while self.emission_timer >= emission_interval && self.particles.len() < self.config.max_particles && available_particles > 0 {
                self.emit_particles(1);
                self.emission_timer -= emission_interval;
            }
        }

        // 更新现有粒子
        self.update_particles(delta_time);
    }

    /// 发射粒子
    fn emit_particles(&mut self, count: usize) {
        let mut rng = thread_rng();
        
        for _ in 0..count {
            if self.particles.len() >= self.config.max_particles {
                break;
            }

            let mut particle = Particle::new(0, Vec3::ZERO, Vec3::ZERO);
            
            // 设置初始位置
            particle.position = self.position + self.get_emission_position(&mut rng);
            
            // 设置初始速度
            let speed = rng.gen_range(self.config.start_speed_range.0..=self.config.start_speed_range.1);
            particle.velocity = self.get_emission_direction(&mut rng) * speed;
            
            // 设置初始属性
            particle.lifetime = rng.gen_range(self.config.start_lifetime_range.0..=self.config.start_lifetime_range.1);
            particle.max_lifetime = particle.lifetime;
            particle.size = rng.gen_range(self.config.start_size_range.0..=self.config.start_size_range.1);
            particle.color = self.config.start_color;
            particle.lifetime = 1.0; // Use lifetime field

            self.particles.push(particle);
        }
    }

    /// 获取发射位置
    fn get_emission_position(&self, rng: &mut impl Rng) -> Vec3 {
        match &self.config.shape {
            EmissionShape::Point => Vec3::ZERO,
            
            EmissionShape::Circle { radius } => {
                let angle = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
                let r = rng.gen::<f32>().sqrt() * radius;
                Vec3::new(r * angle.cos(), 0.0, r * angle.sin())
            }
            
            EmissionShape::Sphere { radius } => {
                let u = rng.gen::<f32>();
                let v = rng.gen::<f32>();
                let theta = u * 2.0 * std::f32::consts::PI;
                let phi = (2.0 * v - 1.0).acos();
                let r = rng.gen::<f32>().cbrt() * radius;
                
                Vec3::new(
                    r * phi.sin() * theta.cos(),
                    r * phi.sin() * theta.sin(),
                    r * phi.cos()
                )
            }
            
            EmissionShape::Box { size } => {
                Vec3::new(
                    (rng.gen::<f32>() - 0.5) * size.x,
                    (rng.gen::<f32>() - 0.5) * size.y,
                    (rng.gen::<f32>() - 0.5) * size.z,
                )
            }
            
            EmissionShape::Cone { angle, radius } => {
                let cone_angle = angle.to_radians();
                let phi = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
                let theta = rng.gen::<f32>() * cone_angle;
                let r = rng.gen::<f32>() * radius;
                
                Vec3::new(
                    r * theta.sin() * phi.cos(),
                    r * theta.cos(),
                    r * theta.sin() * phi.sin()
                )
            }
            
            EmissionShape::Line { start, end } => {
                let t = rng.gen::<f32>();
                start.lerp(*end, t) - self.position
            }
            
            EmissionShape::Mesh { vertices } => {
                if vertices.is_empty() {
                    Vec3::ZERO
                } else {
                    let index = rng.gen_range(0..vertices.len());
                    vertices[index] - self.position
                }
            }
        }
    }

    /// 获取发射方向
    fn get_emission_direction(&self, rng: &mut impl Rng) -> Vec3 {
        match &self.config.shape {
            EmissionShape::Point => {
                // 随机方向
                let theta = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
                let phi = rng.gen::<f32>() * std::f32::consts::PI;
                Vec3::new(
                    phi.sin() * theta.cos(),
                    phi.cos(),
                    phi.sin() * theta.sin()
                ).normalize()
            }
            
            EmissionShape::Circle { .. } => {
                // 向上发射
                Vec3::Y
            }
            
            EmissionShape::Sphere { .. } => {
                // 径向发射
                let emission_pos = self.get_emission_position(rng);
                if emission_pos.length() > 0.0 {
                    emission_pos.normalize()
                } else {
                    Vec3::Y
                }
            }
            
            EmissionShape::Box { .. } => {
                // 随机方向
                let theta = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
                let phi = rng.gen::<f32>() * std::f32::consts::PI;
                Vec3::new(
                    phi.sin() * theta.cos(),
                    phi.cos(),
                    phi.sin() * theta.sin()
                ).normalize()
            }
            
            EmissionShape::Cone { angle, .. } => {
                let cone_angle = angle.to_radians();
                let phi = rng.gen::<f32>() * 2.0 * std::f32::consts::PI;
                let theta = rng.gen::<f32>() * cone_angle;
                
                Vec3::new(
                    theta.sin() * phi.cos(),
                    theta.cos(),
                    theta.sin() * phi.sin()
                ).normalize()
            }
            
            EmissionShape::Line { start, end } => {
                (*end - *start).normalize()
            }
            
            EmissionShape::Mesh { .. } => {
                // 简化：向上发射
                Vec3::Y
            }
        }
    }

    /// 更新粒子
    fn update_particles(&mut self, delta_time: f32) {
        for particle in &mut self.particles {
            if particle.lifetime <= 0.0 { // Check lifetime instead of state
                continue;
            }

            // 更新生命周期
            particle.lifetime -= delta_time;
            if particle.lifetime <= 0.0 {
                particle.lifetime = 0.0; // Set lifetime to 0 instead of Dead state
                continue;
            }

            let lifetime_ratio = 1.0 - (particle.lifetime / particle.max_lifetime);

            // 应用重力
            particle.velocity += self.config.gravity * delta_time;

            // 应用生命周期内的速度变化
            if let Some(ref velocity_curve) = self.config.velocity_over_lifetime {
                let additional_velocity = velocity_curve.evaluate(lifetime_ratio);
                particle.velocity += additional_velocity * delta_time;
            }

            // 更新位置
            particle.position += particle.velocity * delta_time;

            // 应用生命周期内的大小变化
            if let Some(ref size_curve) = self.config.size_over_lifetime {
                let base_size = particle.size; // 假设我们存储了初始大小
                particle.size = base_size * size_curve.evaluate(lifetime_ratio);
            }

            // 应用生命周期内的颜色变化
            if let Some(ref color_curve) = self.config.color_over_lifetime {
                particle.color = color_curve.evaluate(lifetime_ratio);
            } else {
                // 简单的颜色插值
                let start_color = self.config.start_color;
                let end_color = self.config.end_color;
                for i in 0..4 {
                    particle.color[i] = start_color[i] + (end_color[i] - start_color[i]) * lifetime_ratio;
                }
            }
        }
    }

    /// 清理死亡粒子
    pub fn cleanup_dead_particles(&mut self) {
        self.particles.retain(|p| p.lifetime > 0.0); // Check lifetime instead of state
    }

    /// 清除所有粒子
    pub fn clear_particles(&mut self) {
        self.particles.clear();
    }

    /// 获取活跃粒子数
    pub fn get_active_particle_count(&self) -> usize {
        self.particles.iter().filter(|p| p.lifetime > 0.0).count() // Check lifetime instead of state
    }

    /// 渲染粒子
    pub fn render(&self, render_system: &mut RenderSystem) {
        if self.particles.is_empty() {
            return;
        }

        // TODO: 实际渲染逻辑
        // 这里需要根据blend_mode、texture_path等属性来渲染粒子
        // 可能需要将粒子数据上传到GPU进行批量渲染
    }

    /// 获取发射器变换矩阵
    pub fn get_transform_matrix(&self) -> crate::math::Mat4 {
        crate::math::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// 重置发射器
    pub fn reset(&mut self) {
        self.particles.clear();
        self.emission_timer = 0.0;
        self.lifetime_timer = 0.0;
        self.burst_emitted = false;
        self.state = EmitterState::Stopped;
    }

    /// 预热发射器（预先模拟一段时间）
    pub fn prewarm(&mut self, duration: f32, time_step: f32) {
        if duration <= 0.0 || time_step <= 0.0 {
            return;
        }

        let old_state = self.state;
        self.start();

        let mut time = 0.0;
        while time < duration {
            let dt = time_step.min(duration - time);
            self.update(dt, self.config.max_particles);
            time += dt;
        }

        self.state = old_state;
    }
}
