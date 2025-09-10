//! 碰撞体组件

use crate::math::{Vec3, AABB, BoundingSphere};
use serde::{Deserialize, Serialize};

/// 碰撞体形状类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColliderShape {
    /// 球体
    Sphere {
        radius: f32,
    },
    /// 盒子
    Box {
        half_extents: Vec3,
    },
    /// 胶囊体
    Capsule {
        radius: f32,
        height: f32,
    },
    /// 圆柱体
    Cylinder {
        radius: f32,
        height: f32,
    },
    /// 平面
    Plane {
        normal: Vec3,
        distance: f32,
    },
    /// 网格（用于复杂几何体）
    Mesh {
        vertices: Vec<Vec3>,
        indices: Vec<u32>,
    },
    /// 复合形状（多个形状组合）
    Compound {
        shapes: Vec<(Vec3, ColliderShape)>, // (相对位置, 形状)
    },
}

impl Default for ColliderShape {
    fn default() -> Self {
        Self::Box {
            half_extents: Vec3::new(0.5, 0.5, 0.5),
        }
    }
}

impl ColliderShape {
    /// 计算形状的AABB
    pub fn compute_aabb(&self, position: Vec3, rotation: glam::Quat) -> AABB {
        match self {
            ColliderShape::Sphere { radius } => {
                AABB::from_center_size(position, Vec3::splat(*radius * 2.0))
            }
            ColliderShape::Box { half_extents } => {
                // 计算旋转后的AABB
                let corners = [
                    Vec3::new(-half_extents.x, -half_extents.y, -half_extents.z),
                    Vec3::new( half_extents.x, -half_extents.y, -half_extents.z),
                    Vec3::new(-half_extents.x,  half_extents.y, -half_extents.z),
                    Vec3::new( half_extents.x,  half_extents.y, -half_extents.z),
                    Vec3::new(-half_extents.x, -half_extents.y,  half_extents.z),
                    Vec3::new( half_extents.x, -half_extents.y,  half_extents.z),
                    Vec3::new(-half_extents.x,  half_extents.y,  half_extents.z),
                    Vec3::new( half_extents.x,  half_extents.y,  half_extents.z),
                ];
                
                let rotated_corners: Vec<Vec3> = corners
                    .iter()
                    .map(|&corner| position + rotation * corner)
                    .collect();
                    
                AABB::from_points(&rotated_corners).unwrap_or_default()
            }
            ColliderShape::Capsule { radius, height } => {
                let half_height = *height * 0.5;
                let size = Vec3::new(*radius * 2.0, height + *radius * 2.0, *radius * 2.0);
                AABB::from_center_size(position, size)
            }
            ColliderShape::Cylinder { radius, height } => {
                let size = Vec3::new(*radius * 2.0, *height, *radius * 2.0);
                AABB::from_center_size(position, size)
            }
            ColliderShape::Plane { .. } => {
                // 平面的AABB是无限大的，这里返回一个很大的盒子
                AABB::from_center_size(position, Vec3::splat(10000.0))
            }
            ColliderShape::Mesh { vertices, .. } => {
                if vertices.is_empty() {
                    AABB::from_center_size(position, Vec3::splat(1.0))
                } else {
                    let transformed_vertices: Vec<Vec3> = vertices
                        .iter()
                        .map(|&vertex| position + rotation * vertex)
                        .collect();
                    AABB::from_points(&transformed_vertices).unwrap_or_default()
                }
            }
            ColliderShape::Compound { shapes } => {
                if shapes.is_empty() {
                    return AABB::from_center_size(position, Vec3::splat(1.0));
                }
                
                let mut combined_aabb = shapes[0].1.compute_aabb(
                    position + rotation * shapes[0].0, 
                    rotation
                );
                
                for (relative_pos, shape) in shapes.iter().skip(1) {
                    let shape_aabb = shape.compute_aabb(
                        position + rotation * relative_pos, 
                        rotation
                    );
                    combined_aabb = combined_aabb.union(&shape_aabb);
                }
                
                combined_aabb
            }
        }
    }

