//! 网格系统

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

/// 顶点结构
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MeshVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coords: Vec2,
    pub color: Vec3,
}

impl Default for MeshVertex {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            normal: Vec3::Y,
            tex_coords: Vec2::ZERO,
            color: Vec3::ONE,
        }
    }
}

/// 网格数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
    pub name: String,
}

impl Mesh {
    /// 创建新的网格
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            name: name.into(),
        }
    }

    /// 创建立方体网格
    pub fn cube() -> Self {
        let vertices = vec![
            // 前面
            MeshVertex { position: Vec3::new(-0.5, -0.5, 0.5), normal: Vec3::Z, tex_coords: Vec2::new(0.0, 1.0), color: Vec3::ONE },
            MeshVertex { position: Vec3::new(0.5, -0.5, 0.5), normal: Vec3::Z, tex_coords: Vec2::new(1.0, 1.0), color: Vec3::ONE },
            MeshVertex { position: Vec3::new(0.5, 0.5, 0.5), normal: Vec3::Z, tex_coords: Vec2::new(1.0, 0.0), color: Vec3::ONE },
            MeshVertex { position: Vec3::new(-0.5, 0.5, 0.5), normal: Vec3::Z, tex_coords: Vec2::new(0.0, 0.0), color: Vec3::ONE },
            // 其他面...
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0, // 前面
            // 其他面的索引...
        ];

        Self {
            vertices,
            indices,
            name: "立方体".to_string(),
        }
    }

    /// 创建球体网格
    pub fn sphere(radius: f32, segments: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // 简化的球体生成逻辑
        for i in 0..=segments {
            for j in 0..=segments {
                let phi = std::f32::consts::PI * (i as f32) / (segments as f32);
                let theta = 2.0 * std::f32::consts::PI * (j as f32) / (segments as f32);

                let x = radius * phi.sin() * theta.cos();
                let y = radius * phi.cos();
                let z = radius * phi.sin() * theta.sin();

                vertices.push(MeshVertex {
                    position: Vec3::new(x, y, z),
                    normal: Vec3::new(x, y, z).normalize(),
                    tex_coords: Vec2::new(j as f32 / segments as f32, i as f32 / segments as f32),
                    color: Vec3::ONE,
                });
            }
        }

        // 生成索引
        for i in 0..segments {
            for j in 0..segments {
                let first = i * (segments + 1) + j;
                let second = first + segments + 1;

                indices.extend_from_slice(&[first, second, first + 1]);
                indices.extend_from_slice(&[second, second + 1, first + 1]);
            }
        }

        Self {
            vertices,
            indices,
            name: "球体".to_string(),
        }
    }

    /// 计算法线
    pub fn calculate_normals(&mut self) {
        for vertex in &mut self.vertices {
            vertex.normal = Vec3::ZERO;
        }

        // 计算每个三角形的法线并累加到顶点
        for triangle in self.indices.chunks(3) {
            let i0 = triangle[0] as usize;
            let i1 = triangle[1] as usize;
            let i2 = triangle[2] as usize;

            let v0 = self.vertices[i0].position;
            let v1 = self.vertices[i1].position;
            let v2 = self.vertices[i2].position;

            let normal = (v1 - v0).cross(v2 - v0).normalize();

            self.vertices[i0].normal += normal;
            self.vertices[i1].normal += normal;
            self.vertices[i2].normal += normal;
        }

        // 归一化法线
        for vertex in &mut self.vertices {
            vertex.normal = vertex.normal.normalize();
        }
    }
}
