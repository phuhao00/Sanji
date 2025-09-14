//! 渲染系统模块

pub mod render_system;
pub mod shader;
pub mod mesh;
pub mod texture;
pub mod material;
pub mod camera;
pub mod shadows;
pub mod post_processing;

pub use render_system::*;
pub use shader::*;
pub use mesh::*;
pub use texture::*;
pub use material::*;
pub use camera::*;
pub use shadows::*;
pub use post_processing::*;

// 重新导出组件中的Light相关类型，以便向后兼容
pub use crate::ecs::{Light, LightType};