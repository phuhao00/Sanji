//! 粒子特效系统

use crate::math::{Vec3, Vec2, Quat};
use crate::particles::{EmitterConfig, EmitterId, ParticleSystemManager, EmissionShape, EmitterBlendMode, SizeOverLifetime, VelocityOverLifetime, ColorOverLifetime, SimulationSpace};
use crate::ecs::{World, Entity};
use crate::audio::AudioSystem;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 特效事件类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EffectEvent {
    Start,      // 开始播放
    Stop,       // 停止播放
    Pause,      // 暂停播放
    Resume,     // 恢复播放
    Complete,   // 播放完成
}

/// 特效数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectData {
    pub name: String,
    pub duration: f32,              // 特效持续时间
    pub loops: bool,                // 是否循环播放
    pub emitters: Vec<EmitterConfig>, // 粒子发射器配置
    pub audio_clips: Vec<String>,   // 音效文件路径
    pub screen_shake: Option<ScreenShakeData>, // 屏幕震动
    pub light_flash: Option<LightFlashData>,   // 光闪效果
    pub time_dilation: Option<TimeDilationData>, // 时间膨胀效果
}

/// 屏幕震动数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenShakeData {
    pub intensity: f32,     // 震动强度
    pub duration: f32,      // 震动时长
    pub frequency: f32,     // 震动频率
    pub fade_out: bool,     // 是否渐弱
}

/// 光闪效果数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightFlashData {
    pub color: [f32; 3],    // 闪光颜色 (RGB)
    pub intensity: f32,     // 强度
    pub duration: f32,      // 持续时间
    pub fade_curve: Vec<(f32, f32)>, // 衰减曲线 (时间, 强度)
}

/// 时间膨胀效果数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeDilationData {
    pub time_scale: f32,    // 时间缩放比例
    pub duration: f32,      // 持续时间
    pub ease_in: f32,       // 渐入时间
    pub ease_out: f32,      // 渐出时间
}

/// 特效实例
pub struct EffectInstance {
    pub id: EffectInstanceId,
    pub effect_data: EffectData,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    
    // 运行时状态
    pub is_playing: bool,
    pub current_time: f32,
    pub emitter_ids: Vec<EmitterId>,
    pub entity: Option<Entity>,
    
    // 事件回调
    pub on_complete: Option<Box<dyn Fn() + Send + Sync>>,
}

pub type EffectInstanceId = u64;

/// 特效系统管理器
pub struct EffectSystemManager {
    effects: HashMap<String, EffectData>,
    instances: HashMap<EffectInstanceId, EffectInstance>,
    next_instance_id: EffectInstanceId,
    particle_manager: ParticleSystemManager,
}

impl EffectSystemManager {
    pub fn new(max_particles: usize) -> Self {
        let mut manager = Self {
            effects: HashMap::new(),
            instances: HashMap::new(),
            next_instance_id: 1,
            particle_manager: ParticleSystemManager::new(max_particles),
        };
        
        // 加载预设特效
        manager.load_preset_effects();
        manager
    }

