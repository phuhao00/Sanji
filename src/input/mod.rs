//! 输入处理系统

pub mod input_manager;
pub mod keyboard;
pub mod mouse;
pub mod gamepad;
pub mod input_map;

pub use input_manager::*;
pub use keyboard::*;
pub use mouse::*;
pub use gamepad::*;
pub use input_map::*;

// 重新导出winit的输入相关类型
pub use winit::{
    event::{KeyEvent, MouseButton, ElementState},
    keyboard::{KeyCode, PhysicalKey},
    dpi::PhysicalPosition,
};
