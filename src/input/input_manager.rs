//! 输入管理器

use crate::input::{KeyboardState, MouseState, InputMap};
use winit::event::{KeyEvent, MouseButton, ElementState};
use winit::dpi::PhysicalPosition;
use std::collections::HashMap;

/// 输入管理器 - 管理所有输入设备的状态
pub struct InputManager {
    keyboard: KeyboardState,
    mouse: MouseState,
    input_maps: HashMap<String, InputMap>,
    current_input_map: Option<String>,
}

impl InputManager {
    /// 创建新的输入管理器
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardState::new(),
            mouse: MouseState::new(),
            input_maps: HashMap::new(),
            current_input_map: None,
        }
    }

    /// 更新输入状态 (每帧调用)
    pub fn update(&mut self) {
        self.keyboard.update();
        self.mouse.update();
    }

    /// 处理键盘输入事件
    pub fn handle_keyboard_input(&mut self, event: KeyEvent) {
        self.keyboard.handle_key_event(event);
    }

    /// 处理鼠标按键事件
    pub fn handle_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        self.mouse.handle_button_input(button, state);
    }

    /// 处理鼠标移动事件
    pub fn handle_mouse_move(&mut self, position: PhysicalPosition<f64>) {
        self.mouse.handle_mouse_move(position);
    }

    /// 获取键盘状态
    pub fn keyboard(&self) -> &KeyboardState {
        &self.keyboard
    }

    /// 获取鼠标状态
    pub fn mouse(&self) -> &MouseState {
        &self.mouse
    }

    /// 添加输入映射
    pub fn add_input_map(&mut self, name: impl Into<String>, input_map: InputMap) {
        self.input_maps.insert(name.into(), input_map);
    }

    /// 设置当前输入映射
    pub fn set_current_input_map(&mut self, name: impl Into<String>) {
        let name = name.into();
        if self.input_maps.contains_key(&name) {
            self.current_input_map = Some(name);
        }
    }

    /// 获取当前输入映射
    pub fn current_input_map(&self) -> Option<&InputMap> {
        self.current_input_map.as_ref()
            .and_then(|name| self.input_maps.get(name))
    }

    /// 检查动作是否被触发
    pub fn is_action_triggered(&self, action_name: &str) -> bool {
        if let Some(input_map) = self.current_input_map() {
            input_map.is_action_triggered(action_name, &self.keyboard, &self.mouse)
        } else {
            false
        }
    }

    /// 检查动作是否刚被按下
    pub fn is_action_just_pressed(&self, action_name: &str) -> bool {
        if let Some(input_map) = self.current_input_map() {
            input_map.is_action_just_pressed(action_name, &self.keyboard, &self.mouse)
        } else {
            false
        }
    }

    /// 检查动作是否刚被释放
    pub fn is_action_just_released(&self, action_name: &str) -> bool {
        if let Some(input_map) = self.current_input_map() {
            input_map.is_action_just_released(action_name, &self.keyboard, &self.mouse)
        } else {
            false
        }
    }

    /// 获取轴的值
    pub fn get_axis(&self, axis_name: &str) -> f32 {
        if let Some(input_map) = self.current_input_map() {
            input_map.get_axis_value(axis_name, &self.keyboard, &self.mouse)
        } else {
            0.0
        }
    }

    /// 获取2D向量输入
    pub fn get_vector2d(&self, x_axis: &str, y_axis: &str) -> glam::Vec2 {
        glam::Vec2::new(
            self.get_axis(x_axis),
            self.get_axis(y_axis)
        )
    }

    /// 重置所有输入状态
    pub fn reset(&mut self) {
        self.keyboard.reset();
        self.mouse.reset();
    }

    /// 创建默认输入映射
    pub fn create_default_input_map(&mut self) {
        let mut input_map = InputMap::new();

        // 移动控制
        input_map.bind_key_action("move_forward", winit::keyboard::KeyCode::KeyW);
        input_map.bind_key_action("move_backward", winit::keyboard::KeyCode::KeyS);
        input_map.bind_key_action("move_left", winit::keyboard::KeyCode::KeyA);
        input_map.bind_key_action("move_right", winit::keyboard::KeyCode::KeyD);
        
        // 跳跃和蹲下
        input_map.bind_key_action("jump", winit::keyboard::KeyCode::Space);
        input_map.bind_key_action("crouch", winit::keyboard::KeyCode::ControlLeft);
        
        // 鼠标控制
        input_map.bind_mouse_action("primary_action", MouseButton::Left);
        input_map.bind_mouse_action("secondary_action", MouseButton::Right);
        
        // 轴控制
        input_map.bind_key_axis("horizontal", 
            winit::keyboard::KeyCode::KeyA, -1.0,
            winit::keyboard::KeyCode::KeyD, 1.0
        );
        input_map.bind_key_axis("vertical", 
            winit::keyboard::KeyCode::KeyS, -1.0,
            winit::keyboard::KeyCode::KeyW, 1.0
        );
        
        // 鼠标轴
        input_map.bind_mouse_axis("mouse_x");
        input_map.bind_mouse_axis("mouse_y");

        self.add_input_map("default", input_map);
        self.set_current_input_map("default");
    }

    /// 获取所有输入映射名称
    pub fn input_map_names(&self) -> Vec<&String> {
        self.input_maps.keys().collect()
    }

    /// 移除输入映射
    pub fn remove_input_map(&mut self, name: &str) {
        if let Some(current_name) = &self.current_input_map {
            if current_name == name {
                self.current_input_map = None;
            }
        }
        self.input_maps.remove(name);
    }

    /// 清空所有输入映射
    pub fn clear_input_maps(&mut self) {
        self.input_maps.clear();
        self.current_input_map = None;
    }
}

impl Default for InputManager {
    fn default() -> Self {
        let mut manager = Self::new();
        manager.create_default_input_map();
        manager
    }
}
