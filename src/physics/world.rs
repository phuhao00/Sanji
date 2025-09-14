//! 物理世界管理

use crate::{EngineResult, EngineError};
use crate::physics::{PhysicsRigidBody, Collider};
use crate::math::{Vec3, AABB, BoundingSphere};

use std::collections::{HashMap, HashSet};
use specs::Entity;

/// 物理世界配置
#[derive(Debug, Clone)]
pub struct PhysicsConfig {
    /// 重力加速度
    pub gravity: Vec3,
    /// 时间步长
    pub timestep: f32,
    /// 最大子步数
    pub max_substeps: u32,
    /// 启用连续碰撞检测
    pub enable_ccd: bool,
    /// 世界边界
    pub world_bounds: Option<AABB>,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            timestep: 1.0 / 60.0,
            max_substeps: 4,
            enable_ccd: false,
            world_bounds: Some(AABB::from_center_size(Vec3::ZERO, Vec3::splat(1000.0))),
        }
    }
}

/// 碰撞事件
#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub contact_point: Vec3,
    pub contact_normal: Vec3,
    pub penetration_depth: f32,
    pub relative_velocity: Vec3,
}

/// 物理世界管理器
pub struct PhysicsWorld {
    /// 配置
    config: PhysicsConfig,
    /// 刚体映射
    rigid_bodies: HashMap<Entity, PhysicsRigidBody>,
    /// 碰撞体映射
    colliders: HashMap<Entity, Collider>,
    /// 碰撞对
    collision_pairs: HashSet<(Entity, Entity)>,
    /// 碰撞事件缓冲区
    collision_events: Vec<CollisionEvent>,
    /// 累积时间
    accumulated_time: f32,
    /// 是否暂停物理模拟
    paused: bool,
}

impl PhysicsWorld {
    /// 创建新的物理世界
    pub fn new(config: PhysicsConfig) -> Self {
        Self {
            config,
            rigid_bodies: HashMap::new(),
            colliders: HashMap::new(),
            collision_pairs: HashSet::new(),
            collision_events: Vec::new(),
            accumulated_time: 0.0,
            paused: false,
        }
    }

    /// 添加刚体
    pub fn add_rigid_body(&mut self, entity: Entity, rigid_body: PhysicsRigidBody) {
        self.rigid_bodies.insert(entity, rigid_body);
    }

    /// 移除刚体
    pub fn remove_rigid_body(&mut self, entity: Entity) -> Option<PhysicsRigidBody> {
        self.rigid_bodies.remove(&entity)
    }

    /// 添加碰撞体
    pub fn add_collider(&mut self, entity: Entity, collider: Collider) {
        self.colliders.insert(entity, collider);
    }

    /// 移除碰撞体
    pub fn remove_collider(&mut self, entity: Entity) -> Option<Collider> {
        self.colliders.remove(&entity)
    }

    /// 获取刚体
    pub fn get_rigid_body(&self, entity: Entity) -> Option<&PhysicsRigidBody> {
        self.rigid_bodies.get(&entity)
    }

    /// 获取可变刚体
    pub fn get_rigid_body_mut(&mut self, entity: Entity) -> Option<&mut PhysicsRigidBody> {
        self.rigid_bodies.get_mut(&entity)
    }

    /// 获取碰撞体
    pub fn get_collider(&self, entity: Entity) -> Option<&Collider> {
        self.colliders.get(&entity)
    }

    /// 更新物理世界
    pub fn update(&mut self, delta_time: f32) -> EngineResult<()> {
        if self.paused {
            return Ok(());
        }

        self.accumulated_time += delta_time;
        
        let mut substeps = 0;
        while self.accumulated_time >= self.config.timestep && substeps < self.config.max_substeps {
            self.step(self.config.timestep)?;
            self.accumulated_time -= self.config.timestep;
            substeps += 1;
        }
        
        Ok(())
    }

    /// 执行一个物理步骤
    fn step(&mut self, dt: f32) -> EngineResult<()> {
        // 清空上一帧的碰撞事件
        self.collision_events.clear();
        
        // 1. 应用外力和重力
        self.apply_forces(dt);
        
        // 2. 积分速度
        self.integrate_velocities(dt);
        
        // 3. 检测碰撞
        self.detect_collisions();
        
        // 4. 解决碰撞
        self.resolve_collisions(dt);
        
        // 5. 积分位置
        self.integrate_positions(dt);
        
        // 6. 更新变换
        self.update_transforms();
        
        Ok(())
    }

