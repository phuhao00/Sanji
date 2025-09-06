//! 缓动函数库

use std::f32::consts::PI;

/// 缓动函数类型
#[derive(Debug, Clone, Copy)]
pub enum EasingType {
    // 线性
    Linear,
    
    // 二次方
    QuadIn,
    QuadOut,
    QuadInOut,
    
    // 三次方
    CubicIn,
    CubicOut,
    CubicInOut,
    
    // 四次方
    QuartIn,
    QuartOut,
    QuartInOut,
    
    // 五次方
    QuintIn,
    QuintOut,
    QuintInOut,
    
    // 正弦
    SineIn,
    SineOut,
    SineInOut,
    
    // 指数
    ExpoIn,
    ExpoOut,
    ExpoInOut,
    
    // 圆形
    CircIn,
    CircOut,
    CircInOut,
    
    // 弹性
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    
    // 回弹
    BackIn,
    BackOut,
    BackInOut,
    
    // 反弹
    BounceIn,
    BounceOut,
    BounceInOut,
}

/// 缓动函数
pub struct Easing;

impl Easing {
    /// 应用缓动函数
    pub fn ease(easing_type: EasingType, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        
        match easing_type {
            EasingType::Linear => t,
            
            EasingType::QuadIn => Self::quad_in(t),
            EasingType::QuadOut => Self::quad_out(t),
            EasingType::QuadInOut => Self::quad_in_out(t),
            
            EasingType::CubicIn => Self::cubic_in(t),
            EasingType::CubicOut => Self::cubic_out(t),
            EasingType::CubicInOut => Self::cubic_in_out(t),
            
            EasingType::QuartIn => Self::quart_in(t),
            EasingType::QuartOut => Self::quart_out(t),
            EasingType::QuartInOut => Self::quart_in_out(t),
            
            EasingType::QuintIn => Self::quint_in(t),
            EasingType::QuintOut => Self::quint_out(t),
            EasingType::QuintInOut => Self::quint_in_out(t),
            
            EasingType::SineIn => Self::sine_in(t),
            EasingType::SineOut => Self::sine_out(t),
            EasingType::SineInOut => Self::sine_in_out(t),
            
            EasingType::ExpoIn => Self::expo_in(t),
            EasingType::ExpoOut => Self::expo_out(t),
            EasingType::ExpoInOut => Self::expo_in_out(t),
            
            EasingType::CircIn => Self::circ_in(t),
            EasingType::CircOut => Self::circ_out(t),
            EasingType::CircInOut => Self::circ_in_out(t),
            
            EasingType::ElasticIn => Self::elastic_in(t),
            EasingType::ElasticOut => Self::elastic_out(t),
            EasingType::ElasticInOut => Self::elastic_in_out(t),
            
            EasingType::BackIn => Self::back_in(t),
            EasingType::BackOut => Self::back_out(t),
            EasingType::BackInOut => Self::back_in_out(t),
            
            EasingType::BounceIn => Self::bounce_in(t),
            EasingType::BounceOut => Self::bounce_out(t),
            EasingType::BounceInOut => Self::bounce_in_out(t),
        }
    }

    // 二次方缓动
    fn quad_in(t: f32) -> f32 { t * t }
    fn quad_out(t: f32) -> f32 { 1.0 - (1.0 - t) * (1.0 - t) }
    fn quad_in_out(t: f32) -> f32 {
        if t < 0.5 { 2.0 * t * t } else { 1.0 - 2.0 * (1.0 - t) * (1.0 - t) }
    }

    // 三次方缓动
    fn cubic_in(t: f32) -> f32 { t * t * t }
    fn cubic_out(t: f32) -> f32 { 1.0 - (1.0 - t).powi(3) }
    fn cubic_in_out(t: f32) -> f32 {
        if t < 0.5 { 4.0 * t.powi(3) } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
    }

