//! 视锥体工具

use glam::{Vec3, Vec4, Mat4};
use crate::math::bounds::{AABB, BoundingSphere};

/// 平面
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

impl Plane {
    /// 创建新的平面
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal: normal.normalize(), distance }
    }

    /// 从三个点创建平面
    pub fn from_points(p1: Vec3, p2: Vec3, p3: Vec3) -> Self {
        let normal = (p2 - p1).cross(p3 - p1).normalize();
        let distance = normal.dot(p1);
        Self { normal, distance }
    }

    /// 从法线和点创建平面
    pub fn from_normal_and_point(normal: Vec3, point: Vec3) -> Self {
        let normal = normal.normalize();
        let distance = normal.dot(point);
        Self { normal, distance }
    }

    /// 计算点到平面的距离
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) - self.distance
    }

    /// 检查点在平面的哪一边
    pub fn side_of_plane(&self, point: Vec3) -> PlaneSide {
        let distance = self.distance_to_point(point);
        if distance > 0.0 {
            PlaneSide::Front
        } else if distance < 0.0 {
            PlaneSide::Back
        } else {
            PlaneSide::On
        }
    }
}

/// 平面侧面枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaneSide {
    Front,
    Back,
    On,
}

/// 视锥体
#[derive(Debug, Clone)]
pub struct Frustum {
    planes: [Plane; 6],
}

impl Frustum {
    /// 从视图投影矩阵创建视锥体
    pub fn from_view_projection_matrix(view_proj: Mat4) -> Self {
        let m = view_proj.to_cols_array();
        
        // 提取6个平面 (左、右、下、上、近、远)
        let planes = [
            // 左平面
            Plane::new(
                Vec3::new(m[3] + m[0], m[7] + m[4], m[11] + m[8]),
                m[15] + m[12]
            ),
            // 右平面
            Plane::new(
                Vec3::new(m[3] - m[0], m[7] - m[4], m[11] - m[8]),
                m[15] - m[12]
            ),
            // 下平面
            Plane::new(
                Vec3::new(m[3] + m[1], m[7] + m[5], m[11] + m[9]),
                m[15] + m[13]
            ),
            // 上平面
            Plane::new(
                Vec3::new(m[3] - m[1], m[7] - m[5], m[11] - m[9]),
                m[15] - m[13]
            ),
            // 近平面
            Plane::new(
                Vec3::new(m[3] + m[2], m[7] + m[6], m[11] + m[10]),
                m[15] + m[14]
            ),
            // 远平面
            Plane::new(
                Vec3::new(m[3] - m[2], m[7] - m[6], m[11] - m[10]),
                m[15] - m[14]
            ),
        ];

        Self { planes }
    }

    /// 获取指定平面
    pub fn plane(&self, index: usize) -> Option<&Plane> {
        self.planes.get(index)
    }

    /// 检查点是否在视锥体内
    pub fn contains_point(&self, point: Vec3) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(point) < 0.0 {
                return false;
            }
        }
        true
    }

    /// 检查球体是否在视锥体内
    pub fn intersects_sphere(&self, sphere: &BoundingSphere) -> FrustumIntersection {
        let mut inside = true;
        
        for plane in &self.planes {
            let distance = plane.distance_to_point(sphere.center);
            
            if distance < -sphere.radius {
                return FrustumIntersection::Outside;
            } else if distance < sphere.radius {
                inside = false;
            }
        }
        
        if inside {
            FrustumIntersection::Inside
        } else {
            FrustumIntersection::Intersects
        }
    }

    /// 检查AABB是否在视锥体内
    pub fn intersects_aabb(&self, aabb: &AABB) -> FrustumIntersection {
        let mut inside = true;
        
        for plane in &self.planes {
            let mut in_count = 0;
            let mut out_count = 0;
            
            // 检查AABB的8个角点
            for i in 0..8 {
                let corner = Vec3::new(
                    if i & 1 != 0 { aabb.max.x } else { aabb.min.x },
                    if i & 2 != 0 { aabb.max.y } else { aabb.min.y },
                    if i & 4 != 0 { aabb.max.z } else { aabb.min.z },
                );
                
                if plane.distance_to_point(corner) < 0.0 {
                    out_count += 1;
                } else {
                    in_count += 1;
                }
            }
            
            // 如果所有点都在平面外侧，则AABB在视锥体外
            if in_count == 0 {
                return FrustumIntersection::Outside;
            }
            
            // 如果有点在平面外侧，则不完全在内侧
            if out_count > 0 {
                inside = false;
            }
        }
        
        if inside {
            FrustumIntersection::Inside
        } else {
            FrustumIntersection::Intersects
        }
    }

    /// 获取视锥体的8个角点
    pub fn corners(&self) -> [Vec3; 8] {
        // 这是一个简化的实现，实际应该从平面交点计算
        // 这里返回默认值，实际使用时需要完善
        [Vec3::ZERO; 8]
    }
}

/// 视锥体相交结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrustumIntersection {
    Outside,    // 完全在外面
    Inside,     // 完全在里面
    Intersects, // 相交
}

/// 视锥体剔除工具
pub struct FrustumCuller {
    frustum: Frustum,
}

impl FrustumCuller {
    /// 创建新的视锥体剔除器
    pub fn new(view_proj: Mat4) -> Self {
        Self {
            frustum: Frustum::from_view_projection_matrix(view_proj),
        }
    }

    /// 更新视锥体
    pub fn update(&mut self, view_proj: Mat4) {
        self.frustum = Frustum::from_view_projection_matrix(view_proj);
    }

    /// 检查球体是否可见
    pub fn is_sphere_visible(&self, sphere: &BoundingSphere) -> bool {
        self.frustum.intersects_sphere(sphere) != FrustumIntersection::Outside
    }

    /// 检查AABB是否可见
    pub fn is_aabb_visible(&self, aabb: &AABB) -> bool {
        self.frustum.intersects_aabb(aabb) != FrustumIntersection::Outside
    }

    /// 检查点是否可见
    pub fn is_point_visible(&self, point: Vec3) -> bool {
        self.frustum.contains_point(point)
    }
}
