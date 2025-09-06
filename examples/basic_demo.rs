//! Sanji游戏引擎基础演示
//! 
//! 这个演示展示了引擎的基本功能：
//! - 窗口管理
//! - 渲染系统
//! - ECS实体组件系统
//! - 输入处理
//! - 场景管理
//! - 数学工具

use sanji_engine::{
    EngineResult, EngineConfig, AppBuilder, App,
    ecs::{ECSWorld, Prefabs, Transform, MeshRenderer, Camera as CameraComponent, Light, LightType},
    scene::{Scene, SceneManager},
    input::InputManager,
    math::{Vec3, Quat},
    render::Camera,
    time::TimeManager,
    events::EventSystem,
};

use glam::Vec3 as GlamVec3;
use std::sync::{Arc, RwLock};

/// 演示应用程序
struct BasicDemo {
    ecs_world: Option<ECSWorld>,
    scene_manager: Option<SceneManager>,
    event_system: Option<Arc<RwLock<EventSystem>>>,
}

impl BasicDemo {
    fn new() -> Self {
        Self {
            ecs_world: None,
            scene_manager: None,
            event_system: None,
        }
    }
}

impl App for BasicDemo {
    fn startup(&mut self) -> EngineResult<()> {
        println!("🚀 启动Sanji游戏引擎演示!");
        
        // 初始化ECS世界
        let mut ecs_world = ECSWorld::new()?;
        ecs_world.setup_default_resources();
        
        // 初始化事件系统
        let event_system = Arc::new(RwLock::new(EventSystem::new()));
        
        // 初始化场景管理器
        let mut scene_manager = SceneManager::new();
        scene_manager.set_event_system(Arc::clone(&event_system));
        
        // 创建演示场景
        self.create_demo_scene(&mut ecs_world, &mut scene_manager)?;
        
        // 设置事件监听器
        self.setup_event_listeners(&event_system)?;
        
        // 保存到结构体
        self.ecs_world = Some(ecs_world);
        self.scene_manager = Some(scene_manager);
        self.event_system = Some(event_system);
        
        println!("✅ 引擎初始化完成!");
        Ok(())
    }

    fn update(&mut self, delta_time: f32) -> EngineResult<()> {
        // 更新ECS世界
        if let Some(ref mut ecs_world) = self.ecs_world {
            ecs_world.update(delta_time)?;
        }
        
        // 更新场景管理器
        if let Some(ref mut scene_manager) = self.scene_manager {
            scene_manager.update(delta_time)?;
        }
        
        // 处理事件
        if let Some(ref event_system) = self.event_system {
            if let Ok(mut events) = event_system.write() {
                events.process_events();
            }
        }
        
        // 演示旋转立方体
        self.animate_objects(delta_time)?;
        
        Ok(())
    }

    fn shutdown(&mut self) -> EngineResult<()> {
        println!("🛑 关闭引擎演示");
        
        // 清理场景
        if let (Some(ref mut scene_manager), Some(ref mut ecs_world)) = 
            (self.scene_manager.as_mut(), self.ecs_world.as_mut()) {
            scene_manager.clear_all_scenes(ecs_world)?;
        }
        
        println!("✅ 引擎关闭完成!");
        Ok(())
    }

    fn config(&self) -> EngineConfig {
        EngineConfig {
            window: sanji_engine::WindowConfig {
                title: "Sanji游戏引擎 - 基础演示".to_string(),
                width: 1280,
                height: 720,
                vsync: true,
                resizable: true,
            },
            render: sanji_engine::RenderConfig {
                backend: "auto".to_string(),
                msaa_samples: 4,
                max_texture_size: 4096,
            },
            assets: sanji_engine::AssetConfig {
                asset_folder: "assets".to_string(),
                cache_size: 1024 * 1024 * 256, // 256MB
            },
        }
    }
}

impl BasicDemo {
    /// 创建演示场景
    fn create_demo_scene(&mut self, ecs_world: &mut ECSWorld, scene_manager: &mut SceneManager) -> EngineResult<()> {
        println!("🏗️  创建演示场景...");
        
        // 创建场景
        let scene = scene_manager.create_default_scene("demo_scene");
        
        // 创建主相机
        let camera_entity = scene.spawn_prefab(
            ecs_world, 
            sanji_engine::scene::PrefabType::Camera, 
            "main_camera".to_string(), 
            GlamVec3::new(0.0, 2.0, 5.0)
        );
        
        // 调整相机看向原点
        {
            use specs::WorldExt;
            let mut cameras = ecs_world.world_mut().write_storage::<CameraComponent>();
            let mut transforms = ecs_world.world_mut().write_storage::<Transform>();
            
            if let (Some(camera), Some(transform)) = (cameras.get_mut(camera_entity), transforms.get_mut(camera_entity)) {
                camera.camera.look_at(GlamVec3::ZERO, GlamVec3::Y);
                transform.set_position(GlamVec3::new(0.0, 2.0, 5.0));
            }
        }
        
        // 创建一些几何体
        self.create_demo_objects(ecs_world, scene)?;
        
        // 创建光源
        let light_entity = scene.spawn_prefab(
            ecs_world,
            sanji_engine::scene::PrefabType::DirectionalLight,
            "sun_light".to_string(),
            GlamVec3::new(2.0, 4.0, 2.0)
        );
        
        // 设置光源属性
        {
            use specs::WorldExt;
            let mut lights = ecs_world.world_mut().write_storage::<Light>();
            let mut transforms = ecs_world.world_mut().write_storage::<Transform>();
            
            if let (Some(light), Some(transform)) = (lights.get_mut(light_entity), transforms.get_mut(light_entity)) {
                light.color = GlamVec3::new(1.0, 0.9, 0.8);
                light.intensity = 1.5;
                
                // 设置光源朝向
                let rotation = Quat::from_rotation_x(-45.0_f32.to_radians()) * 
                              Quat::from_rotation_y(30.0_f32.to_radians());
                transform.set_rotation(rotation);
            }
        }
        
        // 激活场景
        scene_manager.switch_to_scene("demo_scene")?;
        
        println!("✅ 演示场景创建完成");
        Ok(())
    }
    
