//! 序列化系统模块

pub mod scene_serializer;
pub mod asset_serializer;
pub mod component_serializer;
pub mod binary_format;
pub mod json_format;

pub use scene_serializer::*;
pub use asset_serializer::*;
pub use component_serializer::*;
pub use binary_format::*;
pub use json_format::*;

use crate::EngineResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// 序列化格式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SerializationFormat {
    Json,           // JSON格式 - 人类可读
    Binary,         // 二进制格式 - 高效
    MessagePack,    // MessagePack格式 - 紧凑
    YAML,           // YAML格式 - 配置友好
}

impl SerializationFormat {
    /// 从文件扩展名推断格式
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "bin" | "data" => Some(Self::Binary),
            "msgpack" | "mp" => Some(Self::MessagePack),
            "yaml" | "yml" => Some(Self::YAML),
            _ => None,
        }
    }

    /// 获取默认文件扩展名
    pub fn default_extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Binary => "bin",
            Self::MessagePack => "msgpack",
            Self::YAML => "yaml",
        }
    }
}

/// 序列化上下文
#[derive(Debug, Clone)]
pub struct SerializationContext {
    pub format: SerializationFormat,
    pub pretty_print: bool,
    pub include_metadata: bool,
    pub compress: bool,
    pub version: u32,
    pub custom_data: HashMap<String, String>,
}

impl Default for SerializationContext {
    fn default() -> Self {
        Self {
            format: SerializationFormat::Json,
            pretty_print: true,
            include_metadata: true,
            compress: false,
            version: 1,
            custom_data: HashMap::new(),
        }
    }
}

/// 可序列化对象特征
pub trait Serializable {
    /// 序列化到字节数组
    fn serialize(&self, context: &SerializationContext) -> EngineResult<Vec<u8>>;
    
    /// 从字节数组反序列化
    fn deserialize(data: &[u8], context: &SerializationContext) -> EngineResult<Self>
    where
        Self: Sized;
    
    /// 序列化到文件
    fn serialize_to_file<P: AsRef<Path>>(&self, path: P, context: &SerializationContext) -> EngineResult<()> {
        let data = self.serialize(context)?;
        std::fs::write(path, data)?;
        Ok(())
    }
    
    /// 从文件反序列化
    fn deserialize_from_file<P: AsRef<Path>>(path: P, context: &SerializationContext) -> EngineResult<Self>
    where
        Self: Sized,
    {
        let data = std::fs::read(path)?;
        Self::deserialize(&data, context)
    }
}

/// 序列化元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializationMetadata {
    pub version: u32,
    pub timestamp: i64,
    pub engine_version: String,
    pub format: String,
    pub compressed: bool,
    pub checksum: String,
    pub custom_data: HashMap<String, String>,
}

impl SerializationMetadata {
    pub fn new(context: &SerializationContext) -> Self {
        Self {
            version: context.version,
            timestamp: chrono::Utc::now().timestamp(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            format: format!("{:?}", context.format),
            compressed: context.compress,
            checksum: String::new(), // 在实际序列化时计算
            custom_data: context.custom_data.clone(),
        }
    }
}

/// 序列化包装器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedData<T> {
    pub metadata: Option<SerializationMetadata>,
    pub data: T,
}

impl<T> SerializedData<T> {
    pub fn new(data: T, context: &SerializationContext) -> Self {
        let metadata = if context.include_metadata {
            Some(SerializationMetadata::new(context))
        } else {
            None
        };

        Self { metadata, data }
    }
}

/// 序列化器接口
pub trait Serializer {
    type Error: std::error::Error + Send + Sync + 'static;

    /// 序列化数据
    fn serialize<T: Serialize>(&self, data: &T, context: &SerializationContext) -> Result<Vec<u8>, Self::Error>;
    
    /// 反序列化数据
    fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8], context: &SerializationContext) -> Result<T, Self::Error>;
}