    // 四次方缓动
    fn quart_in(t: f32) -> f32 { t.powi(4) }
    fn quart_out(t: f32) -> f32 { 1.0 - (1.0 - t).powi(4) }
    fn quart_in_out(t: f32) -> f32 {
        if t < 0.5 { 8.0 * t.powi(4) } else { 1.0 - (-2.0 * t + 2.0).powi(4) / 2.0 }
    }

    // 五次方缓动
    fn quint_in(t: f32) -> f32 { t.powi(5) }
    fn quint_out(t: f32) -> f32 { 1.0 - (1.0 - t).powi(5) }
    fn quint_in_out(t: f32) -> f32 {
        if t < 0.5 { 16.0 * t.powi(5) } else { 1.0 - (-2.0 * t + 2.0).powi(5) / 2.0 }
    }

    // 正弦缓动
    fn sine_in(t: f32) -> f32 { 1.0 - (t * PI / 2.0).cos() }
    fn sine_out(t: f32) -> f32 { (t * PI / 2.0).sin() }
    fn sine_in_out(t: f32) -> f32 { -(t * PI).cos() - 1.0) / 2.0 }

    // 指数缓动
    fn expo_in(t: f32) -> f32 {
        if t == 0.0 { 0.0 } else { 2.0_f32.powf(10.0 * (t - 1.0)) }
    }
    fn expo_out(t: f32) -> f32 {
        if t == 1.0 { 1.0 } else { 1.0 - 2.0_f32.powf(-10.0 * t) }
    }
    fn expo_in_out(t: f32) -> f32 {
        if t == 0.0 { 0.0 }
        else if t == 1.0 { 1.0 }
        else if t < 0.5 { 2.0_f32.powf(20.0 * t - 10.0) / 2.0 }
        else { (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0 }
    }

    // 圆形缓动
    fn circ_in(t: f32) -> f32 { 1.0 - (1.0 - t * t).sqrt() }
    fn circ_out(t: f32) -> f32 { (1.0 - (t - 1.0) * (t - 1.0)).sqrt() }
    fn circ_in_out(t: f32) -> f32 {
        if t < 0.5 {
            (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
        } else {
            ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
        }
    }

    // 弹性缓动
    fn elastic_in(t: f32) -> f32 {
        let c4 = (2.0 * PI) / 3.0;
        if t == 0.0 { 0.0 }
        else if t == 1.0 { 1.0 }
        else { -(2.0_f32.powf(10.0 * t - 10.0)) * ((t * 10.0 - 10.75) * c4).sin() }
    }
    fn elastic_out(t: f32) -> f32 {
        let c4 = (2.0 * PI) / 3.0;
        if t == 0.0 { 0.0 }
        else if t == 1.0 { 1.0 }
        else { 2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0 }
    }
    fn elastic_in_out(t: f32) -> f32 {
        let c5 = (2.0 * PI) / 4.5;
        if t == 0.0 { 0.0 }
        else if t == 1.0 { 1.0 }
        else if t < 0.5 {
            -(2.0_f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
        } else {
            (2.0_f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0 + 1.0
        }
    }

    // 回弹缓动
    fn back_in(t: f32) -> f32 {
        let c1 = 1.70158;
        let c3 = c1 + 1.0;
        c3 * t.powi(3) - c1 * t * t
    }
    fn back_out(t: f32) -> f32 {
        let c1 = 1.70158;
        let c3 = c1 + 1.0;
        1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0) * (t - 1.0)
    }
    fn back_in_out(t: f32) -> f32 {
        let c1 = 1.70158;
        let c2 = c1 * 1.525;
        if t < 0.5 {
            ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
        } else {
            ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
        }
    }

    // 反弹缓动
    fn bounce_out(t: f32) -> f32 {
        let n1 = 7.5625;
        let d1 = 2.75;

        if t < 1.0 / d1 {
            n1 * t * t
        } else if t < 2.0 / d1 {
            let t = t - 1.5 / d1;
            n1 * t * t + 0.75
        } else if t < 2.5 / d1 {
            let t = t - 2.25 / d1;
            n1 * t * t + 0.9375
        } else {
            let t = t - 2.625 / d1;
            n1 * t * t + 0.984375
        }
    }
    fn bounce_in(t: f32) -> f32 { 1.0 - Self::bounce_out(1.0 - t) }
    fn bounce_in_out(t: f32) -> f32 {
        if t < 0.5 {
            (1.0 - Self::bounce_out(1.0 - 2.0 * t)) / 2.0
        } else {
            (1.0 + Self::bounce_out(2.0 * t - 1.0)) / 2.0
        }
    }
}

/// 缓动动画器
pub struct EasingAnimator {
    pub start_value: f32,
    pub end_value: f32,
    pub duration: f32,
    pub easing_type: EasingType,
    pub current_time: f32,
    pub is_playing: bool,
    pub is_finished: bool,
}

impl EasingAnimator {
    /// 创建新的缓动动画器
    pub fn new(start: f32, end: f32, duration: f32, easing: EasingType) -> Self {
        Self {
            start_value: start,
            end_value: end,
            duration,
            easing_type: easing,
            current_time: 0.0,
            is_playing: false,
            is_finished: false,
        }
    }

    /// 开始播放动画
    pub fn play(&mut self) {
        self.is_playing = true;
        self.is_finished = false;
        self.current_time = 0.0;
    }

    /// 暂停动画
    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    /// 恢复动画
    pub fn resume(&mut self) {
        if !self.is_finished {
            self.is_playing = true;
        }
    }

    /// 停止并重置动画
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.is_finished = false;
        self.current_time = 0.0;
    }

    /// 更新动画
    pub fn update(&mut self, delta_time: f32) -> f32 {
        if !self.is_playing || self.is_finished {
            return self.current_value();
        }

        self.current_time += delta_time;
        
        if self.current_time >= self.duration {
            self.current_time = self.duration;
            self.is_playing = false;
            self.is_finished = true;
        }

        self.current_value()
    }

    /// 获取当前值
    pub fn current_value(&self) -> f32 {
        if self.duration <= 0.0 {
            return self.end_value;
        }

        let t = (self.current_time / self.duration).clamp(0.0, 1.0);
        let eased_t = Easing::ease(self.easing_type, t);
        
        crate::math::lerp(self.start_value, self.end_value, eased_t)
    }

    /// 获取进度 (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 { 1.0 } else { (self.current_time / self.duration).clamp(0.0, 1.0) }
    }

    /// 设置新的目标值
    pub fn set_target(&mut self, new_target: f32) {
        self.start_value = self.current_value();
        self.end_value = new_target;
        self.current_time = 0.0;
        self.is_finished = false;
    }
}

