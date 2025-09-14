//! 补间动画系统

use crate::math::easing::{EasingType, Easing};
use crate::EngineResult;
use serde::{Serialize, Deserialize};

/// 补间动画
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tween<T> {
    pub start_value: T,
    pub end_value: T,
    pub duration: f32,
    pub current_time: f32,
    pub easing: EasingType,
    pub is_playing: bool,
    pub is_finished: bool,
    pub is_looping: bool,
    pub ping_pong: bool,
    pub reverse: bool,
}

impl<T> Tween<T>
where
    T: Clone + Interpolatable,
{
    /// 创建新的补间动画
    pub fn new(start: T, end: T, duration: f32) -> Self {
        Self {
            start_value: start,
            end_value: end,
            duration,
            current_time: 0.0,
            easing: EasingType::Linear,
            is_playing: false,
            is_finished: false,
            is_looping: false,
            ping_pong: false,
            reverse: false,
        }
    }

    /// 设置缓动类型
    pub fn with_easing(mut self, easing: EasingType) -> Self {
        self.easing = easing;
        self
    }

    /// 设置循环播放
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.is_looping = looping;
        self
    }

    /// 设置乒乓播放
    pub fn with_ping_pong(mut self, ping_pong: bool) -> Self {
        self.ping_pong = ping_pong;
        self
    }

    /// 开始播放
    pub fn play(&mut self) {
        self.is_playing = true;
        self.is_finished = false;
        self.current_time = 0.0;
        self.reverse = false;
    }

    /// 停止播放
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.is_finished = false;
        self.current_time = 0.0;
        self.reverse = false;
    }

    /// 暂停播放
    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    /// 恢复播放
    pub fn resume(&mut self) {
        if !self.is_finished {
            self.is_playing = true;
        }
    }

    /// 更新动画
    pub fn update(&mut self, delta_time: f32) -> T {
        if self.is_playing && !self.is_finished {
            self.current_time += delta_time;

            if self.current_time >= self.duration {
                if self.ping_pong && !self.reverse {
                    // 乒乓模式：反向播放
                    self.reverse = true;
                    self.current_time = self.duration - (self.current_time - self.duration);
                } else if self.is_looping {
                    // 循环模式：重新开始
                    self.current_time = self.current_time % self.duration;
                    if self.ping_pong {
                        self.reverse = false;
                    }
                } else {
                    // 单次播放：结束
                    self.current_time = self.duration;
                    self.is_playing = false;
                    self.is_finished = true;
                }
            } else if self.ping_pong && self.reverse && self.current_time <= 0.0 {
                if self.is_looping {
                    self.reverse = false;
                    self.current_time = -self.current_time;
                } else {
                    self.current_time = 0.0;
                    self.is_playing = false;
                    self.is_finished = true;
                }
            }
        }

        self.current_value()
    }

    /// 获取当前值
    pub fn current_value(&self) -> T {
        if self.duration <= 0.0 {
            return self.end_value.clone();
        }

        let mut t = (self.current_time / self.duration).clamp(0.0, 1.0);
        
        if self.ping_pong && self.reverse {
            t = 1.0 - t;
        }

        let eased_t = Easing::ease(self.easing, t);
        
        if self.reverse && !self.ping_pong {
            self.end_value.interpolate(&self.start_value, eased_t)
        } else {
            self.start_value.interpolate(&self.end_value, eased_t)
        }
    }

    /// 获取进度 (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            (self.current_time / self.duration).clamp(0.0, 1.0)
        }
    }

    /// 设置进度 (0.0 - 1.0)
    pub fn set_progress(&mut self, progress: f32) {
        self.current_time = (progress.clamp(0.0, 1.0) * self.duration).clamp(0.0, self.duration);
        
        if progress >= 1.0 && !self.is_looping {
            self.is_finished = true;
            self.is_playing = false;
        }
    }

    /// 检查是否播放完成
    pub fn is_finished(&self) -> bool {
        self.is_finished
    }

    /// 检查是否正在播放
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// 设置新的目标值
    pub fn set_target(&mut self, new_target: T) {
        self.start_value = self.current_value();
        self.end_value = new_target;
        self.current_time = 0.0;
        self.is_finished = false;
        self.reverse = false;
    }
}

/// 可插值的trait
pub trait Interpolatable {
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

impl Interpolatable for f32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        crate::math::lerp(*self, *other, t)
    }
}

impl Interpolatable for glam::Vec2 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self.lerp(*other, t)
    }
}

impl Interpolatable for glam::Vec3 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self.lerp(*other, t)
    }
}

impl Interpolatable for glam::Vec4 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self.lerp(*other, t)
    }
}

impl Interpolatable for glam::Quat {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self.slerp(*other, t)
    }
}

impl Interpolatable for [f32; 4] {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        [
            crate::math::lerp(self[0], other[0], t),
            crate::math::lerp(self[1], other[1], t),
            crate::math::lerp(self[2], other[2], t),
            crate::math::lerp(self[3], other[3], t),
        ]
    }
}

/// 补间序列
#[derive(Debug, Clone)]
pub struct TweenSequence<T> {
    pub tweens: Vec<Tween<T>>,
    pub current_index: usize,
    pub is_playing: bool,
    pub is_looping: bool,
}

