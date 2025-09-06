# 🚀 Sanji游戏引擎快速开始指南

欢迎使用Sanji游戏引擎！这个指南将帮助您在几分钟内运行您的第一个游戏。

## 📋 系统要求

- **Rust**: 1.70 或更高版本
- **显卡**: 支持Vulkan、Metal或DirectX 12
- **操作系统**: Windows 10+、macOS 10.15+、或Linux

## ⚡ 快速运行

### 1. 克隆项目
```bash
git clone https://github.com/your-username/sanji-engine.git
cd sanji-engine
```

### 2. 构建项目
```bash
cargo build --release
```

### 3. 运行演示
```bash
# 简单演示 - 基础引擎功能
cargo run --example simple_demo

# 完整演示 - 3D场景、光照、动画
cargo run --example basic_demo

# 默认应用
cargo run
```

## 🎮 第一个游戏

创建 `my_game.rs`:

```rust
use sanji_engine::{
    EngineResult, App, AppBuilder,
    ecs::{ECSWorld, Prefabs, PrefabType},
    scene::{Scene, SceneManager},
    math::Vec3,
};

struct MyGame {
    ecs_world: Option<ECSWorld>,
    scene_manager: Option<SceneManager>,
}

impl MyGame {
    fn new() -> Self {
        Self {
            ecs_world: None,
            scene_manager: None,
        }
    }
}

impl App for MyGame {
    fn startup(&mut self) -> EngineResult<()> {
        println!("🎮 我的游戏启动了!");
        
        // 初始化ECS世界
        let mut ecs_world = ECSWorld::new()?;
        let mut scene_manager = SceneManager::new();
        
        // 创建场景
        let scene = scene_manager.create_scene("main");
        
        // 创建相机
        scene.spawn_prefab(
            &mut ecs_world,
            PrefabType::Camera,
            "camera".to_string(),
            Vec3::new(0.0, 1.0, 3.0)
        );
        
        // 创建立方体
        scene.spawn_prefab(
            &mut ecs_world,
            PrefabType::Cube,
            "cube".to_string(),
            Vec3::ZERO
        );
        
        // 创建光源
        scene.spawn_prefab(
            &mut ecs_world,
            PrefabType::DirectionalLight,
            "light".to_string(),
            Vec3::new(1.0, 1.0, 1.0)
        );
        
        // 激活场景
        scene_manager.switch_to_scene("main")?;
        
        self.ecs_world = Some(ecs_world);
        self.scene_manager = Some(scene_manager);
        
        Ok(())
    }

    fn update(&mut self, delta_time: f32) -> EngineResult<()> {
        // 更新ECS世界
        if let Some(ref mut ecs_world) = self.ecs_world {
            ecs_world.update(delta_time)?;
        }
        
        // 更新场景
        if let Some(ref mut scene_manager) = self.scene_manager {
            scene_manager.update(delta_time)?;
        }
        
        Ok(())
    }
}

fn main() -> EngineResult<()> {
    AppBuilder::new(MyGame::new())
        .with_title("我的第一个游戏")
        .with_window_size(1024, 768)
        .run()
}
```

运行您的游戏：
```bash
cargo run --bin my_game
```

## 🎯 核心概念

### 应用程序生命周期
```rust
impl App for MyGame {
    fn startup(&mut self) -> EngineResult<()> {
        // 游戏初始化 - 只运行一次
    }

    fn update(&mut self, delta_time: f32) -> EngineResult<()> {
        // 每帧更新 - 持续运行
    }

    fn shutdown(&mut self) -> EngineResult<()> {
        // 游戏清理 - 退出时运行
    }
}
```

### ECS系统
```rust
// 创建实体
let entity = scene.spawn_prefab(
    &mut ecs_world,
    PrefabType::Cube,     // 预制件类型
    "my_cube".to_string(), // 名称
    Vec3::new(1.0, 0.0, 0.0) // 位置
);

// 查找实体
if let Some(cube) = scene.find_entity("my_cube") {
    // 使用实体
}
```

### 输入处理
```rust
use sanji_engine::input::InputManager;

fn handle_input(input: &InputManager) {
    if input.is_action_just_pressed("jump") {
        println!("玩家跳跃!");
    }
    
    let movement = input.get_vector2d("horizontal", "vertical");
    // 使用movement进行移动
}
```

### 资源加载
```rust
use sanji_engine::assets::AssetManager;

// 加载纹理
let texture_handle = asset_manager.load::<Texture>("player.png")?;

// 使用资源
if let Some(texture) = asset_manager.get(&texture_handle) {
    // 使用纹理
}
```

## 🛠️ 配置选项

修改 `engine.toml` 来自定义引擎行为：

```toml
[window]
title = "我的游戏"
width = 1920
height = 1080

[render] 
msaa_samples = 4
backend = "auto"

[input.actions]
jump = ["Space", "GamepadSouth"]
move_left = ["KeyA"]
move_right = ["KeyD"]
```

## 📚 下一步

1. **阅读完整文档**: [README.md](./README.md)
2. **查看更多示例**: [examples/](./examples/)
3. **学习API**: `cargo doc --open`
4. **加入开发**: [CONTRIBUTING.md](./CONTRIBUTING.md)

## 🎮 预制件类型

引擎提供以下预制件：

- `PrefabType::Cube` - 立方体
- `PrefabType::Sphere` - 球体  
- `PrefabType::Plane` - 平面
- `PrefabType::Camera` - 相机
- `PrefabType::DirectionalLight` - 方向光
- `PrefabType::PointLight` - 点光源

## 🐛 故障排除

### 常见问题

**Q: 编译失败，提示缺少依赖**
```bash
# 更新依赖
cargo update
cargo build
```

**Q: 运行时黑屏**
- 确保您的显卡驱动程序是最新的
- 检查是否支持现代图形API

**Q: 性能问题**
- 使用release模式: `cargo run --release`
- 调整MSAA设置
- 检查GPU兼容性

**Q: 输入不响应**
- 确保窗口获得焦点
- 检查输入映射配置

## 💡 提示

- 使用 `--release` 模式获得最佳性能
- 查看控制台输出了解引擎状态
- 按 `ESC` 键退出大多数演示程序
- 修改演示程序来学习引擎功能

## 🎉 开始创造！

现在您已经准备好使用Sanji游戏引擎创建令人惊叹的游戏了！

---

**需要帮助？** 查看 [issues](https://github.com/your-username/sanji-engine/issues) 或创建新的问题报告。
