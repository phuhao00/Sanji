//! 键盘输入处理

use winit::event::{KeyEvent, ElementState};
use winit::keyboard::{KeyCode, PhysicalKey};
use std::collections::HashSet;

/// 键盘状态管理器
pub struct KeyboardState {
    /// 当前按下的键
    current_keys: HashSet<KeyCode>,
    /// 上一帧按下的键
    previous_keys: HashSet<KeyCode>,
    /// 刚按下的键
    just_pressed: HashSet<KeyCode>,
    /// 刚释放的键
    just_released: HashSet<KeyCode>,
}

impl KeyboardState {
    /// 创建新的键盘状态
    pub fn new() -> Self {
        Self {
            current_keys: HashSet::new(),
            previous_keys: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
        }
    }

    /// 处理键盘事件
    pub fn handle_key_event(&mut self, event: KeyEvent) {
        if let PhysicalKey::Code(key_code) = event.physical_key {
            match event.state {
                ElementState::Pressed => {
                    if !self.current_keys.contains(&key_code) {
                        self.current_keys.insert(key_code);
                        self.just_pressed.insert(key_code);
                    }
                }
                ElementState::Released => {
                    if self.current_keys.contains(&key_code) {
                        self.current_keys.remove(&key_code);
                        self.just_released.insert(key_code);
                    }
                }
            }
        }
    }

    /// 更新状态 (每帧调用)
    pub fn update(&mut self) {
        self.previous_keys.clone_from(&self.current_keys);
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// 检查键是否当前被按下
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.current_keys.contains(&key)
    }

    /// 检查键是否没有被按下
    pub fn is_key_up(&self, key: KeyCode) -> bool {
        !self.current_keys.contains(&key)
    }

    /// 检查键是否刚被按下
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    /// 检查键是否刚被释放
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }

    /// 检查键在上一帧是否被按下
    pub fn was_key_down(&self, key: KeyCode) -> bool {
        self.previous_keys.contains(&key)
    }

    /// 获取所有当前按下的键
    pub fn pressed_keys(&self) -> Vec<KeyCode> {
        self.current_keys.iter().copied().collect()
    }

    /// 获取所有刚按下的键
    pub fn just_pressed_keys(&self) -> Vec<KeyCode> {
        self.just_pressed.iter().copied().collect()
    }

    /// 获取所有刚释放的键
    pub fn just_released_keys(&self) -> Vec<KeyCode> {
        self.just_released.iter().copied().collect()
    }

    /// 检查是否有任何键被按下
    pub fn any_key_down(&self) -> bool {
        !self.current_keys.is_empty()
    }

    /// 检查是否有任何键刚被按下
    pub fn any_key_just_pressed(&self) -> bool {
        !self.just_pressed.is_empty()
    }

    /// 检查多个键是否同时被按下
    pub fn are_keys_down(&self, keys: &[KeyCode]) -> bool {
        keys.iter().all(|key| self.is_key_down(*key))
    }

    /// 检查多个键中是否有任何一个被按下
    pub fn any_keys_down(&self, keys: &[KeyCode]) -> bool {
        keys.iter().any(|key| self.is_key_down(*key))
    }

    /// 重置所有状态
    pub fn reset(&mut self) {
        self.current_keys.clear();
        self.previous_keys.clear();
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// 模拟按键按下 (用于测试)
    pub fn simulate_key_press(&mut self, key: KeyCode) {
        if !self.current_keys.contains(&key) {
            self.current_keys.insert(key);
            self.just_pressed.insert(key);
        }
    }

    /// 模拟按键释放 (用于测试)
    pub fn simulate_key_release(&mut self, key: KeyCode) {
        if self.current_keys.contains(&key) {
            self.current_keys.remove(&key);
            self.just_released.insert(key);
        }
    }
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
    }
}

/// 键盘快捷键组合
#[derive(Debug, Clone, PartialEq)]
pub struct KeyCombination {
    pub keys: Vec<KeyCode>,
    pub require_exact: bool, // 是否要求精确匹配 (不能有其他键按下)
}

impl KeyCombination {
    /// 创建新的键盘组合
    pub fn new(keys: Vec<KeyCode>) -> Self {
        Self {
            keys,
            require_exact: false,
        }
    }

