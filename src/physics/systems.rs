//! 物理系统的ECS集成

use crate::physics::{PhysicsWorld, PhysicsRigidBody, Collider, CollisionEvent};
use crate::ecs::{Transform, ReadStorage, WriteStorage, System, SystemData, Join, World, WorldExt};
use crate::math::Vec3;
use specs::{Entity, Entities};
use std::collections::HashMap;

/// 物理系统 - 将物理世界与ECS集成
pub struct PhysicsSystem {
    physics_world: PhysicsWorld,
    entity_to_physics: HashMap<Entity, Entity>, // ECS实体到物理实体的映射
}

impl Default for PhysicsSystem {
    fn default() -> Self {
        Self {
            physics_world: PhysicsWorld::new(crate::physics::world::PhysicsConfig::default()),
            entity_to_physics: HashMap::new(),
        }
    }
}

impl PhysicsSystem {
    /// 创建新的物理系统
    pub fn new(physics_world: PhysicsWorld) -> Self {
        Self {
            physics_world,
            entity_to_physics: HashMap::new(),
        }
    }

    /// 获取物理世界的引用
    pub fn physics_world(&self) -> &PhysicsWorld {
        &self.physics_world
    }

    /// 获取物理世界的可变引用
    pub fn physics_world_mut(&mut self) -> &mut PhysicsWorld {
        &mut self.physics_world
    }

    /// 为ECS实体添加物理组件
    pub fn add_rigid_body(&mut self, entity: Entity, rigid_body: PhysicsRigidBody) {
        self.physics_world.add_rigid_body(entity, rigid_body);
        self.entity_to_physics.insert(entity, entity);
    }

    /// 为ECS实体添加碰撞体
    pub fn add_collider(&mut self, entity: Entity, collider: Collider) {
        self.physics_world.add_collider(entity, collider);
    }

    /// 移除ECS实体的物理组件
    pub fn remove_physics_entity(&mut self, entity: Entity) {
        self.physics_world.remove_rigid_body(entity);
        self.physics_world.remove_collider(entity);
        self.entity_to_physics.remove(&entity);
    }
}

impl<'a> System<'a> for PhysicsSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, PhysicsRigidBody>,
        ReadStorage<'a, Collider>,
        specs::Read<'a, crate::ecs::TimeResource>,
    );

    fn run(&mut self, (entities, mut transforms, rigid_bodies, colliders, time): Self::SystemData) {
        let delta_time = time.delta_time;
        
        // 1. 同步ECS Transform到物理世界  
        use specs::Join;
        for (entity, transform, rigid_body) in (&entities, &transforms, &rigid_bodies).join() {
            if let Some(physics_rb) = self.physics_world.get_rigid_body_mut(entity) {
                // 只有在Transform被修改时才更新物理世界
                if transform.dirty {
                    physics_rb.position = transform.position;
                    physics_rb.rotation = transform.rotation;
                }
            }
        }
        
        // 2. 更新碰撞体边界
        for (entity, transform, collider) in (&entities, &transforms, &colliders).join() {
            if let Some(physics_collider) = self.physics_world.get_collider(entity) {
                // 这里需要更新碰撞体的边界信息
                // 由于borrowing的限制，我们需要重构这部分代码
            }
        }
        
        // 3. 更新物理世界
        if let Err(e) = self.physics_world.update(delta_time) {
            log::error!("物理世界更新失败: {}", e);
            return;
        }
        
        // 4. 同步物理世界的结果回ECS Transform
        for (entity, mut transform) in (&entities, &mut transforms).join() {
            if let Some(physics_rb) = self.physics_world.get_rigid_body(entity) {
                // 只有动态刚体才会更新Transform
                if physics_rb.is_dynamic() {
                    transform.set_position(physics_rb.position);
                    transform.set_rotation(physics_rb.rotation);
                }
            }
        }
    }
}

/// 碰撞检测系统 - 处理碰撞事件
pub struct CollisionDetectionSystem {
    collision_events: Vec<CollisionEvent>,
}

impl CollisionDetectionSystem {
    pub fn new() -> Self {
        Self {
            collision_events: Vec::new(),
        }
    }

    /// 获取碰撞事件
    pub fn collision_events(&self) -> &[CollisionEvent] {
        &self.collision_events
    }

    /// 清空碰撞事件
    pub fn clear_events(&mut self) {
        self.collision_events.clear();
    }
}

impl Default for CollisionDetectionSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> System<'a> for CollisionDetectionSystem {
    type SystemData = (
        specs::Entities<'a>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Collider>,
        specs::Write<'a, PhysicsSystem>, // 访问物理系统
    );

    fn run(&mut self, (entities, transforms, colliders, mut physics_system): Self::SystemData) {
        self.collision_events.clear();
        
        // 从物理系统获取碰撞事件
        let events = physics_system.physics_world().collision_events();
        self.collision_events.extend_from_slice(events);
        
        // 这里可以添加额外的碰撞处理逻辑
        for event in &self.collision_events {
            log::debug!("碰撞检测: {:?} <-> {:?}", event.entity_a, event.entity_b);
        }
    }
}

/// 物理调试渲染系统
pub struct PhysicsDebugRenderSystem {
    enabled: bool,
}

