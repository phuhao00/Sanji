//! Sanji游戏引擎综合演示
//! 
//! 这个示例展示了引擎的主要功能：
//! - 渲染系统（网格、材质、光照、阴影、后处理）
//! - ECS系统（实体、组件、系统）
//! - 物理系统（刚体、碰撞检测）
//! - 音频系统（3D音效）
//! - 动画系统（关键帧动画）
//! - UI系统（界面控件）
//! - 粒子系统（特效）
//! - 性能监控
//! - 序列化系统

use sanji_engine::*;
use std::time::Instant;

fn main() -> EngineResult<()> {
    // 初始化日志系统
    env_logger::init();
    log::info!("启动Sanji游戏引擎综合演示");

    // 创建引擎实例
    let mut engine = Engine::new("Sanji Engine - 综合演示", 1280, 720)?;
    
    // 设置性能监控
    engine.enable_performance_monitoring(true);
    engine.set_target_fps(60.0);

    // 创建演示场景
    let demo_scene = create_demo_scene(&mut engine)?;
    engine.load_scene(demo_scene)?;

    // 设置渲染配置
    configure_rendering(&mut engine)?;

    // 设置UI界面
    setup_ui(&mut engine)?;

    // 主循环
    log::info!("开始主渲染循环");
    let mut last_time = Instant::now();
    let mut frame_count = 0u64;

    engine.run(move |engine, delta_time| {
        frame_count += 1;
        
        // 更新游戏逻辑
        update_demo_logic(engine, delta_time)?;
        
        // 渲染帧
        render_frame(engine)?;
        
        // 更新性能统计
        if frame_count % 60 == 0 {
            print_performance_stats(engine, frame_count);
        }

        Ok(())
    })?;

    log::info!("演示结束");
    Ok(())
}

/// 创建演示场景
fn create_demo_scene(engine: &mut Engine) -> EngineResult<Scene> {
    let mut scene = Scene::new("Demo Scene".to_string());
    scene.description = "综合功能演示场景".to_string();

    // 创建地面
    let ground = create_ground_entity(engine)?;
    scene.add_entity(ground);

    // 创建一些立方体
    for i in 0..5 {
        for j in 0..5 {
            let cube = create_cube_entity(
                engine,
                Vec3::new(i as f32 * 2.0 - 4.0, 1.0, j as f32 * 2.0 - 4.0),
                i + j
            )?;
            scene.add_entity(cube);
        }
    }

    // 创建球体
    let sphere = create_sphere_entity(engine, Vec3::new(0.0, 5.0, 0.0))?;
    scene.add_entity(sphere);

    // 创建光源
    let sun_light = create_directional_light(engine)?;
    scene.add_entity(sun_light);

    let point_lights = create_point_lights(engine)?;
    for light in point_lights {
        scene.add_entity(light);
    }

    // 创建相机
    let camera = create_demo_camera(engine)?;
    scene.add_entity(camera);

    // 创建粒子系统
    let particle_systems = create_particle_systems(engine)?;
    for ps in particle_systems {
        scene.add_entity(ps);
    }

    // 创建音频源
    let audio_sources = create_audio_sources(engine)?;
    for audio in audio_sources {
        scene.add_entity(audio);
    }

    Ok(scene)
}

/// 创建地面实体
fn create_ground_entity(engine: &mut Engine) -> EngineResult<Entity> {
    let entity = engine.world.create_entity();

    // 变换组件
    let transform = TransformComponent {
        position: Vec3::new(0.0, 0.0, 0.0),
        scale: Vec3::new(20.0, 0.1, 20.0),
        ..Default::default()
    };

    // 渲染组件
    let render = RenderComponent {
        mesh_path: "meshes/cube.obj".to_string(),
        material_path: "materials/ground.mat".to_string(),
        receive_shadows: true,
        ..Default::default()
    };

    // 物理组件
    let physics = PhysicsComponent {
        body_type: PhysicsBodyType::Static,
        ..Default::default()
    };

    // 碰撞体组件
    let collider = ColliderComponent {
        shape: ColliderShape::Box { size: [20.0, 0.1, 20.0] },
        ..Default::default()
    };

    // 名称组件
    let name = NameComponent {
        name: "Ground".to_string(),
    };

    engine.world.add_component(entity, transform);
    engine.world.add_component(entity, render);
    engine.world.add_component(entity, physics);
    engine.world.add_component(entity, collider);
    engine.world.add_component(entity, name);

    Ok(entity)
}

