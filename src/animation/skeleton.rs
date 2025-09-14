//! 骨骼动画系统

use crate::math::{Vec3, Mat4};
use crate::EngineResult;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// 骨骼
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bone {
    pub name: String,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub bind_pose: Transform,
    pub inverse_bind_matrix: Mat4,
}

/// 变换信息
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: glam::Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    /// 转换为矩阵
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// 从矩阵创建变换
    pub fn from_matrix(matrix: &Mat4) -> Self {
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// 插值
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            translation: self.translation.lerp(other.translation, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

/// 骨骼层次结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
    pub bone_name_to_index: HashMap<String, usize>,
    pub root_bones: Vec<usize>,
}

impl Skeleton {
    /// 创建新的骨骼
    pub fn new() -> Self {
        Self {
            bones: Vec::new(),
            bone_name_to_index: HashMap::new(),
            root_bones: Vec::new(),
        }
    }

    /// 添加骨骼
    pub fn add_bone(&mut self, name: impl Into<String>, parent: Option<usize>) -> usize {
        let name = name.into();
        let bone_index = self.bones.len();
        
        let bone = Bone {
            name: name.clone(),
            parent,
            children: Vec::new(),
            bind_pose: Transform::default(),
            inverse_bind_matrix: Mat4::IDENTITY,
        };

        self.bones.push(bone);
        self.bone_name_to_index.insert(name, bone_index);

        // 更新父子关系
        if let Some(parent_index) = parent {
            if parent_index < self.bones.len() {
                self.bones[parent_index].children.push(bone_index);
            }
        } else {
            self.root_bones.push(bone_index);
        }

        bone_index
    }

    /// 根据名称查找骨骼索引
    pub fn find_bone(&self, name: &str) -> Option<usize> {
        self.bone_name_to_index.get(name).copied()
    }

    /// 获取骨骼
    pub fn get_bone(&self, index: usize) -> Option<&Bone> {
        self.bones.get(index)
    }

    /// 获取可变骨骼
    pub fn get_bone_mut(&mut self, index: usize) -> Option<&mut Bone> {
        self.bones.get_mut(index)
    }

    /// 计算全局变换矩阵
    pub fn compute_global_transforms(&self, local_transforms: &[Transform]) -> Vec<Mat4> {
        let mut global_transforms = vec![Mat4::IDENTITY; self.bones.len()];
        
        // 递归计算每个根骨骼的全局变换
        for &root_index in &self.root_bones {
            self.compute_bone_global_transform(
                root_index,
                &Mat4::IDENTITY,
                local_transforms,
                &mut global_transforms,
            );
        }

        global_transforms
    }

    /// 递归计算骨骼的全局变换
    fn compute_bone_global_transform(
        &self,
        bone_index: usize,
        parent_global: &Mat4,
        local_transforms: &[Transform],
        global_transforms: &mut [Mat4],
    ) {
        if bone_index >= self.bones.len() || bone_index >= local_transforms.len() {
            return;
        }

        let local_matrix = local_transforms[bone_index].to_matrix();
        let global_matrix = *parent_global * local_matrix;
        global_transforms[bone_index] = global_matrix;

        // 递归处理子骨骼
        for &child_index in &self.bones[bone_index].children {
            self.compute_bone_global_transform(
                child_index,
                &global_matrix,
                local_transforms,
                global_transforms,
            );
        }
    }

    /// 计算最终的骨骼矩阵（用于顶点蒙皮）
    pub fn compute_skinning_matrices(&self, global_transforms: &[Mat4]) -> Vec<Mat4> {
        global_transforms
            .iter()
            .zip(&self.bones)
            .map(|(global, bone)| *global * bone.inverse_bind_matrix)
            .collect()
    }

    /// 设置骨骼的绑定姿势
    pub fn set_bind_pose(&mut self, bone_index: usize, transform: Transform) {
        if let Some(bone) = self.bones.get_mut(bone_index) {
            bone.bind_pose = transform;
            bone.inverse_bind_matrix = transform.to_matrix().inverse();
        }
    }

    /// 获取骨骼数量
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }

    /// 验证骨骼层次结构的有效性
    pub fn validate(&self) -> EngineResult<()> {
        for (i, bone) in self.bones.iter().enumerate() {
            // 检查父骨骼索引
            if let Some(parent_index) = bone.parent {
                if parent_index >= self.bones.len() {
                    return Err(crate::EngineError::AssetError(
                        format!("Bone {} has invalid parent index {}", i, parent_index)
                    ).into());
                }
            }

            // 检查子骨骼索引
            for &child_index in &bone.children {
                if child_index >= self.bones.len() {
                    return Err(crate::EngineError::AssetError(
                        format!("Bone {} has invalid child index {}", i, child_index)
                    ).into());
                }
            }
        }

        Ok(())
    }
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

/// 骨骼动画姿势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkeletalPose {
    pub bone_transforms: Vec<Transform>,
}

