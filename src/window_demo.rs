//! Sanji游戏引擎 - 图形界面演示
//! 
//! 这个演示展示带有图形窗口的Sanji引擎

use winit::{
    application::ApplicationHandler,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

#[derive(Default)]
struct SanjiApp {
    window: Option<Window>,
}

impl ApplicationHandler for SanjiApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("🎮 Sanji游戏引擎 - 图形界面演示")
                    .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0)),
            )
            .unwrap();
        self.window = Some(window);
        
        println!("🎮 Sanji游戏引擎窗口已启动!");
        println!("📱 窗口大小: 1024x768");
        println!("⌨️  按ESC键退出");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let window = match self.window.as_ref() {
            Some(window) => window,
            None => return,
        };

        if window.id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                println!("👋 关闭窗口");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                println!("⌨️  按下ESC键，退出程序");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                println!("📐 窗口大小改变: {}x{}", physical_size.width, physical_size.height);
            }
            WindowEvent::RedrawRequested => {
                // 这里可以添加渲染逻辑
                window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 启动Sanji游戏引擎图形界面演示...");
    
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = SanjiApp::default();
    event_loop.run_app(&mut app)?;
    
    println!("✨ Sanji游戏引擎演示结束");
    Ok(())
}
