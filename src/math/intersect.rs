//! 几何相交检测

use glam::Vec3;
use crate::math::bounds::{AABB, BoundingSphere};
use crate::math::ray::{Ray, RayHit};

/// 点与几何体相交检测
pub struct PointIntersection;

impl PointIntersection {
    /// 点是否在三角形内 (2D)
    pub fn point_in_triangle_2d(point: glam::Vec2, a: glam::Vec2, b: glam::Vec2, c: glam::Vec2) -> bool {
        let v0 = c - a;
        let v1 = b - a;
        let v2 = point - a;

        let dot00 = v0.dot(v0);
        let dot01 = v0.dot(v1);
        let dot02 = v0.dot(v2);
        let dot11 = v1.dot(v1);
        let dot12 = v1.dot(v2);

        let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

        u >= 0.0 && v >= 0.0 && (u + v) <= 1.0
    }

    /// 点是否在多边形内 (2D, 射线法)
    pub fn point_in_polygon_2d(point: glam::Vec2, vertices: &[glam::Vec2]) -> bool {
        let mut inside = false;
        let mut j = vertices.len() - 1;

        for i in 0..vertices.len() {
            if ((vertices[i].y > point.y) != (vertices[j].y > point.y)) &&
               (point.x < (vertices[j].x - vertices[i].x) * (point.y - vertices[i].y) / (vertices[j].y - vertices[i].y) + vertices[i].x) {
                inside = !inside;
            }
            j = i;
        }

        inside
    }
}

/// 线段相交检测
pub struct LineIntersection;

impl LineIntersection {
    /// 两条2D线段是否相交
    pub fn segments_intersect_2d(
        p1: glam::Vec2, q1: glam::Vec2,
        p2: glam::Vec2, q2: glam::Vec2
    ) -> bool {
        fn orientation(p: glam::Vec2, q: glam::Vec2, r: glam::Vec2) -> i32 {
            let val = (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y);
            if val.abs() < f32::EPSILON { 0 } // 共线
            else if val > 0.0 { 1 } // 顺时针
            else { 2 } // 逆时针
        }

        fn on_segment(p: glam::Vec2, q: glam::Vec2, r: glam::Vec2) -> bool {
            q.x <= p.x.max(r.x) && q.x >= p.x.min(r.x) &&
            q.y <= p.y.max(r.y) && q.y >= p.y.min(r.y)
        }

        let o1 = orientation(p1, q1, p2);
        let o2 = orientation(p1, q1, q2);
        let o3 = orientation(p2, q2, p1);
        let o4 = orientation(p2, q2, q1);

        // 一般情况
        if o1 != o2 && o3 != o4 {
            return true;
        }

        // 特殊情况
        // p1, q1 和 p2 共线，且 p2 在线段 p1q1 上
        if o1 == 0 && on_segment(p1, p2, q1) { return true; }
        // p1, q1 和 q2 共线，且 q2 在线段 p1q1 上
        if o2 == 0 && on_segment(p1, q2, q1) { return true; }
        // p2, q2 和 p1 共线，且 p1 在线段 p2q2 上
        if o3 == 0 && on_segment(p2, p1, q2) { return true; }
        // p2, q2 和 q1 共线，且 q1 在线段 p2q2 上
        if o4 == 0 && on_segment(p2, q1, q2) { return true; }

        false
    }

    /// 计算两条2D线段的交点
    pub fn segment_intersection_2d(
        p1: glam::Vec2, q1: glam::Vec2,
        p2: glam::Vec2, q2: glam::Vec2
    ) -> Option<glam::Vec2> {
        let d1 = q1 - p1;
        let d2 = q2 - p2;
        let d3 = p1 - p2;

        let cross = d1.x * d2.y - d1.y * d2.x;
        if cross.abs() < f32::EPSILON {
            return None; // 平行线
        }

        let t1 = (d3.x * d2.y - d3.y * d2.x) / cross;
        let t2 = (d3.x * d1.y - d3.y * d1.x) / cross;

        if t1 >= 0.0 && t1 <= 1.0 && t2 >= 0.0 && t2 <= 1.0 {
            Some(p1 + d1 * t1)
        } else {
            None
        }
    }
}

/// 球体相交检测
pub struct SphereIntersection;

impl SphereIntersection {
    /// 两个球体是否相交
    pub fn sphere_sphere(sphere1: &BoundingSphere, sphere2: &BoundingSphere) -> bool {
        let distance_sq = (sphere1.center - sphere2.center).length_squared();
        let radius_sum = sphere1.radius + sphere2.radius;
        distance_sq <= radius_sum * radius_sum
    }

    /// 球体与AABB是否相交
    pub fn sphere_aabb(sphere: &BoundingSphere, aabb: &AABB) -> bool {
        let closest_point = aabb.closest_point(sphere.center);
        (closest_point - sphere.center).length_squared() <= sphere.radius * sphere.radius
    }

    /// 球体与射线相交
    pub fn sphere_ray(sphere: &BoundingSphere, ray: &Ray) -> Option<RayHit> {
        ray.intersect_sphere(sphere)
    }
}

/// AABB相交检测
pub struct AABBIntersection;

impl AABBIntersection {
    /// 两个AABB是否相交
    pub fn aabb_aabb(aabb1: &AABB, aabb2: &AABB) -> bool {
        aabb1.intersects(aabb2)
    }

    /// AABB与射线相交
    pub fn aabb_ray(aabb: &AABB, ray: &Ray) -> Option<RayHit> {
        ray.intersect_aabb(aabb)
    }

