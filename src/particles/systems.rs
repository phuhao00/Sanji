//! 粒子系统管理

use crate::math::{Vec3, Vec2, Mat4};
use crate::particles::{ParticleEmitter, EmitterId, EmitterConfig, ParticleStats};
use crate::render::RenderSystem;
use crate::ecs::{World, Component, System, Entity};
use specs::{WorldExt, Builder};
use specs::VecStorage;
use std::collections::HashMap;

/// 粒子系统组件
#[derive(Debug, Clone)]
pub struct ParticleSystemComponent {
    pub emitter_id: EmitterId,
    pub auto_destroy: bool,    // 发射器完成后自动销毁
    pub world_space: bool,     // 是否在世界空间中模拟
}

impl Component for ParticleSystemComponent {
    type Storage = VecStorage<Self>;
}

/// 变换组件（用于粒子系统位置）
#[derive(Debug, Clone)]
pub struct TransformComponent {
    pub position: Vec3,
    pub rotation: crate::math::Quat,
    pub scale: Vec3,
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: crate::math::Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Component for TransformComponent {
    type Storage = VecStorage<Self>;
}

/// 粒子系统更新系统
pub struct ParticleUpdateSystem {
    particle_manager: crate::particles::ParticleSystemManager,
}

impl ParticleUpdateSystem {
    pub fn new(max_particles: usize) -> Self {
        Self {
            particle_manager: crate::particles::ParticleSystemManager::new(max_particles),
        }
    }

    /// 创建粒子发射器
    pub fn create_emitter(&mut self, config: EmitterConfig) -> EmitterId {
        self.particle_manager.create_emitter(config)
    }

    /// 移除粒子发射器
    pub fn remove_emitter(&mut self, id: EmitterId) -> bool {
        self.particle_manager.remove_emitter(id)
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> ParticleStats {
        self.particle_manager.get_stats()
    }

    /// 清除所有粒子
    pub fn clear_all_particles(&mut self) {
        self.particle_manager.clear_all_particles();
    }
}

impl<'a> System<'a> for ParticleUpdateSystem {
    type SystemData = ();
    
    fn run(&mut self, _: Self::SystemData) {
        // Note: 在实际的specs系统中，这个方法会被正确调用
        // 这里保持简化的实现
        // 简化的更新逻辑
        self.particle_manager.update(1.0 / 60.0);
    }
}

/// 粒子渲染系统
pub struct ParticleRenderSystem;

impl ParticleRenderSystem {
    pub fn new() -> Self {
        Self
    }
}

impl<'a> System<'a> for ParticleRenderSystem {
    type SystemData = ();
    
    fn run(&mut self, _: Self::SystemData) {
        // 渲染逻辑在render方法中实现
    }
}

impl ParticleRenderSystem {
    /// 渲染粒子系统
    pub fn render(&self, particle_system: &ParticleUpdateSystem, render_system: &mut RenderSystem) {
        particle_system.particle_manager.render(render_system);
    }
}

/// 粒子系统工厂
pub struct ParticleSystemFactory;

impl ParticleSystemFactory {
    /// 创建简单的粒子系统实体
    pub fn create_particle_system(
        world: &mut World,
        particle_system: &mut ParticleUpdateSystem,
        config: EmitterConfig,
        position: Vec3,
        auto_destroy: bool,
    ) -> Entity {
        let emitter_id = particle_system.create_emitter(config);
        let entity = world.create_entity().build();

        // Note: In specs, components are added via EntityBuilder, not after creation
        // This needs architectural changes to work properly
        // For now, return the entity without components

        // 启动发射器
        particle_system.particle_manager.start_emitter(emitter_id);

        entity
    }

    /// 创建火焰效果
    pub fn create_fire_effect(
        world: &mut World,
        particle_system: &mut ParticleUpdateSystem,
        position: Vec3,
    ) -> Entity {
        Self::create_particle_system(
            world,
            particle_system,
            crate::particles::ParticlePresets::fire(),
            position,
            false,
        )
    }

