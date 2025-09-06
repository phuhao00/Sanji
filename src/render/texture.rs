//! 纹理系统

use crate::{EngineResult, EngineError};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 纹理格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureFormat {
    Rgba8,
    Rgb8,
    R8,
    Rg8,
    Rgba16Float,
    Rgba32Float,
}

/// 纹理过滤模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    Linear,
    Nearest,
}

/// 纹理包装模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WrapMode {
    Repeat,
    MirrorRepeat,
    ClampToEdge,
    ClampToBorder,
}

/// 纹理描述符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub min_filter: FilterMode,
    pub mag_filter: FilterMode,
    pub wrap_u: WrapMode,
    pub wrap_v: WrapMode,
    pub generate_mipmaps: bool,
}

impl Default for TextureDescriptor {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            format: TextureFormat::Rgba8,
            min_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            wrap_u: WrapMode::Repeat,
            wrap_v: WrapMode::Repeat,
            generate_mipmaps: true,
        }
    }
}

/// 纹理数据
#[derive(Debug, Clone)]
pub struct Texture {
    pub descriptor: TextureDescriptor,
    pub data: Vec<u8>,
    pub name: String,
}

impl Texture {
    /// 创建新的纹理
    pub fn new(descriptor: TextureDescriptor, data: Vec<u8>, name: impl Into<String>) -> Self {
        Self {
            descriptor,
            data,
            name: name.into(),
        }
    }

    /// 从文件加载纹理
    pub fn from_file<P: AsRef<Path>>(path: P) -> EngineResult<Self> {
        let path = path.as_ref();
        let img = image::open(path)
            .map_err(|e| EngineError::AssetError(format!("加载纹理失败 {:?}: {}", path, e)))?;

        let img = img.to_rgba8();
        let (width, height) = img.dimensions();

        let descriptor = TextureDescriptor {
            width,
            height,
            format: TextureFormat::Rgba8,
            ..Default::default()
        };

        Ok(Self::new(
            descriptor,
            img.into_raw(),
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("未知纹理")
                .to_string(),
        ))
    }

    /// 创建纯色纹理
    pub fn solid_color(width: u32, height: u32, color: [u8; 4]) -> Self {
        let mut data = Vec::with_capacity((width * height * 4) as usize);
        for _ in 0..(width * height) {
            data.extend_from_slice(&color);
        }

        let descriptor = TextureDescriptor {
            width,
            height,
            format: TextureFormat::Rgba8,
            generate_mipmaps: false,
            ..Default::default()
        };

        Self::new(descriptor, data, "纯色纹理")
    }

    /// 创建棋盘格纹理
    pub fn checkerboard(size: u32, checker_size: u32) -> Self {
        let mut data = Vec::with_capacity((size * size * 4) as usize);
        
        for y in 0..size {
            for x in 0..size {
                let checker_x = (x / checker_size) % 2;
                let checker_y = (y / checker_size) % 2;
                let color = if (checker_x + checker_y) % 2 == 0 {
                    [255, 255, 255, 255] // 白色
                } else {
                    [0, 0, 0, 255] // 黑色
                };
                data.extend_from_slice(&color);
            }
        }

        let descriptor = TextureDescriptor {
            width: size,
            height: size,
            format: TextureFormat::Rgba8,
            ..Default::default()
        };

        Self::new(descriptor, data, "棋盘格纹理")
    }

    /// 获取像素数据大小
    pub fn data_size(&self) -> usize {
        let pixel_size = match self.descriptor.format {
            TextureFormat::Rgba8 => 4,
            TextureFormat::Rgb8 => 3,
            TextureFormat::R8 => 1,
            TextureFormat::Rg8 => 2,
            TextureFormat::Rgba16Float => 8,
            TextureFormat::Rgba32Float => 16,
        };
        (self.descriptor.width * self.descriptor.height) as usize * pixel_size
    }
}