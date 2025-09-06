//! 游戏手柄输入处理

use glam::Vec2;
use std::collections::HashMap;

/// 游戏手柄按键
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    // 面部按键
    South,      // A / X
    East,       // B / Circle  
    West,       // X / Square
    North,      // Y / Triangle
    
    // 肩部按键
    LeftBumper,
    RightBumper,
    
    // 扳机按键
    LeftTrigger,
    RightTrigger,
    
    // 方向键
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    
    // 摇杆按键
    LeftStick,
    RightStick,
    
    // 系统按键
    Select,     // Back / Share
    Start,      // Menu / Options
    Home,       // Guide / PS
    
    // 自定义按键
    Custom(u8),
}

/// 游戏手柄轴
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
    Custom(u8),
}

/// 游戏手柄按键状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamepadButtonState {
    Released,
    Pressed,
    JustPressed,
    JustReleased,
}

/// 游戏手柄状态
#[derive(Debug, Clone)]
pub struct GamepadState {
    /// 按键状态
    button_states: HashMap<GamepadButton, GamepadButtonState>,
    /// 轴值
    axis_values: HashMap<GamepadAxis, f32>,
    /// 是否连接
    connected: bool,
    /// 手柄ID
    id: u32,
    /// 手柄名称
    name: String,
}

impl GamepadState {
    /// 创建新的游戏手柄状态
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            button_states: HashMap::new(),
            axis_values: HashMap::new(),
            connected: true,
            id,
            name: name.into(),
        }
    }

    /// 更新按键状态
    pub fn update(&mut self) {
        for (_, state) in self.button_states.iter_mut() {
            match *state {
                GamepadButtonState::JustPressed => *state = GamepadButtonState::Pressed,
                GamepadButtonState::JustReleased => *state = GamepadButtonState::Released,
                _ => {}
            }
        }
    }

    /// 设置按键状态
    pub fn set_button_state(&mut self, button: GamepadButton, pressed: bool) {
        let current_state = self.button_states.get(&button).unwrap_or(&GamepadButtonState::Released);
        
        let new_state = match (pressed, current_state) {
            (true, GamepadButtonState::Released | GamepadButtonState::JustReleased) => GamepadButtonState::JustPressed,
            (true, _) => GamepadButtonState::Pressed,
            (false, GamepadButtonState::Pressed | GamepadButtonState::JustPressed) => GamepadButtonState::JustReleased,
            (false, _) => GamepadButtonState::Released,
        };
        
        self.button_states.insert(button, new_state);
    }

    /// 设置轴值
    pub fn set_axis_value(&mut self, axis: GamepadAxis, value: f32) {
        self.axis_values.insert(axis, value);
    }

    /// 检查按键是否被按下
    pub fn is_button_pressed(&self, button: GamepadButton) -> bool {
        matches!(
            self.button_states.get(&button),
            Some(GamepadButtonState::Pressed | GamepadButtonState::JustPressed)
        )
    }

    /// 检查按键是否刚被按下
    pub fn is_button_just_pressed(&self, button: GamepadButton) -> bool {
        matches!(
            self.button_states.get(&button),
            Some(GamepadButtonState::JustPressed)
        )
    }

    /// 检查按键是否刚被释放
    pub fn is_button_just_released(&self, button: GamepadButton) -> bool {
        matches!(
            self.button_states.get(&button),
            Some(GamepadButtonState::JustReleased)
        )
    }

    /// 获取轴值
    pub fn get_axis_value(&self, axis: GamepadAxis) -> f32 {
        self.axis_values.get(&axis).copied().unwrap_or(0.0)
    }

    /// 获取左摇杆值
    pub fn left_stick(&self) -> Vec2 {
        Vec2::new(
            self.get_axis_value(GamepadAxis::LeftStickX),
            self.get_axis_value(GamepadAxis::LeftStickY)
        )
    }

    /// 获取右摇杆值
    pub fn right_stick(&self) -> Vec2 {
        Vec2::new(
            self.get_axis_value(GamepadAxis::RightStickX),
            self.get_axis_value(GamepadAxis::RightStickY)
        )
    }

    /// 获取扳机值
    pub fn triggers(&self) -> Vec2 {
        Vec2::new(
            self.get_axis_value(GamepadAxis::LeftTrigger),
            self.get_axis_value(GamepadAxis::RightTrigger)
        )
    }

    /// 是否连接
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// 设置连接状态
    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }

    /// 获取ID
    pub fn id(&self) -> u32 {
        self.id
    }

    /// 获取名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 重置所有状态
    pub fn reset(&mut self) {
        self.button_states.clear();
        self.axis_values.clear();
    }
}

