//! 音频监听器组件

use crate::math::Vec3;
use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};
use specs_derive::Component;

/// 音频监听器 - 3D音频系统中的"耳朵"
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct AudioListener {
    /// 监听器位置
    pub position: Vec3,
    /// 前方向量
    pub forward: Vec3,
    /// 上方向量
    pub up: Vec3,
    /// 右方向量（计算得出）
    #[serde(skip)]
    pub right: Vec3,
    /// 速度（用于多普勒效应）
    pub velocity: Vec3,
    /// 音量比例
    pub volume_scale: f32,
    /// 是否是主监听器
    pub is_main: bool,
    /// 音频过滤设置
    pub low_pass_cutoff: f32,
    pub high_pass_cutoff: f32,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            forward: Vec3::new(0.0, 0.0, -1.0), // 朝向-Z方向
            up: Vec3::new(0.0, 1.0, 0.0),       // 上方向+Y
            right: Vec3::new(1.0, 0.0, 0.0),    // 右方向+X
            velocity: Vec3::ZERO,
            volume_scale: 1.0,
            is_main: true,
            low_pass_cutoff: 22000.0,  // Hz
            high_pass_cutoff: 10.0,    // Hz
        }
    }
}

impl AudioListener {
    /// 创建新的音频监听器
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建主监听器
    pub fn main() -> Self {
        Self {
            is_main: true,
            ..Default::default()
        }
    }

    /// 设置位置
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// 设置方向（前方和上方向量）
    pub fn set_orientation(&mut self, forward: Vec3, up: Vec3) {
        self.forward = forward.normalize();
        self.up = up.normalize();
        self.right = self.forward.cross(self.up).normalize();
        
        // 确保up向量与forward垂直
        self.up = self.right.cross(self.forward).normalize();
    }

    /// 设置速度
    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.velocity = velocity;
    }

    /// 设置音量比例
    pub fn set_volume_scale(&mut self, scale: f32) {
        self.volume_scale = scale.clamp(0.0, 2.0);
    }

    /// 看向指定点
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let forward = (target - self.position).normalize();
        self.set_orientation(forward, up);
    }

    /// 获取位置
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// 获取前方向量
    pub fn forward(&self) -> Vec3 {
        self.forward
    }

    /// 获取上方向量
    pub fn up(&self) -> Vec3 {
        self.up
    }

    /// 获取右方向量
    pub fn right(&self) -> Vec3 {
        self.right
    }

    /// 获取速度
    pub fn velocity(&self) -> Vec3 {
        self.velocity
    }

    /// 计算到音源的方向信息
    pub fn calculate_source_direction(&self, source_position: Vec3) -> AudioDirectionInfo {
        let relative_position = source_position - self.position;
        let distance = relative_position.length();
        
        if distance < f32::EPSILON {
            return AudioDirectionInfo::default();
        }
        
        let direction = relative_position / distance;
        
        // 计算左右平衡 (-1.0 = 完全左边, 1.0 = 完全右边)
        let pan = direction.dot(self.right);
        
        // 计算前后位置 (1.0 = 前方, -1.0 = 后方)
        let front_back = direction.dot(self.forward);
        
        // 计算上下位置 (1.0 = 上方, -1.0 = 下方)
        let up_down = direction.dot(self.up);
        
        AudioDirectionInfo {
            distance,
            pan: pan.clamp(-1.0, 1.0),
            front_back,
            up_down,
            direction,
        }
    }

    /// 计算立体声平衡
    pub fn calculate_stereo_pan(&self, source_position: Vec3) -> f32 {
        let direction_info = self.calculate_source_direction(source_position);
        direction_info.pan
    }

    /// 应用音频过滤
    pub fn apply_audio_filter(&self, distance: f32, base_frequency: f32) -> AudioFilterInfo {
        // 距离越远，高频衰减越多
        let distance_factor = (distance / 100.0).clamp(0.0, 1.0);
        let low_pass = self.low_pass_cutoff * (1.0 - distance_factor * 0.8);
        let high_pass = self.high_pass_cutoff;
        
        AudioFilterInfo {
            low_pass_cutoff: low_pass,
            high_pass_cutoff: high_pass,
            reverb_amount: distance_factor * 0.3,
        }
    }

    /// 设置为主监听器
    pub fn set_as_main(&mut self) {
        self.is_main = true;
    }

    /// 检查是否是主监听器
    pub fn is_main(&self) -> bool {
        self.is_main
    }

    /// 获取变换矩阵
    pub fn transform_matrix(&self) -> glam::Mat4 {
        let rotation = glam::Mat3::from_cols(
            self.right,
            self.up,
            -self.forward // OpenGL风格，相机看向-Z方向
        );
        
        glam::Mat4::from_rotation_translation(glam::Quat::from_mat3(&rotation), self.position)
    }

    /// 世界坐标转听音器坐标
    pub fn world_to_listener(&self, world_position: Vec3) -> Vec3 {
        let relative_pos = world_position - self.position;
        Vec3::new(
            relative_pos.dot(self.right),
            relative_pos.dot(self.up),
            -relative_pos.dot(self.forward)
        )
    }

    /// 听音器坐标转世界坐标
    pub fn listener_to_world(&self, listener_position: Vec3) -> Vec3 {
        self.position + 
            self.right * listener_position.x +
            self.up * listener_position.y +
            self.forward * (-listener_position.z)
    }
}

