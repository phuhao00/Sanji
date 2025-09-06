//! 鼠标输入处理

use winit::event::{MouseButton, ElementState};
use winit::dpi::PhysicalPosition;
use glam::Vec2;
use std::collections::HashMap;

/// 鼠标按键状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButtonState {
    Released,
    Pressed,
    JustPressed,
    JustReleased,
}

/// 鼠标状态管理器
#[derive(Debug)]
pub struct MouseState {
    /// 按键状态
    button_states: HashMap<MouseButton, MouseButtonState>,
    /// 按键持续时间
    button_durations: HashMap<MouseButton, f32>,
    /// 当前鼠标位置
    position: Vec2,
    /// 上一帧鼠标位置
    last_position: Vec2,
    /// 鼠标移动增量
    delta: Vec2,
    /// 滚轮增量
    scroll_delta: Vec2,
    /// 是否在窗口内
    is_in_window: bool,
}

impl MouseState {
    /// 创建新的鼠标状态
    pub fn new() -> Self {
        Self {
            button_states: HashMap::new(),
            button_durations: HashMap::new(),
            position: Vec2::ZERO,
            last_position: Vec2::ZERO,
            delta: Vec2::ZERO,
            scroll_delta: Vec2::ZERO,
            is_in_window: true,
        }
    }

    /// 更新鼠标状态 (每帧调用)
    pub fn update(&mut self) {
        // 更新按键状态
        for (_, state) in self.button_states.iter_mut() {
            match *state {
                MouseButtonState::JustPressed => *state = MouseButtonState::Pressed,
                MouseButtonState::JustReleased => *state = MouseButtonState::Released,
                _ => {}
            }
        }

        // 重置增量
        self.delta = Vec2::ZERO;
        self.scroll_delta = Vec2::ZERO;
    }

    /// 处理鼠标按键输入
    pub fn handle_button_input(&mut self, button: MouseButton, state: ElementState) {
        let new_state = match state {
            ElementState::Pressed => {
                let current_state = self.button_states.get(&button).unwrap_or(&MouseButtonState::Released);
                match current_state {
                    MouseButtonState::Released | MouseButtonState::JustReleased => MouseButtonState::JustPressed,
                    _ => MouseButtonState::Pressed,
                }
            }
            ElementState::Released => {
                let current_state = self.button_states.get(&button).unwrap_or(&MouseButtonState::Released);
                match current_state {
                    MouseButtonState::Pressed | MouseButtonState::JustPressed => MouseButtonState::JustReleased,
                    _ => MouseButtonState::Released,
                }
            }
        };

        self.button_states.insert(button, new_state);
        
        // 重置持续时间如果是新按下
        if new_state == MouseButtonState::JustPressed {
            self.button_durations.insert(button, 0.0);
        }
    }

