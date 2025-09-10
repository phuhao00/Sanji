//! 粒子数据结构

use crate::math::{Vec3, Vec2, Quat};
use serde::{Deserialize, Serialize};

/// 粒子ID类型
pub type ParticleId = u64;

/// 单个粒子
#[derive(Debug, Clone)]
pub struct Particle {
    /// 粒子ID
    pub id: ParticleId,
    
    /// 位置
    pub position: Vec3,
    
    /// 速度
    pub velocity: Vec3,
    
    /// 加速度
    pub acceleration: Vec3,
    
    /// 旋转
    pub rotation: f32,
    
    /// 角速度
    pub angular_velocity: f32,
    
    /// 大小
    pub size: f32,
    
    /// 颜色 (RGBA)
    pub color: [f32; 4],
    
    /// 生命时间（当前）
    pub lifetime: f32,
    
    /// 最大生命时间
    pub max_lifetime: f32,
    
    /// 初始大小
    pub start_size: f32,
    
    /// 初始颜色
    pub start_color: [f32; 4],
    
    /// 是否活跃
    pub is_alive: bool,
    
    /// 重力缩放
    pub gravity_scale: f32,
    
    /// 阻力系数
    pub drag: f32,
    
    /// 自定义数据
    pub user_data: ParticleUserData,
}

/// 粒子自定义数据
#[derive(Debug, Clone, Default)]
pub struct ParticleUserData {
    /// 浮点数据
    pub float_data: Vec<f32>,
    /// 整数数据
    pub int_data: Vec<i32>,
    /// 布尔数据
    pub bool_data: Vec<bool>,
}

impl Particle {
    /// 创建新粒子
    pub fn new(id: ParticleId, position: Vec3, velocity: Vec3) -> Self {
        Self {
            id,
            position,
            velocity,
            acceleration: Vec3::ZERO,
            rotation: 0.0,
            angular_velocity: 0.0,
            size: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            lifetime: 0.0,
            max_lifetime: 1.0,
            start_size: 1.0,
            start_color: [1.0, 1.0, 1.0, 1.0],
            is_alive: true,
            gravity_scale: 1.0,
            drag: 0.0,
            user_data: ParticleUserData::default(),
        }
    }

    /// 设置生命时间
    pub fn set_lifetime(&mut self, lifetime: f32) {
        self.max_lifetime = lifetime;
        self.lifetime = 0.0;
    }

    /// 设置初始属性
    pub fn set_initial_properties(&mut self, size: f32, color: [f32; 4]) {
        self.start_size = size;
        self.size = size;
        self.start_color = color;
        self.color = color;
    }

    /// 更新粒子
    pub fn update(&mut self, delta_time: f32, gravity: Vec3) {
        if !self.is_alive {
            return;
        }

        // 更新生命时间
        self.lifetime += delta_time;
        
        // 检查是否死亡
        if self.lifetime >= self.max_lifetime {
            self.is_alive = false;
            return;
        }

        // 应用重力
        self.acceleration += gravity * self.gravity_scale;

        // 应用阻力
        if self.drag > 0.0 {
            let drag_force = self.velocity * (-self.drag * delta_time);
            self.velocity += drag_force;
        }

        // 更新速度和位置
        self.velocity += self.acceleration * delta_time;
        self.position += self.velocity * delta_time;

        // 更新旋转
        self.rotation += self.angular_velocity * delta_time;

        // 重置加速度（每帧重新计算）
        self.acceleration = Vec3::ZERO;
    }

    /// 获取生命时间比例 (0.0 到 1.0)
    pub fn lifetime_ratio(&self) -> f32 {
        if self.max_lifetime <= 0.0 {
            1.0
        } else {
            (self.lifetime / self.max_lifetime).clamp(0.0, 1.0)
        }
    }

    /// 获取剩余生命时间比例 (1.0 到 0.0)
    pub fn remaining_lifetime_ratio(&self) -> f32 {
        1.0 - self.lifetime_ratio()
    }

    /// 是否即将死亡
    pub fn is_dying(&self, threshold: f32) -> bool {
        self.lifetime_ratio() >= threshold
    }

    /// 重置粒子（重用）
    pub fn reset(&mut self, id: ParticleId, position: Vec3, velocity: Vec3) {
        self.id = id;
        self.position = position;
        self.velocity = velocity;
        self.acceleration = Vec3::ZERO;
        self.lifetime = 0.0;
        self.is_alive = true;
        self.size = self.start_size;
        self.color = self.start_color;
        self.rotation = 0.0;
        self.angular_velocity = 0.0;
    }

    /// 添加力
    pub fn add_force(&mut self, force: Vec3) {
        self.acceleration += force;
    }

    /// 添加冲量
    pub fn add_impulse(&mut self, impulse: Vec3) {
        self.velocity += impulse;
    }

    /// 设置用户浮点数据
    pub fn set_float_data(&mut self, index: usize, value: f32) {
        if index >= self.user_data.float_data.len() {
            self.user_data.float_data.resize(index + 1, 0.0);
        }
        self.user_data.float_data[index] = value;
    }

    /// 获取用户浮点数据
    pub fn get_float_data(&self, index: usize) -> Option<f32> {
        self.user_data.float_data.get(index).copied()
    }

    /// 设置用户整数数据
    pub fn set_int_data(&mut self, index: usize, value: i32) {
        if index >= self.user_data.int_data.len() {
            self.user_data.int_data.resize(index + 1, 0);
        }
        self.user_data.int_data[index] = value;
    }

    /// 获取用户整数数据
    pub fn get_int_data(&self, index: usize) -> Option<i32> {
        self.user_data.int_data.get(index).copied()
    }