    /// AABB与平面相交检测
    pub fn aabb_plane(aabb: &AABB, plane_normal: Vec3, plane_distance: f32) -> bool {
        let center = aabb.center();
        let extents = aabb.extents();
        
        // 计算AABB在平面法线方向上的投影半径
        let r = extents.x * plane_normal.x.abs() +
                extents.y * plane_normal.y.abs() +
                extents.z * plane_normal.z.abs();
        
        // 计算中心到平面的距离
        let distance = plane_normal.dot(center) - plane_distance;
        
        // 如果距离小于等于投影半径，则相交
        distance.abs() <= r
    }
}

/// 三角形相交检测
pub struct TriangleIntersection;

impl TriangleIntersection {
    /// 射线与三角形相交
    pub fn ray_triangle(ray: &Ray, v0: Vec3, v1: Vec3, v2: Vec3) -> Option<RayHit> {
        ray.intersect_triangle(v0, v1, v2)
    }

    /// 点是否在三角形内 (3D重心坐标法)
    pub fn point_in_triangle_3d(point: Vec3, v0: Vec3, v1: Vec3, v2: Vec3) -> bool {
        // 计算三角形法线
        let normal = (v1 - v0).cross(v2 - v0);
        if normal.length_squared() < f32::EPSILON {
            return false; // 退化三角形
        }

        // 将点投影到三角形平面
        let plane_distance = normal.dot(v0);
        let point_distance = normal.dot(point);
        let projected_point = point - normal * ((point_distance - plane_distance) / normal.length_squared());

        // 使用重心坐标
        let v0_to_v1 = v1 - v0;
        let v0_to_v2 = v2 - v0;
        let v0_to_point = projected_point - v0;

        let dot00 = v0_to_v2.dot(v0_to_v2);
        let dot01 = v0_to_v2.dot(v0_to_v1);
        let dot02 = v0_to_v2.dot(v0_to_point);
        let dot11 = v0_to_v1.dot(v0_to_v1);
        let dot12 = v0_to_v1.dot(v0_to_point);

        let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
        let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
        let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

        u >= 0.0 && v >= 0.0 && (u + v) <= 1.0
    }

    /// 两个三角形是否相交 (SAT算法简化版)
    pub fn triangle_triangle(
        t1_v0: Vec3, t1_v1: Vec3, t1_v2: Vec3,
        t2_v0: Vec3, t2_v1: Vec3, t2_v2: Vec3
    ) -> bool {
        // 简化实现：检查是否有任何顶点在另一个三角形内
        Self::point_in_triangle_3d(t1_v0, t2_v0, t2_v1, t2_v2) ||
        Self::point_in_triangle_3d(t1_v1, t2_v0, t2_v1, t2_v2) ||
        Self::point_in_triangle_3d(t1_v2, t2_v0, t2_v1, t2_v2) ||
        Self::point_in_triangle_3d(t2_v0, t1_v0, t1_v1, t1_v2) ||
        Self::point_in_triangle_3d(t2_v1, t1_v0, t1_v1, t1_v2) ||
        Self::point_in_triangle_3d(t2_v2, t1_v0, t1_v1, t1_v2)
    }
}

/// 最近点计算
pub struct ClosestPoint;

impl ClosestPoint {
    /// 点到线段的最近点
    pub fn point_to_segment(point: Vec3, segment_start: Vec3, segment_end: Vec3) -> Vec3 {
        let segment = segment_end - segment_start;
        let point_to_start = point - segment_start;
        
        let segment_length_sq = segment.length_squared();
        if segment_length_sq < f32::EPSILON {
            return segment_start; // 退化线段
        }
        
        let t = (point_to_start.dot(segment) / segment_length_sq).clamp(0.0, 1.0);
        segment_start + segment * t
    }

    /// 两条线段之间的最近点
    pub fn segment_to_segment(
        seg1_start: Vec3, seg1_end: Vec3,
        seg2_start: Vec3, seg2_end: Vec3
    ) -> (Vec3, Vec3) {
        let d1 = seg1_end - seg1_start;
        let d2 = seg2_end - seg2_start;
        let r = seg1_start - seg2_start;
        
        let a = d1.dot(d1);
        let e = d2.dot(d2);
        let f = d2.dot(r);
        
        let mut s = 0.0;
        let mut t = 0.0;
        
        if a <= f32::EPSILON && e <= f32::EPSILON {
            // 两条线段都是点
            return (seg1_start, seg2_start);
        }
        
        if a <= f32::EPSILON {
            // 第一条线段是点
            t = (f / e).clamp(0.0, 1.0);
        } else {
            let c = d1.dot(r);
            if e <= f32::EPSILON {
                // 第二条线段是点
                s = (-c / a).clamp(0.0, 1.0);
            } else {
                // 一般情况
                let b = d1.dot(d2);
                let denom = a * e - b * b;
                
                if denom != 0.0 {
                    s = ((b * f - c * e) / denom).clamp(0.0, 1.0);
                }
                
                t = (b * s + f) / e;
                
                if t < 0.0 {
                    t = 0.0;
                    s = (-c / a).clamp(0.0, 1.0);
                } else if t > 1.0 {
                    t = 1.0;
                    s = ((b - c) / a).clamp(0.0, 1.0);
                }
            }
        }
        
        let point1 = seg1_start + d1 * s;
        let point2 = seg2_start + d2 * t;
        
        (point1, point2)
    }
}