/// 创建立方体实体
fn create_cube_entity(engine: &mut Engine, position: Vec3, variant: usize) -> EngineResult<Entity> {
    let entity = engine.world.create_entity();

    // 变换组件
    let transform = TransformComponent {
        position,
        scale: Vec3::ONE,
        ..Default::default()
    };

    // 渲染组件
    let material = match variant % 3 {
        0 => "materials/metal.mat",
        1 => "materials/wood.mat",
        _ => "materials/stone.mat",
    };

    let render = RenderComponent {
        mesh_path: "meshes/cube.obj".to_string(),
        material_path: material.to_string(),
        cast_shadows: true,
        receive_shadows: true,
        ..Default::default()
    };

    // 物理组件
    let physics = PhysicsComponent {
        body_type: PhysicsBodyType::Dynamic,
        mass: 1.0,
        ..Default::default()
    };

    // 碰撞体组件
    let collider = ColliderComponent {
        shape: ColliderShape::Box { size: [1.0, 1.0, 1.0] },
        ..Default::default()
    };

    // 动画组件
    let animation = AnimationComponent {
        controller_path: "animations/cube_idle.anim".to_string(),
        auto_play: true,
        ..Default::default()
    };

    // 名称组件
    let name = NameComponent {
        name: format!("Cube_{}", variant),
    };

    engine.world.add_component(entity, transform);
    engine.world.add_component(entity, render);
    engine.world.add_component(entity, physics);
    engine.world.add_component(entity, collider);
    engine.world.add_component(entity, animation);
    engine.world.add_component(entity, name);

    Ok(entity)
}

/// 创建球体实体
fn create_sphere_entity(engine: &mut Engine, position: Vec3) -> EngineResult<Entity> {
    let entity = engine.world.create_entity();

    // 变换组件
    let transform = TransformComponent {
        position,
        scale: Vec3::ONE * 0.5,
        ..Default::default()
    };

    // 渲染组件
    let render = RenderComponent {
        mesh_path: "meshes/sphere.obj".to_string(),
        material_path: "materials/glass.mat".to_string(),
        cast_shadows: true,
        receive_shadows: true,
        ..Default::default()
    };

    // 物理组件
    let physics = PhysicsComponent {
        body_type: PhysicsBodyType::Dynamic,
        mass: 0.5,
        ..Default::default()
    };

    // 碰撞体组件
    let collider = ColliderComponent {
        shape: ColliderShape::Sphere { radius: 0.5 },
        ..Default::default()
    };

    // 粒子系统组件
    let particle_system = ParticleSystemComponent {
        effect_name: "magic_sparkles".to_string(),
        auto_start: true,
        max_particles: 50,
        emission_rate: 20.0,
        ..Default::default()
    };

    // 音频源组件
    let audio_source = AudioSourceComponent {
        clip_path: "audio/magic.ogg".to_string(),
        volume: 0.5,
        spatial: true,
        loop_audio: true,
        play_on_awake: true,
        ..Default::default()
    };

    // 名称组件
    let name = NameComponent {
        name: "Magic Sphere".to_string(),
    };

    engine.world.add_component(entity, transform);
    engine.world.add_component(entity, render);
    engine.world.add_component(entity, physics);
    engine.world.add_component(entity, collider);
    engine.world.add_component(entity, particle_system);
    engine.world.add_component(entity, audio_source);
    engine.world.add_component(entity, name);

    Ok(entity)
}