    /// 计算形状的边界球
    pub fn compute_bounding_sphere(&self, position: Vec3) -> BoundingSphere {
        match self {
            ColliderShape::Sphere { radius } => {
                BoundingSphere::new(position, *radius)
            }
            ColliderShape::Box { half_extents } => {
                let radius = half_extents.length();
                BoundingSphere::new(position, radius)
            }
            ColliderShape::Capsule { radius, height } => {
                let half_height = *height * 0.5;
                let radius = (*radius * *radius + half_height * half_height).sqrt();
                BoundingSphere::new(position, radius)
            }
            ColliderShape::Cylinder { radius, height } => {
                let half_height = *height * 0.5;
                let radius = (*radius * *radius + half_height * half_height).sqrt();
                BoundingSphere::new(position, radius)
            }
            ColliderShape::Plane { .. } => {
                BoundingSphere::new(position, 10000.0) // 平面的边界球很大
            }
            ColliderShape::Mesh { vertices, .. } => {
                if vertices.is_empty() {
                    BoundingSphere::new(position, 1.0)
                } else {
                    let transformed_vertices: Vec<Vec3> = vertices
                        .iter()
                        .map(|&vertex| position + vertex)
                        .collect();
                    BoundingSphere::from_points(&transformed_vertices).unwrap_or_default()
                }
            }
            ColliderShape::Compound { shapes } => {
                if shapes.is_empty() {
                    return BoundingSphere::new(position, 1.0);
                }
                
                let mut max_radius = 0.0;
                for (relative_pos, shape) in shapes {
                    let shape_sphere = shape.compute_bounding_sphere(position + *relative_pos);
                    let distance = (shape_sphere.center - position).length() + shape_sphere.radius;
                    max_radius = max_radius.max(distance);
                }
                
                BoundingSphere::new(position, max_radius)
            }
        }
    }

    /// 计算形状的体积
    pub fn volume(&self) -> f32 {
        match self {
            ColliderShape::Sphere { radius } => {
                (4.0 / 3.0) * std::f32::consts::PI * radius.powi(3)
            }
            ColliderShape::Box { half_extents } => {
                8.0 * half_extents.x * half_extents.y * half_extents.z
            }
            ColliderShape::Capsule { radius, height } => {
                // 圆柱体 + 两个半球
                let cylinder_volume = std::f32::consts::PI * radius * radius * height;
                let sphere_volume = (4.0 / 3.0) * std::f32::consts::PI * radius.powi(3);
                cylinder_volume + sphere_volume
            }
            ColliderShape::Cylinder { radius, height } => {
                std::f32::consts::PI * radius * radius * height
            }
            ColliderShape::Plane { .. } => {
                f32::INFINITY // 平面体积无限
            }
            ColliderShape::Mesh { .. } => {
                // 网格体积计算比较复杂，这里返回估算值
                1.0
            }
            ColliderShape::Compound { shapes } => {
                shapes.iter().map(|(_, shape)| shape.volume()).sum()
            }
        }
    }

    /// 创建球体形状
    pub fn sphere(radius: f32) -> Self {
        Self::Sphere { radius }
    }

    /// 创建盒子形状
    pub fn cuboid(half_extents: Vec3) -> Self {
        Self::Box { half_extents }
    }

    /// 创建立方体形状
    pub fn cube(half_extent: f32) -> Self {
        Self::Box {
            half_extents: Vec3::splat(half_extent),
        }
    }

    /// 创建胶囊体形状
    pub fn capsule(radius: f32, height: f32) -> Self {
        Self::Capsule { radius, height }
    }

    /// 创建圆柱体形状
    pub fn cylinder(radius: f32, height: f32) -> Self {
        Self::Cylinder { radius, height }
    }

    /// 创建平面形状
    pub fn plane(normal: Vec3) -> Self {
        Self::Plane {
            normal: normal.normalize(),
            distance: 0.0,
        }
    }

    /// 从顶点创建凸包网格
    pub fn convex_hull(vertices: Vec<Vec3>) -> Self {
        Self::Mesh {
            vertices,
            indices: Vec::new(), // 简化处理，实际需要计算凸包
        }
    }
}

