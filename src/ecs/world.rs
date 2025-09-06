//! ECS世界管理

use crate::{EngineResult, EngineError};
use crate::ecs::component::*;
use crate::ecs::system::*;

use specs::{World, WorldExt, DispatcherBuilder, Dispatcher, RunNow};

/// ECS世界包装器
pub struct ECSWorld {
    world: World,
    dispatcher: Option<Dispatcher<'static, 'static>>,
}

impl ECSWorld {
    /// 创建新的ECS世界
    pub fn new() -> EngineResult<Self> {
        let mut world = World::new();
        
        // 注册核心组件
        world.register::<Transform>();
        world.register::<MeshRenderer>();
        world.register::<Camera>();
        world.register::<Light>();
        world.register::<RigidBody>();
        world.register::<Name>();
        world.register::<Tag>();

        // 创建系统调度器
        let dispatcher = DispatcherBuilder::new()
            .with(TransformSystem::new(), "transform", &[])
            .with(RenderSystem::new(), "render", &["transform"])
            .with(PhysicsSystem::new(), "physics", &[])
            .build();

        Ok(Self {
            world,
            dispatcher: Some(dispatcher),
        })
    }

    /// 获取内部World的可变引用
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// 获取内部World的不可变引用
    pub fn world(&self) -> &World {
        &self.world
    }

    /// 创建实体构建器
    pub fn create_entity(&mut self) -> specs::EntityBuilder {
        self.world.create_entity()
    }

    /// 更新ECS系统
    pub fn update(&mut self, delta_time: f32) -> EngineResult<()> {
        // 更新时间资源
        {
            let mut time_res = self.world.write_resource::<TimeResource>();
            time_res.delta_time = delta_time;
            time_res.total_time += delta_time;
        }

        // 运行系统调度器
        if let Some(ref mut dispatcher) = self.dispatcher {
            dispatcher.dispatch(&self.world);
        }

        // 维护世界状态
        self.world.maintain();

        Ok(())
    }

    /// 添加资源
    pub fn add_resource<T: Send + Sync + 'static>(&mut self, resource: T) {
        self.world.insert(resource);
    }

    /// 获取资源的可变引用
    pub fn get_resource_mut<T: Send + Sync + 'static>(&mut self) -> Option<specs::shred::FetchMut<T>> {
        if self.world.has_value::<T>() {
            Some(self.world.write_resource::<T>())
        } else {
            None
        }
    }

    /// 获取资源的不可变引用
    pub fn get_resource<T: Send + Sync + 'static>(&self) -> Option<specs::shred::Fetch<T>> {
        if self.world.has_value::<T>() {
            Some(self.world.read_resource::<T>())
        } else {
            None
        }
    }

    /// 查找带有指定组件的所有实体
    pub fn find_entities_with<T: Component>(&self) -> Vec<specs::Entity>
    where
        T::Storage: Default,
    {
        use specs::Join;
        
        let entities = self.world.entities();
        let storage = self.world.read_storage::<T>();
        
        (&entities, &storage).join().map(|(e, _)| e).collect()
    }

    /// 删除实体
    pub fn delete_entity(&mut self, entity: specs::Entity) -> EngineResult<()> {
        self.world
            .delete_entity(entity)
            .map_err(|e| EngineError::RenderError(format!("删除实体失败: {:?}", e)))
    }
}

/// 时间资源
#[derive(Debug, Default)]
pub struct TimeResource {
    pub delta_time: f32,
    pub total_time: f32,
}

impl ECSWorld {
    /// 初始化默认资源
    pub fn setup_default_resources(&mut self) {
        self.add_resource(TimeResource::default());
    }
}
