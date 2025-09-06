//! 核心引擎实现

use crate::{EngineConfig, EngineResult, EngineError};
use crate::render::RenderSystem;
use crate::ecs::ECSWorld;
use crate::assets::AssetManager;
use crate::scene::SceneManager;
use crate::input::InputManager;
use crate::time::TimeManager;
use crate::events::EventSystem;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId, WindowAttributes},
};

use std::sync::Arc;

/// 核心游戏引擎
pub struct Engine {
    config: EngineConfig,
    window: Option<Arc<Window>>,
    render_system: Option<RenderSystem>,
    ecs_world: ECSWorld,
    asset_manager: AssetManager,
    scene_manager: SceneManager,
    input_manager: InputManager,
    time_manager: TimeManager,
    event_system: EventSystem,
    running: bool,
}

impl Engine {
    /// 创建新的引擎实例
    pub fn new(config: EngineConfig) -> EngineResult<Self> {
        log::info!("初始化Sanji游戏引擎...");
        
        Ok(Self {
            config,
            window: None,
            render_system: None,
            ecs_world: ECSWorld::new()?,
            asset_manager: AssetManager::new()?,
            scene_manager: SceneManager::new(),
            input_manager: InputManager::new(),
            time_manager: TimeManager::new(),
            event_system: EventSystem::new(),
            running: false,
        })
    }

    /// 使用默认配置创建引擎
    pub fn default() -> EngineResult<Self> {
        Self::new(EngineConfig::default())
    }

    /// 运行引擎
    pub fn run(mut self) -> EngineResult<()> {
        let event_loop = EventLoop::new().map_err(|e| EngineError::RenderError(e.to_string()))?;
        event_loop.set_control_flow(ControlFlow::Poll);
        
        log::info!("启动游戏引擎主循环...");
        self.running = true;
        
        event_loop.run_app(&mut self)
            .map_err(|e| EngineError::RenderError(e.to_string()))?;
        
        Ok(())
    }

    /// 引擎更新循环
    fn update(&mut self) -> EngineResult<()> {
        // 更新时间管理器
        self.time_manager.update();
        let delta_time = self.time_manager.delta_time();
        
        // 更新输入管理器
        self.input_manager.update();
        
        // 更新ECS系统
        self.ecs_world.update(delta_time)?;
        
        // 更新场景管理器
        self.scene_manager.update(delta_time)?;
        
        Ok(())
    }

    /// 引擎渲染
    fn render(&mut self) -> EngineResult<()> {
        if let Some(ref mut render_system) = self.render_system {
            render_system.begin_frame()?;
            
            // 渲染当前场景
            if let Some(current_scene) = self.scene_manager.current_scene() {
                render_system.render_scene(current_scene, &self.ecs_world)?;
            }
            
            render_system.end_frame()?;
        }
        
        Ok(())
    }

    /// 处理窗口事件
    fn handle_window_event(&mut self, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("收到窗口关闭请求");
                self.running = false;
            }
            WindowEvent::Resized(physical_size) => {
                log::info!("窗口大小调整为: {:?}", physical_size);
                if let Some(ref mut render_system) = self.render_system {
                    if let Err(e) = render_system.resize(physical_size.width, physical_size.height) {
                        log::error!("调整渲染系统大小失败: {}", e);
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.input_manager.handle_keyboard_input(event);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.input_manager.handle_mouse_input(button, state);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input_manager.handle_mouse_move(position);
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = self.render() {
                    log::error!("渲染错误: {}", e);
                }
            }
            _ => {}
        }
    }
}

impl ApplicationHandler for Engine {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default()
            .with_title(&self.config.window.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.window.width,
                self.config.window.height,
            ))
            .with_resizable(self.config.window.resizable);

        let window = event_loop.create_window(window_attributes)
            .expect("创建窗口失败");
        
        let window = Arc::new(window);
        
        // 初始化渲染系统
        match pollster::block_on(RenderSystem::new(window.clone(), &self.config.render)) {
            Ok(render_system) => {
                self.render_system = Some(render_system);
                log::info!("渲染系统初始化成功");
            }
            Err(e) => {
                log::error!("渲染系统初始化失败: {}", e);
                return;
            }
        }
        
        self.window = Some(window);
        log::info!("引擎窗口创建成功");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        self.handle_window_event(window_id, event);
        
        if !self.running {
            event_loop.exit();
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // 主更新循环
        if let Err(e) = self.update() {
            log::error!("引擎更新错误: {}", e);
        }
        
        // 请求重绘
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
