//! 关键帧动画系统

use crate::math::{Vec3, lerp, smoothstep};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 关键帧插值类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InterpolationType {
    /// 线性插值
    Linear,
    /// 阶跃插值（无插值）
    Step,
    /// 平滑插值
    Smooth,
    /// 贝塞尔曲线插值
    Bezier,
    /// 弹性插值
    Elastic,
}

/// 3D向量关键帧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vec3Keyframe {
    /// 时间点
    pub time: f32,
    /// 值
    pub value: Vec3,
    /// 插值类型
    pub interpolation: InterpolationType,
    /// 输入切线（贝塞尔曲线用）
    pub in_tangent: Option<Vec3>,
    /// 输出切线（贝塞尔曲线用）
    pub out_tangent: Option<Vec3>,
}

impl Vec3Keyframe {
    /// 创建新的关键帧
    pub fn new(time: f32, value: Vec3) -> Self {
        Self {
            time,
            value,
            interpolation: InterpolationType::Linear,
            in_tangent: None,
            out_tangent: None,
        }
    }

    /// 设置插值类型
    pub fn with_interpolation(mut self, interpolation: InterpolationType) -> Self {
        self.interpolation = interpolation;
        self
    }

    /// 设置贝塞尔切线
    pub fn with_tangents(mut self, in_tangent: Vec3, out_tangent: Vec3) -> Self {
        self.in_tangent = Some(in_tangent);
        self.out_tangent = Some(out_tangent);
        self
    }
}

/// 浮点数关键帧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatKeyframe {
    pub time: f32,
    pub value: f32,
    pub interpolation: InterpolationType,
    pub in_tangent: Option<f32>,
    pub out_tangent: Option<f32>,
}

impl FloatKeyframe {
    pub fn new(time: f32, value: f32) -> Self {
        Self {
            time,
            value,
            interpolation: InterpolationType::Linear,
            in_tangent: None,
            out_tangent: None,
        }
    }

    pub fn with_interpolation(mut self, interpolation: InterpolationType) -> Self {
        self.interpolation = interpolation;
        self
    }
}

/// 四元数关键帧（用于旋转）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuatKeyframe {
    pub time: f32,
    pub value: glam::Quat,
    pub interpolation: InterpolationType,
}

impl QuatKeyframe {
    pub fn new(time: f32, value: glam::Quat) -> Self {
        Self {
            time,
            value,
            interpolation: InterpolationType::Linear,
        }
    }
}

/// 动画曲线 - 存储关键帧序列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationCurve<T> {
    /// 关键帧列表（按时间排序）
    keyframes: BTreeMap<u32, T>, // 使用u32作为键，时间*1000转换为整数以避免浮点精度问题
    /// 动画长度
    duration: f32,
    /// 是否循环
    looping: bool,
    /// 循环前奏时间
    pre_wrap_mode: WrapMode,
    /// 循环后续时间
    post_wrap_mode: WrapMode,
}

/// 包装模式（超出动画范围时的行为）
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WrapMode {
    /// 夹紧到最后一帧
    Clamp,
    /// 循环
    Loop,
    /// 乒乓循环（来回播放）
    PingPong,
    /// 重置到默认值
    Default,
}

impl<T> AnimationCurve<T> {
    /// 创建新的动画曲线
    pub fn new() -> Self {
        Self {
            keyframes: BTreeMap::new(),
            duration: 0.0,
            looping: false,
            pre_wrap_mode: WrapMode::Clamp,
            post_wrap_mode: WrapMode::Clamp,
        }
    }

    /// 添加关键帧
    pub fn add_keyframe(&mut self, keyframe: T) 
    where 
        T: HasTime,
    {
        let time_key = (keyframe.time() * 1000.0) as u32;
        self.duration = self.duration.max(keyframe.time());
        self.keyframes.insert(time_key, keyframe);
    }