    /// 应用力和重力
    fn apply_forces(&mut self, dt: f32) {
        for (_, rigid_body) in self.rigid_bodies.iter_mut() {
            if rigid_body.body_type != crate::physics::RigidBodyType::Dynamic {
                continue;
            }

            // 应用重力
            if rigid_body.use_gravity {
                rigid_body.add_force(self.config.gravity * rigid_body.mass);
            }

            // 应用阻尼
            rigid_body.velocity *= 1.0 - (rigid_body.linear_damping * dt);
            rigid_body.angular_velocity *= 1.0 - (rigid_body.angular_damping * dt);
        }
    }

    /// 积分速度
    fn integrate_velocities(&mut self, dt: f32) {
        for (_, rigid_body) in self.rigid_bodies.iter_mut() {
            if rigid_body.body_type != crate::physics::RigidBodyType::Dynamic {
                continue;
            }

            // 积分线性速度
            if rigid_body.mass > 0.0 {
                rigid_body.velocity += rigid_body.force * dt / rigid_body.mass;
            }

            // 积分角速度 (简化处理)
            rigid_body.angular_velocity += rigid_body.torque * dt / rigid_body.inertia;

            // 清空力
            rigid_body.force = Vec3::ZERO;
            rigid_body.torque = Vec3::ZERO;
        }
    }

    /// 检测碰撞
    fn detect_collisions(&mut self) {
        self.collision_pairs.clear();
        
        let entities: Vec<Entity> = self.colliders.keys().copied().collect();
        
        // 宽相位碰撞检测 (简单的n^2算法)
        for i in 0..entities.len() {
            for j in i + 1..entities.len() {
                let entity_a = entities[i];
                let entity_b = entities[j];
                
                if let (Some(collider_a), Some(collider_b)) = 
                    (self.colliders.get(&entity_a), self.colliders.get(&entity_b)) {
                    
                    // 检查AABB重叠
                    if collider_a.aabb.intersects(&collider_b.aabb) {
                        self.collision_pairs.insert((entity_a, entity_b));
                    }
                }
            }
        }
        
        // 窄相位碰撞检测
        let collision_pairs: Vec<_> = self.collision_pairs.iter().copied().collect();
        for (entity_a, entity_b) in collision_pairs {
            if let Some(collision) = self.narrow_phase_detection(entity_a, entity_b) {
                self.collision_events.push(collision);
            }
        }
    }

    /// 窄相位碰撞检测
    fn narrow_phase_detection(&self, entity_a: Entity, entity_b: Entity) -> Option<CollisionEvent> {
        let collider_a = self.colliders.get(&entity_a)?;
        let collider_b = self.colliders.get(&entity_b)?;
        
        // 简化的球-球碰撞检测
        if let (Some(sphere_a), Some(sphere_b)) = (&collider_a.bounding_sphere, &collider_b.bounding_sphere) {
            let distance = (sphere_a.center - sphere_b.center).length();
            let combined_radius = sphere_a.radius + sphere_b.radius;
            
            if distance < combined_radius {
                let penetration = combined_radius - distance;
                let normal = if distance > 0.0 {
                    (sphere_b.center - sphere_a.center).normalize()
                } else {
                    Vec3::new(1.0, 0.0, 0.0) // 默认法线
                };
                
                let contact_point = sphere_a.center + normal * sphere_a.radius;
                
                // 计算相对速度
                let vel_a = self.rigid_bodies.get(&entity_a).map(|rb| rb.velocity).unwrap_or(Vec3::ZERO);
                let vel_b = self.rigid_bodies.get(&entity_b).map(|rb| rb.velocity).unwrap_or(Vec3::ZERO);
                let relative_velocity = vel_b - vel_a;
                
                return Some(CollisionEvent {
                    entity_a,
                    entity_b,
                    contact_point,
                    contact_normal: normal,
                    penetration_depth: penetration,
                    relative_velocity,
                });
            }
        }
        
        None
    }

    /// 解决碰撞
    fn resolve_collisions(&mut self, dt: f32) {
        let collision_events = self.collision_events.clone();
        for collision in collision_events.iter() {
            self.resolve_collision(collision, dt);
        }
    }

