//! 核心组件定义

use crate::render::{Camera as RenderCamera, Mesh, Material};
use glam::{Vec3, Quat, Mat4};
use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage, DenseVecStorage, HashMapStorage};
use specs_derive::Component;

/// 变换组件 - 定义对象的位置、旋转和缩放
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct Transform {
    /// 位置
    pub position: Vec3,
    /// 旋转
    pub rotation: Quat,
    /// 缩放
    pub scale: Vec3,
    /// 局部变换矩阵(缓存)
    #[serde(skip)]
    pub local_matrix: Mat4,
    /// 世界变换矩阵(缓存)
    #[serde(skip)]
    pub world_matrix: Mat4,
    /// 是否需要更新矩阵
    #[serde(skip)]
    pub dirty: bool,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            local_matrix: Mat4::IDENTITY,
            world_matrix: Mat4::IDENTITY,
            dirty: true,
        }
    }
}

impl Transform {
    /// 创建新的变换
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置位置
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.dirty = true;
    }

    /// 设置旋转
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.dirty = true;
    }

    /// 设置缩放
    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
        self.dirty = true;
    }

    /// 平移
    pub fn translate(&mut self, translation: Vec3) {
        self.position += translation;
        self.dirty = true;
    }

    /// 旋转
    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = self.rotation * rotation;
        self.dirty = true;
    }

    /// 缩放
    pub fn scale_by(&mut self, scale: Vec3) {
        self.scale *= scale;
        self.dirty = true;
    }

    /// 获取前方向量
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    /// 获取右方向量
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// 获取上方向量
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// 更新变换矩阵
    pub fn update_matrices(&mut self) {
        if self.dirty {
            self.local_matrix = Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
            self.world_matrix = self.local_matrix; // 简化版本，不考虑父子关系
            self.dirty = false;
        }
    }

    /// 获取世界矩阵
    pub fn world_matrix(&mut self) -> Mat4 {
        self.update_matrices();
        self.world_matrix
    }
}

/// 网格渲染器组件
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct MeshRenderer {
    pub mesh_name: String,
    pub material_name: String,
    pub visible: bool,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
}

impl Default for MeshRenderer {
    fn default() -> Self {
        Self {
            mesh_name: "default".to_string(),
            material_name: "default".to_string(),
            visible: true,
            cast_shadows: true,
            receive_shadows: true,
        }
    }
}

impl MeshRenderer {
    pub fn new(mesh_name: impl Into<String>, material_name: impl Into<String>) -> Self {
        Self {
            mesh_name: mesh_name.into(),
            material_name: material_name.into(),
            ..Default::default()
        }
    }
}

/// 相机组件
#[derive(Component, Debug, Clone)]
#[storage(VecStorage)]
pub struct Camera {
    pub camera: RenderCamera,
    pub render_target: Option<String>,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            camera: RenderCamera::default(),
            render_target: None,
        }
    }
}

/// 光源类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LightType {
    Directional,
    Point,
    Spot,
}

/// 光源组件
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct Light {
    pub light_type: LightType,
    pub color: Vec3,
    pub intensity: f32,
    pub range: f32, // 点光源和聚光灯的范围
    pub spot_angle: f32, // 聚光灯的角度
    pub cast_shadows: bool,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            light_type: LightType::Directional,
            color: Vec3::ONE,
            intensity: 1.0,
            range: 10.0,
            spot_angle: 45.0_f32.to_radians(),
            cast_shadows: false,
        }
    }
}

/// 刚体组件(简化版物理)
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct RigidBody {
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub mass: f32,
    pub is_kinematic: bool,
    pub use_gravity: bool,
    pub drag: f32,
    pub angular_drag: f32,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass: 1.0,
            is_kinematic: false,
            use_gravity: true,
            drag: 0.0,
            angular_drag: 0.05,
        }
    }
}

/// 名称组件
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct Name {
    pub name: String,
}

impl Name {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// 标签组件
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[storage(HashMapStorage)]
pub struct Tag {
    pub tags: Vec<String>,
}

impl Tag {
    pub fn new() -> Self {
        Self { tags: Vec::new() }
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }
}

impl Default for Tag {
    fn default() -> Self {
        Self::new()
    }
}