impl SkeletalPose {
    /// 创建新的骨骼姿势
    pub fn new(bone_count: usize) -> Self {
        Self {
            bone_transforms: vec![Transform::default(); bone_count],
        }
    }

    /// 设置骨骼变换
    pub fn set_bone_transform(&mut self, bone_index: usize, transform: Transform) {
        if bone_index < self.bone_transforms.len() {
            self.bone_transforms[bone_index] = transform;
        }
    }

    /// 获取骨骼变换
    pub fn get_bone_transform(&self, bone_index: usize) -> Option<&Transform> {
        self.bone_transforms.get(bone_index)
    }

    /// 插值两个姿势
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let bone_count = self.bone_transforms.len().min(other.bone_transforms.len());
        let mut result = Self::new(bone_count);

        for i in 0..bone_count {
            result.bone_transforms[i] = self.bone_transforms[i].lerp(&other.bone_transforms[i], t);
        }

        result
    }

    /// 混合多个姿势
    pub fn blend(poses: &[(Self, f32)]) -> Option<Self> {
        if poses.is_empty() {
            return None;
        }

        if poses.len() == 1 {
            return Some(poses[0].0.clone());
        }

        let bone_count = poses[0].0.bone_transforms.len();
        let mut result = Self::new(bone_count);

        // 归一化权重
        let total_weight: f32 = poses.iter().map(|(_, w)| w).sum();
        if total_weight <= 0.0 {
            return Some(poses[0].0.clone());
        }

        // 混合每个骨骼的变换
        for bone_index in 0..bone_count {
            let mut blended_translation = Vec3::ZERO;
            let mut blended_scale = Vec3::ZERO;
            let mut blended_rotation = glam::Quat::IDENTITY;

            for (i, (pose, weight)) in poses.iter().enumerate() {
                if bone_index < pose.bone_transforms.len() {
                    let normalized_weight = weight / total_weight;
                    let transform = &pose.bone_transforms[bone_index];

                    blended_translation += transform.translation * normalized_weight;
                    blended_scale += transform.scale * normalized_weight;

                    if i == 0 {
                        blended_rotation = transform.rotation;
                    } else {
                        blended_rotation = blended_rotation.slerp(transform.rotation, normalized_weight);
                    }
                }
            }

            result.bone_transforms[bone_index] = Transform {
                translation: blended_translation,
                rotation: blended_rotation,
                scale: blended_scale,
            };
        }

        Some(result)
    }
}

/// 骨骼动画混合器
#[derive(Debug, Clone)]
pub struct SkeletalBlender {
    pub poses: Vec<(SkeletalPose, f32)>,
}

impl SkeletalBlender {
    /// 创建新的骨骼混合器
    pub fn new() -> Self {
        Self {
            poses: Vec::new(),
        }
    }

    /// 添加姿势
    pub fn add_pose(&mut self, pose: SkeletalPose, weight: f32) {
        self.poses.push((pose, weight));
    }

    /// 清除所有姿势
    pub fn clear(&mut self) {
        self.poses.clear();
    }

    /// 混合所有姿势
    pub fn blend(&self) -> Option<SkeletalPose> {
        SkeletalPose::blend(&self.poses)
    }

    /// 设置姿势权重
    pub fn set_weight(&mut self, index: usize, weight: f32) {
        if index < self.poses.len() {
            self.poses[index].1 = weight;
        }
    }
}

impl Default for SkeletalBlender {
    fn default() -> Self {
        Self::new()
    }
}

/// 骨骼动画系统
#[derive(Debug)]
pub struct SkeletalAnimationSystem {
    pub skeleton: Skeleton,
    pub current_pose: SkeletalPose,
    pub blender: SkeletalBlender,
}

impl SkeletalAnimationSystem {
    /// 创建新的骨骼动画系统
    pub fn new(skeleton: Skeleton) -> Self {
        let bone_count = skeleton.bone_count();
        Self {
            skeleton,
            current_pose: SkeletalPose::new(bone_count),
            blender: SkeletalBlender::new(),
        }
    }

    /// 更新动画
    pub fn update(&mut self, delta_time: f32) -> EngineResult<()> {
        // 这里可以添加动画更新逻辑
        // 例如播放动画剪辑、混合姿势等

        // 混合当前的姿势
        if let Some(blended_pose) = self.blender.blend() {
            self.current_pose = blended_pose;
        }

        Ok(())
    }

    /// 获取当前的蒙皮矩阵
    pub fn get_skinning_matrices(&self) -> Vec<Mat4> {
        let global_transforms = self.skeleton.compute_global_transforms(&self.current_pose.bone_transforms);
        self.skeleton.compute_skinning_matrices(&global_transforms)
    }

    /// 设置姿势
    pub fn set_pose(&mut self, pose: SkeletalPose) {
        self.current_pose = pose;
    }

    /// 添加混合姿势
    pub fn add_blend_pose(&mut self, pose: SkeletalPose, weight: f32) {
        self.blender.add_pose(pose, weight);
    }

    /// 清除混合姿势
    pub fn clear_blend_poses(&mut self) {
        self.blender.clear();
    }
}
