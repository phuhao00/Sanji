# Sanji Game Engine

[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Sanji是一个现代化的、高性能的游戏引擎，完全使用Rust语言构建。它提供了完整的游戏开发工具链，包括渲染、物理、音频、动画、UI和性能分析等系统。

## ✨ 主要特性

### 🎨 现代渲染系统
- **基于wgpu的跨平台渲染** - 支持Vulkan、Metal、DX12和WebGPU
- **物理基渲染（PBR）** - 真实感材质和光照
- **高级阴影系统** - 支持Shadow Mapping和Cascaded Shadow Maps
- **后处理效果** - Bloom、色调映射、FXAA、色彩分级等
- **多种光源类型** - 定向光、点光源、聚光灯
- **高效的GPU剔除** - 视锥体剔除和遮挡剔除

### 🏗️ 高性能ECS架构
- **实体组件系统（ECS）** - 基于specs crate的高性能ECS
- **组件化设计** - 灵活的组件组合系统
- **系统并行执行** - 多线程系统调度
- **内存友好** - 缓存友好的数据布局

### ⚡ 集成物理引擎
- **3D物理模拟** - 基于Rapier3D的高性能物理
- **多种碰撞体** - 盒子、球体、胶囊体、网格碰撞
- **刚体动力学** - 支持静态、运动学和动态刚体
- **物理材质** - 摩擦力、弹性、阻尼等物理属性
- **射线投射** - 高效的射线查询系统

### 🔊 3D空间音频
- **3D音效系统** - 基于Rodio的空间音频
- **多种音频格式** - 支持WAV、MP3、OGG、FLAC
- **音频监听器** - 动态音频源和监听器
- **多普勒效应** - 真实的音频体验
- **音频过滤** - 低通、高通、带通滤波器

### 🎬 动画系统
- **关键帧动画** - 位置、旋转、缩放插值
- **多种插值方法** - 线性、贝塞尔、样条插值
- **动画混合** - 多个动画的平滑混合
- **动画状态机** - 复杂动画逻辑控制
- **骨骼动画** - 支持骨骼权重和蒙皮

### 🖥️ 现代UI系统
- **即时模式GUI** - 高效的UI渲染
- **丰富的控件** - 按钮、文本、面板、滑块等
- **布局管理** - 灵活的布局系统
- **事件系统** - 完整的UI事件处理
- **样式系统** - 可定制的UI外观

### ✨ 粒子系统
- **GPU粒子** - 高性能粒子模拟
- **多种发射器** - 点、线、面、体积发射器
- **丰富的效果** - 火焰、烟雾、爆炸、魔法等
- **物理交互** - 粒子与物理世界的交互
- **纹理动画** - 支持精灵序列动画

### 📊 性能分析工具
- **实时性能监控** - FPS、帧时间、CPU、内存使用
- **详细性能分析** - 函数级别的性能剖析
- **内存追踪** - 内存分配、泄漏检测
- **可视化调试** - 性能图表和热力图
- **热重载系统** - 资源实时重载

### 💾 序列化系统
- **多种格式支持** - JSON、二进制、MessagePack、YAML
- **场景序列化** - 完整场景的保存和加载
- **预制件系统** - 可复用的游戏对象模板
- **资源打包** - 高效的资源管理和分发
- **增量序列化** - 智能的差异化保存

## 🚀 快速开始

### 安装依赖

确保你已经安装了Rust（1.70+）：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 创建新项目

```bash
cargo new my_game
cd my_game
```

### 添加依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
sanji_engine = "0.1.0"
```

### 基础示例

```rust
use sanji_engine::*;

fn main() -> EngineResult<()> {
    // 创建引擎实例
    let mut engine = Engine::new("我的游戏", 1280, 720)?;
    
    // 创建场景
    let mut scene = Scene::new("主场景".to_string());
    
    // 创建一个立方体
    let cube = engine.world.create_entity();
    engine.world.add_component(cube, TransformComponent::default());
    engine.world.add_component(cube, RenderComponent {
        mesh_path: "meshes/cube.obj".to_string(),
        material_path: "materials/default.mat".to_string(),
        ..Default::default()
    });
    
    scene.add_entity(cube);
    engine.load_scene(scene)?;
    
    // 主循环
    engine.run(|engine, delta_time| {
        // 游戏逻辑更新
        Ok(())
    })?;
    
    Ok(())
}
```

## 📚 文档和示例

### 运行示例

引擎包含了多个演示示例：

```bash
# 简单演示
cargo run --example simple_demo

# 基础功能演示
cargo run --example basic_demo

# 综合功能演示
cargo run --example comprehensive_demo
```

### 系统架构

```
Sanji Engine
├── Core (核心系统)
│   ├── Engine - 引擎主类
│   ├── Scene - 场景管理
│   └── Resource - 资源管理
├── ECS (实体组件系统)
│   ├── World - ECS世界
│   ├── Entity - 实体管理
│   ├── Component - 组件系统
│   └── System - 系统调度
├── Render (渲染系统)
│   ├── Renderer - 渲染器
│   ├── Mesh - 网格处理
│   ├── Material - 材质系统
│   ├── Shader - 着色器管理
│   ├── Texture - 纹理系统
│   ├── Light - 光照系统
│   ├── Shadow - 阴影渲染
│   └── PostProcess - 后处理
├── Physics (物理系统)
│   ├── World - 物理世界
│   ├── RigidBody - 刚体
│   ├── Collider - 碰撞体
│   └── RayCast - 射线投射
├── Audio (音频系统)
│   ├── AudioEngine - 音频引擎
│   ├── AudioSource - 音频源
│   ├── AudioListener - 音频监听器
│   └── AudioEffects - 音频效果
├── Animation (动画系统)
│   ├── AnimationClip - 动画片段
│   ├── Animator - 动画器
│   ├── Keyframe - 关键帧
│   └── Skeleton - 骨骼系统
├── UI (用户界面)
│   ├── UISystem - UI系统
│   ├── Widgets - UI控件
│   ├── Layout - 布局管理
│   └── Events - 事件处理
├── Particles (粒子系统)
│   ├── ParticleSystem - 粒子系统
│   ├── Emitter - 发射器
│   └── Effects - 粒子效果
├── Performance (性能分析)
│   ├── Profiler - 性能分析器
│   ├── MemoryTracker - 内存追踪
│   ├── FrameAnalyzer - 帧分析器
│   └── Debugger - 调试器
└── Serialization (序列化)
    ├── SceneSerializer - 场景序列化
    ├── AssetSerializer - 资源序列化
    └── ComponentSerializer - 组件序列化
```

## 🛠️ 系统要求

### 最低配置
- **操作系统**: Windows 10, macOS 10.15, Ubuntu 18.04+
- **GPU**: 支持Vulkan 1.1, Metal 2.0, 或 DirectX 12
- **内存**: 4 GB RAM
- **存储**: 1 GB 可用空间

### 推荐配置
- **操作系统**: Windows 11, macOS 12+, Ubuntu 20.04+
- **GPU**: 现代显卡（GTX 1060 / RX 580或更高）
- **内存**: 8 GB RAM
- **存储**: SSD存储

## 🔧 编译和构建

### 开发版本
```bash
git clone https://github.com/your-repo/sanji-engine
cd sanji-engine
cargo build
```

### 发布版本
```bash
cargo build --release
```

### 特性标志
```bash
# 仅渲染系统（无物理和音频）
cargo build --no-default-features

# 启用所有功能
cargo build --features "physics,audio"

# 调试模式
cargo build --features "debug"
```

## 📖 API文档

详细的API文档可以通过以下命令生成：

```bash
cargo doc --open
```

## 🤝 贡献指南

我们欢迎所有形式的贡献！请阅读 [CONTRIBUTING.md](CONTRIBUTING.md) 了解如何参与项目开发。

### 开发流程
1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### 代码规范
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 添加适当的文档注释
- 确保所有测试通过

## 🧪 测试

运行测试套件：

```bash
# 运行所有测试
cargo test

# 运行基准测试
cargo bench

# 运行集成测试
cargo test --test integration
```

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [wgpu](https://github.com/gfx-rs/wgpu) - 现代图形API
- [specs](https://github.com/amethyst/specs) - ECS框架
- [rapier](https://github.com/dimforge/rapier) - 物理引擎
- [rodio](https://github.com/RustAudio/rodio) - 音频库
- [glam](https://github.com/bitshifter/glam-rs) - 数学库

## 📞 联系我们

- 项目主页: [https://github.com/your-repo/sanji-engine](https://github.com/your-repo/sanji-engine)
- 问题反馈: [GitHub Issues](https://github.com/your-repo/sanji-engine/issues)
- 讨论区: [GitHub Discussions](https://github.com/your-repo/sanji-engine/discussions)

## 🗺️ 路线图

### v0.2.0 (即将发布)
- [ ] 网络系统
- [ ] 脚本系统 (Lua/Python)
- [ ] 地形渲染
- [ ] 水体渲染

### v0.3.0
- [ ] VR/AR支持
- [ ] 移动平台支持
- [ ] 可视化编辑器
- [ ] 资源热重载

### v1.0.0
- [ ] 稳定API
- [ ] 完整文档
- [ ] 性能优化
- [ ] 生产就绪

---

**Sanji Engine** - 让游戏开发变得简单而强大 🎮