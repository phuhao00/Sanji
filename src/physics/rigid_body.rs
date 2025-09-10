//! 刚体组件

use crate::math::Vec3;
use serde::{Deserialize, Serialize};

/// 刚体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RigidBodyType {
    /// 静态刚体 - 不移动，不受力影响
    Static,
    /// 运动学刚体 - 可以移动，但不受物理力影响
    Kinematic,
    /// 动态刚体 - 受物理力影响，完全参与物理模拟
    Dynamic,
}

impl Default for RigidBodyType {
    fn default() -> Self {
        Self::Dynamic
    }
}

/// 物理刚体组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsRigidBody {
    /// 刚体类型
    pub body_type: RigidBodyType,
    
    /// 位置
    pub position: Vec3,
    /// 旋转（四元数）
    pub rotation: glam::Quat,
    
    /// 线性速度
    pub velocity: Vec3,
    /// 角速度
    pub angular_velocity: Vec3,
    
    /// 质量
    pub mass: f32,
    /// 转动惯量
    pub inertia: f32,
    
    /// 线性阻尼
    pub linear_damping: f32,
    /// 角阻尼
    pub angular_damping: f32,
    
    /// 是否使用重力
    pub use_gravity: bool,
    /// 重力比例
    pub gravity_scale: f32,
    
    /// 累积的力
    #[serde(skip)]
    pub force: Vec3,
    /// 累积的扭矩
    #[serde(skip)]
    pub torque: Vec3,
    
    /// 是否冻结位置
    pub freeze_position: Vec3, // 0.0 = 不冻结, 1.0 = 冻结
    /// 是否冻结旋转
    pub freeze_rotation: Vec3, // 0.0 = 不冻结, 1.0 = 冻结
    
    /// 是否处于休眠状态
    #[serde(skip)]
    pub is_sleeping: bool,
    /// 休眠阈值
    pub sleep_threshold: f32,
    /// 休眠计时器
    #[serde(skip)]
    pub sleep_timer: f32,
    
    /// 连续碰撞检测
    pub enable_ccd: bool,
    
    /// 自定义用户数据
    pub user_data: Option<String>,
}

impl Default for PhysicsRigidBody {
    fn default() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            position: Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass: 1.0,
            inertia: 1.0,
            linear_damping: 0.01,
            angular_damping: 0.05,
            use_gravity: true,
            gravity_scale: 1.0,
            force: Vec3::ZERO,
            torque: Vec3::ZERO,
            freeze_position: Vec3::ZERO,
            freeze_rotation: Vec3::ZERO,
            is_sleeping: false,
            sleep_threshold: 0.01,
            sleep_timer: 0.0,
            enable_ccd: false,
            user_data: None,
        }
    }
}

impl PhysicsRigidBody {
    /// 创建静态刚体
    pub fn static_body() -> Self {
        Self {
            body_type: RigidBodyType::Static,
            mass: f32::INFINITY,
            use_gravity: false,
            ..Default::default()
        }
    }

    /// 创建运动学刚体
    pub fn kinematic_body() -> Self {
        Self {
            body_type: RigidBodyType::Kinematic,
            use_gravity: false,
            ..Default::default()
        }
    }