/// 音频方向信息
#[derive(Debug, Clone)]
pub struct AudioDirectionInfo {
    /// 距离
    pub distance: f32,
    /// 左右平衡 (-1.0 到 1.0)
    pub pan: f32,
    /// 前后位置 (-1.0 到 1.0)
    pub front_back: f32,
    /// 上下位置 (-1.0 到 1.0)
    pub up_down: f32,
    /// 归一化方向向量
    pub direction: Vec3,
}

impl Default for AudioDirectionInfo {
    fn default() -> Self {
        Self {
            distance: 0.0,
            pan: 0.0,
            front_back: 1.0,
            up_down: 0.0,
            direction: Vec3::new(0.0, 0.0, -1.0),
        }
    }
}

/// 音频过滤信息
#[derive(Debug, Clone)]
pub struct AudioFilterInfo {
    /// 低通滤波器截止频率
    pub low_pass_cutoff: f32,
    /// 高通滤波器截止频率
    pub high_pass_cutoff: f32,
    /// 混响量 (0.0 - 1.0)
    pub reverb_amount: f32,
}

/// 监听器预设
pub struct AudioListenerPresets;

impl AudioListenerPresets {
    /// 第一人称视角监听器
    pub fn first_person() -> AudioListener {
        AudioListener {
            volume_scale: 1.0,
            is_main: true,
            ..Default::default()
        }
    }

    /// 第三人称视角监听器
    pub fn third_person() -> AudioListener {
        AudioListener {
            volume_scale: 0.8,
            is_main: true,
            ..Default::default()
        }
    }

    /// 电影模式监听器（更宽的立体声场）
    pub fn cinematic() -> AudioListener {
        AudioListener {
            volume_scale: 1.2,
            is_main: true,
            low_pass_cutoff: 20000.0,
            high_pass_cutoff: 20.0,
            ..Default::default()
        }
    }

    /// 环境监听器（用于环境音效）
    pub fn ambient() -> AudioListener {
        AudioListener {
            volume_scale: 0.6,
            is_main: false,
            low_pass_cutoff: 8000.0,
            high_pass_cutoff: 50.0,
            ..Default::default()
        }
    }
}

/// 监听器管理器
pub struct ListenerManager {
    listeners: Vec<AudioListener>,
    main_listener_index: Option<usize>,
}

impl ListenerManager {
    /// 创建新的监听器管理器
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
            main_listener_index: None,
        }
    }

    /// 添加监听器
    pub fn add_listener(&mut self, listener: AudioListener) -> usize {
        let index = self.listeners.len();
        
        if listener.is_main {
            // 如果新监听器是主监听器，取消其他主监听器状态
            for existing_listener in &mut self.listeners {
                existing_listener.is_main = false;
            }
            self.main_listener_index = Some(index);
        }
        
        self.listeners.push(listener);
        index
    }

    /// 获取主监听器
    pub fn main_listener(&self) -> Option<&AudioListener> {
        self.main_listener_index.and_then(|i| self.listeners.get(i))
    }

    /// 获取主监听器的可变引用
    pub fn main_listener_mut(&mut self) -> Option<&mut AudioListener> {
        self.main_listener_index.and_then(|i| self.listeners.get_mut(i))
    }

    /// 设置主监听器
    pub fn set_main_listener(&mut self, index: usize) -> bool {
        if index < self.listeners.len() {
            // 清除所有主监听器标记
            for listener in &mut self.listeners {
                listener.is_main = false;
            }
            
            // 设置新的主监听器
            self.listeners[index].is_main = true;
            self.main_listener_index = Some(index);
            true
        } else {
            false
        }
    }

    /// 获取所有监听器
    pub fn listeners(&self) -> &[AudioListener] {
        &self.listeners
    }

    /// 移除监听器
    pub fn remove_listener(&mut self, index: usize) -> Option<AudioListener> {
        if index < self.listeners.len() {
            let removed = self.listeners.remove(index);
            
            // 如果移除的是主监听器，重新分配主监听器
            if Some(index) == self.main_listener_index {
                self.main_listener_index = None;
                // 尝试将第一个监听器设为主监听器
                if !self.listeners.is_empty() {
                    self.listeners[0].is_main = true;
                    self.main_listener_index = Some(0);
                }
            }
            
            Some(removed)
        } else {
            None
        }
    }

    /// 清空所有监听器
    pub fn clear(&mut self) {
        self.listeners.clear();
        self.main_listener_index = None;
    }

    /// 监听器数量
    pub fn count(&self) -> usize {
        self.listeners.len()
    }
}

impl Default for ListenerManager {
    fn default() -> Self {
        let mut manager = Self::new();
        manager.add_listener(AudioListener::main());
        manager
    }
}