impl<T> TweenSequence<T>
where
    T: Clone + Interpolatable,
{
    /// 创建新的补间序列
    pub fn new() -> Self {
        Self {
            tweens: Vec::new(),
            current_index: 0,
            is_playing: false,
            is_looping: false,
        }
    }

    /// 添加补间动画
    pub fn add_tween(&mut self, tween: Tween<T>) {
        self.tweens.push(tween);
    }

    /// 设置循环播放
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.is_looping = looping;
        self
    }

    /// 开始播放
    pub fn play(&mut self) {
        if !self.tweens.is_empty() {
            self.is_playing = true;
            self.current_index = 0;
            self.tweens[0].play();
        }
    }

    /// 停止播放
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_index = 0;
        for tween in &mut self.tweens {
            tween.stop();
        }
    }

    /// 更新序列
    pub fn update(&mut self, delta_time: f32) -> Option<T> {
        if !self.is_playing || self.tweens.is_empty() {
            return None;
        }

        let current_value = self.tweens[self.current_index].update(delta_time);

        // 检查当前补间是否完成
        if self.tweens[self.current_index].is_finished() {
            self.current_index += 1;

            if self.current_index >= self.tweens.len() {
                if self.is_looping {
                    // 重新开始序列
                    self.current_index = 0;
                    self.tweens[0].play();
                } else {
                    // 序列完成
                    self.is_playing = false;
                    return Some(current_value);
                }
            } else {
                // 播放下一个补间
                self.tweens[self.current_index].play();
            }
        }

        Some(current_value)
    }

    /// 检查序列是否完成
    pub fn is_finished(&self) -> bool {
        !self.is_playing && self.current_index >= self.tweens.len()
    }

    /// 检查序列是否正在播放
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
}

impl<T> Default for TweenSequence<T>
where
    T: Clone + Interpolatable,
{
    fn default() -> Self {
        Self::new()
    }
}

/// 补间管理器
#[derive(Debug, Default)]
pub struct TweenManager {
    pub float_tweens: Vec<(String, Tween<f32>)>,
    pub vec2_tweens: Vec<(String, Tween<glam::Vec2>)>,
    pub vec3_tweens: Vec<(String, Tween<glam::Vec3>)>,
    pub vec4_tweens: Vec<(String, Tween<glam::Vec4>)>,
    pub color_tweens: Vec<(String, Tween<[f32; 4]>)>,
}

impl TweenManager {
    /// 创建新的补间管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加浮点数补间
    pub fn add_float_tween(&mut self, id: impl Into<String>, tween: Tween<f32>) {
        self.float_tweens.push((id.into(), tween));
    }

    /// 添加Vec3补间
    pub fn add_vec3_tween(&mut self, id: impl Into<String>, tween: Tween<glam::Vec3>) {
        self.vec3_tweens.push((id.into(), tween));
    }

    /// 更新所有补间
    pub fn update(&mut self, delta_time: f32) {
        // 更新并移除已完成的补间
        self.float_tweens.retain_mut(|(_, tween)| {
            tween.update(delta_time);
            !tween.is_finished()
        });

        self.vec2_tweens.retain_mut(|(_, tween)| {
            tween.update(delta_time);
            !tween.is_finished()
        });

        self.vec3_tweens.retain_mut(|(_, tween)| {
            tween.update(delta_time);
            !tween.is_finished()
        });

        self.vec4_tweens.retain_mut(|(_, tween)| {
            tween.update(delta_time);
            !tween.is_finished()
        });

        self.color_tweens.retain_mut(|(_, tween)| {
            tween.update(delta_time);
            !tween.is_finished()
        });
    }

    /// 获取浮点数补间的当前值
    pub fn get_float(&self, id: &str) -> Option<f32> {
        self.float_tweens
            .iter()
            .find(|(tween_id, _)| tween_id == id)
            .map(|(_, tween)| tween.current_value())
    }

    /// 获取Vec3补间的当前值
    pub fn get_vec3(&self, id: &str) -> Option<glam::Vec3> {
        self.vec3_tweens
            .iter()
            .find(|(tween_id, _)| tween_id == id)
            .map(|(_, tween)| tween.current_value())
    }

    /// 停止指定的补间
    pub fn stop_tween(&mut self, id: &str) {
        for (tween_id, tween) in &mut self.float_tweens {
            if tween_id == id {
                tween.stop();
            }
        }
        for (tween_id, tween) in &mut self.vec3_tweens {
            if tween_id == id {
                tween.stop();
            }
        }
        // ... 其他类型类似
    }

    /// 清除所有已完成的补间
    pub fn clear_finished(&mut self) {
        self.float_tweens.retain(|(_, tween)| !tween.is_finished());
        self.vec2_tweens.retain(|(_, tween)| !tween.is_finished());
        self.vec3_tweens.retain(|(_, tween)| !tween.is_finished());
        self.vec4_tweens.retain(|(_, tween)| !tween.is_finished());
        self.color_tweens.retain(|(_, tween)| !tween.is_finished());
    }

    /// 清除所有补间
    pub fn clear_all(&mut self) {
        self.float_tweens.clear();
        self.vec2_tweens.clear();
        self.vec3_tweens.clear();
        self.vec4_tweens.clear();
        self.color_tweens.clear();
    }
}
