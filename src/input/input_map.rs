//! 输入映射系统

use crate::input::{KeyboardState, MouseState, GamepadState, GamepadButton, GamepadAxis};
use winit::event::MouseButton;
use winit::keyboard::KeyCode;
use glam::Vec2;
use std::collections::HashMap;

/// 输入绑定类型
#[derive(Debug, Clone)]
pub enum InputBinding {
    /// 键盘按键
    Key(KeyCode),
    /// 鼠标按键
    Mouse(MouseButton),
    /// 游戏手柄按键
    GamepadButton(GamepadButton),
    /// 键盘轴 (两个按键控制一个轴)
    KeyAxis {
        negative_key: KeyCode,
        positive_key: KeyCode,
        negative_value: f32,
        positive_value: f32,
    },
    /// 鼠标轴
    MouseAxis {
        axis: MouseAxisType,
    },
    /// 游戏手柄轴
    GamepadAxis(GamepadAxis),
}

/// 鼠标轴类型
#[derive(Debug, Clone, Copy)]
pub enum MouseAxisType {
    X,
    Y,
    ScrollX,
    ScrollY,
}

/// 输入动作配置
#[derive(Debug, Clone)]
pub struct ActionConfig {
    /// 绑定列表
    bindings: Vec<InputBinding>,
    /// 是否需要所有绑定同时触发
    require_all: bool,
}

impl ActionConfig {
    /// 创建新的动作配置
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
            require_all: false,
        }
    }

    /// 添加绑定
    pub fn add_binding(mut self, binding: InputBinding) -> Self {
        self.bindings.push(binding);
        self
    }

    /// 设置是否需要所有绑定
    pub fn require_all(mut self, require_all: bool) -> Self {
        self.require_all = require_all;
        self
    }
}

/// 轴配置
#[derive(Debug, Clone)]
pub struct AxisConfig {
    /// 绑定
    binding: InputBinding,
    /// 是否反转
    invert: bool,
    /// 灵敏度
    sensitivity: f32,
    /// 死区
    deadzone: f32,
}

impl AxisConfig {
    /// 创建新的轴配置
    pub fn new(binding: InputBinding) -> Self {
        Self {
            binding,
            invert: false,
            sensitivity: 1.0,
            deadzone: 0.0,
        }
    }

    /// 设置反转
    pub fn invert(mut self, invert: bool) -> Self {
        self.invert = invert;
        self
    }

    /// 设置灵敏度
    pub fn sensitivity(mut self, sensitivity: f32) -> Self {
        self.sensitivity = sensitivity;
        self
    }

    /// 设置死区
    pub fn deadzone(mut self, deadzone: f32) -> Self {
        self.deadzone = deadzone;
        self
    }
}

/// 输入映射
#[derive(Debug)]
pub struct InputMap {
    /// 动作映射
    actions: HashMap<String, ActionConfig>,
    /// 轴映射
    axes: HashMap<String, AxisConfig>,
}

