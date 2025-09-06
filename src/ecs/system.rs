//! ECS系统定义

use crate::ecs::component::*;
use crate::ecs::world::TimeResource;

use specs::{System, SystemData, ReadStorage, WriteStorage, Read, Join};
use glam::Vec3;

/// 变换系统 - 更新变换矩阵
pub struct TransformSystem;

impl TransformSystem {
    pub fn new() -> Self {
        Self
    }
}

impl<'a> System<'a> for TransformSystem {
    type SystemData = WriteStorage<'a, Transform>;

    fn run(&mut self, mut transforms: Self::SystemData) {
        for transform in (&mut transforms).join() {
            transform.update_matrices();
        }
    }
}

/// 渲染系统 - 处理渲染相关逻辑
pub struct RenderSystem;

impl RenderSystem {
    pub fn new() -> Self {
        Self
    }
}

impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        ReadStorage<'a, Transform>,
        ReadStorage<'a, MeshRenderer>,
        ReadStorage<'a, Camera>,
    );

    fn run(&mut self, (transforms, renderers, cameras): Self::SystemData) {
        // 收集所有需要渲染的对象
        let mut render_items = Vec::new();
        
        for (transform, renderer) in (&transforms, &renderers).join() {
            if renderer.visible {
                render_items.push((transform, renderer));
            }
        }

        // 这里可以进行视锥体剔除、深度排序等优化
        // 目前保持简单
        
        // 对于每个相机，渲染场景
        for (camera_transform, camera) in (&transforms, &cameras).join() {
            if camera.camera.is_main {
                // 主相机渲染逻辑
                // 这里可以调用实际的渲染命令
            }
        }
    }
}

/// 物理系统 - 简单的物理模拟
pub struct PhysicsSystem {
    gravity: Vec3,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
        }
    }

    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity;
    }
}

impl<'a> System<'a> for PhysicsSystem {
    type SystemData = (
        WriteStorage<'a, Transform>,
        WriteStorage<'a, RigidBody>,
        Read<'a, TimeResource>,
    );

    fn run(&mut self, (mut transforms, mut rigidbodies, time): Self::SystemData) {
        let delta_time = time.delta_time;
        
        for (transform, rigidbody) in (&mut transforms, &mut rigidbodies).join() {
            if rigidbody.is_kinematic {
                continue;
            }

            // 应用重力
            if rigidbody.use_gravity {
                rigidbody.velocity += self.gravity * delta_time;
            }

            // 应用阻力
            rigidbody.velocity *= 1.0 - (rigidbody.drag * delta_time);
            rigidbody.angular_velocity *= 1.0 - (rigidbody.angular_drag * delta_time);

            // 更新位置和旋转
            let velocity_delta = rigidbody.velocity * delta_time;
            transform.translate(velocity_delta);

            // 简化的角速度处理
            if rigidbody.angular_velocity.length() > 0.001 {
                let axis = rigidbody.angular_velocity.normalize();
                let angle = rigidbody.angular_velocity.length() * delta_time;
                let rotation = glam::Quat::from_axis_angle(axis, angle);
                transform.rotate(rotation);
            }
        }
    }
}

/// 相机系统 - 更新相机相关逻辑
pub struct CameraSystem;

impl CameraSystem {
    pub fn new() -> Self {
        Self
    }
}

impl<'a> System<'a> for CameraSystem {
    type SystemData = (
        ReadStorage<'a, Transform>,
        WriteStorage<'a, Camera>,
    );

    fn run(&mut self, (transforms, mut cameras): Self::SystemData) {
        for (transform, camera) in (&transforms, &mut cameras).join() {
            // 更新相机的位置和旋转
            camera.camera.set_position(transform.position);
            camera.camera.set_rotation(transform.rotation);
        }
    }
}

/// 生命周期系统 - 处理对象的生命周期
pub struct LifecycleSystem;

impl LifecycleSystem {
    pub fn new() -> Self {
        Self
    }
}

impl<'a> System<'a> for LifecycleSystem {
    type SystemData = (
        specs::Entities<'a>,
        ReadStorage<'a, Tag>,
    );

    fn run(&mut self, (entities, tags): Self::SystemData) {
        let mut to_delete = Vec::new();
        
        for (entity, tag) in (&entities, &tags).join() {
            if tag.has_tag("destroy") {
                to_delete.push(entity);
            }
        }

        // 删除标记为销毁的实体
        for entity in to_delete {
            if let Err(e) = entities.delete(entity) {
                log::error!("删除实体失败: {:?}", e);
            }
        }
    }
}
