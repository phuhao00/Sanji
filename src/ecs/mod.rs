//! 实体组件系统模块

pub mod world;
pub mod entity;
pub mod component;
pub mod system;
pub mod query;

pub use world::*;
pub use entity::*;
pub use component::*;
pub use system::*;
pub use query::*;

// 重新导出specs的常用类型
pub use specs::{
    Component, 
    DenseVecStorage, 
    VecStorage, 
    HashMapStorage,
    Entity,
    ReadStorage,
    WriteStorage,
    Read,
    Write,
    SystemData,
    System,
    Builder,
    WorldExt,
};