/// 序列化管理器
pub struct SerializationManager {
    serializers: HashMap<SerializationFormat, Box<dyn Serializer<Error = Box<dyn std::error::Error + Send + Sync>>>>,
    default_context: SerializationContext,
}

impl SerializationManager {
    pub fn new() -> Self {
        let mut manager = Self {
            serializers: HashMap::new(),
            default_context: SerializationContext::default(),
        };

        // 注册默认序列化器
        manager.register_serializer(SerializationFormat::Json, Box::new(JsonSerializer::new()));
        manager.register_serializer(SerializationFormat::Binary, Box::new(BinarySerializer::new()));

        manager
    }

    /// 注册序列化器
    pub fn register_serializer(&mut self, format: SerializationFormat, serializer: Box<dyn Serializer<Error = Box<dyn std::error::Error + Send + Sync>>>) {
        self.serializers.insert(format, serializer);
    }

    /// 设置默认上下文
    pub fn set_default_context(&mut self, context: SerializationContext) {
        self.default_context = context;
    }

    /// 序列化数据
    pub fn serialize<T: Serialize>(&self, data: &T, context: Option<&SerializationContext>) -> EngineResult<Vec<u8>> {
        let ctx = context.unwrap_or(&self.default_context);
        
        if let Some(serializer) = self.serializers.get(&ctx.format) {
            let wrapped_data = SerializedData::new(data, ctx);
            let result = serializer.serialize(&wrapped_data, ctx)
                .map_err(|e| anyhow::anyhow!("Serialization failed: {}", e))?;

            if ctx.compress {
                self.compress_data(&result)
            } else {
                Ok(result)
            }
        } else {
            Err(anyhow::anyhow!("No serializer registered for format: {:?}", ctx.format))
        }
    }

    /// 反序列化数据
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8], context: Option<&SerializationContext>) -> EngineResult<T> {
        let ctx = context.unwrap_or(&self.default_context);
        
        if let Some(serializer) = self.serializers.get(&ctx.format) {
            let decompressed_data = if ctx.compress {
                self.decompress_data(data)?
            } else {
                data.to_vec()
            };

            let wrapped: SerializedData<T> = serializer.deserialize(&decompressed_data, ctx)
                .map_err(|e| anyhow::anyhow!("Deserialization failed: {}", e))?;

            // 验证元数据
            if let Some(ref metadata) = wrapped.metadata {
                self.validate_metadata(metadata, ctx)?;
            }

            Ok(wrapped.data)
        } else {
            Err(anyhow::anyhow!("No serializer registered for format: {:?}", ctx.format))
        }
    }

    /// 序列化到文件
    pub fn serialize_to_file<T: Serialize, P: AsRef<Path>>(
        &self, 
        data: &T, 
        path: P, 
        context: Option<&SerializationContext>
    ) -> EngineResult<()> {
        let serialized = self.serialize(data, context)?;
        std::fs::write(path, serialized)?;
        Ok(())
    }

    /// 从文件反序列化
    pub fn deserialize_from_file<T: for<'de> Deserialize<'de>, P: AsRef<Path>>(
        &self, 
        path: P, 
        context: Option<&SerializationContext>
    ) -> EngineResult<T> {
        let data = std::fs::read(path)?;
        self.deserialize(&data, context)
    }

    /// 压缩数据
    fn compress_data(&self, data: &[u8]) -> EngineResult<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        let compressed = encoder.finish()?;
        Ok(compressed)
    }

    /// 解压缩数据
    fn decompress_data(&self, data: &[u8]) -> EngineResult<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    /// 验证元数据
    fn validate_metadata(&self, metadata: &SerializationMetadata, context: &SerializationContext) -> EngineResult<()> {
        // 版本兼容性检查
        if metadata.version > context.version {
            return Err(anyhow::anyhow!(
                "Data version {} is newer than supported version {}",
                metadata.version,
                context.version
            ));
        }

        // 格式检查
        let expected_format = format!("{:?}", context.format);
        if metadata.format != expected_format {
            return Err(anyhow::anyhow!(
                "Data format mismatch: expected {}, got {}",
                expected_format,
                metadata.format
            ));
        }

        // 压缩设置检查
        if metadata.compressed != context.compress {
            log::warn!(
                "Compression setting mismatch: expected {}, got {}",
                context.compress,
                metadata.compressed
            );
        }

        Ok(())
    }

    /// 获取支持的格式列表
    pub fn supported_formats(&self) -> Vec<SerializationFormat> {
        self.serializers.keys().copied().collect()
    }

    /// 估计序列化后的大小
    pub fn estimate_size<T: Serialize>(&self, data: &T, context: Option<&SerializationContext>) -> EngineResult<usize> {
        let serialized = self.serialize(data, context)?;
        Ok(serialized.len())
    }
}