    /// 处理鼠标移动
    pub fn handle_mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.last_position = self.position;
        self.position = Vec2::new(position.x as f32, position.y as f32);
        self.delta = self.position - self.last_position;
    }

    /// 处理滚轮滚动
    pub fn handle_scroll(&mut self, delta: Vec2) {
        self.scroll_delta += delta;
    }

    /// 设置鼠标是否在窗口内
    pub fn set_in_window(&mut self, in_window: bool) {
        self.is_in_window = in_window;
    }

    /// 检查按键是否被按下
    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        matches!(
            self.button_states.get(&button),
            Some(MouseButtonState::Pressed | MouseButtonState::JustPressed)
        )
    }

    /// 检查按键是否刚被按下
    pub fn is_button_just_pressed(&self, button: MouseButton) -> bool {
        matches!(
            self.button_states.get(&button),
            Some(MouseButtonState::JustPressed)
        )
    }

    /// 检查按键是否刚被释放
    pub fn is_button_just_released(&self, button: MouseButton) -> bool {
        matches!(
            self.button_states.get(&button),
            Some(MouseButtonState::JustReleased)
        )
    }

    /// 获取按键持续按下时间
    pub fn get_button_duration(&self, button: MouseButton) -> f32 {
        self.button_durations.get(&button).copied().unwrap_or(0.0)
    }

    /// 获取当前鼠标位置
    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// 获取鼠标移动增量
    pub fn delta(&self) -> Vec2 {
        self.delta
    }

    /// 获取滚轮增量
    pub fn scroll_delta(&self) -> Vec2 {
        self.scroll_delta
    }

    /// 检查鼠标是否在窗口内
    pub fn is_in_window(&self) -> bool {
        self.is_in_window
    }

    /// 获取上一帧鼠标位置
    pub fn last_position(&self) -> Vec2 {
        self.last_position
    }

    /// 重置所有状态
    pub fn reset(&mut self) {
        self.button_states.clear();
        self.button_durations.clear();
        self.position = Vec2::ZERO;
        self.last_position = Vec2::ZERO;
        self.delta = Vec2::ZERO;
        self.scroll_delta = Vec2::ZERO;
        self.is_in_window = true;
    }

    /// 获取所有当前按下的按键
    pub fn pressed_buttons(&self) -> Vec<MouseButton> {
        self.button_states
            .iter()
            .filter_map(|(&button, &state)| {
                if matches!(state, MouseButtonState::Pressed | MouseButtonState::JustPressed) {
                    Some(button)
                } else {
                    None
                }
            })
            .collect()
    }

    /// 检查是否有任何按键被按下
    pub fn any_button_pressed(&self) -> bool {
        self.button_states.values().any(|&state| {
            matches!(state, MouseButtonState::Pressed | MouseButtonState::JustPressed)
        })
    }

    /// 检查是否有任何按键刚被按下
    pub fn any_button_just_pressed(&self) -> bool {
        self.button_states.values().any(|&state| {
            matches!(state, MouseButtonState::JustPressed)
        })
    }

    /// 更新按键持续时间
    pub fn update_button_durations(&mut self, delta_time: f32) {
        for (&button, duration) in self.button_durations.iter_mut() {
            if self.is_button_pressed(button) {
                *duration += delta_time;
            }
        }
    }
}

impl Default for MouseState {
    fn default() -> Self {
        Self::new()
    }
}

/// 鼠标配置
#[derive(Debug, Clone)]
pub struct MouseConfig {
    /// 鼠标灵敏度
    pub sensitivity: f32,
    /// 是否反转Y轴
    pub invert_y: bool,
    /// 是否反转X轴
    pub invert_x: bool,
    /// 鼠标平滑度 (0.0 = 无平滑, 1.0 = 完全平滑)
    pub smoothing: f32,
    /// 死区大小
    pub dead_zone: f32,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            sensitivity: 1.0,
            invert_y: false,
            invert_x: false,
            smoothing: 0.0,
            dead_zone: 0.0,
        }
    }
}

/// 鼠标控制器
pub struct MouseController {
    config: MouseConfig,
    smoothed_delta: Vec2,
}

impl MouseController {
    /// 创建新的鼠标控制器
    pub fn new(config: MouseConfig) -> Self {
        Self {
            config,
            smoothed_delta: Vec2::ZERO,
        }
    }

    /// 处理鼠标输入并应用配置
    pub fn process_input(&mut self, raw_delta: Vec2) -> Vec2 {
        let mut processed_delta = raw_delta * self.config.sensitivity;

        // 应用反转
        if self.config.invert_x {
            processed_delta.x = -processed_delta.x;
        }
        if self.config.invert_y {
            processed_delta.y = -processed_delta.y;
        }

        // 应用死区
        if processed_delta.length() < self.config.dead_zone {
            processed_delta = Vec2::ZERO;
        }

        // 应用平滑
        if self.config.smoothing > 0.0 {
            self.smoothed_delta = self.smoothed_delta.lerp(processed_delta, 1.0 - self.config.smoothing);
            self.smoothed_delta
        } else {
            processed_delta
        }
    }

    /// 获取配置
    pub fn config(&self) -> &MouseConfig {
        &self.config
    }

    /// 设置配置
    pub fn set_config(&mut self, config: MouseConfig) {
        self.config = config;
    }

    /// 重置平滑状态
    pub fn reset_smoothing(&mut self) {
        self.smoothed_delta = Vec2::ZERO;
    }
}

impl Default for MouseController {
    fn default() -> Self {
        Self::new(MouseConfig::default())
    }
}
