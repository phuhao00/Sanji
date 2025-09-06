//! 应用程序抽象层

use crate::{EngineConfig, EngineResult, Engine};

/// 游戏应用程序trait
pub trait App {
    /// 应用程序启动时调用
    fn startup(&mut self) -> EngineResult<()> {
        Ok(())
    }

    /// 每帧更新时调用
    fn update(&mut self, delta_time: f32) -> EngineResult<()> {
        Ok(())
    }

    /// 应用程序关闭时调用
    fn shutdown(&mut self) -> EngineResult<()> {
        Ok(())
    }

    /// 获取引擎配置
    fn config(&self) -> EngineConfig {
        EngineConfig::default()
    }
}

/// 应用程序构建器
pub struct AppBuilder<T: App> {
    app: T,
    config: Option<EngineConfig>,
}

impl<T: App> AppBuilder<T> {
    /// 创建新的应用程序构建器
    pub fn new(app: T) -> Self {
        Self {
            app,
            config: None,
        }
    }

    /// 设置引擎配置
    pub fn with_config(mut self, config: EngineConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// 设置窗口标题
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        let mut config = self.config.unwrap_or_default();
        config.window.title = title.into();
        self.config = Some(config);
        self
    }

    /// 设置窗口大小
    pub fn with_window_size(mut self, width: u32, height: u32) -> Self {
        let mut config = self.config.unwrap_or_default();
        config.window.width = width;
        config.window.height = height;
        self.config = Some(config);
        self
    }

    /// 运行应用程序
    pub fn run(mut self) -> EngineResult<()> {
        let config = self.config.unwrap_or_else(|| self.app.config());
        
        // 初始化日志
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();

        log::info!("启动应用程序: {}", config.window.title);
        
        // 启动应用程序
        self.app.startup()?;
        
        // 创建并运行引擎
        let engine = Engine::new(config)?;
        engine.run()?;
        
        // 关闭应用程序
        self.app.shutdown()?;
        
        log::info!("应用程序已关闭");
        Ok(())
    }
}

/// 便捷宏，用于创建简单的应用程序
#[macro_export]
macro_rules! simple_app {
    ($name:ident) => {
        struct $name;
        
        impl $crate::App for $name {}
        
        fn main() -> $crate::EngineResult<()> {
            $crate::AppBuilder::new($name).run()
        }
    };
    
    ($name:ident, config: $config:expr) => {
        struct $name;
        
        impl $crate::App for $name {
            fn config(&self) -> $crate::EngineConfig {
                $config
            }
        }
        
        fn main() -> $crate::EngineResult<()> {
            $crate::AppBuilder::new($name).run()
        }
    };
}
