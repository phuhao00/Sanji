//! 音频系统实现

use crate::{EngineResult, EngineError};
use crate::audio::{AudioSource, AudioListener};
use crate::math::Vec3;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::Path;
use specs::Entity;

/// 音频后端类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioBackend {
    /// 自动选择
    Auto,
    /// ALSA (Linux)
    Alsa,
    /// PulseAudio (Linux)
    PulseAudio,
    /// WASAPI (Windows)
    Wasapi,
    /// DirectSound (Windows)
    DirectSound,
    /// CoreAudio (macOS)
    CoreAudio,
}

/// 音频配置
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// 主音量 (0.0 - 1.0)
    pub master_volume: f32,
    /// 最大同时播放的音频源数量
    pub max_sources: usize,
    /// 3D音频的最大距离
    pub max_distance: f32,
    /// 多普勒效应强度
    pub doppler_factor: f32,
    /// 音频后端
    pub backend: AudioBackend,
    /// 缓冲区大小
    pub buffer_size: usize,
    /// 采样率
    pub sample_rate: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            max_sources: 64,
            max_distance: 100.0,
            doppler_factor: 1.0,
            backend: AudioBackend::Auto,
            buffer_size: 4096,
            sample_rate: 44100,
        }
    }
}

/// 音频剪辑数据
#[derive(Debug, Clone)]
pub struct AudioClip {
    /// 音频名称
    pub name: String,
    /// 音频数据
    pub data: Vec<f32>,
    /// 采样率
    pub sample_rate: u32,
    /// 声道数
    pub channels: u16,
    /// 时长（秒）
    pub duration: f32,
    /// 是否循环
    pub looping: bool,
}

impl AudioClip {
    /// 创建新的音频剪辑
    pub fn new(name: impl Into<String>, data: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        let duration = data.len() as f32 / (sample_rate as f32 * channels as f32);
        Self {
            name: name.into(),
            data,
            sample_rate,
            channels,
            duration,
            looping: false,
        }
    }

    /// 从文件加载音频剪辑
    pub fn from_file<P: AsRef<Path>>(path: P) -> EngineResult<Self> {
        let path = path.as_ref();
        
        // 这里应该使用音频库来加载文件
        // 简化实现：创建一个测试音频剪辑
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
            
        // 生成1秒的440Hz正弦波作为测试
        let sample_rate = 44100;
        let duration = 1.0;
        let frequency = 440.0;
        let mut data = Vec::with_capacity((sample_rate as f32 * duration) as usize);
        
        for i in 0..(sample_rate as f32 * duration) as usize {
            let t = i as f32 / sample_rate as f32;
            let sample = (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.3;
            data.push(sample);
        }
        
        Ok(Self::new(name, data, sample_rate, 1))
    }

    /// 设置循环
    pub fn set_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }
}

/// 音频播放状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// 音频系统
pub struct AudioSystem {
    config: AudioConfig,
    /// 音频剪辑库
    clips: HashMap<String, Arc<AudioClip>>,
    /// 活跃的音频源
    active_sources: HashMap<Entity, AudioSourceState>,
    /// 音频监听器
    listener: AudioListener,
    /// 是否初始化
    initialized: bool,
    /// 是否静音
    muted: bool,
}

/// 音频源状态
#[derive(Debug)]
struct AudioSourceState {
    clip: Arc<AudioClip>,
    position: usize,
    state: PlaybackState,
    volume: f32,
    pitch: f32,
    looping: bool,
    position_3d: Option<Vec3>,
    velocity_3d: Option<Vec3>,
}

impl AudioSystem {
    /// 创建新的音频系统
    pub fn new(config: AudioConfig) -> EngineResult<Self> {
        let mut system = Self {
            config,
            clips: HashMap::new(),
            active_sources: HashMap::new(),
            listener: AudioListener::new(),
            initialized: false,
            muted: false,
        };
        
        system.initialize()?;
        Ok(system)
    }