impl Default for SerializationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 序列化工具函数
pub mod utils {
    use super::*;
    use std::path::Path;

    /// 快速序列化到JSON
    pub fn to_json<T: Serialize>(data: &T, pretty: bool) -> EngineResult<String> {
        let context = SerializationContext {
            format: SerializationFormat::Json,
            pretty_print: pretty,
            ..Default::default()
        };

        let manager = SerializationManager::new();
        let bytes = manager.serialize(data, Some(&context))?;
        Ok(String::from_utf8(bytes)?)
    }

    /// 快速从JSON反序列化
    pub fn from_json<T: for<'de> Deserialize<'de>>(json: &str) -> EngineResult<T> {
        let context = SerializationContext {
            format: SerializationFormat::Json,
            ..Default::default()
        };

        let manager = SerializationManager::new();
        manager.deserialize(json.as_bytes(), Some(&context))
    }

    /// 快速序列化到二进制
    pub fn to_binary<T: Serialize>(data: &T, compress: bool) -> EngineResult<Vec<u8>> {
        let context = SerializationContext {
            format: SerializationFormat::Binary,
            compress,
            ..Default::default()
        };

        let manager = SerializationManager::new();
        manager.serialize(data, Some(&context))
    }

    /// 快速从二进制反序列化
    pub fn from_binary<T: for<'de> Deserialize<'de>>(data: &[u8], compress: bool) -> EngineResult<T> {
        let context = SerializationContext {
            format: SerializationFormat::Binary,
            compress,
            ..Default::default()
        };

        let manager = SerializationManager::new();
        manager.deserialize(data, Some(&context))
    }

    /// 自动检测文件格式并序列化
    pub fn serialize_auto<T: Serialize, P: AsRef<Path>>(
        data: &T, 
        path: P, 
        pretty: bool
    ) -> EngineResult<()> {
        let path = path.as_ref();
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");

        let format = SerializationFormat::from_extension(extension)
            .unwrap_or(SerializationFormat::Json);

        let context = SerializationContext {
            format,
            pretty_print: pretty,
            ..Default::default()
        };

        let manager = SerializationManager::new();
        manager.serialize_to_file(data, path, Some(&context))
    }

    /// 自动检测文件格式并反序列化
    pub fn deserialize_auto<T: for<'de> Deserialize<'de>, P: AsRef<Path>>(
        path: P
    ) -> EngineResult<T> {
        let path = path.as_ref();
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json");

        let format = SerializationFormat::from_extension(extension)
            .unwrap_or(SerializationFormat::Json);

        let context = SerializationContext {
            format,
            ..Default::default()
        };

        let manager = SerializationManager::new();
        manager.deserialize_from_file(path, Some(&context))
    }

    /// 计算数据的校验和
    pub fn calculate_checksum(data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// 验证数据完整性
    pub fn verify_checksum(data: &[u8], expected: &str) -> bool {
        calculate_checksum(data) == expected
    }
}
