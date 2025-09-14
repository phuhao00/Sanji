//! 动画剪辑系统

use crate::math::Vec3;
use crate::EngineResult;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// 动画剪辑
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub tracks: Vec<AnimationTrack>,
}

/// 动画轨道
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationTrack {
    pub target: String,
    pub property: AnimationProperty,
    pub keyframes: Vec<Keyframe>,
}

/// 动画属性类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationProperty {
    Position,
    Rotation,
    Scale,
    Color,
    Alpha,
}

/// 关键帧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    pub time: f32,
    pub value: KeyframeValue,
    pub tangent_in: Option<Vec3>,
    pub tangent_out: Option<Vec3>,
}

/// 关键帧值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyframeValue {
    Float(f32),
    Vec3(Vec3),
    Quaternion(glam::Quat),
    Color([f32; 4]),
}

impl AnimationClip {
    /// 创建新的动画剪辑
    pub fn new(name: impl Into<String>, duration: f32) -> Self {
        Self {
            name: name.into(),
            duration,
            tracks: Vec::new(),
        }
    }

    /// 添加动画轨道
    pub fn add_track(&mut self, track: AnimationTrack) {
        self.tracks.push(track);
    }

    /// 获取指定时间的动画值
    pub fn sample(&self, time: f32) -> HashMap<String, KeyframeValue> {
        let mut result = HashMap::new();
        
        for track in &self.tracks {
            if let Some(value) = track.sample(time) {
                result.insert(track.target.clone(), value);
            }
        }
        
        result
    }
}

impl AnimationTrack {
    /// 创建新的动画轨道
    pub fn new(target: impl Into<String>, property: AnimationProperty) -> Self {
        Self {
            target: target.into(),
            property,
            keyframes: Vec::new(),
        }
    }

    /// 添加关键帧
    pub fn add_keyframe(&mut self, keyframe: Keyframe) {
        self.keyframes.push(keyframe);
        // 按时间排序
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    /// 采样指定时间的值
    pub fn sample(&self, time: f32) -> Option<KeyframeValue> {
        if self.keyframes.is_empty() {
            return None;
        }

        // 查找相邻的关键帧
        let mut prev_index = 0;
        let mut next_index = 0;

        for (i, keyframe) in self.keyframes.iter().enumerate() {
            if keyframe.time <= time {
                prev_index = i;
            }
            if keyframe.time >= time {
                next_index = i;
                break;
            }
        }

        // 如果时间超出范围，返回边界值
        if time <= self.keyframes[0].time {
            return Some(self.keyframes[0].value.clone());
        }
        if time >= self.keyframes.last().unwrap().time {
            return Some(self.keyframes.last().unwrap().value.clone());
        }

        // 插值计算
        if prev_index == next_index {
            return Some(self.keyframes[prev_index].value.clone());
        }

        let prev = &self.keyframes[prev_index];
        let next = &self.keyframes[next_index];
        let t = (time - prev.time) / (next.time - prev.time);

        Some(self.interpolate(&prev.value, &next.value, t))
    }

    /// 插值函数
    fn interpolate(&self, a: &KeyframeValue, b: &KeyframeValue, t: f32) -> KeyframeValue {
        match (a, b) {
            (KeyframeValue::Float(a), KeyframeValue::Float(b)) => {
                KeyframeValue::Float(crate::math::lerp(*a, *b, t))
            }
            (KeyframeValue::Vec3(a), KeyframeValue::Vec3(b)) => {
                KeyframeValue::Vec3(a.lerp(*b, t))
            }
            (KeyframeValue::Quaternion(a), KeyframeValue::Quaternion(b)) => {
                KeyframeValue::Quaternion(a.slerp(*b, t))
            }
            (KeyframeValue::Color(a), KeyframeValue::Color(b)) => {
                KeyframeValue::Color([
                    crate::math::lerp(a[0], b[0], t),
                    crate::math::lerp(a[1], b[1], t),
                    crate::math::lerp(a[2], b[2], t),
                    crate::math::lerp(a[3], b[3], t),
                ])
            }
            _ => a.clone(), // 类型不匹配时返回第一个值
        }
    }
}