    /// 解决单个碰撞
    fn resolve_collision(&mut self, collision: &CollisionEvent, dt: f32) {
        let restitution = 0.5; // 恢复系数
        let friction = 0.3;    // 摩擦系数
        
        // 获取刚体
        let (mass_a, vel_a) = if let Some(rb) = self.rigid_bodies.get(&collision.entity_a) {
            (rb.mass, rb.velocity)
        } else {
            return;
        };
        
        let (mass_b, vel_b) = if let Some(rb) = self.rigid_bodies.get(&collision.entity_b) {
            (rb.mass, rb.velocity)
        } else {
            return;
        };
        
        // 计算冲量
        let relative_velocity = collision.relative_velocity;
        let velocity_along_normal = relative_velocity.dot(collision.contact_normal);
        
        // 如果物体正在分离，不需要解决
        if velocity_along_normal > 0.0 {
            return;
        }
        
        // 计算冲量大小
        let impulse_magnitude = -(1.0 + restitution) * velocity_along_normal;
        let impulse_magnitude = impulse_magnitude / (1.0 / mass_a + 1.0 / mass_b);
        
        let impulse = collision.contact_normal * impulse_magnitude;
        
        // 应用冲量
        if let Some(rb_a) = self.rigid_bodies.get_mut(&collision.entity_a) {
            if rb_a.body_type == crate::physics::RigidBodyType::Dynamic {
                rb_a.velocity -= impulse / mass_a;
            }
        }
        
        if let Some(rb_b) = self.rigid_bodies.get_mut(&collision.entity_b) {
            if rb_b.body_type == crate::physics::RigidBodyType::Dynamic {
                rb_b.velocity += impulse / mass_b;
            }
        }
        
        // 位置修正（防止穿透）
        let correction_percent = 0.8;
        let slop = 0.01;
        let correction_magnitude = (collision.penetration_depth - slop).max(0.0) / (1.0 / mass_a + 1.0 / mass_b) * correction_percent;
        let correction = collision.contact_normal * correction_magnitude;
        
        // 应用位置修正
        if let Some(rb_a) = self.rigid_bodies.get_mut(&collision.entity_a) {
            if rb_a.body_type == crate::physics::RigidBodyType::Dynamic {
                rb_a.position -= correction / mass_a;
            }
        }
        
        if let Some(rb_b) = self.rigid_bodies.get_mut(&collision.entity_b) {
            if rb_b.body_type == crate::physics::RigidBodyType::Dynamic {
                rb_b.position += correction / mass_b;
            }
        }
    }

    /// 积分位置
    fn integrate_positions(&mut self, dt: f32) {
        for (_, rigid_body) in self.rigid_bodies.iter_mut() {
            if rigid_body.body_type != crate::physics::RigidBodyType::Dynamic {
                continue;
            }

            // 积分位置
            rigid_body.position += rigid_body.velocity * dt;
            
            // 积分旋转 (简化处理)
            if rigid_body.angular_velocity.length_squared() > 0.0 {
                let angular_vel_magnitude = rigid_body.angular_velocity.length();
                let rotation_axis = rigid_body.angular_velocity.normalize();
                let rotation_angle = angular_vel_magnitude * dt;
                let rotation_quat = glam::Quat::from_axis_angle(rotation_axis, rotation_angle);
                rigid_body.rotation = rotation_quat * rigid_body.rotation;
            }

            // 检查世界边界
            if let Some(bounds) = &self.config.world_bounds {
                rigid_body.position = rigid_body.position.clamp(bounds.min, bounds.max);
            }
        }
    }

    /// 更新变换（这个方法在实际使用时应该与ECS集成）
    fn update_transforms(&self) {
        // 这里应该更新ECS中的Transform组件
        // 在实际集成时，这个方法会被物理系统调用
    }

    /// 暂停物理模拟
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// 恢复物理模拟
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// 检查是否暂停
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// 获取碰撞事件
    pub fn collision_events(&self) -> &[CollisionEvent] {
        &self.collision_events
    }

    /// 射线投射
    pub fn raycast(&self, ray: &crate::math::Ray, max_distance: f32) -> Vec<RaycastHit> {
        let mut hits = Vec::new();
        
        for (entity, collider) in &self.colliders {
            if let Some(bounding_sphere) = &collider.bounding_sphere {
                if let Some(hit) = ray.intersect_sphere(bounding_sphere) {
                    if hit.distance <= max_distance {
                        hits.push(RaycastHit {
                            entity: *entity,
                            point: hit.point,
                            normal: hit.normal,
                            distance: hit.distance,
                        });
                    }
                }
            }
        }
        
        // 按距离排序
        hits.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        hits
    }

    /// 设置重力
    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.config.gravity = gravity;
    }

    /// 获取重力
    pub fn gravity(&self) -> Vec3 {
        self.config.gravity
    }

    /// 获取统计信息
    pub fn stats(&self) -> PhysicsStats {
        PhysicsStats {
            rigid_body_count: self.rigid_bodies.len(),
            collider_count: self.colliders.len(),
            active_collision_pairs: self.collision_pairs.len(),
            collision_events: self.collision_events.len(),
        }
    }
}

/// 射线投射结果
#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub entity: Entity,
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

/// 物理统计信息
#[derive(Debug, Clone)]
pub struct PhysicsStats {
    pub rigid_body_count: usize,
    pub collider_count: usize,
    pub active_collision_pairs: usize,
    pub collision_events: usize,
}