/// 碰撞体组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collider {
    /// 碰撞体形状
    pub shape: ColliderShape,
    /// 是否是触发器（不产生物理碰撞，只检测）
    pub is_trigger: bool,
    /// 物理材质
    pub material: ColliderMaterial,
    /// 碰撞组（用于碰撞过滤）
    pub collision_groups: u32,
    /// 碰撞掩码（与哪些组发生碰撞）
    pub collision_mask: u32,
    /// 缓存的AABB
    #[serde(skip)]
    pub aabb: AABB,
    /// 缓存的边界球
    #[serde(skip)]
    pub bounding_sphere: Option<BoundingSphere>,
    /// 是否启用
    pub enabled: bool,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: ColliderShape::default(),
            is_trigger: false,
            material: ColliderMaterial::default(),
            collision_groups: 1,
            collision_mask: u32::MAX,
            aabb: AABB::default(),
            bounding_sphere: None,
            enabled: true,
        }
    }
}

impl Collider {
    /// 创建新的碰撞体
    pub fn new(shape: ColliderShape) -> Self {
        Self {
            shape,
            ..Default::default()
        }
    }

    /// 设置为触发器
    pub fn as_trigger(mut self) -> Self {
        self.is_trigger = true;
        self
    }

    /// 设置碰撞组
    pub fn with_collision_groups(mut self, groups: u32) -> Self {
        self.collision_groups = groups;
        self
    }

    /// 设置碰撞掩码
    pub fn with_collision_mask(mut self, mask: u32) -> Self {
        self.collision_mask = mask;
        self
    }

    /// 设置物理材质
    pub fn with_material(mut self, material: ColliderMaterial) -> Self {
        self.material = material;
        self
    }

    /// 更新缓存的边界信息
    pub fn update_bounds(&mut self, position: Vec3, rotation: glam::Quat) {
        self.aabb = self.shape.compute_aabb(position, rotation);
        self.bounding_sphere = Some(self.shape.compute_bounding_sphere(position));
    }

    /// 检查是否与另一个碰撞体的组匹配
    pub fn can_collide_with(&self, other: &Collider) -> bool {
        (self.collision_mask & other.collision_groups) != 0 &&
        (other.collision_mask & self.collision_groups) != 0
    }

    /// 计算质量（基于形状和密度）
    pub fn compute_mass(&self, density: f32) -> f32 {
        self.shape.volume() * density
    }
}

/// 碰撞体物理材质
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColliderMaterial {
    /// 摩擦系数
    pub friction: f32,
    /// 恢复系数（弹性）
    pub restitution: f32,
    /// 密度
    pub density: f32,
}

impl Default for ColliderMaterial {
    fn default() -> Self {
        Self {
            friction: 0.5,
            restitution: 0.3,
            density: 1.0,
        }
    }
}

impl ColliderMaterial {
    /// 创建摩擦材质
    pub fn friction(friction: f32) -> Self {
        Self {
            friction,
            ..Default::default()
        }
    }

    /// 创建弹性材质
    pub fn bouncy(restitution: f32) -> Self {
        Self {
            restitution,
            ..Default::default()
        }
    }

    /// 创建无摩擦材质
    pub fn frictionless() -> Self {
        Self {
            friction: 0.0,
            ..Default::default()
        }
    }

    /// 创建高弹性材质
    pub fn super_bouncy() -> Self {
        Self {
            restitution: 0.95,
            ..Default::default()
        }
    }

    /// 创建自定义材质
    pub fn new(friction: f32, restitution: f32, density: f32) -> Self {
        Self {
            friction: friction.clamp(0.0, f32::INFINITY),
            restitution: restitution.clamp(0.0, 1.0),
            density: density.max(0.001),
        }
    }

    /// 组合两个材质的属性
    pub fn combine(&self, other: &ColliderMaterial) -> ColliderMaterial {
        ColliderMaterial {
            friction: (self.friction * other.friction).sqrt(),
            restitution: self.restitution.max(other.restitution),
            density: (self.density + other.density) * 0.5,
        }
    }
}

/// 碰撞组预定义常量
pub mod collision_groups {
    pub const ALL: u32 = u32::MAX;
    pub const NONE: u32 = 0;
    pub const DEFAULT: u32 = 1;
    pub const STATIC: u32 = 2;
    pub const KINEMATIC: u32 = 4;
    pub const DEBRIS: u32 = 8;
    pub const SENSOR_TRIGGER: u32 = 16;
    pub const CHARACTER_CONTROLLER: u32 = 32;
    pub const USER_1: u32 = 64;
    pub const USER_2: u32 = 128;
    pub const USER_3: u32 = 256;
    pub const USER_4: u32 = 512;
}