    /// 初始化音频系统
    fn initialize(&mut self) -> EngineResult<()> {
        // 这里应该初始化音频后端
        // 简化实现：仅标记为已初始化
        self.initialized = true;
        log::info!("音频系统初始化完成");
        Ok(())
    }

    /// 加载音频剪辑
    pub fn load_clip<P: AsRef<Path>>(&mut self, path: P) -> EngineResult<String> {
        let clip = AudioClip::from_file(path)?;
        let name = clip.name.clone();
        self.clips.insert(name.clone(), Arc::new(clip));
        log::info!("音频剪辑已加载: {}", name);
        Ok(name)
    }

    /// 添加音频剪辑
    pub fn add_clip(&mut self, clip: AudioClip) {
        let name = clip.name.clone();
        self.clips.insert(name.clone(), Arc::new(clip));
        log::info!("音频剪辑已添加: {}", name);
    }

    /// 播放音频剪辑
    pub fn play_clip(&mut self, clip_name: &str, entity: Entity) -> EngineResult<()> {
        let clip = self.clips.get(clip_name)
            .ok_or_else(|| EngineError::AssetError(format!("音频剪辑未找到: {}", clip_name)))?
            .clone();

        let source_state = AudioSourceState {
            clip,
            position: 0,
            state: PlaybackState::Playing,
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            position_3d: None,
            velocity_3d: None,
        };

        self.active_sources.insert(entity, source_state);
        log::debug!("开始播放音频: {} (实体: {:?})", clip_name, entity);
        Ok(())
    }

    /// 播放一次性音频（不需要实体）
    pub fn play_one_shot(&mut self, clip_name: &str, volume: f32) -> EngineResult<()> {
        let clip = self.clips.get(clip_name)
            .ok_or_else(|| EngineError::AssetError(format!("音频剪辑未找到: {}", clip_name)))?;

        // 简化实现：创建临时实体ID
        let temp_entity = specs::Entity::from_bits(rand::random()).unwrap();
        
        let source_state = AudioSourceState {
            clip: clip.clone(),
            position: 0,
            state: PlaybackState::Playing,
            volume,
            pitch: 1.0,
            looping: false,
            position_3d: None,
            velocity_3d: None,
        };

        self.active_sources.insert(temp_entity, source_state);
        log::debug!("播放一次性音频: {}", clip_name);
        Ok(())
    }

    /// 停止音频播放
    pub fn stop(&mut self, entity: Entity) {
        if let Some(source) = self.active_sources.get_mut(&entity) {
            source.state = PlaybackState::Stopped;
            source.position = 0;
        }
    }

    /// 暂停音频播放
    pub fn pause(&mut self, entity: Entity) {
        if let Some(source) = self.active_sources.get_mut(&entity) {
            source.state = PlaybackState::Paused;
        }
    }

    /// 恢复音频播放
    pub fn resume(&mut self, entity: Entity) {
        if let Some(source) = self.active_sources.get_mut(&entity) {
            if source.state == PlaybackState::Paused {
                source.state = PlaybackState::Playing;
            }
        }
    }

    /// 设置音频源音量
    pub fn set_volume(&mut self, entity: Entity, volume: f32) {
        if let Some(source) = self.active_sources.get_mut(&entity) {
            source.volume = volume.clamp(0.0, 1.0);
        }
    }

    /// 设置音频源音调
    pub fn set_pitch(&mut self, entity: Entity, pitch: f32) {
        if let Some(source) = self.active_sources.get_mut(&entity) {
            source.pitch = pitch.clamp(0.1, 3.0);
        }
    }

    /// 设置音频源循环
    pub fn set_looping(&mut self, entity: Entity, looping: bool) {
        if let Some(source) = self.active_sources.get_mut(&entity) {
            source.looping = looping;
        }
    }

    /// 设置3D音频位置
    pub fn set_3d_position(&mut self, entity: Entity, position: Vec3) {
        if let Some(source) = self.active_sources.get_mut(&entity) {
            source.position_3d = Some(position);
        }
    }