impl PhysicsDebugRenderSystem {
    pub fn new() -> Self {
        Self { enabled: false }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for PhysicsDebugRenderSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> System<'a> for PhysicsDebugRenderSystem {
    type SystemData = (
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Collider>,
        ReadStorage<'a, PhysicsRigidBody>,
    );

    fn run(&mut self, (transforms, colliders, rigid_bodies): Self::SystemData) {
        if !self.enabled {
            return;
        }

        // 这里可以实现物理形状的调试渲染
        // 例如渲染碰撞体的边界框、速度向量等
        
        for (transform, collider, rigid_body) in (&transforms, &colliders, &rigid_bodies).join() {
            // 渲染AABB
            // render_debug_aabb(transform.position, collider.aabb);
            
            // 渲染速度向量
            if rigid_body.is_dynamic() {
                // render_debug_vector(transform.position, rigid_body.velocity);
            }
        }
    }
}

/// 简化的物理系统管理器
pub struct SimplePhysicsSystem;

impl SimplePhysicsSystem {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimplePhysicsSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> System<'a> for SimplePhysicsSystem {
    type SystemData = (
        WriteStorage<'a, Transform>,
        WriteStorage<'a, crate::ecs::RigidBody>, // 使用原来的简单RigidBody
        specs::Read<'a, crate::ecs::TimeResource>,
    );

    fn run(&mut self, (mut transforms, mut rigid_bodies, time): Self::SystemData) {
        let delta_time = time.delta_time;
        let gravity = Vec3::new(0.0, -9.81, 0.0);

        for (mut transform, mut rigid_body) in (&mut transforms, &mut rigid_bodies).join() {
            if rigid_body.is_kinematic {
                continue;
            }

            // 应用重力
            if rigid_body.use_gravity {
                rigid_body.velocity += gravity * delta_time;
            }

            // 应用阻尼
            rigid_body.velocity *= 1.0 - (rigid_body.drag * delta_time);
            rigid_body.angular_velocity *= 1.0 - (rigid_body.angular_drag * delta_time);

            // 更新位置
            let position_delta = rigid_body.velocity * delta_time;
            transform.translate(position_delta);

            // 更新旋转
            if rigid_body.angular_velocity.length_squared() > 0.001 {
                let axis = rigid_body.angular_velocity.normalize();
                let angle = rigid_body.angular_velocity.length() * delta_time;
                let rotation = glam::Quat::from_axis_angle(axis, angle);
                transform.rotate(rotation);
            }

            // 简单的地面碰撞检测
            if transform.position.y < 0.0 {
                transform.set_position(Vec3::new(
                    transform.position.x,
                    0.0,
                    transform.position.z
                ));
                
                // 反弹
                if rigid_body.velocity.y < 0.0 {
                    rigid_body.velocity.y = -rigid_body.velocity.y * 0.5; // 恢复系数
                }
            }
        }
    }
}

/// 物理事件资源 - 用于在系统间共享物理事件
pub struct PhysicsEvents {
    pub collision_events: Vec<CollisionEvent>,
}

impl Default for PhysicsEvents {
    fn default() -> Self {
        Self {
            collision_events: Vec::new(),
        }
    }
}

/// 物理配置资源
pub struct PhysicsConfig {
    pub gravity: Vec3,
    pub timestep: f32,
    pub enabled: bool,
    pub debug_render: bool,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            timestep: 1.0 / 60.0,
            enabled: true,
            debug_render: false,
        }
    }
}

/// 物理系统工具函数
pub mod physics_utils {
    use super::*;

    /// 创建基本的物理世界设置
    pub fn setup_physics_world(world: &mut World) {
        // 注册物理组件
        world.register::<PhysicsRigidBody>();
        world.register::<Collider>();
        
        // 添加物理资源
        world.insert(PhysicsEvents::default());
        world.insert(PhysicsConfig::default());
        
        log::info!("物理系统已初始化");
    }

    /// 为实体添加基本的物理组件
    pub fn add_physics_to_entity(
        world: &mut World,
        entity: Entity,
        rigid_body_type: crate::physics::RigidBodyType
    ) -> crate::EngineResult<()> {
        let rigid_body = match rigid_body_type {
            crate::physics::RigidBodyType::Static => PhysicsRigidBody::static_body(),
            crate::physics::RigidBodyType::Kinematic => PhysicsRigidBody::kinematic_body(),
            crate::physics::RigidBodyType::Dynamic => PhysicsRigidBody::dynamic_body(),
        };

        let collider = Collider::new(crate::physics::ColliderShape::cube(0.5));

        world.write_storage::<PhysicsRigidBody>().insert(entity, rigid_body)?;
        world.write_storage::<Collider>().insert(entity, collider)?;

        Ok(())
    }

    /// 应用冲量到实体
    pub fn apply_impulse_to_entity(
        world: &mut World,
        entity: Entity,
        impulse: Vec3
    ) {
        if let Some(mut rigid_body) = world.write_storage::<PhysicsRigidBody>().get_mut(entity) {
            rigid_body.add_impulse(impulse);
        }
    }

    /// 设置实体速度
    pub fn set_entity_velocity(
        world: &mut World,
        entity: Entity,
        velocity: Vec3
    ) {
        if let Some(mut rigid_body) = world.write_storage::<PhysicsRigidBody>().get_mut(entity) {
            rigid_body.set_velocity(velocity);
        }
    }
}
