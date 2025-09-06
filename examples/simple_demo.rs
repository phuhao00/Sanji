//! 简单的引擎演示
//! 
//! 一个最基础的演示程序，展示如何使用Sanji游戏引擎

use sanji_engine::{
    EngineResult, EngineConfig, WindowConfig, RenderConfig, AssetConfig,
    AppBuilder, App,
};

/// 简单演示应用
struct SimpleDemo;

impl App for SimpleDemo {
    fn startup(&mut self) -> EngineResult<()> {
        println!("🚀 Sanji游戏引擎启动!");
        println!("📦 这是一个简化的演示");
        Ok(())
    }

    fn update(&mut self, delta_time: f32) -> EngineResult<()> {
        // 简单的更新逻辑 - 每秒输出一次FPS信息
        static mut TIMER: f32 = 0.0;
        unsafe {
            TIMER += delta_time;
            if TIMER >= 1.0 {
                let fps = 1.0 / delta_time;
                println!("🎮 FPS: {:.1}, 帧时间: {:.2}ms", fps, delta_time * 1000.0);
                TIMER = 0.0;
            }
        }
        Ok(())
    }

    fn shutdown(&mut self) -> EngineResult<()> {
        println!("👋 引擎关闭");
        Ok(())
    }

    fn config(&self) -> EngineConfig {
        EngineConfig {
            window: WindowConfig {
                title: "Sanji引擎 - 简单演示".to_string(),
                width: 800,
                height: 600,
                vsync: true,
                resizable: true,
            },
            render: RenderConfig {
                backend: "auto".to_string(),
                msaa_samples: 1,
                max_texture_size: 2048,
            },
            assets: AssetConfig {
                asset_folder: "assets".to_string(),
                cache_size: 1024 * 1024 * 128, // 128MB
            },
        }
    }
}

fn main() -> EngineResult<()> {
    println!("🎯 启动Sanji游戏引擎简单演示");
    println!("⏱️  你将看到每秒更新的FPS信息");
    println!("🔚 关闭窗口来退出演示");
    println!();

    AppBuilder::new(SimpleDemo)
        .with_title("Sanji游戏引擎 - 简单演示")
        .with_window_size(800, 600)
        .run()
}