/// 创建定向光源
fn create_directional_light(engine: &mut Engine) -> EngineResult<Entity> {
    let entity = engine.world.create_entity();

    // 变换组件
    let transform = TransformComponent {
        position: Vec3::new(0.0, 10.0, 0.0),
        rotation: Quat::from_euler(glam::EulerRot::XYZ, -0.8, 0.3, 0.0),
        ..Default::default()
    };

    // 光源组件
    let light = LightComponent {
        light_type: LightType::Directional,
        color: [1.0, 0.95, 0.8],
        intensity: 3.0,
        cast_shadows: true,
        ..Default::default()
    };

    // 名称组件
    let name = NameComponent {
        name: "Sun Light".to_string(),
    };

    engine.world.add_component(entity, transform);
    engine.world.add_component(entity, light);
    engine.world.add_component(entity, name);

    Ok(entity)
}

/// 创建点光源
fn create_point_lights(engine: &mut Engine) -> EngineResult<Vec<Entity>> {
    let mut lights = Vec::new();

    let light_positions = [
        Vec3::new(-5.0, 2.0, -5.0),
        Vec3::new(5.0, 2.0, -5.0),
        Vec3::new(-5.0, 2.0, 5.0),
        Vec3::new(5.0, 2.0, 5.0),
    ];

    let light_colors = [
        [1.0, 0.2, 0.2], // 红色
        [0.2, 1.0, 0.2], // 绿色
        [0.2, 0.2, 1.0], // 蓝色
        [1.0, 1.0, 0.2], // 黄色
    ];

    for (i, (&position, &color)) in light_positions.iter().zip(light_colors.iter()).enumerate() {
        let entity = engine.world.create_entity();

        // 变换组件
        let transform = TransformComponent {
            position,
            ..Default::default()
        };

        // 光源组件
        let light = LightComponent {
            light_type: LightType::Point,
            color,
            intensity: 2.0,
            range: 8.0,
            cast_shadows: false,
            ..Default::default()
        };

        // 名称组件
        let name = NameComponent {
            name: format!("Point Light {}", i + 1),
        };

        engine.world.add_component(entity, transform);
        engine.world.add_component(entity, light);
        engine.world.add_component(entity, name);

        lights.push(entity);
    }

    Ok(lights)
}

/// 创建演示相机
fn create_demo_camera(engine: &mut Engine) -> EngineResult<Entity> {
    let entity = engine.world.create_entity();

    // 变换组件
    let transform = TransformComponent {
        position: Vec3::new(0.0, 5.0, 10.0),
        rotation: Quat::from_euler(glam::EulerRot::XYZ, -0.3, 0.0, 0.0),
        ..Default::default()
    };

    // 相机组件
    let camera = CameraComponent {
        is_active: true,
        fov: 60.0,
        near_plane: 0.1,
        far_plane: 100.0,
        clear_color: [0.1, 0.2, 0.3, 1.0],
        ..Default::default()
    };

    // 名称组件
    let name = NameComponent {
        name: "Main Camera".to_string(),
    };

    engine.world.add_component(entity, transform);
    engine.world.add_component(entity, camera);
    engine.world.add_component(entity, name);

    Ok(entity)
}

