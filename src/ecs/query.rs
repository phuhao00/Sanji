//! ECS查询系统

use specs::{World, Entity, ReadStorage, WriteStorage, Join, WorldExt};
use crate::ecs::component::*;

/// 查询构建器
pub struct QueryBuilder<'a> {
    world: &'a World,
}

impl<'a> QueryBuilder<'a> {
    /// 创建新的查询构建器
    pub fn new(world: &'a World) -> Self {
        Self { world }
    }

    /// 查询所有带有Transform组件的实体
    pub fn with_transform(self) -> TransformQuery<'a> {
        TransformQuery::new(self.world)
    }

    /// 查询所有渲染对象
    pub fn renderable(self) -> RenderableQuery<'a> {
        RenderableQuery::new(self.world)
    }

    /// 查询所有相机
    pub fn cameras(self) -> CameraQuery<'a> {
        CameraQuery::new(self.world)
    }

    /// 查询所有光源
    pub fn lights(self) -> LightQuery<'a> {
        LightQuery::new(self.world)
    }
}

/// Transform查询
pub struct TransformQuery<'a> {
    world: &'a World,
}

impl<'a> TransformQuery<'a> {
    fn new(world: &'a World) -> Self {
        Self { world }
    }

    /// 执行查询，返回所有结果
    pub fn execute(&self) -> Vec<(Entity, Transform)> {
        let entities = self.world.entities();
        let transforms = self.world.read_storage::<Transform>();
        
        (&entities, &transforms).join().map(|(e, t)| (e, t.clone())).collect()
    }

    /// 按条件过滤
    pub fn filter<F>(self, predicate: F) -> Vec<(Entity, Transform)>
    where
        F: Fn(&Transform) -> bool,
    {
        self.execute()
            .into_iter()
            .filter(|(_, transform)| predicate(transform))
            .map(|(entity, transform)| (entity, transform.clone()))
            .collect()
    }
}

/// 可渲染对象查询
pub struct RenderableQuery<'a> {
    world: &'a World,
}

impl<'a> RenderableQuery<'a> {
    fn new(world: &'a World) -> Self {
        Self { world }
    }

    /// 执行查询
    pub fn execute(&self) -> Vec<(Entity, Transform, MeshRenderer)> {
        let entities = self.world.entities();
        let transforms = self.world.read_storage::<Transform>();
        let renderers = self.world.read_storage::<MeshRenderer>();
        
        (&entities, &transforms, &renderers).join().map(|(e, t, r)| (e, t.clone(), r.clone())).collect()
    }

    /// 只获取可见的对象
    pub fn visible_only(&self) -> Vec<(Entity, Transform, MeshRenderer)> {
        self.execute()
            .into_iter()
            .filter(|(_, _, renderer)| renderer.visible)
            .collect()
    }

    /// 按材质过滤
    pub fn with_material(&self, material_name: &str) -> Vec<(Entity, Transform, MeshRenderer)> {
        self.execute()
            .into_iter()
            .filter(|(_, _, renderer)| renderer.material_name == material_name)
            .collect()
    }
}

/// 相机查询
pub struct CameraQuery<'a> {
    world: &'a World,
}

impl<'a> CameraQuery<'a> {
    fn new(world: &'a World) -> Self {
        Self { world }
    }

    /// 执行查询
    pub fn execute(&self) -> Vec<(Entity, Transform, Camera)> {
        let entities = self.world.entities();
        let transforms = self.world.read_storage::<Transform>();
        let cameras = self.world.read_storage::<Camera>();
        
        (&entities, &transforms, &cameras).join().map(|(e, t, c)| (e, t.clone(), c.clone())).collect()
    }

    /// 获取主相机
    pub fn main_camera(&self) -> Option<(Entity, Transform, Camera)> {
        self.execute()
            .into_iter()
            .find(|(_, _, camera)| camera.camera.is_main)
    }
}

/// 光源查询
pub struct LightQuery<'a> {
    world: &'a World,
}

impl<'a> LightQuery<'a> {
    fn new(world: &'a World) -> Self {
        Self { world }
    }

    /// 执行查询
    pub fn execute(&self) -> Vec<(Entity, Transform, Light)> {
        let entities = self.world.entities();
        let transforms = self.world.read_storage::<Transform>();
        let lights = self.world.read_storage::<Light>();
        
        (&entities, &transforms, &lights).join().map(|(e, t, l)| (e, t.clone(), l.clone())).collect()
    }

    /// 按光源类型过滤
    pub fn by_type(&self, light_type: crate::ecs::component::LightType) -> Vec<(Entity, Transform, Light)> {
        self.execute()
            .into_iter()
            .filter(|(_, _, light)| light.light_type == light_type)
            .collect()
    }

    /// 获取所有方向光
    pub fn directional_lights(&self) -> Vec<(Entity, Transform, Light)> {
        self.by_type(crate::ecs::component::LightType::Directional)
    }

    /// 获取所有点光源
    pub fn point_lights(&self) -> Vec<(Entity, Transform, Light)> {
        self.by_type(crate::ecs::component::LightType::Point)
    }

    /// 获取所有聚光灯
    pub fn spot_lights(&self) -> Vec<(Entity, Transform, Light)> {
        self.by_type(crate::ecs::component::LightType::Spot)
    }
}

/// 扩展World以支持查询构建器
pub trait WorldQueryExt {
    /// 创建查询构建器
    fn query(&self) -> QueryBuilder;
}

impl WorldQueryExt for World {
    fn query(&self) -> QueryBuilder {
        QueryBuilder::new(self)
    }
}