/// Vec3缓动动画器
pub struct Vec3EasingAnimator {
    pub x_animator: EasingAnimator,
    pub y_animator: EasingAnimator,
    pub z_animator: EasingAnimator,
}

impl Vec3EasingAnimator {
    /// 创建新的Vec3缓动动画器
    pub fn new(start: glam::Vec3, end: glam::Vec3, duration: f32, easing: EasingType) -> Self {
        Self {
            x_animator: EasingAnimator::new(start.x, end.x, duration, easing),
            y_animator: EasingAnimator::new(start.y, end.y, duration, easing),
            z_animator: EasingAnimator::new(start.z, end.z, duration, easing),
        }
    }

    /// 开始播放动画
    pub fn play(&mut self) {
        self.x_animator.play();
        self.y_animator.play();
        self.z_animator.play();
    }

    /// 暂停动画
    pub fn pause(&mut self) {
        self.x_animator.pause();
        self.y_animator.pause();
        self.z_animator.pause();
    }

    /// 更新动画
    pub fn update(&mut self, delta_time: f32) -> glam::Vec3 {
        glam::Vec3::new(
            self.x_animator.update(delta_time),
            self.y_animator.update(delta_time),
            self.z_animator.update(delta_time),
        )
    }

    /// 检查动画是否完成
    pub fn is_finished(&self) -> bool {
        self.x_animator.is_finished && self.y_animator.is_finished && self.z_animator.is_finished
    }
}
