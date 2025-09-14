//! 资源加载器

use crate::{EngineResult, EngineError};
use crate::render::{Texture, Mesh, Material, Shader};
use std::path::Path;
use std::sync::Arc;
use std::any::Any;

/// 资源加载器trait
pub trait AssetLoader: Send + Sync {
    /// 资源类型
    type Asset: Send + Sync + 'static;

    /// 支持的文件扩展名
    fn extensions(&self) -> &[&str];

    /// 加载资源
    fn load(&self, path: &Path) -> EngineResult<Self::Asset>;

    /// 检查是否支持该文件
    fn supports_extension(&self, extension: &str) -> bool {
        self.extensions().contains(&extension)
    }
}

/// 纹理加载器
pub struct TextureLoader;

impl AssetLoader for TextureLoader {
    type Asset = Texture;

    fn extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "bmp", "tga", "dds", "hdr"]
    }

    fn load(&self, path: &Path) -> EngineResult<Self::Asset> {
        Texture::from_file(path)
    }
}

/// 着色器加载器
pub struct ShaderLoader;

impl AssetLoader for ShaderLoader {
    type Asset = Shader;

    fn extensions(&self) -> &[&str] {
        &["wgsl", "glsl", "hlsl"]
    }

    fn load(&self, path: &Path) -> EngineResult<Self::Asset> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| EngineError::AssetError(format!("读取着色器文件失败: {}", e)))?;
        
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        Ok(Shader::new(name).with_wgsl_source(source))
    }
}

/// 材质加载器
pub struct MaterialLoader;

impl AssetLoader for MaterialLoader {
    type Asset = Material;

    fn extensions(&self) -> &[&str] {
        &["mat", "json"]
    }

    fn load(&self, path: &Path) -> EngineResult<Self::Asset> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| EngineError::AssetError(format!("读取材质文件失败: {}", e)))?;
        
        let material: Material = serde_json::from_str(&content)
            .map_err(|e| EngineError::SerializationError(e))?;
        
        Ok(material)
    }
}

/// 网格加载器 (简化版)
pub struct MeshLoader;

impl AssetLoader for MeshLoader {
    type Asset = Mesh;

    fn extensions(&self) -> &[&str] {
        &["obj", "fbx", "gltf", "glb"]
    }

    fn load(&self, path: &Path) -> EngineResult<Self::Asset> {
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        match extension.to_lowercase().as_str() {
            "obj" => self.load_obj(path),
            _ => {
                // 对于其他格式，返回默认立方体
                log::warn!("不支持的网格格式: {}, 使用默认立方体", extension);
                Ok(Mesh::cube())
            }
        }
    }
}

impl MeshLoader {
    fn load_obj(&self, path: &Path) -> EngineResult<Mesh> {
        // 简化的OBJ加载器实现
        let content = std::fs::read_to_string(path)
            .map_err(|e| EngineError::AssetError(format!("读取OBJ文件失败: {}", e)))?;
        
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut tex_coords = Vec::new();
        let mut indices = Vec::new();
        
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            
            match parts[0] {
                "v" => {
                    if parts.len() >= 4 {
                        let x: f32 = parts[1].parse().unwrap_or(0.0);
                        let y: f32 = parts[2].parse().unwrap_or(0.0);
                        let z: f32 = parts[3].parse().unwrap_or(0.0);
                        positions.push(glam::Vec3::new(x, y, z));
                    }
                }
                "vn" => {
                    if parts.len() >= 4 {
                        let x: f32 = parts[1].parse().unwrap_or(0.0);
                        let y: f32 = parts[2].parse().unwrap_or(0.0);
                        let z: f32 = parts[3].parse().unwrap_or(0.0);
                        normals.push(glam::Vec3::new(x, y, z));
                    }
                }
                "vt" => {
                    if parts.len() >= 3 {
                        let u: f32 = parts[1].parse().unwrap_or(0.0);
                        let v: f32 = parts[2].parse().unwrap_or(0.0);
                        tex_coords.push(glam::Vec2::new(u, v));
                    }
                }
                "f" => {
                    // 简化处理，假设是三角形面
                    if parts.len() >= 4 {
                        for i in 1..4 {
                            if let Some(vertex_info) = parts.get(i) {
                                let vertex_parts: Vec<&str> = vertex_info.split('/').collect();
                                if let Ok(pos_idx) = vertex_parts[0].parse::<usize>() {
                                    indices.push((pos_idx - 1) as u32); // OBJ索引从1开始
                                }
                            }
                        }
                    }
                }
                _ => {} // 忽略其他行
            }
        }
        
