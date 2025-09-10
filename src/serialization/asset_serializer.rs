//! 资源序列化器

use super::{Serializable, SerializationContext, SerializationFormat};
use crate::assets::{AssetHandle, AssetLoader, AssetCache};
use crate::EngineResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 资源元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub id: String,
    pub name: String,
    pub asset_type: String,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub created_at: String,
    pub modified_at: String,
    pub checksum: String,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub custom_data: HashMap<String, serde_json::Value>,
}

/// 序列化的资源包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedAssetBundle {
    pub metadata: AssetBundleMetadata,
    pub assets: Vec<SerializedAsset>,
    pub dependencies: HashMap<String, String>, // asset_id -> bundle_id
}

/// 资源包元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBundleMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub created_at: String,
    pub compression: CompressionType,
    pub platform: String,
    pub tags: Vec<String>,
}

/// 序列化的资源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedAsset {
    pub metadata: AssetMetadata,
    pub data: AssetData,
}

/// 资源数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetData {
    /// 二进制数据（如纹理、音频）
    Binary(Vec<u8>),
    /// 文本数据（如着色器、配置文件）
    Text(String),
    /// 结构化数据（如材质、网格定义）
    Structured(serde_json::Value),
    /// 外部引用（大文件）
    External { path: PathBuf, size: u64 },
}

/// 压缩类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Lz4,
    Zstd,
}

/// 纹理资源数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureAssetData {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub mip_levels: u32,
    pub data: Vec<u8>,
    pub srgb: bool,
}

/// 纹理格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextureFormat {
    R8,
    RG8,
    RGB8,
    RGBA8,
    R16F,
    RG16F,
    RGB16F,
    RGBA16F,
    R32F,
    RG32F,
    RGB32F,
    RGBA32F,
    BC1,
    BC2,
    BC3,
    BC4,
    BC5,
    BC6H,
    BC7,
}

/// 网格资源数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshAssetData {
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    pub submeshes: Vec<SubMeshData>,
    pub bounds: BoundsData,
}

/// 顶点数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexData {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub tangent: [f32; 4],
    pub color: [f32; 4],
}

/// 子网格数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubMeshData {
    pub start_index: u32,
    pub index_count: u32,
    pub material_index: u32,
}

/// 包围盒数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundsData {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub center: [f32; 3],
    pub extents: [f32; 3],
}

/// 材质资源数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialAssetData {
    pub shader_path: String,
    pub properties: HashMap<String, MaterialProperty>,
    pub textures: HashMap<String, String>, // property_name -> texture_path
    pub render_queue: u32,
    pub blend_mode: BlendMode,
    pub cull_mode: CullMode,
}

/// 材质属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaterialProperty {
    Float(f32),
    Vector2([f32; 2]),
    Vector3([f32; 3]),
    Vector4([f32; 4]),
    Color([f32; 4]),
    Int(i32),
    Bool(bool),
    Matrix4([[f32; 4]; 4]),
}

/// 混合模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlendMode {
    Opaque,
    Transparent,
    Additive,
    Multiply,
    Custom { src: BlendFactor, dst: BlendFactor },
}

/// 混合因子
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
}

/// 裁剪模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CullMode {
    None,
    Front,
    Back,
}

/// 音频资源数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAssetData {
    pub channels: u32,
    pub sample_rate: u32,
    pub bits_per_sample: u32,
    pub duration: f32,
    pub format: AudioFormat,
    pub data: Vec<u8>,
    pub loop_start: Option<u32>,
    pub loop_end: Option<u32>,
}

/// 音频格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFormat {
    WAV,
    MP3,
    OGG,
    FLAC,
    Raw,
}

/// 动画资源数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationAssetData {
    pub duration: f32,
    pub frame_rate: f32,
    pub tracks: Vec<AnimationTrack>,
    pub events: Vec<AnimationEvent>,
}

/// 动画轨道
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationTrack {
    pub target: String, // 目标对象路径
    pub property: String, // 属性名
    pub keyframes: Vec<Keyframe>,
    pub interpolation: InterpolationType,
}

/// 关键帧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    pub time: f32,
    pub value: KeyframeValue,
    pub in_tangent: Option<[f32; 2]>,
    pub out_tangent: Option<[f32; 2]>,
}

/// 关键帧值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyframeValue {
    Float(f32),
    Vector3([f32; 3]),
    Quaternion([f32; 4]),
    Bool(bool),
}

/// 插值类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterpolationType {
    Linear,
    Step,
    Bezier,
}

/// 动画事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationEvent {
    pub time: f32,
    pub name: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// 资源序列化器
pub struct AssetSerializer {
    compression: CompressionType,
    platform_specific: bool,
}