/// 创建粒子系统
fn create_particle_systems(engine: &mut Engine) -> EngineResult<Vec<Entity>> {
    let mut systems = Vec::new();

    // 火焰效果
    let fire_entity = engine.world.create_entity();
    let fire_transform = TransformComponent {
        position: Vec3::new(-8.0, 1.0, 0.0),
        ..Default::default()
    };
    let fire_particles = ParticleSystemComponent {
        effect_name: "fire".to_string(),
        auto_start: true,
        max_particles: 200,
        emission_rate: 50.0,
        start_color: [1.0, 0.5, 0.0, 1.0],
        ..Default::default()
    };
    let fire_name = NameComponent {
        name: "Fire Effect".to_string(),
    };

    engine.world.add_component(fire_entity, fire_transform);
    engine.world.add_component(fire_entity, fire_particles);
    engine.world.add_component(fire_entity, fire_name);
    systems.push(fire_entity);

    // 烟雾效果
    let smoke_entity = engine.world.create_entity();
    let smoke_transform = TransformComponent {
        position: Vec3::new(8.0, 1.0, 0.0),
        ..Default::default()
    };
    let smoke_particles = ParticleSystemComponent {
        effect_name: "smoke".to_string(),
        auto_start: true,
        max_particles: 100,
        emission_rate: 25.0,
        start_color: [0.7, 0.7, 0.7, 0.5],
        ..Default::default()
    };
    let smoke_name = NameComponent {
        name: "Smoke Effect".to_string(),
    };

    engine.world.add_component(smoke_entity, smoke_transform);
    engine.world.add_component(smoke_entity, smoke_particles);
    engine.world.add_component(smoke_entity, smoke_name);
    systems.push(smoke_entity);

    Ok(systems)
}

/// 创建音频源
fn create_audio_sources(engine: &mut Engine) -> EngineResult<Vec<Entity>> {
    let mut sources = Vec::new();

    // 环境音乐
    let ambient_entity = engine.world.create_entity();
    let ambient_audio = AudioSourceComponent {
        clip_path: "audio/ambient.ogg".to_string(),
        volume: 0.3,
        loop_audio: true,
        spatial: false,
        play_on_awake: true,
        ..Default::default()
    };
    let ambient_name = NameComponent {
        name: "Ambient Music".to_string(),
    };

    engine.world.add_component(ambient_entity, ambient_audio);
    engine.world.add_component(ambient_entity, ambient_name);
    sources.push(ambient_entity);

    // 风声
    let wind_entity = engine.world.create_entity();
    let wind_transform = TransformComponent {
        position: Vec3::new(0.0, 5.0, 0.0),
        ..Default::default()
    };
    let wind_audio = AudioSourceComponent {
        clip_path: "audio/wind.ogg".to_string(),
        volume: 0.2,
        loop_audio: true,
        spatial: true,
        play_on_awake: true,
        ..Default::default()
    };
    let wind_name = NameComponent {
        name: "Wind Sound".to_string(),
    };

    engine.world.add_component(wind_entity, wind_transform);
    engine.world.add_component(wind_entity, wind_audio);
    engine.world.add_component(wind_entity, wind_name);
    sources.push(wind_entity);

    Ok(sources)
}

/// 配置渲染设置
fn configure_rendering(engine: &mut Engine) -> EngineResult<()> {
    // 启用阴影渲染
    engine.renderer.enable_shadows(true);
    engine.renderer.set_shadow_resolution(2048);
    engine.renderer.set_shadow_cascade_count(4);

    // 启用后处理效果
    engine.renderer.enable_post_processing(true);
    engine.renderer.enable_bloom(true, 1.5);
    engine.renderer.enable_tone_mapping(true, ToneMappingType::ACES);
    engine.renderer.enable_fxaa(true);

    // 设置渲染质量
    engine.renderer.set_msaa_samples(4);
    engine.renderer.set_anisotropic_filtering(16);

    log::info!("渲染配置完成");
    Ok(())
}

/// 设置UI界面
fn setup_ui(engine: &mut Engine) -> EngineResult<()> {
    // 创建FPS显示面板
    let fps_panel = create_fps_panel();
    engine.ui_system.add_widget(fps_panel);

    // 创建性能监控面板
    let perf_panel = create_performance_panel();
    engine.ui_system.add_widget(perf_panel);

    // 创建控制面板
    let control_panel = create_control_panel();
    engine.ui_system.add_widget(control_panel);

    // 创建场景信息面板
    let scene_panel = create_scene_info_panel();
    engine.ui_system.add_widget(scene_panel);

    log::info!("UI界面设置完成");
    Ok(())
}