    /// 创建爆炸效果
    pub fn create_explosion_effect(
        world: &mut World,
        particle_system: &mut ParticleUpdateSystem,
        position: Vec3,
    ) -> Entity {
        Self::create_particle_system(
            world,
            particle_system,
            crate::particles::ParticlePresets::explosion(),
            position,
            true, // 爆炸效果播放完后自动销毁
        )
    }

    /// 创建治疗效果
    pub fn create_healing_effect(
        world: &mut World,
        particle_system: &mut ParticleUpdateSystem,
        position: Vec3,
    ) -> Entity {
        Self::create_particle_system(
            world,
            particle_system,
            crate::particles::ParticlePresets::healing(),
            position,
            false,
        )
    }
}

/// 粒子系统帮助工具
pub struct ParticleSystemHelper;

impl ParticleSystemHelper {
    /// 在指定位置播放一次性特效
    pub fn play_one_shot_effect(
        world: &mut World,
        particle_system: &mut ParticleUpdateSystem,
        effect_name: &str,
        position: Vec3,
    ) -> Option<Entity> {
        let config = match effect_name {
            "fire" => crate::particles::ParticlePresets::fire(),
            "explosion" => crate::particles::ParticlePresets::explosion(),
            "smoke" => crate::particles::ParticlePresets::smoke(),
            "healing" => crate::particles::ParticlePresets::healing(),
            "magic_orb" => crate::particles::ParticlePresets::magic_orb(),
            _ => return None,
        };

        Some(ParticleSystemFactory::create_particle_system(
            world,
            particle_system,
            config,
            position,
            true, // 一次性特效播放完后自动销毁
        ))
    }

    /// 创建持续性环境特效
    pub fn create_ambient_effect(
        world: &mut World,
        particle_system: &mut ParticleUpdateSystem,
        effect_name: &str,
        position: Vec3,
    ) -> Option<Entity> {
        let config = match effect_name {
            "snow" => crate::particles::ParticlePresets::snow(),
            "rain" => crate::particles::ParticlePresets::rain(),
            "fire" => crate::particles::ParticlePresets::fire(),
            _ => return None,
        };

        Some(ParticleSystemFactory::create_particle_system(
            world,
            particle_system,
            config,
            position,
            false, // 环境特效不自动销毁
        ))
    }

    /// 停止指定实体的粒子发射
    pub fn stop_particle_emission(
        world: &World,
        particle_system: &mut ParticleUpdateSystem,
        entity: Entity,
    ) {
        if let Some(particle_comp) = None::<&ParticleSystemComponent> { // TODO: Fix specs component access
            particle_system.particle_manager.stop_emitter(particle_comp.emitter_id);
        }
    }

    /// 暂停指定实体的粒子发射
    pub fn pause_particle_emission(
        world: &World,
        particle_system: &mut ParticleUpdateSystem,
        entity: Entity,
    ) {
        if let Some(particle_comp) = None::<&ParticleSystemComponent> { // TODO: Fix specs component access
            particle_system.particle_manager.pause_emitter(particle_comp.emitter_id);
        }
    }

    /// 恢复指定实体的粒子发射
    pub fn resume_particle_emission(
        world: &World,
        particle_system: &mut ParticleUpdateSystem,
        entity: Entity,
    ) {
        if let Some(particle_comp) = None::<&ParticleSystemComponent> { // TODO: Fix specs component access
            particle_system.particle_manager.start_emitter(particle_comp.emitter_id);
        }
    }