        // 创建网格顶点
        let mut mesh = Mesh::new(
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("mesh")
                .to_string()
        );
        
        for &index in &indices {
            let pos = positions.get(index as usize).copied().unwrap_or(glam::Vec3::ZERO);
            let normal = normals.get(index as usize).copied().unwrap_or(glam::Vec3::Y);
            let tex_coord = tex_coords.get(index as usize).copied().unwrap_or(glam::Vec2::ZERO);
            
            mesh.vertices.push(crate::render::MeshVertex {
                position: pos,
                normal,
                tex_coords: tex_coord,
                color: glam::Vec3::ONE,
            });
        }
        
        // 生成索引
        mesh.indices = (0..mesh.vertices.len()).map(|i| i as u32).collect();
        
        // 如果没有法线，计算法线
        if normals.is_empty() {
            mesh.calculate_normals();
        }
        
        Ok(mesh)
    }
}

/// 音频加载器占位符
pub struct AudioLoader;

impl AssetLoader for AudioLoader {
    type Asset = AudioClip;

    fn extensions(&self) -> &[&str] {
        &["wav", "mp3", "ogg", "flac"]
    }

    fn load(&self, path: &Path) -> EngineResult<Self::Asset> {
        // 简化的音频加载实现
        let data = std::fs::read(path)
            .map_err(|e| EngineError::AssetError(format!("读取音频文件失败: {}", e)))?;
        
        Ok(AudioClip {
            name: path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("audio")
                .to_string(),
            data,
            format: AudioFormat::Unknown,
        })
    }
}

/// 音频剪辑 (简化)
#[derive(Debug, Clone)]
pub struct AudioClip {
    pub name: String,
    pub data: Vec<u8>,
    pub format: AudioFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum AudioFormat {
    Wav,
    Mp3,
    Ogg,
    Flac,
    Unknown,
}

/// 资源加载器注册表
pub struct AssetLoaderRegistry {
    loaders: Vec<Box<dyn ErasedAssetLoader>>,
}

/// 类型擦除的资源加载器
pub trait ErasedAssetLoader: Send + Sync {
    fn extensions(&self) -> &[&str];
    fn load(&self, path: &Path) -> EngineResult<Arc<dyn Any + Send + Sync>>;
    fn type_name(&self) -> &'static str;
}

/// 类型擦除包装器
struct TypeErasedLoader<L: AssetLoader> {
    loader: L,
}

impl<L: AssetLoader> ErasedAssetLoader for TypeErasedLoader<L> {
    fn extensions(&self) -> &[&str] {
        self.loader.extensions()
    }

    fn load(&self, path: &Path) -> EngineResult<Arc<dyn Any + Send + Sync>> {
        let asset = self.loader.load(path)?;
        Ok(Arc::new(asset) as Arc<dyn Any + Send + Sync>)
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<L::Asset>()
    }
}

impl AssetLoaderRegistry {
    /// 创建新的资源加载器注册表
    pub fn new() -> Self {
        Self {
            loaders: Vec::new(),
        }
    }

    /// 注册资源加载器
    pub fn register<L: AssetLoader + 'static>(&mut self, loader: L) {
        self.loaders.push(Box::new(TypeErasedLoader { loader }));
    }

    /// 根据文件扩展名查找加载器
    pub fn find_loader(&self, extension: &str) -> Option<&dyn ErasedAssetLoader> {
        self.loaders
            .iter()
            .find(|loader| loader.extensions().contains(&extension))
            .map(|loader| loader.as_ref())
    }

    /// 加载资源
    pub fn load_asset(&self, path: &Path) -> EngineResult<Arc<dyn Any + Send + Sync>> {
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| EngineError::AssetError("文件没有扩展名".to_string()))?;
        
        let loader = self.find_loader(extension)
            .ok_or_else(|| EngineError::AssetError(format!("不支持的文件格式: {}", extension)))?;
        
        loader.load(path)
    }

    /// 获取支持的文件扩展名
    pub fn supported_extensions(&self) -> Vec<&str> {
        self.loaders
            .iter()
            .flat_map(|loader| loader.extensions())
            .cloned()
            .collect()
    }
}

impl Default for AssetLoaderRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        
        // 注册内置加载器
        registry.register(TextureLoader);
        registry.register(ShaderLoader);
        registry.register(MaterialLoader);
        registry.register(MeshLoader);
        registry.register(AudioLoader);
        
        registry
    }
}