    /// 设置用户布尔数据
    pub fn set_bool_data(&mut self, index: usize, value: bool) {
        if index >= self.user_data.bool_data.len() {
            self.user_data.bool_data.resize(index + 1, false);
        }
        self.user_data.bool_data[index] = value;
    }

    /// 获取用户布尔数据
    pub fn get_bool_data(&self, index: usize) -> Option<bool> {
        self.user_data.bool_data.get(index).copied()
    }
}

/// 粒子状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleState {
    /// 出生
    Birth,
    /// 活跃
    Alive,
    /// 死亡
    Dead,
}

/// 粒子事件
#[derive(Debug, Clone)]
pub enum ParticleEvent {
    /// 粒子出生
    Birth { particle_id: ParticleId },
    /// 粒子死亡
    Death { particle_id: ParticleId },
    /// 粒子碰撞
    Collision { 
        particle_id: ParticleId, 
        position: Vec3, 
        normal: Vec3 
    },
}

/// 粒子属性插值器
#[derive(Debug, Clone)]
pub struct ParticleInterpolator<T> {
    keyframes: Vec<(f32, T)>, // (时间比例, 值)
}

impl<T> ParticleInterpolator<T>
where
    T: Clone + std::ops::Add<Output = T> + std::ops::Mul<f32, Output = T>,
{
    pub fn new(keyframes: Vec<(f32, T)>) -> Self {
        let mut sorted_keyframes = keyframes;
        sorted_keyframes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        
        Self {
            keyframes: sorted_keyframes,
        }
    }

    /// 在指定时间插值
    pub fn interpolate(&self, time_ratio: f32) -> Option<T> {
        if self.keyframes.is_empty() {
            return None;
        }

        let clamped_time = time_ratio.clamp(0.0, 1.0);

        // 找到时间范围
        for i in 0..self.keyframes.len() {
            let (t1, ref v1) = self.keyframes[i];
            
            if clamped_time <= t1 {
                return Some(v1.clone());
            }

            if i + 1 < self.keyframes.len() {
                let (t2, ref v2) = self.keyframes[i + 1];
                
                if clamped_time >= t1 && clamped_time <= t2 {
                    // 线性插值
                    let factor = if t2 - t1 > 0.0 {
                        (clamped_time - t1) / (t2 - t1)
                    } else {
                        0.0
                    };
                    
                    return Some(v1.clone() * (1.0 - factor) + v2.clone() * factor);
                }
            }
        }

        // 返回最后一个值
        self.keyframes.last().map(|(_, v)| v.clone())
    }

    /// 添加关键帧
    pub fn add_keyframe(&mut self, time: f32, value: T) {
        self.keyframes.push((time, value));
        self.keyframes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    /// 移除关键帧
    pub fn remove_keyframe(&mut self, index: usize) -> bool {
        if index < self.keyframes.len() {
            self.keyframes.remove(index);
            true
        } else {
            false
        }
    }

    /// 清除所有关键帧
    pub fn clear(&mut self) {
        self.keyframes.clear();
    }

    /// 获取关键帧数量
    pub fn keyframe_count(&self) -> usize {
        self.keyframes.len()
    }
}

/// 粒子颜色插值器
pub type ColorInterpolator = ParticleInterpolator<[f32; 4]>;

/// 粒子大小插值器
pub type SizeInterpolator = ParticleInterpolator<f32>;

/// 粒子速度插值器
pub type VelocityInterpolator = ParticleInterpolator<Vec3>;

/// 粒子批次（用于高效渲染）
#[derive(Debug, Clone)]
pub struct ParticleBatch {
    /// 粒子数据
    pub particles: Vec<Particle>,
    /// 纹理路径
    pub texture_path: Option<String>,
    /// 混合模式
    pub blend_mode: BlendMode,
    /// 排序层级
    pub sorting_layer: i32,
    /// 层内顺序
    pub order_in_layer: i32,
}

/// 混合模式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BlendMode {
    /// 普通混合
    Alpha,
    /// 加法混合
    Additive,
    /// 乘法混合
    Multiply,
    /// 减法混合
    Subtract,
    /// 屏幕混合
    Screen,
}

impl Default for BlendMode {
    fn default() -> Self {
        BlendMode::Alpha
    }
}

impl ParticleBatch {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            texture_path: None,
            blend_mode: BlendMode::Alpha,
            sorting_layer: 0,
            order_in_layer: 0,
        }
    }

    /// 添加粒子
    pub fn add_particle(&mut self, particle: Particle) {
        self.particles.push(particle);
    }

    /// 移除死亡粒子
    pub fn remove_dead_particles(&mut self) {
        self.particles.retain(|p| p.is_alive);
    }

    /// 更新所有粒子
    pub fn update(&mut self, delta_time: f32, gravity: Vec3) {
        for particle in &mut self.particles {
            particle.update(delta_time, gravity);
        }
    }

    /// 按距离排序粒子（用于透明度渲染）
    pub fn sort_by_distance(&mut self, camera_position: Vec3) {
        self.particles.sort_by(|a, b| {
            let dist_a = (a.position - camera_position).length_squared();
            let dist_b = (b.position - camera_position).length_squared();
            dist_b.partial_cmp(&dist_a).unwrap() // 远到近排序
        });
    }

    /// 清除所有粒子
    pub fn clear(&mut self) {
        self.particles.clear();
    }

    /// 获取活跃粒子数量
    pub fn alive_count(&self) -> usize {
        self.particles.iter().filter(|p| p.is_alive).count()
    }
}

impl Default for ParticleBatch {
    fn default() -> Self {
        Self::new()
    }
}
