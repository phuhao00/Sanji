//! 音频源组件

use crate::math::Vec3;
use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

/// 音频源组件 - 3D空间中的音频发射器
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct AudioSource {
    /// 要播放的音频剪辑名称
    pub clip_name: String,
    /// 音量 (0.0 - 1.0)
    pub volume: f32,
    /// 音调 (0.1 - 3.0)
    pub pitch: f32,
    /// 是否循环播放
    pub looping: bool,
    /// 是否在启动时自动播放
    pub play_on_awake: bool,
    /// 是否3D音频
    pub spatial: bool,
    /// 最小听到距离
    pub min_distance: f32,
    /// 最大听到距离
    pub max_distance: f32,
    /// 音频衰减曲线类型
    pub rolloff_mode: AudioRolloffMode,
    /// 多普勒效应等级 (0.0 - 5.0)
    pub doppler_level: f32,
    /// 传播延迟
    pub spread: f32,
    /// 优先级 (0 = 最高优先级, 256 = 最低优先级)
    pub priority: u8,
    /// 是否正在播放
    #[serde(skip)]
    pub is_playing: bool,
    /// 是否暂停
    #[serde(skip)]
    pub is_paused: bool,
    /// 当前播放时间
    #[serde(skip)]
    pub time: f32,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            clip_name: String::new(),
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            play_on_awake: true,
            spatial: true,
            min_distance: 1.0,
            max_distance: 500.0,
            rolloff_mode: AudioRolloffMode::Logarithmic,
            doppler_level: 1.0,
            spread: 0.0,
            priority: 128,
            is_playing: false,
            is_paused: false,
            time: 0.0,
        }
    }
}

/// 音频衰减模式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AudioRolloffMode {
    /// 线性衰减
    Linear,
    /// 对数衰减（真实物理衰减）
    Logarithmic,
    /// 自定义衰减曲线
    Custom,
}

impl AudioSource {
    /// 创建新的音频源
    pub fn new(clip_name: impl Into<String>) -> Self {
        Self {
            clip_name: clip_name.into(),
            ..Default::default()
        }
    }

    /// 创建2D音频源（非空间化）
    pub fn new_2d(clip_name: impl Into<String>) -> Self {
        Self {
            clip_name: clip_name.into(),
            spatial: false,
            ..Default::default()
        }
    }

    /// 创建3D音频源
    pub fn new_3d(clip_name: impl Into<String>, min_distance: f32, max_distance: f32) -> Self {
        Self {
            clip_name: clip_name.into(),
            spatial: true,
            min_distance,
            max_distance,
            ..Default::default()
        }
    }

    /// 设置音量
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// 设置音调
    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.clamp(0.1, 3.0);
        self
    }

    /// 设置循环
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// 设置自动播放
    pub fn with_play_on_awake(mut self, play_on_awake: bool) -> Self {
        self.play_on_awake = play_on_awake;
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// 设置空间化
    pub fn with_spatial(mut self, spatial: bool) -> Self {
        self.spatial = spatial;
        self
    }

    /// 设置距离衰减
    pub fn with_distance_attenuation(mut self, min_distance: f32, max_distance: f32) -> Self {
        self.min_distance = min_distance.max(0.0);
        self.max_distance = max_distance.max(self.min_distance);
        self
    }

    /// 设置衰减模式
    pub fn with_rolloff_mode(mut self, mode: AudioRolloffMode) -> Self {
        self.rolloff_mode = mode;
        self
    }

    /// 设置多普勒效应等级
    pub fn with_doppler_level(mut self, level: f32) -> Self {
        self.doppler_level = level.clamp(0.0, 5.0);
        self
    }

    /// 开始播放
    pub fn play(&mut self) {
        self.is_playing = true;
        self.is_paused = false;
    }

    /// 停止播放
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.is_paused = false;
        self.time = 0.0;
    }

    /// 暂停播放
    pub fn pause(&mut self) {
        if self.is_playing {
            self.is_paused = true;
        }
    }

    /// 恢复播放
    pub fn resume(&mut self) {
        if self.is_paused {
            self.is_paused = false;
        }
    }

    /// 检查是否正在播放
    pub fn is_playing(&self) -> bool {
        self.is_playing && !self.is_paused
    }

    /// 检查是否暂停
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    /// 检查是否停止
    pub fn is_stopped(&self) -> bool {
        !self.is_playing
    }

    /// 设置播放时间
    pub fn set_time(&mut self, time: f32) {
        self.time = time.max(0.0);
    }

    /// 获取播放时间
    pub fn time(&self) -> f32 {
        self.time
    }

    /// 计算基于距离的音量衰减
    pub fn calculate_volume_attenuation(&self, distance: f32) -> f32 {
        if !self.spatial || distance <= self.min_distance {
            return 1.0;
        }

        if distance >= self.max_distance {
            return 0.0;
        }

        match self.rolloff_mode {
            AudioRolloffMode::Linear => {
                1.0 - (distance - self.min_distance) / (self.max_distance - self.min_distance)
            }
            AudioRolloffMode::Logarithmic => {
                self.min_distance / distance
            }
            AudioRolloffMode::Custom => {
                // 可以在这里实现自定义衰减曲线
                self.min_distance / distance
            }
        }
    }

    /// 计算多普勒效应
    pub fn calculate_doppler_shift(&self, listener_velocity: Vec3, source_velocity: Vec3, relative_position: Vec3) -> f32 {
        if self.doppler_level <= 0.0 || relative_position.length() < f32::EPSILON {
            return 1.0;
        }

        let sound_speed = 343.0; // 音速 (m/s)
        let direction = relative_position.normalize();
        
        // 监听器和音源在连线上的速度分量
        let listener_speed = listener_velocity.dot(direction);
        let source_speed = source_velocity.dot(direction);
        
        // 多普勒效应公式
        let doppler_factor = (sound_speed + listener_speed) / (sound_speed + source_speed);
        
        // 应用多普勒等级
        1.0 + (doppler_factor - 1.0) * self.doppler_level
    }
}

