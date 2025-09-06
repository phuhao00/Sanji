//! 边界盒和边界球

use glam::{Vec3, Mat4};
use serde::{Deserialize, Serialize};

/// 轴对齐边界盒(AABB)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    /// 创建新的AABB
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// 创建从中心点和大小的AABB
    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half_size = size * 0.5;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    /// 创建包含所有点的AABB
    pub fn from_points(points: &[Vec3]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let mut min = points[0];
        let mut max = points[0];

        for &point in points.iter().skip(1) {
            min = min.min(point);
            max = max.max(point);
        }

        Some(Self::new(min, max))
    }

    /// 获取中心点
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// 获取大小
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// 获取半大小
    pub fn extents(&self) -> Vec3 {
        self.size() * 0.5
    }

    /// 获取表面积
    pub fn surface_area(&self) -> f32 {
        let size = self.size();
        2.0 * (size.x * size.y + size.y * size.z + size.z * size.x)
    }

    /// 获取体积
    pub fn volume(&self) -> f32 {
        let size = self.size();
        size.x * size.y * size.z
    }

    /// 检查点是否在边界盒内
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }

    /// 检查另一个AABB是否完全在此AABB内
    pub fn contains(&self, other: &AABB) -> bool {
        other.min.x >= self.min.x && other.max.x <= self.max.x &&
        other.min.y >= self.min.y && other.max.y <= self.max.y &&
        other.min.z >= self.min.z && other.max.z <= self.max.z
    }

    /// 检查与另一个AABB是否相交
    pub fn intersects(&self, other: &AABB) -> bool {
        self.max.x >= other.min.x && self.min.x <= other.max.x &&
        self.max.y >= other.min.y && self.min.y <= other.max.y &&
        self.max.z >= other.min.z && self.min.z <= other.max.z
    }

    /// 扩展AABB包含指定点
    pub fn expand_to_include(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// 扩展AABB包含另一个AABB
    pub fn expand_to_include_aabb(&mut self, other: &AABB) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    /// 计算与另一个AABB的交集
    pub fn intersection(&self, other: &AABB) -> Option<AABB> {
        let min = self.min.max(other.min);
        let max = self.max.min(other.max);
        
        if min.x <= max.x && min.y <= max.y && min.z <= max.z {
            Some(AABB::new(min, max))
        } else {
            None
        }
    }

    /// 计算与另一个AABB的合并
    pub fn union(&self, other: &AABB) -> AABB {
        AABB::new(
            self.min.min(other.min),
            self.max.max(other.max)
        )
    }

    /// 通过变换矩阵变换AABB
    pub fn transform(&self, matrix: &Mat4) -> AABB {
        let corners = [
            self.min,
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            self.max,
        ];

        let transformed_corners: Vec<Vec3> = corners
            .iter()
            .map(|&corner| matrix.transform_point3(corner))
            .collect();

        AABB::from_points(&transformed_corners).unwrap_or(*self)
    }

    /// 获取最近的点
    pub fn closest_point(&self, point: Vec3) -> Vec3 {
        point.clamp(self.min, self.max)
    }

    /// 计算到点的距离
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        let closest = self.closest_point(point);
        (point - closest).length()
    }
}

impl Default for AABB {
    fn default() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }
}

/// 边界球
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}

impl BoundingSphere {
    /// 创建新的边界球
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    /// 从AABB创建边界球
    pub fn from_aabb(aabb: &AABB) -> Self {
        let center = aabb.center();
        let radius = aabb.extents().length();
        Self::new(center, radius)
    }

    /// 从点集创建边界球
    pub fn from_points(points: &[Vec3]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        // 简单实现：使用AABB的外接球
        let aabb = AABB::from_points(points)?;
        Some(Self::from_aabb(&aabb))
    }

    /// 检查点是否在球内
    pub fn contains_point(&self, point: Vec3) -> bool {
        (point - self.center).length_squared() <= self.radius * self.radius
    }

    /// 检查与另一个球是否相交
    pub fn intersects(&self, other: &BoundingSphere) -> bool {
        let distance_squared = (self.center - other.center).length_squared();
        let radius_sum = self.radius + other.radius;
        distance_squared <= radius_sum * radius_sum
    }

    /// 检查与AABB是否相交
    pub fn intersects_aabb(&self, aabb: &AABB) -> bool {
        let closest_point = aabb.closest_point(self.center);
        (closest_point - self.center).length_squared() <= self.radius * self.radius
    }

    /// 扩展包含指定点
    pub fn expand_to_include(&mut self, point: Vec3) {
        let distance = (point - self.center).length();
        if distance > self.radius {
            self.radius = distance;
        }
    }

    /// 计算表面积
    pub fn surface_area(&self) -> f32 {
        4.0 * std::f32::consts::PI * self.radius * self.radius
    }

    /// 计算体积
    pub fn volume(&self) -> f32 {
        (4.0 / 3.0) * std::f32::consts::PI * self.radius.powi(3)
    }

    /// 通过变换矩阵变换边界球
    pub fn transform(&self, matrix: &Mat4) -> Self {
        let transformed_center = matrix.transform_point3(self.center);
        // 简化处理：使用变换矩阵的最大缩放因子
        let scale = matrix.to_scale_rotation_translation().0;
        let max_scale = scale.x.max(scale.y).max(scale.z);
        
        Self {
            center: transformed_center,
            radius: self.radius * max_scale,
        }
    }
}

impl Default for BoundingSphere {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            radius: 0.0,
        }
    }
}