    /// 创建精确匹配的键盘组合
    pub fn exact(keys: Vec<KeyCode>) -> Self {
        Self {
            keys,
            require_exact: true,
        }
    }

    /// 单个键
    pub fn single(key: KeyCode) -> Self {
        Self::new(vec![key])
    }

    /// 检查组合是否被触发
    pub fn is_triggered(&self, keyboard: &KeyboardState) -> bool {
        if self.require_exact {
            let pressed_keys = keyboard.pressed_keys();
            pressed_keys.len() == self.keys.len() && 
            self.keys.iter().all(|key| pressed_keys.contains(key))
        } else {
            keyboard.are_keys_down(&self.keys)
        }
    }

    /// 检查组合是否刚被触发
    pub fn is_just_triggered(&self, keyboard: &KeyboardState) -> bool {
        if self.keys.is_empty() {
            return false;
        }

        // 至少有一个键刚被按下，并且所有键都当前被按下
        self.keys.iter().any(|key| keyboard.is_key_just_pressed(*key)) &&
        self.keys.iter().all(|key| keyboard.is_key_down(*key))
    }
}

/// 常用键盘组合
pub struct CommonKeyCombinations;

impl CommonKeyCombinations {
    /// Ctrl+C
    pub fn copy() -> KeyCombination {
        KeyCombination::new(vec![KeyCode::ControlLeft, KeyCode::KeyC])
    }

    /// Ctrl+V
    pub fn paste() -> KeyCombination {
        KeyCombination::new(vec![KeyCode::ControlLeft, KeyCode::KeyV])
    }

    /// Ctrl+Z
    pub fn undo() -> KeyCombination {
        KeyCombination::new(vec![KeyCode::ControlLeft, KeyCode::KeyZ])
    }

    /// Ctrl+Y 或 Ctrl+Shift+Z
    pub fn redo() -> Vec<KeyCombination> {
        vec![
            KeyCombination::new(vec![KeyCode::ControlLeft, KeyCode::KeyY]),
            KeyCombination::new(vec![KeyCode::ControlLeft, KeyCode::ShiftLeft, KeyCode::KeyZ]),
        ]
    }

    /// Alt+Tab
    pub fn alt_tab() -> KeyCombination {
        KeyCombination::new(vec![KeyCode::AltLeft, KeyCode::Tab])
    }

    /// Alt+F4
    pub fn alt_f4() -> KeyCombination {
        KeyCombination::new(vec![KeyCode::AltLeft, KeyCode::F4])
    }

    /// 方向键组合
    pub fn arrow_keys() -> Vec<KeyCode> {
        vec![KeyCode::ArrowUp, KeyCode::ArrowDown, KeyCode::ArrowLeft, KeyCode::ArrowRight]
    }

    /// WASD键
    pub fn wasd_keys() -> Vec<KeyCode> {
        vec![KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD]
    }

    /// 功能键
    pub fn function_keys() -> Vec<KeyCode> {
        vec![
            KeyCode::F1, KeyCode::F2, KeyCode::F3, KeyCode::F4,
            KeyCode::F5, KeyCode::F6, KeyCode::F7, KeyCode::F8,
            KeyCode::F9, KeyCode::F10, KeyCode::F11, KeyCode::F12,
        ]
    }

    /// 数字键
    pub fn number_keys() -> Vec<KeyCode> {
        vec![
            KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5,
            KeyCode::Digit6, KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9, KeyCode::Digit0,
        ]
    }

    /// 字母键
    pub fn letter_keys() -> Vec<KeyCode> {
        vec![
            KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC, KeyCode::KeyD, KeyCode::KeyE,
            KeyCode::KeyF, KeyCode::KeyG, KeyCode::KeyH, KeyCode::KeyI, KeyCode::KeyJ,
            KeyCode::KeyK, KeyCode::KeyL, KeyCode::KeyM, KeyCode::KeyN, KeyCode::KeyO,
            KeyCode::KeyP, KeyCode::KeyQ, KeyCode::KeyR, KeyCode::KeyS, KeyCode::KeyT,
            KeyCode::KeyU, KeyCode::KeyV, KeyCode::KeyW, KeyCode::KeyX, KeyCode::KeyY,
            KeyCode::KeyZ,
        ]
    }
}
