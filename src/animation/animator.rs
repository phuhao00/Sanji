//! 动画播放器系统

use crate::animation::{AnimationClip, KeyframeValue};
use crate::ecs::{Component, Entity};
use crate::EngineResult;
use serde::{Serialize, Deserialize};
use specs::VecStorage;
use std::collections::HashMap;

/// 动画播放器组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animator {
    pub current_clip: Option<String>,
    pub time: f32,
    pub speed: f32,
    pub is_playing: bool,
    pub is_looping: bool,
    pub clips: HashMap<String, AnimationClip>,
}

impl Component for Animator {
    type Storage = VecStorage<Self>;
}

impl Default for Animator {
    fn default() -> Self {
        Self {
            current_clip: None,
            time: 0.0,
            speed: 1.0,
            is_playing: false,
            is_looping: true,
            clips: HashMap::new(),
        }
    }
}

impl Animator {
    /// 创建新的动画播放器
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加动画剪辑
    pub fn add_clip(&mut self, clip: AnimationClip) {
        self.clips.insert(clip.name.clone(), clip);
    }

    /// 播放动画
    pub fn play(&mut self, clip_name: &str) -> EngineResult<()> {
        if !self.clips.contains_key(clip_name) {
            return Err(crate::EngineError::AssetError(
                format!("Animation clip '{}' not found", clip_name)
            ).into());
        }

        self.current_clip = Some(clip_name.to_string());
        self.time = 0.0;
        self.is_playing = true;
        Ok(())
    }

    /// 停止动画
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.time = 0.0;
    }

    /// 暂停动画
    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    /// 恢复动画
    pub fn resume(&mut self) {
        self.is_playing = true;
    }

    /// 设置播放速度
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    /// 设置循环播放
    pub fn set_looping(&mut self, looping: bool) {
        self.is_looping = looping;
    }

    /// 更新动画
    pub fn update(&mut self, delta_time: f32) -> Option<HashMap<String, KeyframeValue>> {
        if !self.is_playing {
            return None;
        }

        let clip_name = self.current_clip.as_ref()?;
        let clip = self.clips.get(clip_name)?;

        self.time += delta_time * self.speed;

        // 处理循环和结束
        if self.time >= clip.duration {
            if self.is_looping {
                self.time = self.time % clip.duration;
            } else {
                self.time = clip.duration;
                self.is_playing = false;
            }
        }

        Some(clip.sample(self.time))
    }

    /// 获取当前播放进度 (0.0 - 1.0)
    pub fn get_progress(&self) -> f32 {
        if let Some(clip_name) = &self.current_clip {
            if let Some(clip) = self.clips.get(clip_name) {
                if clip.duration > 0.0 {
                    return (self.time / clip.duration).clamp(0.0, 1.0);
                }
            }
        }
        0.0
    }

    /// 设置播放进度 (0.0 - 1.0)
    pub fn set_progress(&mut self, progress: f32) {
        if let Some(clip_name) = &self.current_clip {
            if let Some(clip) = self.clips.get(clip_name) {
                self.time = (progress.clamp(0.0, 1.0) * clip.duration).clamp(0.0, clip.duration);
            }
        }
    }

    /// 检查动画是否正在播放
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// 获取当前动画剪辑名称
    pub fn current_clip(&self) -> Option<&str> {
        self.current_clip.as_deref()
    }

    /// 获取所有动画剪辑名称
    pub fn clip_names(&self) -> Vec<&str> {
        self.clips.keys().map(|s| s.as_str()).collect()
    }
}

/// 动画混合器
#[derive(Debug, Clone)]
pub struct AnimationBlender {
    pub animations: Vec<BlendedAnimation>,
}

/// 混合动画
#[derive(Debug, Clone)]
pub struct BlendedAnimation {
    pub animator: Animator,
    pub weight: f32,
}

impl AnimationBlender {
    /// 创建新的动画混合器
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
        }
    }

    /// 添加动画到混合器
    pub fn add_animation(&mut self, animator: Animator, weight: f32) {
        self.animations.push(BlendedAnimation { animator, weight });
    }

    /// 更新所有动画并混合结果
    pub fn update(&mut self, delta_time: f32) -> HashMap<String, KeyframeValue> {
        let mut blended_values: HashMap<String, Vec<(KeyframeValue, f32)>> = HashMap::new();

        // 收集所有动画的采样值
        for blend_anim in &mut self.animations {
            if let Some(values) = blend_anim.animator.update(delta_time) {
                for (target, value) in values {
                    blended_values
                        .entry(target)
                        .or_insert_with(Vec::new)
                        .push((value, blend_anim.weight));
                }
            }
        }

        // 混合所有值
        let mut result = HashMap::new();
        for (target, values) in blended_values {
            if let Some(blended) = self.blend_values(&values) {
                result.insert(target, blended);
            }
        }

        result
    }

    /// 混合多个动画值
    fn blend_values(&self, values: &[(KeyframeValue, f32)]) -> Option<KeyframeValue> {
        if values.is_empty() {
            return None;
        }

        if values.len() == 1 {
            return Some(values[0].0.clone());
        }

        // 归一化权重
        let total_weight: f32 = values.iter().map(|(_, w)| w).sum();
        if total_weight <= 0.0 {
            return Some(values[0].0.clone());
        }

        // 按类型混合
        match &values[0].0 {
            KeyframeValue::Float(_) => {
                let mut result = 0.0;
                for (value, weight) in values {
                    if let KeyframeValue::Float(v) = value {
                        result += v * (weight / total_weight);
                    }
                }
                Some(KeyframeValue::Float(result))
            }
            KeyframeValue::Vec3(_) => {
                let mut result = glam::Vec3::ZERO;
                for (value, weight) in values {
                    if let KeyframeValue::Vec3(v) = value {
                        result += *v * (weight / total_weight);
                    }
                }
                Some(KeyframeValue::Vec3(result))
            }
            KeyframeValue::Quaternion(_) => {
                // 四元数球面线性插值
                let mut result = glam::Quat::IDENTITY;
                for (i, (value, weight)) in values.iter().enumerate() {
                    if let KeyframeValue::Quaternion(q) = value {
                        if i == 0 {
                            result = *q;
                        } else {
                            let t = weight / total_weight;
                            result = result.slerp(*q, t);
                        }
                    }
                }
                Some(KeyframeValue::Quaternion(result))
            }
            KeyframeValue::Color(_) => {
                let mut result = [0.0; 4];
                for (value, weight) in values {
                    if let KeyframeValue::Color(c) = value {
                        let w = weight / total_weight;
                        result[0] += c[0] * w;
                        result[1] += c[1] * w;
                        result[2] += c[2] * w;
                        result[3] += c[3] * w;
                    }
                }
                Some(KeyframeValue::Color(result))
            }
        }
    }
}

impl Default for AnimationBlender {
    fn default() -> Self {
        Self::new()
    }
}