/// 游戏手柄管理器
#[derive(Debug)]
pub struct GamepadManager {
    gamepads: HashMap<u32, GamepadState>,
    next_id: u32,
}

impl GamepadManager {
    /// 创建新的游戏手柄管理器
    pub fn new() -> Self {
        Self {
            gamepads: HashMap::new(),
            next_id: 0,
        }
    }

    /// 更新所有游戏手柄
    pub fn update(&mut self) {
        for gamepad in self.gamepads.values_mut() {
            gamepad.update();
        }
    }

    /// 连接新的游戏手柄
    pub fn connect_gamepad(&mut self, name: impl Into<String>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let gamepad = GamepadState::new(id, name);
        self.gamepads.insert(id, gamepad);
        
        id
    }

    /// 断开游戏手柄
    pub fn disconnect_gamepad(&mut self, id: u32) {
        if let Some(gamepad) = self.gamepads.get_mut(&id) {
            gamepad.set_connected(false);
        }
    }

    /// 移除游戏手柄
    pub fn remove_gamepad(&mut self, id: u32) {
        self.gamepads.remove(&id);
    }

    /// 获取游戏手柄
    pub fn get_gamepad(&self, id: u32) -> Option<&GamepadState> {
        self.gamepads.get(&id)
    }

    /// 获取可变游戏手柄
    pub fn get_gamepad_mut(&mut self, id: u32) -> Option<&mut GamepadState> {
        self.gamepads.get_mut(&id)
    }

    /// 获取第一个连接的游戏手柄
    pub fn first_gamepad(&self) -> Option<&GamepadState> {
        self.gamepads.values().find(|g| g.is_connected())
    }

    /// 获取所有连接的游戏手柄
    pub fn connected_gamepads(&self) -> Vec<&GamepadState> {
        self.gamepads.values().filter(|g| g.is_connected()).collect()
    }

    /// 获取所有游戏手柄ID
    pub fn gamepad_ids(&self) -> Vec<u32> {
        self.gamepads.keys().copied().collect()
    }

    /// 检查是否有游戏手柄连接
    pub fn has_gamepad(&self) -> bool {
        self.gamepads.values().any(|g| g.is_connected())
    }

    /// 获取连接的游戏手柄数量
    pub fn connected_count(&self) -> usize {
        self.gamepads.values().filter(|g| g.is_connected()).count()
    }
}

impl Default for GamepadManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 游戏手柄配置
#[derive(Debug, Clone)]
pub struct GamepadConfig {
    /// 摇杆死区
    pub stick_deadzone: f32,
    /// 扳机死区
    pub trigger_deadzone: f32,
    /// 是否反转Y轴
    pub invert_y: bool,
    /// 振动强度
    pub vibration_strength: f32,
    /// 是否启用振动
    pub vibration_enabled: bool,
}

impl Default for GamepadConfig {
    fn default() -> Self {
        Self {
            stick_deadzone: 0.1,
            trigger_deadzone: 0.05,
            invert_y: false,
            vibration_strength: 1.0,
            vibration_enabled: true,
        }
    }
}

/// 应用死区处理
pub fn apply_deadzone(value: f32, deadzone: f32) -> f32 {
    if value.abs() < deadzone {
        0.0
    } else {
        // 重新映射到去除死区后的范围
        let sign = value.signum();
        let abs_value = value.abs();
        let adjusted = (abs_value - deadzone) / (1.0 - deadzone);
        sign * adjusted.clamp(0.0, 1.0)
    }
}

/// 应用向量死区处理
pub fn apply_vector_deadzone(vec: Vec2, deadzone: f32) -> Vec2 {
    let magnitude = vec.length();
    if magnitude < deadzone {
        Vec2::ZERO
    } else {
        let adjusted_magnitude = (magnitude - deadzone) / (1.0 - deadzone);
        vec.normalize() * adjusted_magnitude.clamp(0.0, 1.0)
    }
}
