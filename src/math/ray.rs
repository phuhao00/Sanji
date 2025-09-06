//! 射线和射线投射

use glam::{Vec3, Mat4};
use crate::math::bounds::{AABB, BoundingSphere};
use serde::{Deserialize, Serialize};

/// 射线
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// 创建新的射线
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// 获取射线上的点
    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    /// 与AABB相交测试
    pub fn intersect_aabb(&self, aabb: &AABB) -> Option<RayHit> {
        let inv_dir = Vec3::ONE / self.direction;
        
        let t1 = (aabb.min - self.origin) * inv_dir;
        let t2 = (aabb.max - self.origin) * inv_dir;
        
        let t_min = t1.min(t2);
        let t_max = t1.max(t2);
        
        let t_near = t_min.x.max(t_min.y).max(t_min.z);
        let t_far = t_max.x.min(t_max.y).min(t_max.z);
        
        if t_near > t_far || t_far < 0.0 {
            None
        } else {
            let distance = if t_near > 0.0 { t_near } else { t_far };
            let point = self.point_at(distance);
            
            // 计算法线
            let center = aabb.center();
            let local_point = point - center;
            let extents = aabb.extents();
            
            let normal = if local_point.x.abs() / extents.x > local_point.y.abs() / extents.y &&
                           local_point.x.abs() / extents.x > local_point.z.abs() / extents.z {
                Vec3::new(local_point.x.signum(), 0.0, 0.0)
            } else if local_point.y.abs() / extents.y > local_point.z.abs() / extents.z {
                Vec3::new(0.0, local_point.y.signum(), 0.0)
            } else {
                Vec3::new(0.0, 0.0, local_point.z.signum())
            };
            
            Some(RayHit {
                point,
                normal,
                distance,
            })
        }
    }

    /// 与球体相交测试
    pub fn intersect_sphere(&self, sphere: &BoundingSphere) -> Option<RayHit> {
        let oc = self.origin - sphere.center;
        let a = self.direction.dot(self.direction);
        let b = 2.0 * oc.dot(self.direction);
        let c = oc.dot(oc) - sphere.radius * sphere.radius;
        
        let discriminant = b * b - 4.0 * a * c;
        
        if discriminant < 0.0 {
            None
        } else {
            let sqrt_discriminant = discriminant.sqrt();
            let t1 = (-b - sqrt_discriminant) / (2.0 * a);
            let t2 = (-b + sqrt_discriminant) / (2.0 * a);
            
            let t = if t1 > 0.0 { t1 } else if t2 > 0.0 { t2 } else { return None };
            
            let point = self.point_at(t);
            let normal = (point - sphere.center).normalize();
            
            Some(RayHit {
                point,
                normal,
                distance: t,
            })
        }
    }

    /// 与平面相交测试
    pub fn intersect_plane(&self, plane_point: Vec3, plane_normal: Vec3) -> Option<RayHit> {
        let denom = plane_normal.dot(self.direction);
        
        if denom.abs() < 1e-6 {
            None // 射线与平面平行
        } else {
            let t = (plane_point - self.origin).dot(plane_normal) / denom;
            
            if t >= 0.0 {
                let point = self.point_at(t);
                Some(RayHit {
                    point,
                    normal: plane_normal,
                    distance: t,
                })
            } else {
                None
            }
        }
    }

    /// 与三角形相交测试 (Möller-Trumbore算法)
    pub fn intersect_triangle(&self, v0: Vec3, v1: Vec3, v2: Vec3) -> Option<RayHit> {
        const EPSILON: f32 = 1e-8;
        
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let h = self.direction.cross(edge2);
        let a = edge1.dot(h);
        
        if a > -EPSILON && a < EPSILON {
            return None; // 射线与三角形平行
        }
        
        let f = 1.0 / a;
        let s = self.origin - v0;
        let u = f * s.dot(h);
        
        if u < 0.0 || u > 1.0 {
            return None;
        }
        
        let q = s.cross(edge1);
        let v = f * self.direction.dot(q);
        
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        
        let t = f * edge2.dot(q);
        
        if t > EPSILON {
            let point = self.point_at(t);
            let normal = edge1.cross(edge2).normalize();
            
            Some(RayHit {
                point,
                normal,
                distance: t,
            })
        } else {
            None
        }
    }

    /// 变换射线
    pub fn transform(&self, matrix: &Mat4) -> Self {
        let transformed_origin = matrix.transform_point3(self.origin);
        let transformed_end = matrix.transform_point3(self.origin + self.direction);
        let transformed_direction = (transformed_end - transformed_origin).normalize();
        
        Self::new(transformed_origin, transformed_direction)
    }
}

/// 射线击中信息
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

impl RayHit {
    /// 创建新的射线击中信息
    pub fn new(point: Vec3, normal: Vec3, distance: f32) -> Self {
        Self { point, normal, distance }
    }
}

/// 射线投射结果
#[derive(Debug, Clone)]
pub struct RaycastResult {
    pub hits: Vec<RayHit>,
}

impl RaycastResult {
    /// 创建新的射线投射结果
    pub fn new() -> Self {
        Self { hits: Vec::new() }
    }

    /// 添加击中
    pub fn add_hit(&mut self, hit: RayHit) {
        self.hits.push(hit);
    }

    /// 获取最近的击中
    pub fn closest_hit(&self) -> Option<&RayHit> {
        self.hits.iter().min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
    }

    /// 是否有任何击中
    pub fn has_hit(&self) -> bool {
        !self.hits.is_empty()
    }

    /// 按距离排序击中
    pub fn sort_by_distance(&mut self) {
        self.hits.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
    }
}

impl Default for RaycastResult {
    fn default() -> Self {
        Self::new()
    }
}

/// 从屏幕空间创建射线的工具
pub struct RayFromScreen;

impl RayFromScreen {
    /// 从屏幕坐标创建世界空间射线
    pub fn create_ray(
        screen_pos: glam::Vec2,
        screen_size: glam::Vec2,
        view_matrix: Mat4,
        projection_matrix: Mat4,
    ) -> Ray {
        // 转换到NDC坐标
        let ndc_x = (2.0 * screen_pos.x) / screen_size.x - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y) / screen_size.y;
        
        // 创建射线方向 (在裁剪空间中)
        let clip_coords = glam::Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
        
        // 转换到观察空间
        let view_projection_inv = (projection_matrix * view_matrix).inverse();
        let world_coords = view_projection_inv * clip_coords;
        
        if world_coords.w == 0.0 {
            // 平行投影
            let direction = (view_matrix.inverse() * glam::Vec4::new(0.0, 0.0, -1.0, 0.0)).xyz().normalize();
            let origin = glam::Vec3::new(world_coords.x, world_coords.y, world_coords.z);
            Ray::new(origin, direction)
        } else {
            // 透视投影
            let world_pos = world_coords.xyz() / world_coords.w;
            let camera_pos = view_matrix.inverse().transform_point3(Vec3::ZERO);
            let direction = (world_pos - camera_pos).normalize();
            Ray::new(camera_pos, direction)
        }
    }
}