    /// 加载预设特效
    fn load_preset_effects(&mut self) {
        // 爆炸特效
        self.register_effect("explosion".to_string(), EffectData {
            name: "explosion".to_string(),
            duration: 2.0,
            loops: false,
            emitters: vec![
                crate::particles::ParticlePresets::explosion(),
                // 添加烟雾效果
                {
                    let mut smoke = crate::particles::ParticlePresets::smoke();
                    smoke.emission_rate = 50.0;
                    smoke.start_lifetime_range = (3.0, 4.0);
                    smoke
                }
            ],
            audio_clips: vec!["assets/audio/explosion.wav".to_string()],
            screen_shake: Some(ScreenShakeData {
                intensity: 0.5,
                duration: 0.8,
                frequency: 30.0,
                fade_out: true,
            }),
            light_flash: Some(LightFlashData {
                color: [1.0, 0.8, 0.4],
                intensity: 2.0,
                duration: 0.3,
                fade_curve: vec![(0.0, 1.0), (0.1, 0.8), (0.3, 0.0)],
            }),
            time_dilation: Some(TimeDilationData {
                time_scale: 0.3,
                duration: 0.2,
                ease_in: 0.0,
                ease_out: 0.2,
            }),
        });

        // 治疗特效
        self.register_effect("healing".to_string(), EffectData {
            name: "healing".to_string(),
            duration: 3.0,
            loops: false,
            emitters: vec![crate::particles::ParticlePresets::healing()],
            audio_clips: vec!["assets/audio/healing.wav".to_string()],
            screen_shake: None,
            light_flash: Some(LightFlashData {
                color: [0.3, 1.0, 0.3],
                intensity: 0.8,
                duration: 2.0,
                fade_curve: vec![(0.0, 0.0), (0.3, 1.0), (1.0, 0.0)],
            }),
            time_dilation: None,
        });

        // 魔法施法特效
        self.register_effect("magic_cast".to_string(), EffectData {
            name: "magic_cast".to_string(),
            duration: 1.5,
            loops: false,
            emitters: vec![crate::particles::ParticlePresets::magic_orb()],
            audio_clips: vec!["assets/audio/magic_cast.wav".to_string()],
            screen_shake: None,
            light_flash: Some(LightFlashData {
                color: [0.3, 0.7, 1.0],
                intensity: 1.2,
                duration: 1.0,
                fade_curve: vec![(0.0, 0.0), (0.5, 1.0), (1.0, 0.2)],
            }),
            time_dilation: None,
        });

        // 火焰持续特效
        self.register_effect("fire_torch".to_string(), EffectData {
            name: "fire_torch".to_string(),
            duration: 0.0, // 无限持续
            loops: true,
            emitters: vec![
                crate::particles::ParticlePresets::fire(),
                {
                    let mut smoke = crate::particles::ParticlePresets::smoke();
                    smoke.emission_rate = 10.0;
                    smoke.start_lifetime_range = (2.0, 3.0);
                    smoke.start_size_range = (0.1, 0.2);
                    smoke
                }
            ],
            audio_clips: vec!["assets/audio/fire_loop.wav".to_string()],
            screen_shake: None,
            light_flash: Some(LightFlashData {
                color: [1.0, 0.6, 0.2],
                intensity: 0.5,
                duration: 0.0, // 持续光照
                fade_curve: vec![],
            }),
            time_dilation: None,
        });
    }

    /// 注册特效
    pub fn register_effect(&mut self, name: String, effect_data: EffectData) {
        self.effects.insert(name, effect_data);
    }

    /// 播放特效
    pub fn play_effect(
        &mut self, 
        effect_name: &str, 
        position: Vec3, 
        rotation: Option<Quat>,
        scale: Option<Vec3>
    ) -> Option<EffectInstanceId> {
        let effect_data = self.effects.get(effect_name)?.clone();
        let instance_id = self.next_instance_id;
        self.next_instance_id += 1;

        // 创建粒子发射器
        let mut emitter_ids = Vec::new();
        for emitter_config in &effect_data.emitters {
            let emitter_id = self.particle_manager.create_emitter(emitter_config.clone());
            self.particle_manager.set_emitter_position(emitter_id, position);
            self.particle_manager.start_emitter(emitter_id);
            emitter_ids.push(emitter_id);
        }

        let instance = EffectInstance {
            id: instance_id,
            effect_data,
            position,
            rotation: rotation.unwrap_or(Quat::IDENTITY),
            scale: scale.unwrap_or(Vec3::ONE),
            is_playing: true,
            current_time: 0.0,
            emitter_ids,
            entity: None,
            on_complete: None,
        };

        self.instances.insert(instance_id, instance);
        Some(instance_id)
    }

    /// 停止特效
    pub fn stop_effect(&mut self, instance_id: EffectInstanceId) {
        if let Some(instance) = self.instances.get_mut(&instance_id) {
            instance.is_playing = false;
            
            // 停止所有粒子发射器
            for &emitter_id in &instance.emitter_ids {
                self.particle_manager.stop_emitter(emitter_id);
            }
        }
    }