    /// 创建动态刚体
    pub fn dynamic_body() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            ..Default::default()
        }
    }

    /// 设置质量
    pub fn with_mass(mut self, mass: f32) -> Self {
        self.set_mass(mass);
        self
    }

    /// 设置质量和计算转动惯量
    pub fn set_mass(&mut self, mass: f32) {
        self.mass = mass.max(0.001); // 防止零质量
        
        // 简化的转动惯量计算（假设为球体）
        self.inertia = (2.0 / 5.0) * self.mass;
        
        // 如果是无限质量，设置为静态
        if mass == f32::INFINITY {
            self.body_type = RigidBodyType::Static;
        }
    }

    /// 设置位置
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.wake_up();
    }

    /// 设置旋转
    pub fn set_rotation(&mut self, rotation: glam::Quat) {
        self.rotation = rotation.normalize();
        self.wake_up();
    }

    /// 设置速度
    pub fn set_velocity(&mut self, velocity: Vec3) {
        if self.body_type == RigidBodyType::Dynamic {
            self.velocity = self.apply_position_constraints(velocity);
            self.wake_up();
        }
    }

    /// 设置角速度
    pub fn set_angular_velocity(&mut self, angular_velocity: Vec3) {
        if self.body_type == RigidBodyType::Dynamic {
            self.angular_velocity = self.apply_rotation_constraints(angular_velocity);
            self.wake_up();
        }
    }

    /// 添加力
    pub fn add_force(&mut self, force: Vec3) {
        if self.body_type == RigidBodyType::Dynamic {
            self.force += self.apply_position_constraints(force);
            self.wake_up();
        }
    }

    /// 在指定点添加力
    pub fn add_force_at_position(&mut self, force: Vec3, position: Vec3) {
        if self.body_type == RigidBodyType::Dynamic {
            self.add_force(force);
            
            // 计算扭矩
            let r = position - self.position;
            let torque = r.cross(force);
            self.add_torque(torque);
        }
    }

    /// 添加冲量
    pub fn add_impulse(&mut self, impulse: Vec3) {
        if self.body_type == RigidBodyType::Dynamic && self.mass > 0.0 {
            let velocity_change = impulse / self.mass;
            self.velocity += self.apply_position_constraints(velocity_change);
            self.wake_up();
        }
    }

    /// 在指定点添加冲量
    pub fn add_impulse_at_position(&mut self, impulse: Vec3, position: Vec3) {
        if self.body_type == RigidBodyType::Dynamic {
            self.add_impulse(impulse);
            
            // 计算角冲量
            let r = position - self.position;
            let angular_impulse = r.cross(impulse);
            self.add_angular_impulse(angular_impulse);
        }
    }

    /// 添加扭矩
    pub fn add_torque(&mut self, torque: Vec3) {
        if self.body_type == RigidBodyType::Dynamic {
            self.torque += self.apply_rotation_constraints(torque);
            self.wake_up();
        }
    }

    /// 添加角冲量
    pub fn add_angular_impulse(&mut self, angular_impulse: Vec3) {
        if self.body_type == RigidBodyType::Dynamic && self.inertia > 0.0 {
            let angular_velocity_change = angular_impulse / self.inertia;
            self.angular_velocity += self.apply_rotation_constraints(angular_velocity_change);
            self.wake_up();
        }
    }

    /// 获取动能
    pub fn kinetic_energy(&self) -> f32 {
        if self.mass == f32::INFINITY {
            return 0.0;
        }
        
        let linear_energy = 0.5 * self.mass * self.velocity.length_squared();
        let angular_energy = 0.5 * self.inertia * self.angular_velocity.length_squared();
        
        linear_energy + angular_energy
    }

    /// 获取动量
    pub fn momentum(&self) -> Vec3 {
        if self.mass == f32::INFINITY {
            Vec3::ZERO
        } else {
            self.velocity * self.mass
        }
    }

    /// 获取角动量
    pub fn angular_momentum(&self) -> Vec3 {
        if self.inertia == f32::INFINITY {
            Vec3::ZERO
        } else {
            self.angular_velocity * self.inertia
        }
    }

    /// 冻结位置轴
    pub fn freeze_position_x(&mut self) {
        self.freeze_position.x = 1.0;
    }

    pub fn freeze_position_y(&mut self) {
        self.freeze_position.y = 1.0;
    }

    pub fn freeze_position_z(&mut self) {
        self.freeze_position.z = 1.0;
    }

    /// 冻结旋转轴
    pub fn freeze_rotation_x(&mut self) {
        self.freeze_rotation.x = 1.0;
    }

    pub fn freeze_rotation_y(&mut self) {
        self.freeze_rotation.y = 1.0;
    }

    pub fn freeze_rotation_z(&mut self) {
        self.freeze_rotation.z = 1.0;
    }

    /// 应用位置约束
    fn apply_position_constraints(&self, mut value: Vec3) -> Vec3 {
        value.x *= 1.0 - self.freeze_position.x;
        value.y *= 1.0 - self.freeze_position.y;
        value.z *= 1.0 - self.freeze_position.z;
        value
    }

    /// 应用旋转约束
    fn apply_rotation_constraints(&self, mut value: Vec3) -> Vec3 {
        value.x *= 1.0 - self.freeze_rotation.x;
        value.y *= 1.0 - self.freeze_rotation.y;
        value.z *= 1.0 - self.freeze_rotation.z;
        value
    }

    /// 唤醒刚体
    pub fn wake_up(&mut self) {
        self.is_sleeping = false;
        self.sleep_timer = 0.0;
    }

    /// 强制休眠
    pub fn sleep(&mut self) {
        self.is_sleeping = true;
        self.velocity = Vec3::ZERO;
        self.angular_velocity = Vec3::ZERO;
        self.force = Vec3::ZERO;
        self.torque = Vec3::ZERO;
    }

    /// 检查是否应该休眠
    pub fn should_sleep(&self, dt: f32) -> bool {
        if self.body_type != RigidBodyType::Dynamic {
            return false;
        }
        
        let energy = self.kinetic_energy();
        energy < self.sleep_threshold
    }

    /// 更新休眠状态
    pub fn update_sleep_state(&mut self, dt: f32) {
        if self.body_type != RigidBodyType::Dynamic {
            return;
        }
        
        if self.should_sleep(dt) {
            self.sleep_timer += dt;
            if self.sleep_timer > 1.0 { // 1秒后休眠
                self.sleep();
            }
        } else {
            self.sleep_timer = 0.0;
            self.is_sleeping = false;
        }
    }

    /// 清空累积的力和扭矩
    pub fn clear_forces(&mut self) {
        self.force = Vec3::ZERO;
        self.torque = Vec3::ZERO;
    }

    /// 设置线性阻尼
    pub fn with_linear_damping(mut self, damping: f32) -> Self {
        self.linear_damping = damping.clamp(0.0, 1.0);
        self
    }

    /// 设置角阻尼
    pub fn with_angular_damping(mut self, damping: f32) -> Self {
        self.angular_damping = damping.clamp(0.0, 1.0);
        self
    }

    /// 禁用重力
    pub fn without_gravity(mut self) -> Self {
        self.use_gravity = false;
        self
    }

    /// 设置重力比例
    pub fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    /// 启用连续碰撞检测
    pub fn with_ccd(mut self) -> Self {
        self.enable_ccd = true;
        self
    }

    /// 设置用户数据
    pub fn with_user_data(mut self, data: impl Into<String>) -> Self {
        self.user_data = Some(data.into());
        self
    }

    /// 检查是否为静态刚体
    pub fn is_static(&self) -> bool {
        self.body_type == RigidBodyType::Static
    }

    /// 检查是否为动态刚体
    pub fn is_dynamic(&self) -> bool {
        self.body_type == RigidBodyType::Dynamic
    }

    /// 检查是否为运动学刚体
    pub fn is_kinematic(&self) -> bool {
        self.body_type == RigidBodyType::Kinematic
    }

    /// 获取世界变换矩阵
    pub fn world_transform(&self) -> glam::Mat4 {
        glam::Mat4::from_rotation_translation(self.rotation, self.position)
    }

    /// 从世界坐标转换为局部坐标
    pub fn world_to_local(&self, world_point: Vec3) -> Vec3 {
        let local_point = world_point - self.position;
        self.rotation.conjugate() * local_point
    }

    /// 从局部坐标转换为世界坐标
    pub fn local_to_world(&self, local_point: Vec3) -> Vec3 {
        self.position + self.rotation * local_point
    }

    /// 从世界方向转换为局部方向
    pub fn world_to_local_direction(&self, world_direction: Vec3) -> Vec3 {
        self.rotation.conjugate() * world_direction
    }

    /// 从局部方向转换为世界方向
    pub fn local_to_world_direction(&self, local_direction: Vec3) -> Vec3 {
        self.rotation * local_direction
    }
}