    /// 设置循环
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    /// 设置包装模式
    pub fn set_wrap_mode(&mut self, pre: WrapMode, post: WrapMode) {
        self.pre_wrap_mode = pre;
        self.post_wrap_mode = post;
    }

    /// 获取动画时长
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// 获取关键帧数量
    pub fn keyframe_count(&self) -> usize {
        self.keyframes.len()
    }

    /// 清空关键帧
    pub fn clear(&mut self) {
        self.keyframes.clear();
        self.duration = 0.0;
    }

    /// 获取所有关键帧
    pub fn keyframes(&self) -> impl Iterator<Item = &T> {
        self.keyframes.values()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }
}

impl<T> Default for AnimationCurve<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// 为关键帧类型定义时间访问trait
pub trait HasTime {
    fn time(&self) -> f32;
}

impl HasTime for Vec3Keyframe {
    fn time(&self) -> f32 {
        self.time
    }
}

impl HasTime for FloatKeyframe {
    fn time(&self) -> f32 {
        self.time
    }
}

impl HasTime for QuatKeyframe {
    fn time(&self) -> f32 {
        self.time
    }
}

/// Vec3动画曲线的特化实现
impl AnimationCurve<Vec3Keyframe> {
    /// 计算指定时间的Vec3值
    pub fn evaluate(&self, time: f32) -> Vec3 {
        if self.keyframes.is_empty() {
            return Vec3::ZERO;
        }

        // 处理时间包装
        let wrapped_time = self.wrap_time(time);
        let time_key = (wrapped_time * 1000.0) as u32;

        // 查找相邻的关键帧
        let mut iter = self.keyframes.range(..=time_key);
        let before = iter.next_back();
        let mut iter = self.keyframes.range(time_key..);
        let after = iter.next();

        match (before, after) {
            (Some((_, before_kf)), Some((_, after_kf))) if before_kf.time != after_kf.time => {
                // 在两个关键帧之间插值
                let t = (wrapped_time - before_kf.time) / (after_kf.time - before_kf.time);
                self.interpolate_vec3(before_kf, after_kf, t)
            }
            (Some((_, kf)), _) | (_, Some((_, kf))) => {
                // 只有一个关键帧或正好在关键帧上
                kf.value
            }
            (None, None) => Vec3::ZERO,
        }
    }

    /// 包装时间到有效范围
    fn wrap_time(&self, time: f32) -> f32 {
        if self.duration <= 0.0 {
            return 0.0;
        }

        if time < 0.0 {
            match self.pre_wrap_mode {
                WrapMode::Clamp => 0.0,
                WrapMode::Loop => {
                    let cycles = (-time / self.duration).floor();
                    time + (cycles + 1.0) * self.duration
                }
                WrapMode::PingPong => {
                    let abs_time = -time;
                    let cycle = (abs_time / self.duration).floor() as i32;
                    let local_time = abs_time % self.duration;
                    if cycle % 2 == 0 {
                        self.duration - local_time
                    } else {
                        local_time
                    }
                }
                WrapMode::Default => 0.0,
            }
        } else if time > self.duration {
            match self.post_wrap_mode {
                WrapMode::Clamp => self.duration,
                WrapMode::Loop => time % self.duration,
                WrapMode::PingPong => {
                    let cycle = (time / self.duration).floor() as i32;
                    let local_time = time % self.duration;
                    if cycle % 2 == 0 {
                        local_time
                    } else {
                        self.duration - local_time
                    }
                }
                WrapMode::Default => self.duration,
            }
        } else {
            time
        }
    }