    /// 移除特效实例
    pub fn remove_effect(&mut self, instance_id: EffectInstanceId) {
        if let Some(instance) = self.instances.remove(&instance_id) {
            // 移除所有粒子发射器
            for emitter_id in instance.emitter_ids {
                self.particle_manager.remove_emitter(emitter_id);
            }
        }
    }

    /// 暂停特效
    pub fn pause_effect(&mut self, instance_id: EffectInstanceId) {
        if let Some(instance) = self.instances.get_mut(&instance_id) {
            instance.is_playing = false;
            
            // 暂停所有粒子发射器
            for &emitter_id in &instance.emitter_ids {
                self.particle_manager.pause_emitter(emitter_id);
            }
        }
    }

    /// 恢复特效
    pub fn resume_effect(&mut self, instance_id: EffectInstanceId) {
        if let Some(instance) = self.instances.get_mut(&instance_id) {
            instance.is_playing = true;
            
            // 恢复所有粒子发射器
            for &emitter_id in &instance.emitter_ids {
                self.particle_manager.start_emitter(emitter_id);
            }
        }
    }

    /// 更新特效系统
    pub fn update(&mut self, delta_time: f32) {
        let mut completed_instances = Vec::new();

        // 更新所有特效实例
        for (instance_id, instance) in &mut self.instances {
            if !instance.is_playing {
                continue;
            }

            instance.current_time += delta_time;

            // 检查是否完成
            if instance.effect_data.duration > 0.0 && instance.current_time >= instance.effect_data.duration {
                if instance.effect_data.loops {
                    // 重置时间，继续循环
                    instance.current_time = 0.0;
                } else {
                    // 标记为完成
                    completed_instances.push(*instance_id);
                    instance.is_playing = false;
                    
                    // 停止粒子发射
                    for &emitter_id in &instance.emitter_ids {
                        self.particle_manager.stop_emitter(emitter_id);
                    }
                }
            }
        }

        // 处理完成的特效
        for instance_id in completed_instances {
            if let Some(instance) = self.instances.get(&instance_id) {
                // 调用完成回调
                if let Some(ref callback) = instance.on_complete {
                    callback();
                }
            }
        }

        // 更新粒子系统
        self.particle_manager.update(delta_time);
    }

    /// 渲染特效
    pub fn render(&self, render_system: &mut crate::render::RenderSystem) {
        self.particle_manager.render(render_system);
        
        // TODO: 渲染其他特效元素（光闪、UI效果等）
    }

    /// 获取特效实例
    pub fn get_effect_instance(&self, instance_id: EffectInstanceId) -> Option<&EffectInstance> {
        self.instances.get(&instance_id)
    }

    /// 获取可变特效实例
    pub fn get_effect_instance_mut(&mut self, instance_id: EffectInstanceId) -> Option<&mut EffectInstance> {
        self.instances.get_mut(&instance_id)
    }

    /// 设置特效位置
    pub fn set_effect_position(&mut self, instance_id: EffectInstanceId, position: Vec3) {
        if let Some(instance) = self.instances.get_mut(&instance_id) {
            instance.position = position;
            
            // 更新粒子发射器位置
            for &emitter_id in &instance.emitter_ids {
                self.particle_manager.set_emitter_position(emitter_id, position);
            }
        }
    }

    /// 列出所有注册的特效
    pub fn list_effects(&self) -> Vec<&String> {
        self.effects.keys().collect()
    }

    /// 获取活跃的特效实例数量
    pub fn get_active_instance_count(&self) -> usize {
        self.instances.values().filter(|instance| instance.is_playing).count()
    }

    /// 清除所有特效
    pub fn clear_all_effects(&mut self) {
        for instance in self.instances.values() {
            for &emitter_id in &instance.emitter_ids {
                self.particle_manager.remove_emitter(emitter_id);
            }
        }
        self.instances.clear();
    }

    /// 获取粒子系统统计信息
    pub fn get_particle_stats(&self) -> crate::particles::ParticleStats {
        self.particle_manager.get_stats()
    }
}

/// 特效组合器 - 用于创建复杂的特效组合
pub struct EffectComposer {
    effects: Vec<(String, f32, Vec3, Option<Quat>)>, // (特效名, 延迟时间, 位置偏移, 旋转)
    total_duration: f32,
}