/// 音频源构建器
pub struct AudioSourceBuilder {
    source: AudioSource,
}

impl AudioSourceBuilder {
    /// 创建新的音频源构建器
    pub fn new(clip_name: impl Into<String>) -> Self {
        Self {
            source: AudioSource::new(clip_name),
        }
    }

    /// 设置为2D音频
    pub fn as_2d(mut self) -> Self {
        self.source.spatial = false;
        self
    }

    /// 设置为3D音频
    pub fn as_3d(mut self, min_distance: f32, max_distance: f32) -> Self {
        self.source.spatial = true;
        self.source.min_distance = min_distance;
        self.source.max_distance = max_distance;
        self
    }

    /// 设置音量
    pub fn volume(mut self, volume: f32) -> Self {
        self.source.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// 设置循环播放
    pub fn looping(mut self) -> Self {
        self.source.looping = true;
        self
    }

    /// 设置不自动播放
    pub fn no_auto_play(mut self) -> Self {
        self.source.play_on_awake = false;
        self
    }

    /// 设置高优先级
    pub fn high_priority(mut self) -> Self {
        self.source.priority = 0;
        self
    }

    /// 设置低优先级
    pub fn low_priority(mut self) -> Self {
        self.source.priority = 255;
        self
    }

    /// 构建音频源
    pub fn build(self) -> AudioSource {
        self.source
    }
}

/// 音频源预设
pub struct AudioSourcePresets;

impl AudioSourcePresets {
    /// 背景音乐预设
    pub fn background_music(clip_name: impl Into<String>) -> AudioSource {
        AudioSource::new(clip_name)
            .with_volume(0.5)
            .with_looping(true)
            .with_spatial(false)
            .with_priority(64)
    }

    /// 音效预设
    pub fn sound_effect(clip_name: impl Into<String>) -> AudioSource {
        AudioSource::new(clip_name)
            .with_volume(0.8)
            .with_spatial(true)
            .with_distance_attenuation(1.0, 20.0)
            .with_priority(128)
    }

    /// 环境音预设
    pub fn ambient_sound(clip_name: impl Into<String>) -> AudioSource {
        AudioSource::new(clip_name)
            .with_volume(0.3)
            .with_looping(true)
            .with_spatial(true)
            .with_distance_attenuation(5.0, 50.0)
            .with_priority(200)
    }

    /// 语音预设
    pub fn voice(clip_name: impl Into<String>) -> AudioSource {
        AudioSource::new(clip_name)
            .with_volume(1.0)
            .with_spatial(false)
            .with_priority(32)
    }

    /// UI音效预设
    pub fn ui_sound(clip_name: impl Into<String>) -> AudioSource {
        AudioSource::new(clip_name)
            .with_volume(0.7)
            .with_spatial(false)
            .with_priority(16)
    }
}
