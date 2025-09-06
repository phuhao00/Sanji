//! 相机系统

use glam::{Mat4, Vec3, Vec4, Quat};
use serde::{Deserialize, Serialize};

/// 相机投影类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ProjectionType {
    Perspective,
    Orthographic,
}

/// 相机组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    /// 相机位置
    pub position: Vec3,
    /// 相机旋转
    pub rotation: Quat,
    /// 投影类型
    pub projection_type: ProjectionType,
    /// 视野角度(透视投影)
    pub fovy: f32,
    /// 长宽比
    pub aspect_ratio: f32,
    /// 近裁剪面
    pub near_plane: f32,
    /// 远裁剪面
    pub far_plane: f32,
    /// 正交投影大小
    pub orthographic_size: f32,
    /// 是否是主相机
    pub is_main: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 3.0),
            rotation: Quat::IDENTITY,
            projection_type: ProjectionType::Perspective,
            fovy: 45.0_f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            near_plane: 0.1,
            far_plane: 100.0,
            orthographic_size: 5.0,
            is_main: true,
        }
    }
}

impl Camera {
    /// 创建新的透视相机
    pub fn perspective(fovy: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            projection_type: ProjectionType::Perspective,
            fovy: fovy.to_radians(),
            aspect_ratio,
            near_plane: near,
            far_plane: far,
            ..Default::default()
        }
    }

    /// 创建新的正交相机
    pub fn orthographic(size: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        Self {
            projection_type: ProjectionType::Orthographic,
            orthographic_size: size,
            aspect_ratio,
            near_plane: near,
            far_plane: far,
            ..Default::default()
        }
    }

    /// 获取视图矩阵
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, self.position).inverse()
    }

    /// 获取投影矩阵
    pub fn projection_matrix(&self) -> Mat4 {
        match self.projection_type {
            ProjectionType::Perspective => {
                Mat4::perspective_rh(self.fovy, self.aspect_ratio, self.near_plane, self.far_plane)
            }
            ProjectionType::Orthographic => {
                let height = self.orthographic_size;
                let width = height * self.aspect_ratio;
                Mat4::orthographic_rh(
                    -width / 2.0,
                    width / 2.0,
                    -height / 2.0,
                    height / 2.0,
                    self.near_plane,
                    self.far_plane,
                )
            }
        }
    }

    /// 获取视图投影矩阵
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// 向前移动
    pub fn move_forward(&mut self, distance: f32) {
        let forward = self.forward();
        self.position += forward * distance;
    }

    /// 向右移动
    pub fn move_right(&mut self, distance: f32) {
        let right = self.right();
        self.position += right * distance;
    }

    /// 向上移动
    pub fn move_up(&mut self, distance: f32) {
        let up = self.up();
        self.position += up * distance;
    }

    /// 获取前方向量
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    /// 获取右方向量
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// 获取上方向量
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// 设置位置
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// 设置旋转
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
    }

    /// 看向目标点
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let forward = (target - self.position).normalize();
        let right = forward.cross(up).normalize();
        let up = right.cross(forward).normalize();
        
        let rotation_matrix = Mat4::from_cols(
            Vec4::new(right.x, up.x, -forward.x, 0.0),
            Vec4::new(right.y, up.y, -forward.y, 0.0),
            Vec4::new(right.z, up.z, -forward.z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );
        
        self.rotation = Quat::from_mat4(&rotation_matrix);
    }

    /// 更新长宽比
    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }
}

/// 相机控制器
pub struct CameraController {
    /// 移动速度
    pub move_speed: f32,
    /// 旋转速度
    pub rotation_speed: f32,
    /// 缩放速度
    pub zoom_speed: f32,
    /// 是否启用鼠标控制
    pub mouse_enabled: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            move_speed: 5.0,
            rotation_speed: 1.0,
            zoom_speed: 2.0,
            mouse_enabled: true,
        }
    }
}

impl CameraController {
    /// 更新相机控制
    pub fn update(&self, camera: &mut Camera, delta_time: f32) {
        // 这里可以添加基于输入的相机控制逻辑
        // 目前保持简单
    }
}