    /// 创建演示对象
    fn create_demo_objects(&self, ecs_world: &mut ECSWorld, scene: &mut Scene) -> EngineResult<()> {
        // 创建旋转的立方体
        let cube1 = scene.spawn_prefab(
            ecs_world,
            sanji_engine::scene::PrefabType::Cube,
            "rotating_cube".to_string(),
            GlamVec3::new(-2.0, 0.0, 0.0)
        );
        
        let cube2 = scene.spawn_prefab(
            ecs_world,
            sanji_engine::scene::PrefabType::Cube,
            "floating_cube".to_string(),
            GlamVec3::new(2.0, 1.0, 0.0)
        );
        
        // 创建球体
        let sphere = scene.spawn_prefab(
            ecs_world,
            sanji_engine::scene::PrefabType::Sphere,
            "bouncing_sphere".to_string(),
            GlamVec3::new(0.0, 0.0, -2.0)
        );
        
        // 创建地面平面
        let ground = scene.spawn_prefab(
            ecs_world,
            sanji_engine::scene::PrefabType::Plane,
            "ground".to_string(),
            GlamVec3::new(0.0, -1.0, 0.0)
        );
        
        // 缩放地面
        {
            use specs::WorldExt;
            let mut transforms = ecs_world.world_mut().write_storage::<Transform>();
            if let Some(transform) = transforms.get_mut(ground) {
                transform.set_scale(GlamVec3::new(10.0, 1.0, 10.0));
            }
        }
        
        println!("📦 创建了演示对象: 立方体、球体、地面");
        Ok(())
    }
    
    /// 设置事件监听器
    fn setup_event_listeners(&self, event_system: &Arc<RwLock<EventSystem>>) -> EngineResult<()> {
        if let Ok(mut events) = event_system.write() {
            // 监听窗口调整大小事件
            events.subscribe::<sanji_engine::events::WindowResizedEvent, _>(|event| {
                println!("🖼️  窗口大小调整为: {}x{}", event.width, event.height);
            });
            
            // 监听按键事件
            events.subscribe::<sanji_engine::events::KeyPressedEvent, _>(|event| {
                println!("⌨️  按键按下: {:?}", event.key_code);
                
                // ESC键退出
                if event.key_code == winit::keyboard::KeyCode::Escape {
                    println!("👋 按下ESC键，准备退出...");
                }
            });
            
            // 监听鼠标事件
            events.subscribe::<sanji_engine::events::MouseButtonPressedEvent, _>(|event| {
                println!("🖱️  鼠标按钮按下: {:?} at ({:.1}, {:.1})", event.button, event.position.x, event.position.y);
            });
            
            // 监听场景事件
            events.subscribe::<sanji_engine::events::SceneLoadedEvent, _>(|event| {
                println!("🎬 场景已加载: {}", event.scene_name);
            });
        }
        
        println!("👂 事件监听器设置完成");
        Ok(())
    }
    
    /// 动画演示对象
    fn animate_objects(&mut self, delta_time: f32) -> EngineResult<()> {
        if let Some(ref mut ecs_world) = self.ecs_world {
            use specs::{WorldExt, Join};
            use sanji_engine::ecs::{Name, ReadStorage};
            
            let entities = ecs_world.world().entities();
            let names = ecs_world.world().read_storage::<Name>();
            let mut transforms = ecs_world.world_mut().write_storage::<Transform>();
            
            for (entity, name, transform) in (&entities, &names, &mut transforms).join() {
                match name.name.as_str() {
                    "rotating_cube" => {
                        // 旋转立方体
                        let rotation = Quat::from_rotation_y(delta_time * 2.0) * 
                                      Quat::from_rotation_x(delta_time * 1.5);
                        transform.rotate(rotation);
                    }
                    "floating_cube" => {
                        // 浮动立方体
                        let time = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f32();
                        let y = 1.0 + (time * 2.0).sin() * 0.5;
                        transform.set_position(GlamVec3::new(2.0, y, 0.0));
                    }
                    "bouncing_sphere" => {
                        // 弹跳球体
                        let time = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f32();
                        let y = (time * 3.0).sin().abs() * 2.0;
                        transform.set_position(GlamVec3::new(0.0, y, -2.0));
                    }
                    _ => {}
                }
            }
        }
        
        Ok(())
    }
}

fn main() -> EngineResult<()> {
    // 打印欢迎信息
    println!("🎮 欢迎使用Sanji游戏引擎!");
    println!("📖 这是一个基础演示，展示引擎的核心功能");
    println!("🎯 控制:");
    println!("   - ESC: 退出演示");
    println!("   - 鼠标: 与场景交互");
    println!("   - 观察动画中的几何体");
    println!();
    
    // 创建并运行应用程序
    AppBuilder::new(BasicDemo::new())
        .with_title("Sanji游戏引擎 - 基础演示 v0.1.0")
        .with_window_size(1280, 720)
        .run()
}
