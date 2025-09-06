//! Sanji游戏引擎默认入口
//! 
//! 这个文件提供了一个默认的引擎入口点
//! 运行: cargo run

use sanji_engine::{
    EngineResult, EngineConfig, WindowConfig, RenderConfig, AssetConfig,
    AppBuilder, App,
};

/// 默认应用程序
struct DefaultApp;

impl App for DefaultApp {
    fn startup(&mut self) -> EngineResult<()> {
        println!("🎮 欢迎使用Sanji游戏引擎!");
        println!("📋 当前运行默认应用程序");
        println!("💡 要查看更多演示，请运行:");
        println!("   cargo run --example simple_demo");
        println!("   cargo run --example basic_demo");
        println!();
        Ok(())
    }

    fn update(&mut self, delta_time: f32) -> EngineResult<()> {
        // 显示基本的运行信息
        static mut TIMER: f32 = 0.0;
        static mut FRAME_COUNT: u32 = 0;
        
        unsafe {
            FRAME_COUNT += 1;
            TIMER += delta_time;
            
            if TIMER >= 2.0 {
                let fps = FRAME_COUNT as f32 / TIMER;
                println!("📊 运行状态: FPS {:.1}, 平均帧时间 {:.1}ms", fps, (TIMER / FRAME_COUNT as f32) * 1000.0);
                TIMER = 0.0;
                FRAME_COUNT = 0;
            }
        }
        
        Ok(())
    }

    fn shutdown(&mut self) -> EngineResult<()> {
        println!("👋 感谢使用Sanji游戏引擎!");
        Ok(())
    }

    fn config(&self) -> EngineConfig {
        EngineConfig {
            window: WindowConfig {
                title: "Sanji游戏引擎 v0.1.0".to_string(),
                width: 1024,
                height: 768,
                vsync: true,
                resizable: true,
            },
            render: RenderConfig {
                backend: "auto".to_string(),
                msaa_samples: 4,
                max_texture_size: 4096,
            },
            assets: AssetConfig {
                asset_folder: "assets".to_string(),
                cache_size: 1024 * 1024 * 256, // 256MB
            },
        }
    }
}

fn main() -> EngineResult<()> {
    println!("🚀 启动Sanji游戏引擎...");
    
    AppBuilder::new(DefaultApp)
        .with_title("Sanji游戏引擎")
        .with_window_size(1024, 768)
        .run()
}
