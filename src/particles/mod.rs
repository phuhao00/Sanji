//! 粒子系统

pub mod particle;
pub mod emitter;
pub mod systems;
pub mod effects;

pub use particle::{Particle, ParticleState};
pub use emitter::{ParticleEmitter, EmitterId, EmitterConfig, EmissionShape, BlendMode as EmitterBlendMode, SizeOverLifetime, VelocityOverLifetime, ColorOverLifetime, SimulationSpace};
pub use systems::*;
pub use effects::*;

use crate::math::{Vec3, Vec2};
use crate::render::RenderSystem;
use std::collections::HashMap;

/// 粒子系统管理器
pub struct ParticleSystemManager {
    emitters: HashMap<EmitterId, ParticleEmitter>,
    next_id: EmitterId,
    max_particles: usize,
    current_particle_count: usize,
}

impl ParticleSystemManager {
    pub fn new(max_particles: usize) -> Self {
        Self {
            emitters: HashMap::new(),
            next_id: 1,
            max_particles,
            current_particle_count: 0,
        }
    }

    /// 创建粒子发射器
    pub fn create_emitter(&mut self, config: EmitterConfig) -> EmitterId {
        let id = self.next_id;
        self.next_id += 1;

        let emitter = ParticleEmitter::new(id, config);
        self.emitters.insert(id, emitter);
        id
    }

    /// 移除粒子发射器
    pub fn remove_emitter(&mut self, id: EmitterId) -> bool {
        if let Some(emitter) = self.emitters.remove(&id) {
            self.current_particle_count -= emitter.particles.len();
            true
        } else {
            false
        }
    }

    /// 获取发射器
    pub fn get_emitter(&self, id: EmitterId) -> Option<&ParticleEmitter> {
        self.emitters.get(&id)
    }

    /// 获取可变发射器
    pub fn get_emitter_mut(&mut self, id: EmitterId) -> Option<&mut ParticleEmitter> {
        self.emitters.get_mut(&id)
    }

    /// 设置发射器位置
    pub fn set_emitter_position(&mut self, id: EmitterId, position: Vec3) {
        if let Some(emitter) = self.emitters.get_mut(&id) {
            emitter.set_position(position);
        }
    }

    /// 启动发射器
    pub fn start_emitter(&mut self, id: EmitterId) {
        if let Some(emitter) = self.emitters.get_mut(&id) {
            emitter.start();
        }
    }

    /// 停止发射器
    pub fn stop_emitter(&mut self, id: EmitterId) {
        if let Some(emitter) = self.emitters.get_mut(&id) {
            emitter.stop();
        }
    }

    /// 暂停发射器
    pub fn pause_emitter(&mut self, id: EmitterId) {
        if let Some(emitter) = self.emitters.get_mut(&id) {
            emitter.pause();
        }
    }

    /// 更新所有粒子系统
    pub fn update(&mut self, delta_time: f32) {
        self.current_particle_count = 0;

        for emitter in self.emitters.values_mut() {
            emitter.update(delta_time, self.max_particles - self.current_particle_count);
            self.current_particle_count += emitter.particles.len();
        }
    }