    /// 获取粒子系统性能统计
    pub fn get_performance_stats(particle_system: &ParticleUpdateSystem) -> ParticlePerformanceStats {
        let stats = particle_system.get_stats();
        
        ParticlePerformanceStats {
            total_emitters: stats.total_emitters,
            active_emitters: stats.active_emitters,
            total_particles: stats.total_particles,
            max_particles: stats.max_particles,
            memory_usage_mb: (stats.total_particles * std::mem::size_of::<crate::particles::Particle>()) as f32 / (1024.0 * 1024.0),
            particle_usage_ratio: stats.particle_usage_ratio(),
        }
    }
}

/// 粒子系统性能统计
#[derive(Debug, Clone)]
pub struct ParticlePerformanceStats {
    pub total_emitters: usize,
    pub active_emitters: usize,
    pub total_particles: usize,
    pub max_particles: usize,
    pub memory_usage_mb: f32,
    pub particle_usage_ratio: f32,
}

/// 粒子系统LOD（细节层次）管理器
pub struct ParticleLODManager {
    camera_position: Vec3,
    lod_distances: Vec<f32>, // LOD距离阈值
}

impl ParticleLODManager {
    pub fn new(lod_distances: Vec<f32>) -> Self {
        Self {
            camera_position: Vec3::ZERO,
            lod_distances,
        }
    }

    /// 设置摄像机位置
    pub fn set_camera_position(&mut self, position: Vec3) {
        self.camera_position = position;
    }

    /// 计算LOD等级
    pub fn calculate_lod_level(&self, particle_position: Vec3) -> usize {
        let distance = (particle_position - self.camera_position).length();
        
        for (i, &lod_distance) in self.lod_distances.iter().enumerate() {
            if distance <= lod_distance {
                return i;
            }
        }
        
        self.lod_distances.len() // 最低LOD等级
    }

    /// 根据LOD等级调整发射器配置
    pub fn adjust_emitter_config(&self, config: &mut EmitterConfig, lod_level: usize) {
        match lod_level {
            0 => {
                // 高质量LOD - 不调整
            }
            1 => {
                // 中等质量LOD - 减少50%粒子
                config.max_particles = (config.max_particles as f32 * 0.5) as usize;
                config.emission_rate *= 0.5;
            }
            2 => {
                // 低质量LOD - 减少75%粒子
                config.max_particles = (config.max_particles as f32 * 0.25) as usize;
                config.emission_rate *= 0.25;
            }
            _ => {
                // 最低质量LOD - 禁用发射器
                config.max_particles = 0;
                config.emission_rate = 0.0;
            }
        }
    }
}

/// 粒子系统配置管理器
pub struct ParticleConfigManager {
    configs: HashMap<String, EmitterConfig>,
}

impl ParticleConfigManager {
    pub fn new() -> Self {
        let mut manager = Self {
            configs: HashMap::new(),
        };
        
        // 加载预设配置
        manager.load_presets();
        manager
    }

    /// 加载预设配置
    fn load_presets(&mut self) {
        self.configs.insert("fire".to_string(), crate::particles::ParticlePresets::fire());
        self.configs.insert("smoke".to_string(), crate::particles::ParticlePresets::smoke());
        self.configs.insert("explosion".to_string(), crate::particles::ParticlePresets::explosion());
        self.configs.insert("snow".to_string(), crate::particles::ParticlePresets::snow());
        self.configs.insert("rain".to_string(), crate::particles::ParticlePresets::rain());
        self.configs.insert("healing".to_string(), crate::particles::ParticlePresets::healing());
        self.configs.insert("magic_orb".to_string(), crate::particles::ParticlePresets::magic_orb());
    }

    /// 注册配置
    pub fn register_config(&mut self, name: String, config: EmitterConfig) {
        self.configs.insert(name, config);
    }

    /// 获取配置
    pub fn get_config(&self, name: &str) -> Option<&EmitterConfig> {
        self.configs.get(name)
    }

    /// 获取配置副本
    pub fn get_config_copy(&self, name: &str) -> Option<EmitterConfig> {
        self.configs.get(name).cloned()
    }

    /// 列出所有配置名称
    pub fn list_configs(&self) -> Vec<&String> {
        self.configs.keys().collect()
    }

    /// 移除配置
    pub fn remove_config(&mut self, name: &str) -> bool {
        self.configs.remove(name).is_some()
    }
}

impl Default for ParticleConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