/// 创建FPS显示面板
fn create_fps_panel() -> ui::Panel {
    ui::Panel::new("fps_panel")
        .with_title("FPS")
        .with_position(10.0, 10.0)
        .with_size(150.0, 60.0)
        .with_background_color([0.0, 0.0, 0.0, 0.7])
        .add_child(
            ui::Text::new("fps_text")
                .with_text("FPS: 60")
                .with_color([1.0, 1.0, 1.0, 1.0])
                .with_font_size(16.0)
        )
}

/// 创建性能监控面板
fn create_performance_panel() -> ui::Panel {
    ui::Panel::new("perf_panel")
        .with_title("Performance")
        .with_position(10.0, 80.0)
        .with_size(200.0, 120.0)
        .with_background_color([0.0, 0.0, 0.0, 0.7])
        .add_child(
            ui::Text::new("cpu_text")
                .with_text("CPU: 0%")
                .with_color([1.0, 1.0, 1.0, 1.0])
        )
        .add_child(
            ui::Text::new("memory_text")
                .with_text("Memory: 0 MB")
                .with_color([1.0, 1.0, 1.0, 1.0])
        )
        .add_child(
            ui::Text::new("draw_calls_text")
                .with_text("Draw Calls: 0")
                .with_color([1.0, 1.0, 1.0, 1.0])
        )
}

/// 创建控制面板
fn create_control_panel() -> ui::Panel {
    ui::Panel::new("control_panel")
        .with_title("Controls")
        .with_position(10.0, 210.0)
        .with_size(180.0, 200.0)
        .with_background_color([0.0, 0.0, 0.0, 0.7])
        .add_child(
            ui::Button::new("toggle_wireframe")
                .with_text("Toggle Wireframe")
                .with_on_click(|_| log::info!("Wireframe toggled"))
        )
        .add_child(
            ui::Button::new("reload_shaders")
                .with_text("Reload Shaders")
                .with_on_click(|_| log::info!("Shaders reloaded"))
        )
        .add_child(
            ui::Slider::new("exposure")
                .with_label("Exposure")
                .with_range(0.1, 3.0)
                .with_value(1.0)
        )
}

/// 创建场景信息面板
fn create_scene_info_panel() -> ui::Panel {
    ui::Panel::new("scene_panel")
        .with_title("Scene Info")
        .with_position(10.0, 420.0)
        .with_size(200.0, 100.0)
        .with_background_color([0.0, 0.0, 0.0, 0.7])
        .add_child(
            ui::Text::new("entities_text")
                .with_text("Entities: 0")
                .with_color([1.0, 1.0, 1.0, 1.0])
        )
        .add_child(
            ui::Text::new("lights_text")
                .with_text("Lights: 0")
                .with_color([1.0, 1.0, 1.0, 1.0])
        )
}

/// 更新演示逻辑
fn update_demo_logic(engine: &mut Engine, delta_time: f32) -> EngineResult<()> {
    // 旋转魔法球
    if let Some(sphere_entity) = engine.find_entity_by_name("Magic Sphere") {
        if let Some(transform) = engine.world.get_component_mut::<TransformComponent>(sphere_entity) {
            transform.rotation *= Quat::from_euler(glam::EulerRot::Y, delta_time, 0.0, 0.0);
        }
    }

    // 让点光源围绕场景中心旋转
    let time = engine.get_time();
    for i in 1..=4 {
        let light_name = format!("Point Light {}", i);
        if let Some(light_entity) = engine.find_entity_by_name(&light_name) {
            if let Some(transform) = engine.world.get_component_mut::<TransformComponent>(light_entity) {
                let angle = time * 0.5 + (i as f32 - 1.0) * std::f32::consts::PI * 0.5;
                transform.position.x = angle.cos() * 6.0;
                transform.position.z = angle.sin() * 6.0;
            }
        }
    }

    // 更新UI文本
    update_ui_text(engine)?;

    Ok(())
}