    /// Vec3插值
    fn interpolate_vec3(&self, before: &Vec3Keyframe, after: &Vec3Keyframe, t: f32) -> Vec3 {
        match before.interpolation {
            InterpolationType::Step => before.value,
            InterpolationType::Linear => {
                Vec3::new(
                    lerp(before.value.x, after.value.x, t),
                    lerp(before.value.y, after.value.y, t),
                    lerp(before.value.z, after.value.z, t)
                )
            }
            InterpolationType::Smooth => {
                let smooth_t = smoothstep(0.0, 1.0, t);
                Vec3::new(
                    lerp(before.value.x, after.value.x, smooth_t),
                    lerp(before.value.y, after.value.y, smooth_t),
                    lerp(before.value.z, after.value.z, smooth_t)
                )
            }
            InterpolationType::Bezier => {
                if let (Some(out_tan), Some(in_tan)) = (&before.out_tangent, &after.in_tangent) {
                    // 贝塞尔插值
                    self.bezier_interpolate(before.value, before.value + *out_tan, 
                                          after.value + *in_tan, after.value, t)
                } else {
                    // 回退到线性插值
                    Vec3::new(
                        lerp(before.value.x, after.value.x, t),
                        lerp(before.value.y, after.value.y, t),
                        lerp(before.value.z, after.value.z, t)
                    )
                }
            }
            InterpolationType::Elastic => {
                let elastic_t = self.elastic_ease_in_out(t);
                Vec3::new(
                    lerp(before.value.x, after.value.x, elastic_t),
                    lerp(before.value.y, after.value.y, elastic_t),
                    lerp(before.value.z, after.value.z, elastic_t)
                )
            }
        }
    }

    /// 贝塞尔插值
    fn bezier_interpolate(&self, p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        p0 * mt3 + p1 * (3.0 * mt2 * t) + p2 * (3.0 * mt * t2) + p3 * t3
    }

    /// 弹性缓动
    fn elastic_ease_in_out(&self, t: f32) -> f32 {
        if t == 0.0 { return 0.0; }
        if t == 1.0 { return 1.0; }

        let p = 0.3 * 1.5;
        let a = 1.0;
        let s = p / 4.0;

        let t = t * 2.0;
        if t < 1.0 {
            let t = t - 1.0;
            -0.5 * (a * 2.0_f32.powf(10.0 * t) * ((t - s) * (2.0 * std::f32::consts::PI) / p).sin())
        } else {
            let t = t - 1.0;
            a * 2.0_f32.powf(-10.0 * t) * ((t - s) * (2.0 * std::f32::consts::PI) / p).sin() * 0.5 + 1.0
        }
    }
}

/// 浮点动画曲线的特化实现
impl AnimationCurve<FloatKeyframe> {
    pub fn evaluate(&self, time: f32) -> f32 {
        if self.keyframes.is_empty() {
            return 0.0;
        }

        let wrapped_time = self.wrap_time(time);
        let time_key = (wrapped_time * 1000.0) as u32;

        let mut iter = self.keyframes.range(..=time_key);
        let before = iter.next_back();
        let mut iter = self.keyframes.range(time_key..);
        let after = iter.next();

        match (before, after) {
            (Some((_, before_kf)), Some((_, after_kf))) if before_kf.time != after_kf.time => {
                let t = (wrapped_time - before_kf.time) / (after_kf.time - before_kf.time);
                self.interpolate_float(before_kf, after_kf, t)
            }
            (Some((_, kf)), _) | (_, Some((_, kf))) => kf.value,
            (None, None) => 0.0,
        }
    }

    fn wrap_time(&self, time: f32) -> f32 {
        // 类似Vec3的时间包装逻辑
        if self.duration <= 0.0 {
            return 0.0;
        }
        time.clamp(0.0, self.duration)
    }

    fn interpolate_float(&self, before: &FloatKeyframe, after: &FloatKeyframe, t: f32) -> f32 {
        match before.interpolation {
            InterpolationType::Step => before.value,
            InterpolationType::Linear => lerp(before.value, after.value, t),
            InterpolationType::Smooth => {
                let smooth_t = smoothstep(0.0, 1.0, t);
                lerp(before.value, after.value, smooth_t)
            }
            _ => lerp(before.value, after.value, t), // 其他插值回退到线性
        }
    }
}

