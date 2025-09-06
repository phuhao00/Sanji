//! 材质系统

use crate::render::{Texture, TextureDescriptor};
use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 材质属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialProperties {
    /// 基础颜色
    pub base_color: Vec4,
    /// 金属度
    pub metallic: f32,
    /// 粗糙度
    pub roughness: f32,
    /// 法线强度
    pub normal_strength: f32,
    /// 自发光颜色
    pub emission: Vec3,
    /// 透明度
    pub alpha: f32,
    /// 是否双面渲染
    pub double_sided: bool,
}

impl Default for MaterialProperties {
    fn default() -> Self {
        Self {
            base_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            metallic: 0.0,
            roughness: 0.5,
            normal_strength: 1.0,
            emission: Vec3::ZERO,
            alpha: 1.0,
            double_sided: false,
        }
    }
}

/// 纹理槽类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextureSlot {
    BaseColor,
    Metallic,
    Roughness,
    Normal,
    Emission,
    Occlusion,
    Height,
}

/// 材质
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub name: String,
    pub properties: MaterialProperties,
    pub textures: HashMap<TextureSlot, String>, // 纹理资源路径
    pub shader_name: String,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "默认材质".to_string(),
            properties: MaterialProperties::default(),
            textures: HashMap::new(),
            shader_name: "标准".to_string(),
        }
    }
}

impl Material {
    /// 创建新的材质
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// 创建PBR材质
    pub fn pbr(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            shader_name: "pbr".to_string(),
            ..Default::default()
        }
    }

    /// 创建不受光照材质
    pub fn unlit(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            shader_name: "unlit".to_string(),
            ..Default::default()
        }
    }

    /// 设置基础颜色
    pub fn with_base_color(mut self, color: Vec4) -> Self {
        self.properties.base_color = color;
        self
    }

    /// 设置金属度
    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.properties.metallic = metallic.clamp(0.0, 1.0);
        self
    }

    /// 设置粗糙度
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.properties.roughness = roughness.clamp(0.0, 1.0);
        self
    }

    /// 设置自发光
    pub fn with_emission(mut self, emission: Vec3) -> Self {
        self.properties.emission = emission;
        self
    }

    /// 设置纹理
    pub fn with_texture(mut self, slot: TextureSlot, texture_path: impl Into<String>) -> Self {
        self.textures.insert(slot, texture_path.into());
        self
    }

    /// 移除纹理
    pub fn remove_texture(&mut self, slot: TextureSlot) {
        self.textures.remove(&slot);
    }

    /// 获取纹理路径
    pub fn get_texture(&self, slot: TextureSlot) -> Option<&String> {
        self.textures.get(&slot)
    }

    /// 设置着色器
    pub fn with_shader(mut self, shader_name: impl Into<String>) -> Self {
        self.shader_name = shader_name.into();
        self
    }
}
