//! Sanji Game Engine - 一个用Rust编写的现代游戏引擎
//! 
//! 这个引擎提供了类似Unity和Unreal Engine的功能，包括：
//! - 现代化的渲染管线
//! - 实体组件系统(ECS)
//! - 资源管理
//! - 场景系统
//! - 输入处理
//! - 物理引擎集成

pub mod core;
pub mod render;
pub mod ecs;
pub mod assets;
pub mod scene;
pub mod input;
pub mod math;
pub mod time;
pub mod events;
pub mod physics;
pub mod audio;
pub mod animation;
pub mod ui;
pub mod particles;
pub mod serialization;
pub mod performance;

pub use core::*;

/// 引擎结果类型
pub type EngineResult<T> = anyhow::Result<T>;

/// 引擎错误类型
#[derive(thiserror::Error, Debug)]
pub enum EngineError {
    #[error("渲染错误: {0}")]
    RenderError(String),
    
    #[error("资源加载错误: {0}")]
    AssetError(String),
    
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// 引擎配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EngineConfig {
    pub window: WindowConfig,
    pub render: RenderConfig,
    pub assets: AssetConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            render: RenderConfig::default(),
            assets: AssetConfig::default(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub resizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Sanji Game Engine".to_string(),
            width: 1920,
            height: 1080,
            vsync: true,
            resizable: true,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RenderConfig {
    pub backend: String,
    pub msaa_samples: u32,
    pub max_texture_size: u32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            backend: "auto".to_string(),
            msaa_samples: 4,
            max_texture_size: 8192,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssetConfig {
    pub asset_folder: String,
    pub cache_size: usize,
}

impl Default for AssetConfig {
    fn default() -> Self {
        Self {
            asset_folder: "assets".to_string(),
            cache_size: 1024 * 1024 * 512, // 512MB
        }
    }
}
