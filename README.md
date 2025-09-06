# Sanji 游戏引擎

一个用Rust语言编写的现代化游戏引擎，提供类似Unity和Unreal Engine的功能。

## 🚀 功能特色

### 核心系统
- **现代化渲染管线** - 基于wgpu的跨平台图形渲染
- **实体组件系统(ECS)** - 使用specs库实现的高性能ECS架构  
- **场景管理** - 层次化场景图和场景切换系统
- **资源管理** - 智能资源缓存和异步加载系统
- **输入处理** - 完整的键盘、鼠标和游戏手柄支持
- **事件系统** - 灵活的事件发布/订阅机制
- **数学工具库** - 3D数学、碰撞检测、噪声生成等

### 渲染特性
- **PBR材质系统** - 物理基础渲染
- **多种光源** - 方向光、点光源、聚光灯
- **纹理管理** - 多格式纹理支持
- **着色器系统** - WGSL着色器支持
- **相机系统** - 透视和正交投影

### 工具和便利性
- **缓动动画** - 内置多种缓动函数
- **噪声生成** - Perlin噪声和其他程序化生成工具
- **碰撞检测** - AABB、球体、射线等碰撞检测
- **预制件系统** - 常用对象的快速创建

## 🏗️ 架构设计

### 模块结构

```
sanji_engine/
├── core/           # 核心引擎和应用程序框架
├── render/         # 渲染系统和图形API
├── ecs/           # 实体组件系统
├── scene/         # 场景管理和场景图
├── assets/        # 资源管理和加载
├── input/         # 输入处理系统
├── math/          # 数学工具库
├── events/        # 事件系统
└── time/          # 时间管理
```

### 核心组件

#### 引擎核心 (`core/`)
- `Engine` - 主引擎类
- `App` trait - 应用程序接口
- `AppBuilder` - 应用程序构建器

#### 渲染系统 (`render/`)
- `RenderSystem` - 主渲染器
- `Camera` - 相机管理
- `Material` - 材质系统
- `Mesh` - 几何体数据
- `Texture` - 纹理管理
- `Shader` - 着色器系统

#### ECS系统 (`ecs/`)
- `ECSWorld` - ECS世界管理器
- `Transform` - 变换组件
- `MeshRenderer` - 网格渲染器组件
- `Camera` - 相机组件
- `Light` - 光源组件
- `RigidBody` - 物理组件

## 📖 使用指南

### 快速开始

1. **添加依赖**

```toml
[dependencies]
sanji_engine = "0.1.0"
```

2. **创建简单应用**

```rust
use sanji_engine::{App, AppBuilder, EngineResult};

struct MyGame;

impl App for MyGame {
    fn startup(&mut self) -> EngineResult<()> {
        println!("游戏启动!");
        Ok(())
    }

    fn update(&mut self, delta_time: f32) -> EngineResult<()> {
        // 游戏逻辑更新
        Ok(())
    }
}

fn main() -> EngineResult<()> {
    AppBuilder::new(MyGame)
        .with_title("我的游戏")
        .with_window_size(1280, 720)
        .run()
}
```

### 创建3D场景

```rust
use sanji_engine::{
    ecs::Prefabs,
    scene::{Scene, PrefabType},
    math::Vec3,
};

// 在应用startup中创建场景
fn create_scene(ecs_world: &mut ECSWorld, scene: &mut Scene) -> EngineResult<()> {
    // 创建相机
    let camera = scene.spawn_prefab(
        ecs_world, 
        PrefabType::Camera, 
        "main_camera".to_string(), 
        Vec3::new(0.0, 2.0, 5.0)
    );

    // 创建立方体
    let cube = scene.spawn_prefab(
        ecs_world,
        PrefabType::Cube,
        "my_cube".to_string(),
        Vec3::ZERO
    );

    // 创建光源
    let light = scene.spawn_prefab(
        ecs_world,
        PrefabType::DirectionalLight,
        "sun".to_string(),
        Vec3::new(2.0, 4.0, 2.0)
    );

    Ok(())
}
```

### 输入处理

```rust
use sanji_engine::input::{InputManager, KeyCode};

// 在update中处理输入
fn handle_input(input: &InputManager) {
    if input.is_action_triggered("move_forward") {
        // 向前移动
    }
    
    if input.is_action_just_pressed("jump") {
        // 跳跃
    }
    
    let movement = input.get_vector2d("horizontal", "vertical");
    // 使用movement向量进行移动
}
```

## 🎮 运行演示

项目包含多个演示程序：

```bash
# 简单演示 - 基础引擎功能
cargo run --example simple_demo

# 完整演示 - 展示所有系统功能  
cargo run --example basic_demo
```

## 🛠️ 开发环境

### 系统要求
- Rust 1.70+
- 支持Vulkan、Metal或DirectX 12的显卡
- Windows, macOS, 或 Linux

### 构建项目
```bash
# 克隆仓库
git clone https://github.com/your-username/sanji-engine.git
cd sanji-engine

# 构建项目
cargo build --release

# 运行测试
cargo test

# 运行演示
cargo run --example simple_demo
```

### 可选功能
```toml
[dependencies]
sanji_engine = { version = "0.1.0", features = ["physics", "audio"] }
```

- `physics` - 启用物理引擎支持 (Rapier3D)
- `audio` - 启用音频系统支持 (Rodio)

## 📚 文档

- [API文档](https://docs.rs/sanji_engine) - 完整的API参考
- [教程](./docs/tutorial.md) - 分步教程
- [示例](./examples/) - 各种使用示例
- [架构设计](./docs/architecture.md) - 详细的架构说明

## 🤝 贡献

欢迎贡献代码！请查看 [贡献指南](./CONTRIBUTING.md) 了解详情。

### 开发路线图

- [ ] 更多几何体类型支持
- [ ] 阴影渲染
- [ ] 后处理效果
- [ ] 动画系统
- [ ] UI系统
- [ ] 网络系统
- [ ] 脚本支持

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](./LICENSE) 文件了解详情。

## 🙏 致谢

感谢以下开源项目的支持：

- [wgpu](https://github.com/gfx-rs/wgpu) - 现代图形API
- [winit](https://github.com/rust-windowing/winit) - 跨平台窗口管理
- [specs](https://github.com/amethyst/specs) - 实体组件系统
- [glam](https://github.com/bitshifter/glam-rs) - 数学库
- [image](https://github.com/image-rs/image) - 图像处理

---

**Sanji引擎** - 为创造而生 🎮✨
