//! 场景系统

use crate::{EngineResult, EngineError};
use crate::ecs::{ECSWorld, Entity, EntityBuilder, Prefabs};
use crate::scene::SceneGraph;
use crate::render::Camera;

use specs::{WorldExt, Builder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use glam::{Vec3, Quat};

/// 场景 - 包含游戏对象和场景图的容器
#[derive(Debug)]
pub struct Scene {
    /// 场景名称
    pub name: String,
    /// 场景图
    scene_graph: SceneGraph,
    /// 实体映射 (名称到实体)
    entity_map: HashMap<String, Entity>,
    /// 场景是否激活
    active: bool,
    /// 场景元数据
    metadata: SceneMetadata,
}

/// 场景元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneMetadata {
    /// 场景描述
    pub description: String,
    /// 创建时间
    pub created_at: String,
    /// 版本
    pub version: String,
    /// 标签
    pub tags: Vec<String>,
}

impl Default for SceneMetadata {
    fn default() -> Self {
        Self {
            description: String::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            version: "1.0.0".to_string(),
            tags: Vec::new(),
        }
    }
}

impl Scene {
    /// 创建新场景
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            scene_graph: SceneGraph::new(),
            entity_map: HashMap::new(),
            active: false,
            metadata: SceneMetadata::default(),
        }
    }

    /// 创建默认场景
    pub fn create_default_scene(name: impl Into<String>) -> Self {
        let mut scene = Self::new(name);
        
        // 添加一些默认对象
        scene.metadata.description = "默认场景包含基础的相机和光照".to_string();
        scene.metadata.tags.push("default".to_string());
        
        scene
    }

    /// 设置场景元数据
    pub fn set_metadata(&mut self, metadata: SceneMetadata) {
        self.metadata = metadata;
    }

    /// 获取场景元数据
    pub fn metadata(&self) -> &SceneMetadata {
        &self.metadata
    }

    /// 激活场景
    pub fn activate(&mut self) {
        self.active = true;
    }

    /// 停用场景
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// 检查场景是否激活
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// 获取场景图
    pub fn scene_graph(&self) -> &SceneGraph {
        &self.scene_graph
    }

    /// 获取可变场景图
    pub fn scene_graph_mut(&mut self) -> &mut SceneGraph {
        &mut self.scene_graph
    }

    /// 在ECS世界中创建实体
    pub fn create_entity(&mut self, world: &mut ECSWorld, name: impl Into<String>) -> Entity {
        let name = name.into();
        let entity = world.create_entity().build();
        
        // 添加到场景图根节点
        self.scene_graph.add_entity(entity, None);
        
        // 添加到实体映射
        self.entity_map.insert(name, entity);
        
        entity
    }

    /// 创建带父节点的实体
    pub fn create_child_entity(&mut self, world: &mut ECSWorld, name: impl Into<String>, parent: Entity) -> EngineResult<Entity> {
        let name = name.into();
        let entity = world.create_entity().build();
        
        // 添加到场景图
        self.scene_graph.add_entity(entity, Some(parent))?;
        
        // 添加到实体映射
        self.entity_map.insert(name, entity);
        
        Ok(entity)
    }

    /// 通过名称查找实体
    pub fn find_entity(&self, name: &str) -> Option<Entity> {
        self.entity_map.get(name).copied()
    }

    /// 移除实体
    pub fn remove_entity(&mut self, world: &mut ECSWorld, entity: Entity) -> EngineResult<()> {
        // 从场景图中移除
        self.scene_graph.remove_entity(entity)?;
        
        // 从实体映射中移除
        self.entity_map.retain(|_, &mut e| e != entity);
        
        // 从ECS世界中删除
        world.delete_entity(entity)?;
        
        Ok(())
    }

    /// 通过名称移除实体
    pub fn remove_entity_by_name(&mut self, world: &mut ECSWorld, name: &str) -> EngineResult<()> {
        if let Some(entity) = self.find_entity(name) {
            self.remove_entity(world, entity)?;
        }
        Ok(())
    }

    /// 设置实体的父节点
    pub fn set_parent(&mut self, child: Entity, parent: Option<Entity>) -> EngineResult<()> {
        self.scene_graph.set_parent(child, parent)
    }

    /// 获取实体的子节点
    pub fn get_children(&self, entity: Entity) -> Vec<Entity> {
        self.scene_graph.get_children(entity)
    }

    /// 获取实体的父节点
    pub fn get_parent(&self, entity: Entity) -> Option<Entity> {
        self.scene_graph.get_parent(entity)
    }

    /// 获取所有根实体
    pub fn get_root_entities(&self) -> Vec<Entity> {
        self.scene_graph.get_root_entities()
    }

    /// 创建预制件
    pub fn spawn_prefab(&mut self, world: &mut ECSWorld, prefab_type: PrefabType, name: impl Into<String>, position: Vec3) -> Entity {
        let name = name.into();
        let entity = match prefab_type {
            PrefabType::Cube => Prefabs::cube(world.world_mut(), position),
            PrefabType::Sphere => Prefabs::sphere(world.world_mut(), position),
            PrefabType::Plane => Prefabs::plane(world.world_mut(), position),
            PrefabType::Camera => Prefabs::main_camera(world.world_mut(), position),
            PrefabType::DirectionalLight => Prefabs::directional_light(world.world_mut(), Vec3::ONE, 1.0),
            PrefabType::PointLight => Prefabs::point_light(world.world_mut(), position, Vec3::ONE, 1.0, 10.0),
        };
        
        // 添加到场景
        self.scene_graph.add_entity(entity, None);
        self.entity_map.insert(name, entity);
        
        entity
    }

    /// 克隆实体 (简化版本)
    pub fn clone_entity(&mut self, world: &mut ECSWorld, source: Entity, name: impl Into<String>) -> EngineResult<Entity> {
        // 这是一个简化的实现，实际需要深度复制所有组件
        let new_entity = world.create_entity().build();
        let name = name.into();
        
        // 添加到场景
        self.scene_graph.add_entity(new_entity, None);
        self.entity_map.insert(name, new_entity);
        
        Ok(new_entity)
    }

    /// 获取场景中的所有实体
    pub fn get_all_entities(&self) -> Vec<Entity> {
        self.entity_map.values().copied().collect()
    }

    /// 按类型查找实体
    pub fn find_entities_by_tag(&self, world: &ECSWorld, tag: &str) -> Vec<Entity> {
        use crate::ecs::query::WorldQueryExt;
        
        world.world().query()
            .renderable()
            .execute()
            .into_iter()
            .map(|(entity, _, _)| entity)
            .filter(|&entity| self.entity_map.values().any(|&e| e == entity))
            .collect()
    }

    /// 查找主相机
    pub fn find_main_camera(&self, world: &ECSWorld) -> Option<Entity> {
        use crate::ecs::query::WorldQueryExt;
        
        world.world().query()
            .cameras()
            .main_camera()
            .map(|(entity, _, _)| entity)
    }

    /// 更新场景 (每帧调用)
    pub fn update(&mut self, delta_time: f32) -> EngineResult<()> {
        if !self.active {
            return Ok(());
        }

        // 更新场景图
        self.scene_graph.update(delta_time);
        
        Ok(())
    }

    /// 序列化场景 (简化版本)
    pub fn serialize(&self) -> EngineResult<String> {
        let scene_data = SceneData {
            name: self.name.clone(),
            metadata: self.metadata.clone(),
            entities: self.entity_map.keys().cloned().collect(),
        };
        
        Ok(serde_json::to_string_pretty(&scene_data)
            .map_err(|e| EngineError::SerializationError(e))?)
    }

    /// 清空场景
    pub fn clear(&mut self, world: &mut ECSWorld) -> EngineResult<()> {
        // 删除所有实体
        let entities: Vec<Entity> = self.entity_map.values().copied().collect();
        for entity in entities {
            world.delete_entity(entity)?;
        }
        
        // 清空映射和场景图
        self.entity_map.clear();
        self.scene_graph.clear();
        
        Ok(())
    }

    /// 获取实体数量
    pub fn entity_count(&self) -> usize {
        self.entity_map.len()
    }

    /// 获取所有实体名称
    pub fn entity_names(&self) -> Vec<&String> {
        self.entity_map.keys().collect()
    }

    /// 重命名实体
    pub fn rename_entity(&mut self, old_name: &str, new_name: impl Into<String>) -> EngineResult<()> {
        if let Some(entity) = self.entity_map.remove(old_name) {
            self.entity_map.insert(new_name.into(), entity);
            Ok(())
        } else {
            Err(EngineError::AssetError(format!("未找到实体: {}", old_name)).into())
        }
    }
}

/// 预制件类型
#[derive(Debug, Clone, Copy)]
pub enum PrefabType {
    Cube,
    Sphere,
    Plane,
    Camera,
    DirectionalLight,
    PointLight,
}

/// 用于序列化的场景数据
#[derive(Debug, Serialize, Deserialize)]
struct SceneData {
    name: String,
    metadata: SceneMetadata,
    entities: Vec<String>,
}

/// 场景构建器
pub struct SceneBuilder {
    scene: Scene,
}

impl SceneBuilder {
    /// 创建新的场景构建器
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            scene: Scene::new(name),
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.scene.metadata.description = description.into();
        self
    }

    /// 添加标签
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.scene.metadata.tags.push(tag.into());
        self
    }

    /// 添加多个标签
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.scene.metadata.tags.extend(tags);
        self
    }

    /// 构建场景
    pub fn build(self) -> Scene {
        self.scene
    }
}