/// 四元数动画曲线的特化实现
impl AnimationCurve<QuatKeyframe> {
    pub fn evaluate(&self, time: f32) -> glam::Quat {
        if self.keyframes.is_empty() {
            return glam::Quat::IDENTITY;
        }

        let wrapped_time = self.wrap_time(time);
        let time_key = (wrapped_time * 1000.0) as u32;

        let mut iter = self.keyframes.range(..=time_key);
        let before = iter.next_back();
        let mut iter = self.keyframes.range(time_key..);
        let after = iter.next();

        match (before, after) {
            (Some((_, before_kf)), Some((_, after_kf))) if before_kf.time != after_kf.time => {
                let t = (wrapped_time - before_kf.time) / (after_kf.time - before_kf.time);
                match before_kf.interpolation {
                    InterpolationType::Step => before_kf.value,
                    _ => before_kf.value.slerp(after_kf.value, t), // 球面线性插值
                }
            }
            (Some((_, kf)), _) | (_, Some((_, kf))) => kf.value,
            (None, None) => glam::Quat::IDENTITY,
        }
    }

    fn wrap_time(&self, time: f32) -> f32 {
        if self.duration <= 0.0 {
            return 0.0;
        }
        time.clamp(0.0, self.duration)
    }
}

/// 动画曲线构建器
pub struct AnimationCurveBuilder<T> {
    curve: AnimationCurve<T>,
}

impl<T> AnimationCurveBuilder<T> 
where 
    T: HasTime,
{
    pub fn new() -> Self {
        Self {
            curve: AnimationCurve::new(),
        }
    }

    pub fn add_keyframe(mut self, keyframe: T) -> Self {
        self.curve.add_keyframe(keyframe);
        self
    }

    pub fn looping(mut self, looping: bool) -> Self {
        self.curve.set_looping(looping);
        self
    }

    pub fn wrap_mode(mut self, pre: WrapMode, post: WrapMode) -> Self {
        self.curve.set_wrap_mode(pre, post);
        self
    }

    pub fn build(self) -> AnimationCurve<T> {
        self.curve
    }
}

/// 常用动画曲线预设
pub struct AnimationPresets;

impl AnimationPresets {
    /// 创建淡入曲线
    pub fn fade_in(duration: f32) -> AnimationCurve<FloatKeyframe> {
        AnimationCurveBuilder::new()
            .add_keyframe(FloatKeyframe::new(0.0, 0.0))
            .add_keyframe(FloatKeyframe::new(duration, 1.0).with_interpolation(InterpolationType::Smooth))
            .build()
    }

    /// 创建淡出曲线
    pub fn fade_out(duration: f32) -> AnimationCurve<FloatKeyframe> {
        AnimationCurveBuilder::new()
            .add_keyframe(FloatKeyframe::new(0.0, 1.0))
            .add_keyframe(FloatKeyframe::new(duration, 0.0).with_interpolation(InterpolationType::Smooth))
            .build()
    }

    /// 创建弹跳动画
    pub fn bounce(duration: f32, height: f32) -> AnimationCurve<Vec3Keyframe> {
        AnimationCurveBuilder::new()
            .add_keyframe(Vec3Keyframe::new(0.0, Vec3::ZERO))
            .add_keyframe(Vec3Keyframe::new(duration * 0.5, Vec3::new(0.0, height, 0.0))
                .with_interpolation(InterpolationType::Elastic))
            .add_keyframe(Vec3Keyframe::new(duration, Vec3::ZERO))
            .build()
    }

    /// 创建旋转动画
    pub fn rotate_y(duration: f32, angle_degrees: f32) -> AnimationCurve<QuatKeyframe> {
        let end_rotation = glam::Quat::from_rotation_y(angle_degrees.to_radians());
        AnimationCurveBuilder::new()
            .add_keyframe(QuatKeyframe::new(0.0, glam::Quat::IDENTITY))
            .add_keyframe(QuatKeyframe::new(duration, end_rotation))
            .looping(true)
            .build()
    }
}