/// 更新UI文本
fn update_ui_text(engine: &mut Engine) -> EngineResult<()> {
    let stats = engine.get_performance_stats();

    // 更新FPS显示
    if let Some(fps_text) = engine.ui_system.get_widget_mut("fps_text") {
        if let Some(text_widget) = fps_text.as_any_mut().downcast_mut::<ui::Text>() {
            text_widget.set_text(format!("FPS: {:.1}", stats.fps));
        }
    }

    // 更新性能信息
    if let Some(cpu_text) = engine.ui_system.get_widget_mut("cpu_text") {
        if let Some(text_widget) = cpu_text.as_any_mut().downcast_mut::<ui::Text>() {
            text_widget.set_text(format!("CPU: {:.1}%", stats.cpu_usage));
        }
    }

    if let Some(memory_text) = engine.ui_system.get_widget_mut("memory_text") {
        if let Some(text_widget) = memory_text.as_any_mut().downcast_mut::<ui::Text>() {
            let memory_mb = stats.memory_usage.current_allocated as f32 / (1024.0 * 1024.0);
            text_widget.set_text(format!("Memory: {:.1} MB", memory_mb));
        }
    }

    if let Some(draw_calls_text) = engine.ui_system.get_widget_mut("draw_calls_text") {
        if let Some(text_widget) = draw_calls_text.as_any_mut().downcast_mut::<ui::Text>() {
            text_widget.set_text(format!("Draw Calls: {}", stats.render_stats.draw_calls));
        }
    }

    // 更新场景信息
    let entity_count = engine.world.get_entity_count();
    let light_count = engine.world.count_components::<LightComponent>();

    if let Some(entities_text) = engine.ui_system.get_widget_mut("entities_text") {
        if let Some(text_widget) = entities_text.as_any_mut().downcast_mut::<ui::Text>() {
            text_widget.set_text(format!("Entities: {}", entity_count));
        }
    }

    if let Some(lights_text) = engine.ui_system.get_widget_mut("lights_text") {
        if let Some(text_widget) = lights_text.as_any_mut().downcast_mut::<ui::Text>() {
            text_widget.set_text(format!("Lights: {}", light_count));
        }
    }

    Ok(())
}

/// 渲染帧
fn render_frame(engine: &mut Engine) -> EngineResult<()> {
    engine.begin_frame();
    
    // 渲染场景
    engine.render_scene()?;
    
    // 渲染UI
    engine.render_ui()?;
    
    engine.end_frame();
    
    Ok(())
}

/// 打印性能统计
fn print_performance_stats(engine: &Engine, frame_count: u64) {
    let stats = engine.get_performance_stats();
    
    log::info!("=== 性能统计 (Frame {}) ===", frame_count);
    log::info!("FPS: {:.1}", stats.fps);
    log::info!("帧时间: {:.2}ms", stats.frame_time.as_millis());
    log::info!("CPU使用率: {:.1}%", stats.cpu_usage);
    log::info!("内存使用: {:.1}MB", stats.memory_usage.current_allocated as f32 / (1024.0 * 1024.0));
    log::info!("绘制调用: {}", stats.render_stats.draw_calls);
    log::info!("三角形数: {}", stats.render_stats.triangles);
    log::info!("活跃物理体: {}", stats.physics_stats.active_bodies);
    log::info!("音频源: {}", stats.audio_stats.active_sources);
}

// 引擎扩展trait，提供便利方法
trait EngineExt {
    fn find_entity_by_name(&self, name: &str) -> Option<Entity>;
    fn get_time(&self) -> f32;
    fn get_performance_stats(&self) -> PerformanceStats;
}

impl EngineExt for Engine {
    fn find_entity_by_name(&self, name: &str) -> Option<Entity> {
        // TODO: 实现根据名称查找实体
        None
    }

    fn get_time(&self) -> f32 {
        // TODO: 实现获取引擎运行时间
        0.0
    }

    fn get_performance_stats(&self) -> PerformanceStats {
        // TODO: 实现获取性能统计
        PerformanceStats::default()
    }
}
