//! 着色器系统

use crate::{EngineResult, EngineError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 着色器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Compute,
}

/// 着色器属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderAttribute {
    pub name: String,
    pub location: u32,
    pub format: String,
}

/// 着色器uniform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderUniform {
    pub name: String,
    pub binding: u32,
    pub uniform_type: String,
    pub size: usize,
}

/// 着色器模块
#[derive(Debug, Clone)]
pub struct ShaderModule {
    pub name: String,
    pub source: String,
    pub shader_type: ShaderType,
    pub entry_point: String,
}

impl ShaderModule {
    /// 创建新的着色器模块
    pub fn new(
        name: impl Into<String>,
        source: impl Into<String>,
        shader_type: ShaderType,
        entry_point: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            source: source.into(),
            shader_type,
            entry_point: entry_point.into(),
        }
    }

    /// 从文件加载着色器
    pub fn from_file(
        name: impl Into<String>,
        path: impl AsRef<std::path::Path>,
        shader_type: ShaderType,
        entry_point: impl Into<String>,
    ) -> EngineResult<Self> {
        let source = std::fs::read_to_string(path.as_ref())
            .map_err(|e| EngineError::AssetError(format!("加载着色器文件失败: {}", e)))?;

        Ok(Self::new(name, source, shader_type, entry_point))
    }
}

/// 着色器程序
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shader {
    pub name: String,
    pub vertex_entry: String,
    pub fragment_entry: String,
    pub attributes: Vec<ShaderAttribute>,
    pub uniforms: Vec<ShaderUniform>,
    pub source: String,
}

impl Shader {
    /// 创建新的着色器
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vertex_entry: "vs_main".to_string(),
            fragment_entry: "fs_main".to_string(),
            attributes: Vec::new(),
            uniforms: Vec::new(),
            source: String::new(),
        }
    }

    /// 设置WGSL源码
    pub fn with_wgsl_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// 添加属性
    pub fn add_attribute(&mut self, name: impl Into<String>, location: u32, format: impl Into<String>) {
        self.attributes.push(ShaderAttribute {
            name: name.into(),
            location,
            format: format.into(),
        });
    }

    /// 添加uniform
    pub fn add_uniform(&mut self, name: impl Into<String>, binding: u32, uniform_type: impl Into<String>, size: usize) {
        self.uniforms.push(ShaderUniform {
            name: name.into(),
            binding,
            uniform_type: uniform_type.into(),
            size,
        });
    }
}

/// 着色器管理器
pub struct ShaderManager {
    shaders: HashMap<String, Shader>,
}

impl Default for ShaderManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaderManager {
    /// 创建新的着色器管理器
    pub fn new() -> Self {
        let mut manager = Self {
            shaders: HashMap::new(),
        };

        // 添加内置着色器
        manager.add_builtin_shaders();
        manager
    }

    /// 添加内置着色器
    fn add_builtin_shaders(&mut self) {
        // 基础着色器
        let basic_shader = Shader::new("basic")
            .with_wgsl_source(include_str!("shaders/basic.wgsl"));
        
        self.shaders.insert("basic".to_string(), basic_shader);

        // PBR着色器
        let pbr_source = r#"
// PBR着色器
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    position: vec3<f32>,
}

struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: mat3x3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> model: ModelUniform;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    let world_position = model.model * vec4<f32>(input.position, 1.0);
    out.world_position = world_position.xyz;
    out.world_normal = normalize(model.normal_matrix * input.normal);
    out.tex_coords = input.tex_coords;
    out.clip_position = camera.view_proj * world_position;
    
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = vec3<f32>(0.8, 0.8, 0.8);
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let normal = normalize(input.world_normal);
    
    let diffuse = max(dot(normal, light_dir), 0.0);
    let color = base_color * diffuse;
    
    return vec4<f32>(color, 1.0);
}
        "#;

        let pbr_shader = Shader::new("pbr")
            .with_wgsl_source(pbr_source);
        
        self.shaders.insert("pbr".to_string(), pbr_shader);
    }

    /// 添加着色器
    pub fn add_shader(&mut self, shader: Shader) {
        self.shaders.insert(shader.name.clone(), shader);
    }

    /// 获取着色器
    pub fn get_shader(&self, name: &str) -> Option<&Shader> {
        self.shaders.get(name)
    }

    /// 从文件加载着色器
    pub fn load_shader_from_file(&mut self, name: impl Into<String>, path: impl AsRef<std::path::Path>) -> EngineResult<()> {
        let name = name.into();
        let source = std::fs::read_to_string(path.as_ref())
            .map_err(|e| EngineError::AssetError(format!("加载着色器文件失败: {}", e)))?;

        let shader = Shader::new(name.clone()).with_wgsl_source(source);
        self.shaders.insert(name, shader);
        
        Ok(())
    }

    /// 获取所有着色器名称
    pub fn shader_names(&self) -> Vec<&String> {
        self.shaders.keys().collect()
    }
}