    /// 渲染所有粒子
    pub fn render(&self, render_system: &mut RenderSystem) {
        for emitter in self.emitters.values() {
            emitter.render(render_system);
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> ParticleStats {
        let mut stats = ParticleStats::default();
        
        for emitter in self.emitters.values() {
            stats.total_emitters += 1;
            stats.total_particles += emitter.particles.len();
            
            if emitter.is_active() {
                stats.active_emitters += 1;
            }
        }
        
        stats.max_particles = self.max_particles;
        stats
    }

    /// 清除所有死亡粒子
    pub fn cleanup_dead_particles(&mut self) {
        for emitter in self.emitters.values_mut() {
            emitter.cleanup_dead_particles();
        }
    }

    /// 清除所有粒子
    pub fn clear_all_particles(&mut self) {
        for emitter in self.emitters.values_mut() {
            emitter.clear_particles();
        }
        self.current_particle_count = 0;
    }

    /// 获取所有发射器ID
    pub fn get_emitter_ids(&self) -> Vec<EmitterId> {
        self.emitters.keys().copied().collect()
    }

    /// 批量更新发射器
    pub fn batch_update_emitters<F>(&mut self, mut updater: F)
    where
        F: FnMut(&mut ParticleEmitter),
    {
        for emitter in self.emitters.values_mut() {
            updater(emitter);
        }
    }
}

/// 粒子统计信息
#[derive(Debug, Default, Clone)]
pub struct ParticleStats {
    pub total_emitters: usize,
    pub active_emitters: usize,
    pub total_particles: usize,
    pub max_particles: usize,
}

impl ParticleStats {
    /// 获取粒子使用率
    pub fn particle_usage_ratio(&self) -> f32 {
        if self.max_particles == 0 {
            0.0
        } else {
            self.total_particles as f32 / self.max_particles as f32
        }
    }

    /// 是否接近粒子上限
    pub fn is_near_limit(&self, threshold: f32) -> bool {
        self.particle_usage_ratio() >= threshold
    }
}

/// 粒子系统预设效果
pub struct ParticlePresets;

impl ParticlePresets {
    /// 火焰效果
    pub fn fire() -> EmitterConfig {
        EmitterConfig {
            max_particles: 200,
            emission_rate: 50.0,
            burst_count: 0,
            lifetime: 2.0,
            start_lifetime_range: (1.5, 2.5),
            start_speed_range: (2.0, 4.0),
            start_size_range: (0.1, 0.3),
            start_color: [1.0, 0.4, 0.0, 1.0],
            end_color: [1.0, 0.0, 0.0, 0.0],
            gravity: Vec3::new(0.0, 1.0, 0.0),
            shape: EmissionShape::Circle { radius: 0.5 },
            texture_path: Some("assets/textures/fire_particle.png".to_string()),
            blend_mode: EmitterBlendMode::Additive,
            size_over_lifetime: Some(SizeOverLifetime::new(vec![
                (0.0, 0.5),
                (0.3, 1.0),
                (1.0, 0.0),
            ])),
            velocity_over_lifetime: Some(VelocityOverLifetime::new(vec![
                (0.0, Vec3::new(0.0, 3.0, 0.0)),
                (1.0, Vec3::new(0.0, 1.0, 0.0)),
            ])),
            color_over_lifetime: Some(ColorOverLifetime::new(vec![
                (0.0, [1.0, 0.8, 0.0, 1.0]),
                (0.5, [1.0, 0.4, 0.0, 0.8]),
                (1.0, [0.5, 0.0, 0.0, 0.0]),
            ])),
            simulation_space: SimulationSpace::World,
            sorting_layer: 0,
            order_in_layer: 0,
        }
    }

    /// 烟雾效果
    pub fn smoke() -> EmitterConfig {
        EmitterConfig {
            max_particles: 150,
            emission_rate: 30.0,
            burst_count: 0,
            lifetime: 3.0,
            start_lifetime_range: (2.5, 3.5),
            start_speed_range: (0.5, 1.5),
            start_size_range: (0.2, 0.5),
            start_color: [0.5, 0.5, 0.5, 0.8],
            end_color: [0.3, 0.3, 0.3, 0.0],
            gravity: Vec3::new(0.0, -0.5, 0.0),
            shape: EmissionShape::Circle { radius: 0.3 },
            texture_path: Some("assets/textures/smoke_particle.png".to_string()),
            blend_mode: EmitterBlendMode::Alpha,
            size_over_lifetime: Some(SizeOverLifetime::new(vec![
                (0.0, 0.3),
                (0.5, 1.0),
                (1.0, 1.5),
            ])),
            velocity_over_lifetime: None,
            color_over_lifetime: Some(ColorOverLifetime::new(vec![
                (0.0, [0.8, 0.8, 0.8, 0.8]),
                (0.7, [0.5, 0.5, 0.5, 0.4]),
                (1.0, [0.3, 0.3, 0.3, 0.0]),
            ])),
            simulation_space: SimulationSpace::World,
            sorting_layer: 0,
            order_in_layer: -1,
        }
    }

    /// 爆炸效果
    pub fn explosion() -> EmitterConfig {
        EmitterConfig {
            max_particles: 100,
            emission_rate: 0.0,
            burst_count: 100,
            lifetime: 1.5,
            start_lifetime_range: (1.0, 2.0),
            start_speed_range: (5.0, 10.0),
            start_size_range: (0.1, 0.2),
            start_color: [1.0, 1.0, 0.5, 1.0],
            end_color: [1.0, 0.0, 0.0, 0.0],
            gravity: Vec3::new(0.0, -2.0, 0.0),
            shape: EmissionShape::Sphere { radius: 0.1 },
            texture_path: Some("assets/textures/spark_particle.png".to_string()),
            blend_mode: EmitterBlendMode::Additive,
            size_over_lifetime: Some(SizeOverLifetime::new(vec![
                (0.0, 1.0),
                (1.0, 0.0),
            ])),
            velocity_over_lifetime: Some(VelocityOverLifetime::new(vec![
                (0.0, Vec3::ZERO),
                (0.5, Vec3::new(0.0, -5.0, 0.0)),
                (1.0, Vec3::new(0.0, -10.0, 0.0)),
            ])),
            color_over_lifetime: Some(ColorOverLifetime::new(vec![
                (0.0, [1.0, 1.0, 0.8, 1.0]),
                (0.3, [1.0, 0.5, 0.0, 0.8]),
                (1.0, [0.5, 0.0, 0.0, 0.0]),
            ])),
            simulation_space: SimulationSpace::World,
            sorting_layer: 1,
            order_in_layer: 0,
        }
    }

    /// 雪花效果
    pub fn snow() -> EmitterConfig {
        EmitterConfig {
            max_particles: 300,
            emission_rate: 20.0,
            burst_count: 0,
            lifetime: 10.0,
            start_lifetime_range: (8.0, 12.0),
            start_speed_range: (1.0, 2.0),
            start_size_range: (0.05, 0.1),
            start_color: [1.0, 1.0, 1.0, 0.8],
            end_color: [1.0, 1.0, 1.0, 0.8],
            gravity: Vec3::new(0.0, -1.0, 0.0),
            shape: EmissionShape::Box { 
                size: Vec3::new(20.0, 1.0, 20.0) 
            },
            texture_path: Some("assets/textures/snowflake_particle.png".to_string()),
            blend_mode: EmitterBlendMode::Alpha,
            size_over_lifetime: None,
            velocity_over_lifetime: Some(VelocityOverLifetime::new(vec![
                (0.0, Vec3::new(0.0, -1.0, 0.0)),
                (1.0, Vec3::new(0.5, -1.5, 0.0)),
            ])),
            color_over_lifetime: None,
            simulation_space: SimulationSpace::World,
            sorting_layer: 0,
            order_in_layer: 0,
        }
    }

    /// 魔法光球效果
    pub fn magic_orb() -> EmitterConfig {
        EmitterConfig {
            max_particles: 50,
            emission_rate: 25.0,
            burst_count: 0,
            lifetime: 2.0,
            start_lifetime_range: (1.5, 2.5),
            start_speed_range: (1.0, 2.0),
            start_size_range: (0.05, 0.15),
            start_color: [0.3, 0.7, 1.0, 1.0],
            end_color: [0.0, 0.3, 1.0, 0.0],
            gravity: Vec3::ZERO,
            shape: EmissionShape::Circle { radius: 1.0 },
            texture_path: Some("assets/textures/magic_particle.png".to_string()),
            blend_mode: EmitterBlendMode::Additive,
            size_over_lifetime: Some(SizeOverLifetime::new(vec![
                (0.0, 0.2),
                (0.5, 1.0),
                (1.0, 0.0),
            ])),
            velocity_over_lifetime: Some(VelocityOverLifetime::new(vec![
                (0.0, Vec3::ZERO),
                (1.0, Vec3::new(0.0, 2.0, 0.0)),
            ])),
            color_over_lifetime: Some(ColorOverLifetime::new(vec![
                (0.0, [0.5, 0.8, 1.0, 1.0]),
                (0.5, [0.3, 0.7, 1.0, 0.8]),
                (1.0, [0.0, 0.3, 1.0, 0.0]),
            ])),
            simulation_space: SimulationSpace::World,
            sorting_layer: 1,
            order_in_layer: 1,
        }
    }

    /// 雨滴效果
    pub fn rain() -> EmitterConfig {
        EmitterConfig {
            max_particles: 500,
            emission_rate: 100.0,
            burst_count: 0,
            lifetime: 3.0,
            start_lifetime_range: (2.5, 3.5),
            start_speed_range: (8.0, 12.0),
            start_size_range: (0.02, 0.05),
            start_color: [0.7, 0.8, 1.0, 0.6],
            end_color: [0.7, 0.8, 1.0, 0.6],
            gravity: Vec3::new(0.0, -15.0, 0.0),
            shape: EmissionShape::Box { 
                size: Vec3::new(50.0, 1.0, 50.0) 
            },
            texture_path: Some("assets/textures/rain_particle.png".to_string()),
            blend_mode: EmitterBlendMode::Alpha,
            size_over_lifetime: None,
            velocity_over_lifetime: None,
            color_over_lifetime: None,
            simulation_space: SimulationSpace::World,
            sorting_layer: -1,
            order_in_layer: 0,
        }
    }

    /// 治疗效果
    pub fn healing() -> EmitterConfig {
        EmitterConfig {
            max_particles: 80,
            emission_rate: 40.0,
            burst_count: 0,
            lifetime: 2.5,
            start_lifetime_range: (2.0, 3.0),
            start_speed_range: (1.0, 2.0),
            start_size_range: (0.1, 0.2),
            start_color: [0.3, 1.0, 0.3, 0.8],
            end_color: [0.8, 1.0, 0.8, 0.0],
            gravity: Vec3::new(0.0, -0.5, 0.0),
            shape: EmissionShape::Circle { radius: 0.5 },
            texture_path: Some("assets/textures/healing_particle.png".to_string()),
            blend_mode: EmitterBlendMode::Additive,
            size_over_lifetime: Some(SizeOverLifetime::new(vec![
                (0.0, 0.5),
                (0.3, 1.0),
                (1.0, 0.2),
            ])),
            velocity_over_lifetime: Some(VelocityOverLifetime::new(vec![
                (0.0, Vec3::new(0.0, 2.0, 0.0)),
                (1.0, Vec3::new(0.0, 1.0, 0.0)),
            ])),
            color_over_lifetime: Some(ColorOverLifetime::new(vec![
                (0.0, [0.5, 1.0, 0.5, 0.8]),
                (0.5, [0.3, 1.0, 0.3, 0.6]),
                (1.0, [0.8, 1.0, 0.8, 0.0]),
            ])),
            simulation_space: SimulationSpace::World,
            sorting_layer: 1,
            order_in_layer: 2,
        }
    }
}