impl AssetSerializer {
    pub fn new() -> Self {
        Self {
            compression: CompressionType::Gzip,
            platform_specific: false,
        }
    }

    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.compression = compression;
        self
    }

    pub fn with_platform_specific(mut self, platform_specific: bool) -> Self {
        self.platform_specific = platform_specific;
        self
    }

    /// 序列化单个资源
    pub fn serialize_asset<P: AsRef<Path>>(&self, asset_path: P, asset_type: &str) -> EngineResult<SerializedAsset> {
        let path = asset_path.as_ref();
        let file_data = std::fs::read(path)?;
        let file_metadata = std::fs::metadata(path)?;

        let metadata = AssetMetadata {
            id: self.generate_asset_id(path),
            name: path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
            asset_type: asset_type.to_string(),
            file_path: path.to_path_buf(),
            file_size: file_metadata.len(),
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            checksum: self.calculate_checksum(&file_data),
            dependencies: Vec::new(),
            tags: Vec::new(),
            custom_data: HashMap::new(),
        };

        let data = match asset_type {
            "texture" => self.serialize_texture_data(&file_data)?,
            "mesh" => self.serialize_mesh_data(&file_data)?,
            "material" => self.serialize_material_data(&file_data)?,
            "audio" => self.serialize_audio_data(&file_data)?,
            "animation" => self.serialize_animation_data(&file_data)?,
            "shader" => AssetData::Text(String::from_utf8(file_data)?),
            _ => AssetData::Binary(file_data),
        };

        Ok(SerializedAsset { metadata, data })
    }

    /// 反序列化单个资源
    pub fn deserialize_asset<P: AsRef<Path>>(&self, asset: &SerializedAsset, output_path: P) -> EngineResult<()> {
        let path = output_path.as_ref();
        
        // 创建目录
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let data = match &asset.data {
            AssetData::Binary(data) => data.clone(),
            AssetData::Text(text) => text.as_bytes().to_vec(),
            AssetData::Structured(value) => {
                match asset.metadata.asset_type.as_str() {
                    "texture" => self.deserialize_texture_data(value)?,
                    "mesh" => self.deserialize_mesh_data(value)?,
                    "material" => self.deserialize_material_data(value)?,
                    "audio" => self.deserialize_audio_data(value)?,
                    "animation" => self.deserialize_animation_data(value)?,
                    _ => serde_json::to_vec_pretty(value)?,
                }
            }
            AssetData::External { path: external_path, .. } => {
                return self.copy_external_asset(external_path, path);
            }
        };

        std::fs::write(path, data)?;
        Ok(())
    }

    /// 创建资源包
    pub fn create_asset_bundle(&self, assets: Vec<SerializedAsset>, name: String) -> EngineResult<SerializedAssetBundle> {
        let metadata = AssetBundleMetadata {
            name,
            version: "1.0".to_string(),
            description: "Asset bundle created by Sanji Engine".to_string(),
            author: "Sanji Engine".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            compression: self.compression,
            platform: if self.platform_specific {
                std::env::consts::OS.to_string()
            } else {
                "universal".to_string()
            },
            tags: Vec::new(),
        };

        // 分析依赖关系
        let dependencies = self.analyze_dependencies(&assets)?;

        Ok(SerializedAssetBundle {
            metadata,
            assets,
            dependencies,
        })
    }

    /// 解包资源包
    pub fn extract_asset_bundle<P: AsRef<Path>>(&self, bundle: &SerializedAssetBundle, output_dir: P) -> EngineResult<()> {
        let output_path = output_dir.as_ref();
        
        for asset in &bundle.assets {
            let asset_path = output_path.join(&asset.metadata.file_path);
            self.deserialize_asset(asset, asset_path)?;
        }

        Ok(())
    }

    /// 压缩数据
    fn compress_data(&self, data: &[u8]) -> EngineResult<Vec<u8>> {
        match self.compression {
            CompressionType::None => Ok(data.to_vec()),
            CompressionType::Gzip => {
                use flate2::write::GzEncoder;
                use flate2::Compression;
                use std::io::Write;

                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(data)?;
                Ok(encoder.finish()?)
            }
            CompressionType::Lz4 => {
                // TODO: 实现LZ4压缩
                Ok(data.to_vec())
            }
            CompressionType::Zstd => {
                // TODO: 实现Zstd压缩
                Ok(data.to_vec())
            }
        }
    }

    /// 解压缩数据
    fn decompress_data(&self, data: &[u8]) -> EngineResult<Vec<u8>> {
        match self.compression {
            CompressionType::None => Ok(data.to_vec()),
            CompressionType::Gzip => {
                use flate2::read::GzDecoder;
                use std::io::Read;

                let mut decoder = GzDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            }
            CompressionType::Lz4 => {
                // TODO: 实现LZ4解压缩
                Ok(data.to_vec())
            }
            CompressionType::Zstd => {
                // TODO: 实现Zstd解压缩
                Ok(data.to_vec())
            }
        }
    }

    /// 生成资源ID
    fn generate_asset_id(&self, path: &Path) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(path.to_string_lossy().as_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    /// 计算校验和
    fn calculate_checksum(&self, data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// 分析依赖关系
    fn analyze_dependencies(&self, assets: &[SerializedAsset]) -> EngineResult<HashMap<String, String>> {
        let mut dependencies = HashMap::new();
        
        for asset in assets {
            // TODO: 分析资源之间的依赖关系
            // 例如：材质依赖纹理，模型依赖材质等
        }

        Ok(dependencies)
    }

    /// 序列化纹理数据
    fn serialize_texture_data(&self, data: &[u8]) -> EngineResult<AssetData> {
        // TODO: 解析图像文件并创建TextureAssetData
        // 这里需要使用image库来解析不同格式的图像
        Ok(AssetData::Binary(data.to_vec()))
    }

    /// 序列化网格数据
    fn serialize_mesh_data(&self, data: &[u8]) -> EngineResult<AssetData> {
        // TODO: 解析3D模型文件（如OBJ、GLTF等）
        Ok(AssetData::Binary(data.to_vec()))
    }

    /// 序列化材质数据
    fn serialize_material_data(&self, data: &[u8]) -> EngineResult<AssetData> {
        // TODO: 解析材质文件
        let text = String::from_utf8(data.to_vec())?;
        Ok(AssetData::Text(text))
    }

    /// 序列化音频数据
    fn serialize_audio_data(&self, data: &[u8]) -> EngineResult<AssetData> {
        // TODO: 解析音频文件并创建AudioAssetData
        Ok(AssetData::Binary(data.to_vec()))
    }

    /// 序列化动画数据
    fn serialize_animation_data(&self, data: &[u8]) -> EngineResult<AssetData> {
        // TODO: 解析动画文件并创建AnimationAssetData
        Ok(AssetData::Binary(data.to_vec()))
    }

    /// 反序列化纹理数据
    fn deserialize_texture_data(&self, _value: &serde_json::Value) -> EngineResult<Vec<u8>> {
        // TODO: 将TextureAssetData转换回图像文件
        Ok(Vec::new())
    }

    /// 反序列化网格数据
    fn deserialize_mesh_data(&self, _value: &serde_json::Value) -> EngineResult<Vec<u8>> {
        // TODO: 将MeshAssetData转换回模型文件
        Ok(Vec::new())
    }

    /// 反序列化材质数据
    fn deserialize_material_data(&self, value: &serde_json::Value) -> EngineResult<Vec<u8>> {
        let json = serde_json::to_string_pretty(value)?;
        Ok(json.into_bytes())
    }

    /// 反序列化音频数据
    fn deserialize_audio_data(&self, _value: &serde_json::Value) -> EngineResult<Vec<u8>> {
        // TODO: 将AudioAssetData转换回音频文件
        Ok(Vec::new())
    }

    /// 反序列化动画数据
    fn deserialize_animation_data(&self, _value: &serde_json::Value) -> EngineResult<Vec<u8>> {
        // TODO: 将AnimationAssetData转换回动画文件
        Ok(Vec::new())
    }

    /// 复制外部资源
    fn copy_external_asset<P: AsRef<Path>, Q: AsRef<Path>>(&self, src: P, dst: Q) -> EngineResult<()> {
        std::fs::copy(src, dst)?;
        Ok(())
    }
}

impl Default for AssetSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serializable for SerializedAssetBundle {
    fn serialize(&self, context: &SerializationContext) -> EngineResult<Vec<u8>> {
        match context.format {
            SerializationFormat::Json => {
                if context.pretty_print {
                    Ok(serde_json::to_vec_pretty(self)?)
                } else {
                    Ok(serde_json::to_vec(self)?)
                }
            }
            SerializationFormat::Binary => {
                Ok(bincode::serialize(self)?)
            }
            SerializationFormat::MessagePack => {
                Ok(rmp_serde::to_vec(self)?)
            }
            SerializationFormat::YAML => {
                let yaml_string = serde_yaml::to_string(self)?;
                Ok(yaml_string.into_bytes())
            }
        }
    }

    fn deserialize(data: &[u8], context: &SerializationContext) -> EngineResult<Self> {
        match context.format {
            SerializationFormat::Json => {
                Ok(serde_json::from_slice(data)?)
            }
            SerializationFormat::Binary => {
                Ok(bincode::deserialize(data)?)
            }
            SerializationFormat::MessagePack => {
                Ok(rmp_serde::from_slice(data)?)
            }
            SerializationFormat::YAML => {
                let yaml_string = String::from_utf8(data.to_vec())?;
                Ok(serde_yaml::from_str(&yaml_string)?)
            }
        }
    }
}