    /// 设置3D音频速度（用于多普勒效应）
    pub fn set_3d_velocity(&mut self, entity: Entity, velocity: Vec3) {
        if let Some(source) = self.active_sources.get_mut(&entity) {
            source.velocity_3d = Some(velocity);
        }
    }

    /// 更新音频系统
    pub fn update(&mut self, delta_time: f32) -> EngineResult<()> {
        if !self.initialized || self.muted {
            return Ok(());
        }

        let mut finished_sources = Vec::new();

        // 更新所有活跃的音频源
        for (entity, source) in self.active_sources.iter_mut() {
            if source.state == PlaybackState::Playing {
                // 简化的音频播放逻辑
                let samples_per_frame = (source.clip.sample_rate as f32 * delta_time) as usize;
                source.position += samples_per_frame;

                // 检查是否播放完毕
                if source.position >= source.clip.data.len() {
                    if source.looping || source.clip.looping {
                        source.position = 0; // 重新开始
                    } else {
                        source.state = PlaybackState::Stopped;
                        finished_sources.push(*entity);
                    }
                }
            }
        }

        // 清理已完成的音频源
        for entity in finished_sources {
            self.active_sources.remove(&entity);
        }

        Ok(())
    }

    /// 设置监听器位置
    pub fn set_listener_position(&mut self, position: Vec3) {
        self.listener.set_position(position);
    }

    /// 设置监听器方向
    pub fn set_listener_orientation(&mut self, forward: Vec3, up: Vec3) {
        self.listener.set_orientation(forward, up);
    }

    /// 设置监听器速度
    pub fn set_listener_velocity(&mut self, velocity: Vec3) {
        self.listener.set_velocity(velocity);
    }

    /// 设置主音量
    pub fn set_master_volume(&mut self, volume: f32) {
        self.config.master_volume = volume.clamp(0.0, 1.0);
    }

    /// 获取主音量
    pub fn master_volume(&self) -> f32 {
        self.config.master_volume
    }

    /// 静音/取消静音
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        if muted {
            log::info!("音频系统已静音");
        } else {
            log::info!("音频系统取消静音");
        }
    }

    /// 检查是否静音
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// 获取活跃音频源数量
    pub fn active_source_count(&self) -> usize {
        self.active_sources.len()
    }

    /// 获取音频剪辑数量
    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }

    /// 检查音频剪辑是否存在
    pub fn has_clip(&self, name: &str) -> bool {
        self.clips.contains_key(name)
    }

    /// 移除音频剪辑
    pub fn remove_clip(&mut self, name: &str) -> bool {
        self.clips.remove(name).is_some()
    }

    /// 清空所有音频剪辑
    pub fn clear_clips(&mut self) {
        self.clips.clear();
    }

    /// 停止所有音频
    pub fn stop_all(&mut self) {
        for (_, source) in self.active_sources.iter_mut() {
            source.state = PlaybackState::Stopped;
        }
        self.active_sources.clear();
    }

    /// 获取音频统计信息
    pub fn stats(&self) -> AudioStats {
        let playing_count = self.active_sources
            .values()
            .filter(|s| s.state == PlaybackState::Playing)
            .count();
            
        let paused_count = self.active_sources
            .values()
            .filter(|s| s.state == PlaybackState::Paused)
            .count();

        AudioStats {
            total_clips: self.clips.len(),
            active_sources: self.active_sources.len(),
            playing_sources: playing_count,
            paused_sources: paused_count,
            master_volume: self.config.master_volume,
            is_muted: self.muted,
        }
    }
}

/// 音频统计信息
#[derive(Debug, Clone)]
pub struct AudioStats {
    pub total_clips: usize,
    pub active_sources: usize,
    pub playing_sources: usize,
    pub paused_sources: usize,
    pub master_volume: f32,
    pub is_muted: bool,
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new(AudioConfig::default()).unwrap()
    }
}