impl EffectComposer {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            total_duration: 0.0,
        }
    }

    /// 添加特效到组合
    pub fn add_effect(
        mut self, 
        effect_name: String, 
        delay: f32, 
        position_offset: Vec3,
        rotation: Option<Quat>
    ) -> Self {
        self.effects.push((effect_name, delay, position_offset, rotation));
        self.total_duration = self.total_duration.max(delay);
        self
    }

    /// 播放组合特效
    pub fn play(
        &self, 
        effect_manager: &mut EffectSystemManager, 
        base_position: Vec3
    ) -> Vec<EffectInstanceId> {
        let mut instance_ids = Vec::new();

        for (effect_name, delay, position_offset, rotation) in &self.effects {
            // TODO: 实现延迟播放机制
            let position = base_position + *position_offset;
            if let Some(instance_id) = effect_manager.play_effect(effect_name, position, *rotation, None) {
                instance_ids.push(instance_id);
            }
        }

        instance_ids
    }
}

impl Default for EffectComposer {
    fn default() -> Self {
        Self::new()
    }
}

/// 特效触发器 - 基于条件自动播放特效
pub struct EffectTrigger {
    pub effect_name: String,
    pub condition: TriggerCondition,
    pub cooldown: f32,
    pub last_trigger_time: f32,
}

/// 触发条件
#[derive(Debug, Clone)]
pub enum TriggerCondition {
    HealthBelow(f32),           // 生命值低于某个值
    DistanceFromPlayer(f32),    // 与玩家距离
    TimeInterval(f32),          // 时间间隔
    Custom(String),             // 自定义条件
}

impl EffectTrigger {
    pub fn new(effect_name: String, condition: TriggerCondition, cooldown: f32) -> Self {
        Self {
            effect_name,
            condition,
            cooldown,
            last_trigger_time: 0.0,
        }
    }

    /// 检查是否应该触发
    pub fn should_trigger(&mut self, current_time: f32, context: &TriggerContext) -> bool {
        if current_time - self.last_trigger_time < self.cooldown {
            return false;
        }

        let triggered = match &self.condition {
            TriggerCondition::HealthBelow(threshold) => {
                context.health_ratio < *threshold
            }
            TriggerCondition::DistanceFromPlayer(distance) => {
                context.distance_to_player <= *distance
            }
            TriggerCondition::TimeInterval(interval) => {
                current_time - self.last_trigger_time >= *interval
            }
            TriggerCondition::Custom(condition_name) => {
                context.custom_conditions.get(condition_name).copied().unwrap_or(false)
            }
        };

        if triggered {
            self.last_trigger_time = current_time;
        }

        triggered
    }
}

/// 触发器上下文
pub struct TriggerContext {
    pub health_ratio: f32,
    pub distance_to_player: f32,
    pub custom_conditions: HashMap<String, bool>,
}

/// 特效预设库
pub struct EffectPresets;

impl EffectPresets {
    /// 大爆炸特效组合
    pub fn big_explosion() -> EffectComposer {
        EffectComposer::new()
            .add_effect("explosion".to_string(), 0.0, Vec3::ZERO, None)
            .add_effect("fire_torch".to_string(), 0.2, Vec3::new(0.0, 0.5, 0.0), None)
            .add_effect("explosion".to_string(), 0.5, Vec3::new(2.0, 0.0, 0.0), None)
            .add_effect("explosion".to_string(), 0.7, Vec3::new(-1.5, 0.0, 1.0), None)
    }

    /// 治疗光环特效
    pub fn healing_aura() -> EffectComposer {
        EffectComposer::new()
            .add_effect("healing".to_string(), 0.0, Vec3::ZERO, None)
            .add_effect("magic_cast".to_string(), 0.5, Vec3::new(0.0, 1.0, 0.0), None)
    }

    /// 魔法攻击特效
    pub fn magic_attack() -> EffectComposer {
        EffectComposer::new()
            .add_effect("magic_cast".to_string(), 0.0, Vec3::ZERO, None)
            .add_effect("explosion".to_string(), 1.0, Vec3::new(0.0, 0.0, 5.0), None)
    }
}