impl InputMap {
    /// 创建新的输入映射
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
            axes: HashMap::new(),
        }
    }

    /// 绑定键盘动作
    pub fn bind_key_action(&mut self, action_name: impl Into<String>, key: KeyCode) {
        let config = ActionConfig::new().add_binding(InputBinding::Key(key));
        self.actions.insert(action_name.into(), config);
    }

    /// 绑定鼠标动作
    pub fn bind_mouse_action(&mut self, action_name: impl Into<String>, button: MouseButton) {
        let config = ActionConfig::new().add_binding(InputBinding::Mouse(button));
        self.actions.insert(action_name.into(), config);
    }

    /// 绑定游戏手柄动作
    pub fn bind_gamepad_action(&mut self, action_name: impl Into<String>, button: GamepadButton) {
        let config = ActionConfig::new().add_binding(InputBinding::GamepadButton(button));
        self.actions.insert(action_name.into(), config);
    }

    /// 绑定键盘轴
    pub fn bind_key_axis(&mut self, 
        axis_name: impl Into<String>, 
        negative_key: KeyCode, 
        negative_value: f32,
        positive_key: KeyCode,
        positive_value: f32
    ) {
        let binding = InputBinding::KeyAxis {
            negative_key,
            positive_key,
            negative_value,
            positive_value,
        };
        let config = AxisConfig::new(binding);
        self.axes.insert(axis_name.into(), config);
    }

    /// 绑定鼠标轴
    pub fn bind_mouse_axis(&mut self, axis_name: impl Into<String> + AsRef<str>) {
        let axis_type = match axis_name.as_ref() {
            name if name.contains("x") => MouseAxisType::X,
            name if name.contains("y") => MouseAxisType::Y,
            name if name.contains("scroll_x") => MouseAxisType::ScrollX,
            name if name.contains("scroll_y") => MouseAxisType::ScrollY,
            _ => MouseAxisType::X,
        };
        
        let binding = InputBinding::MouseAxis { axis: axis_type };
        let config = AxisConfig::new(binding);
        self.axes.insert(axis_name.into(), config);
    }

    /// 绑定游戏手柄轴
    pub fn bind_gamepad_axis(&mut self, axis_name: impl Into<String>, axis: GamepadAxis) {
        let config = AxisConfig::new(InputBinding::GamepadAxis(axis));
        self.axes.insert(axis_name.into(), config);
    }

    /// 添加动作配置
    pub fn add_action(&mut self, action_name: impl Into<String>, config: ActionConfig) {
        self.actions.insert(action_name.into(), config);
    }

    /// 添加轴配置
    pub fn add_axis(&mut self, axis_name: impl Into<String>, config: AxisConfig) {
        self.axes.insert(axis_name.into(), config);
    }

    /// 检查动作是否被触发
    pub fn is_action_triggered(&self, action_name: &str, keyboard: &KeyboardState, mouse: &MouseState) -> bool {
        if let Some(config) = self.actions.get(action_name) {
            self.evaluate_action(config, keyboard, mouse, None)
        } else {
            false
        }
    }

    /// 检查动作是否刚被按下
    pub fn is_action_just_pressed(&self, action_name: &str, keyboard: &KeyboardState, mouse: &MouseState) -> bool {
        if let Some(config) = self.actions.get(action_name) {
            self.evaluate_action_just_pressed(config, keyboard, mouse, None)
        } else {
            false
        }
    }

    /// 检查动作是否刚被释放
    pub fn is_action_just_released(&self, action_name: &str, keyboard: &KeyboardState, mouse: &MouseState) -> bool {
        if let Some(config) = self.actions.get(action_name) {
            self.evaluate_action_just_released(config, keyboard, mouse, None)
        } else {
            false
        }
    }

    /// 获取轴值
    pub fn get_axis_value(&self, axis_name: &str, keyboard: &KeyboardState, mouse: &MouseState) -> f32 {
        if let Some(config) = self.axes.get(axis_name) {
            self.evaluate_axis(config, keyboard, mouse, None)
        } else {
            0.0
        }
    }

    /// 支持游戏手柄的动作检查
    pub fn is_action_triggered_with_gamepad(&self, 
        action_name: &str, 
        keyboard: &KeyboardState, 
        mouse: &MouseState,
        gamepad: Option<&GamepadState>
    ) -> bool {
        if let Some(config) = self.actions.get(action_name) {
            self.evaluate_action(config, keyboard, mouse, gamepad)
        } else {
            false
        }
    }

    /// 支持游戏手柄的轴值获取
    pub fn get_axis_value_with_gamepad(&self, 
        axis_name: &str, 
        keyboard: &KeyboardState, 
        mouse: &MouseState,
        gamepad: Option<&GamepadState>
    ) -> f32 {
        if let Some(config) = self.axes.get(axis_name) {
            self.evaluate_axis(config, keyboard, mouse, gamepad)
        } else {
            0.0
        }
    }

    /// 评估动作
    fn evaluate_action(&self, 
        config: &ActionConfig, 
        keyboard: &KeyboardState, 
        mouse: &MouseState,
        gamepad: Option<&GamepadState>
    ) -> bool {
        let results: Vec<bool> = config.bindings.iter().map(|binding| {
            match binding {
                InputBinding::Key(key) => keyboard.is_key_just_pressed(*key),
                InputBinding::Mouse(button) => mouse.is_button_pressed(*button),
                InputBinding::GamepadButton(button) => {
                    gamepad.map_or(false, |g| g.is_button_pressed(*button))
                },
                _ => false,
            }
        }).collect();

        if config.require_all {
            results.iter().all(|&result| result)
        } else {
            results.iter().any(|&result| result)
        }
    }

    /// 评估动作刚按下
    fn evaluate_action_just_pressed(&self, 
        config: &ActionConfig, 
        keyboard: &KeyboardState, 
        mouse: &MouseState,
        gamepad: Option<&GamepadState>
    ) -> bool {
        let results: Vec<bool> = config.bindings.iter().map(|binding| {
            match binding {
                InputBinding::Key(key) => keyboard.is_key_just_pressed(*key),
                InputBinding::Mouse(button) => mouse.is_button_just_pressed(*button),
                InputBinding::GamepadButton(button) => {
                    gamepad.map_or(false, |g| g.is_button_just_pressed(*button))
                },
                _ => false,
            }
        }).collect();

        if config.require_all {
            results.iter().all(|&result| result)
        } else {
            results.iter().any(|&result| result)
        }
    }

    /// 评估动作刚释放
    fn evaluate_action_just_released(&self, 
        config: &ActionConfig, 
        keyboard: &KeyboardState, 
        mouse: &MouseState,
        gamepad: Option<&GamepadState>
    ) -> bool {
        let results: Vec<bool> = config.bindings.iter().map(|binding| {
            match binding {
                InputBinding::Key(key) => keyboard.is_key_just_released(*key),
                InputBinding::Mouse(button) => mouse.is_button_just_released(*button),
                InputBinding::GamepadButton(button) => {
                    gamepad.map_or(false, |g| g.is_button_just_released(*button))
                },
                _ => false,
            }
        }).collect();

        if config.require_all {
            results.iter().all(|&result| result)
        } else {
            results.iter().any(|&result| result)
        }
    }

    /// 评估轴
    fn evaluate_axis(&self, 
        config: &AxisConfig, 
        keyboard: &KeyboardState, 
        mouse: &MouseState,
        gamepad: Option<&GamepadState>
    ) -> f32 {
        let mut value = match &config.binding {
            InputBinding::KeyAxis { negative_key, positive_key, negative_value, positive_value } => {
                let mut axis_value = 0.0;
                if keyboard.is_key_just_pressed(*negative_key) {
                    axis_value += *negative_value;
                }
                if keyboard.is_key_just_pressed(*positive_key) {
                    axis_value += *positive_value;
                }
                axis_value
            },
            InputBinding::MouseAxis { axis } => {
                match axis {
                    MouseAxisType::X => mouse.delta().x,
                    MouseAxisType::Y => mouse.delta().y,
                    MouseAxisType::ScrollX => mouse.scroll_delta().x,
                    MouseAxisType::ScrollY => mouse.scroll_delta().y,
                }
            },
            InputBinding::GamepadAxis(axis) => {
                gamepad.map_or(0.0, |g| g.get_axis_value(*axis))
            },
            _ => 0.0,
        };

        // 应用死区
        if value.abs() < config.deadzone {
            value = 0.0;
        }

        // 应用灵敏度
        value *= config.sensitivity;

        // 应用反转
        if config.invert {
            value = -value;
        }

        value
    }

    /// 移除动作
    pub fn remove_action(&mut self, action_name: &str) {
        self.actions.remove(action_name);
    }

    /// 移除轴
    pub fn remove_axis(&mut self, axis_name: &str) {
        self.axes.remove(axis_name);
    }

    /// 清空所有绑定
    pub fn clear(&mut self) {
        self.actions.clear();
        self.axes.clear();
    }

    /// 获取所有动作名称
    pub fn action_names(&self) -> Vec<&String> {
        self.actions.keys().collect()
    }

    /// 获取所有轴名称
    pub fn axis_names(&self) -> Vec<&String> {
        self.axes.keys().collect()
    }
}

impl Default for InputMap {
    fn default() -> Self {
        Self::new()
    }
}
