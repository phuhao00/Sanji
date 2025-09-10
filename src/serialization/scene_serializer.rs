//! 场景序列化器

use super::{Serializable, SerializationContext, SerializationFormat};
use crate::ecs::{World, Entity, Component};
use crate::scene::{Scene, SceneNode, SceneManager};
use crate::math::{Vec3, Quat};
use crate::EngineResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// 场景序列化数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedScene {
    pub metadata: SceneMetadata,
    pub entities: Vec<SerializedEntity>,
    pub scene_graph: SerializedSceneGraph,
    pub resources: HashMap<String, String>, // 资源ID -> 资源路径
    pub custom_data: HashMap<String, serde_json::Value>,
}

/// 场景元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub created_at: String,
    pub modified_at: String,
    pub author: String,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>, // 依赖的其他场景或资源
}

/// 序列化的实体数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEntity {
    pub id: u64,
    pub name: String,
    pub active: bool,
    pub components: HashMap<String, serde_json::Value>,
    pub parent: Option<u64>,
    pub children: Vec<u64>,
}

/// 序列化的场景图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedSceneGraph {
    pub root_nodes: Vec<u64>,
    pub nodes: HashMap<u64, SerializedSceneNode>,
}

/// 序列化的场景节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedSceneNode {
    pub entity_id: u64,
    pub transform: SerializedTransform,
    pub local_bounds: Option<SerializedBounds>,
    pub world_bounds: Option<SerializedBounds>,
    pub layer: u32,
    pub tag: String,
}

/// 序列化的变换数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedTransform {
    pub position: [f32; 3],
    pub rotation: [f32; 4], // 四元数 [x, y, z, w]
    pub scale: [f32; 3],
}

impl SerializedTransform {
    pub fn from_components(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position: [position.x, position.y, position.z],
            rotation: [rotation.x, rotation.y, rotation.z, rotation.w],
            scale: [scale.x, scale.y, scale.z],
        }
    }

    pub fn to_components(&self) -> (Vec3, Quat, Vec3) {
        let position = Vec3::new(self.position[0], self.position[1], self.position[2]);
        let rotation = Quat::from_xyzw(self.rotation[0], self.rotation[1], self.rotation[2], self.rotation[3]);
        let scale = Vec3::new(self.scale[0], self.scale[1], self.scale[2]);
        (position, rotation, scale)
    }
}

/// 序列化的包围盒
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl SerializedBounds {
    pub fn from_bounds(bounds: &crate::math::bounds::AABB) -> Self {
        Self {
            min: [bounds.min.x, bounds.min.y, bounds.min.z],
            max: [bounds.max.x, bounds.max.y, bounds.max.z],
        }
    }

    pub fn to_bounds(&self) -> crate::math::bounds::AABB {
        crate::math::bounds::AABB::new(
            Vec3::new(self.min[0], self.min[1], self.min[2]),
            Vec3::new(self.max[0], self.max[1], self.max[2]),
        )
    }
}

/// 场景序列化器
pub struct SceneSerializer {
    component_serializers: HashMap<String, Box<dyn ComponentSerializerTrait>>,
    resource_manager: Option<crate::assets::AssetManager>,
}

impl SceneSerializer {
    pub fn new() -> Self {
        Self {
            component_serializers: HashMap::new(),
            resource_manager: None,
        }
    }

    /// 注册组件序列化器
    pub fn register_component_serializer<T: Component + Serialize + for<'de> Deserialize<'de> + 'static>(
        &mut self,
        name: String,
        serializer: Box<dyn ComponentSerializerTrait>,
    ) {
        self.component_serializers.insert(name, serializer);
    }

    /// 设置资源管理器
    pub fn set_resource_manager(&mut self, manager: crate::assets::AssetManager) {
        self.resource_manager = Some(manager);
    }

    /// 序列化场景
    pub fn serialize_scene(&self, scene: &Scene, world: &World) -> EngineResult<SerializedScene> {
        let metadata = SceneMetadata {
            name: scene.name.clone(),
            description: scene.description.clone(),
            version: "1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            author: "Sanji Engine".to_string(),
            tags: scene.tags.clone(),
            dependencies: Vec::new(),
        };

        // 序列化实体
        let mut entities = Vec::new();
        for entity in world.get_all_entities() {
            let serialized_entity = self.serialize_entity(*entity, world)?;
            entities.push(serialized_entity);
        }

        // 序列化场景图
        let scene_graph = self.serialize_scene_graph(&scene.scene_graph)?;

        // 收集资源引用
        let resources = self.collect_resource_references(world)?;

        Ok(SerializedScene {
            metadata,
            entities,
            scene_graph,
            resources,
            custom_data: HashMap::new(),
        })
    }

    /// 反序列化场景
    pub fn deserialize_scene(&self, data: &SerializedScene, world: &mut World, scene_manager: &mut SceneManager) -> EngineResult<Scene> {
        // 创建场景
        let mut scene = Scene::new(data.metadata.name.clone());
        scene.description = data.metadata.description.clone();
        scene.tags = data.metadata.tags.clone();

        // 创建实体映射（旧ID -> 新ID）
        let mut entity_mapping = HashMap::new();

        // 第一阶段：创建所有实体
        for serialized_entity in &data.entities {
            let new_entity = world.create_entity();
            entity_mapping.insert(serialized_entity.id, new_entity);
        }

        // 第二阶段：反序列化组件和设置层次关系
        for serialized_entity in &data.entities {
            let entity = entity_mapping[&serialized_entity.id];
            
            // 反序列化组件
            for (component_type, component_data) in &serialized_entity.components {
                if let Some(serializer) = self.component_serializers.get(component_type) {
                    serializer.deserialize_component(entity, component_data, world)?;
                }
            }

            // 设置父子关系
            if let Some(parent_id) = serialized_entity.parent {
                if let Some(&parent_entity) = entity_mapping.get(&parent_id) {
                    // 设置父子关系的逻辑
                    // TODO: 实现实体层次关系
                }
            }
        }

        // 反序列化场景图
        self.deserialize_scene_graph(&data.scene_graph, &entity_mapping, &mut scene)?;

        // 加载资源引用
        self.load_resource_references(&data.resources)?;

        Ok(scene)
    }

    /// 序列化实体
    fn serialize_entity(&self, entity: Entity, world: &World) -> EngineResult<SerializedEntity> {
        let mut components = HashMap::new();

        // 序列化所有组件
        for (component_type, serializer) in &self.component_serializers {
            if let Some(component_data) = serializer.serialize_component(entity, world)? {
                components.insert(component_type.clone(), component_data);
            }
        }

        Ok(SerializedEntity {
            id: entity as u64,
            name: format!("Entity_{}", entity), // 默认名称
            active: true,
            components,
            parent: None, // TODO: 从场景图获取
            children: Vec::new(), // TODO: 从场景图获取
        })
    }

    /// 序列化场景图
    fn serialize_scene_graph(&self, scene_graph: &crate::scene::SceneGraph) -> EngineResult<SerializedSceneGraph> {
        let mut serialized_nodes = HashMap::new();
        let root_nodes = Vec::new(); // TODO: 从场景图获取根节点

        // TODO: 遍历场景图并序列化所有节点
        // for (node_id, node) in scene_graph.nodes() {
        //     let serialized_node = self.serialize_scene_node(node)?;
        //     serialized_nodes.insert(node_id, serialized_node);
        // }

        Ok(SerializedSceneGraph {
            root_nodes,
            nodes: serialized_nodes,
        })
    }

    /// 反序列化场景图
    fn deserialize_scene_graph(
        &self,
        data: &SerializedSceneGraph,
        entity_mapping: &HashMap<u64, Entity>,
        scene: &mut Scene,
    ) -> EngineResult<()> {
        // TODO: 重建场景图结构
        Ok(())
    }

    /// 收集资源引用
    fn collect_resource_references(&self, world: &World) -> EngineResult<HashMap<String, String>> {
        let mut resources = HashMap::new();
        
        // TODO: 遍历所有实体的组件，收集资源引用
        // 例如：纹理、模型、音频等资源的路径

        Ok(resources)
    }

    /// 加载资源引用
    fn load_resource_references(&self, resources: &HashMap<String, String>) -> EngineResult<()> {
        if let Some(ref resource_manager) = self.resource_manager {
            for (resource_id, resource_path) in resources {
                // TODO: 使用资源管理器加载资源
                log::info!("Loading resource: {} from {}", resource_id, resource_path);
            }
        }
        Ok(())
    }
}

impl Default for SceneSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serializable for SerializedScene {
    fn serialize(&self, context: &SerializationContext) -> EngineResult<Vec<u8>> {
        match context.format {
            SerializationFormat::Json => {
                if context.pretty_print {
                    Ok(serde_json::to_vec_pretty(self)?)
                } else {
                    Ok(serde_json::to_vec(self)?)
                }
            }
            SerializationFormat::Binary => {
                Ok(bincode::serialize(self)?)
            }
            SerializationFormat::MessagePack => {
                Ok(rmp_serde::to_vec(self)?)
            }
            SerializationFormat::YAML => {
                let yaml_string = serde_yaml::to_string(self)?;
                Ok(yaml_string.into_bytes())
            }
        }
    }

    fn deserialize(data: &[u8], context: &SerializationContext) -> EngineResult<Self> {
        match context.format {
            SerializationFormat::Json => {
                Ok(serde_json::from_slice(data)?)
            }
            SerializationFormat::Binary => {
                Ok(bincode::deserialize(data)?)
            }
            SerializationFormat::MessagePack => {
                Ok(rmp_serde::from_slice(data)?)
            }
            SerializationFormat::YAML => {
                let yaml_string = String::from_utf8(data.to_vec())?;
                Ok(serde_yaml::from_str(&yaml_string)?)
            }
        }
    }
}

/// 组件序列化器特征
pub trait ComponentSerializerTrait: Send + Sync {
    /// 序列化组件
    fn serialize_component(&self, entity: Entity, world: &World) -> EngineResult<Option<serde_json::Value>>;
    
    /// 反序列化组件
    fn deserialize_component(&self, entity: Entity, data: &serde_json::Value, world: &mut World) -> EngineResult<()>;
    
    /// 获取组件类型名
    fn component_type_name(&self) -> &'static str;
}

/// 通用组件序列化器
pub struct GenericComponentSerializer<T: Component + Serialize + for<'de> Deserialize<'de>> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Component + Serialize + for<'de> Deserialize<'de>> GenericComponentSerializer<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Component + Serialize + for<'de> Deserialize<'de>> ComponentSerializerTrait for GenericComponentSerializer<T> {
    fn serialize_component(&self, entity: Entity, world: &World) -> EngineResult<Option<serde_json::Value>> {
        if let Some(component) = world.get_component::<T>(entity) {
            let value = serde_json::to_value(component)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn deserialize_component(&self, entity: Entity, data: &serde_json::Value, world: &mut World) -> EngineResult<()> {
        let component: T = serde_json::from_value(data.clone())?;
        world.add_component(entity, component);
        Ok(())
    }

    fn component_type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

/// 预制件数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabData {
    pub metadata: PrefabMetadata,
    pub root_entity: SerializedEntity,
    pub entities: Vec<SerializedEntity>,
    pub resources: HashMap<String, String>,
}

/// 预制件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabMetadata {
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub version: String,
    pub author: String,
    pub icon_path: Option<String>,
}

/// 预制件序列化器
pub struct PrefabSerializer {
    scene_serializer: SceneSerializer,
}

impl PrefabSerializer {
    pub fn new() -> Self {
        Self {
            scene_serializer: SceneSerializer::new(),
        }
    }

    /// 序列化预制件
    pub fn serialize_prefab(&self, root_entity: Entity, world: &World) -> EngineResult<PrefabData> {
        let mut entities = Vec::new();
        
        // 收集根实体及其所有子实体
        let mut entities_to_process = vec![root_entity];
        while let Some(entity) = entities_to_process.pop() {
            let serialized = self.scene_serializer.serialize_entity(entity, world)?;
            entities_to_process.extend(&serialized.children);
            entities.push(serialized);
        }

        let root_serialized = entities.iter().find(|e| e.id == root_entity as u64).unwrap().clone();

        let metadata = PrefabMetadata {
            name: format!("Prefab_{}", root_entity),
            description: "Generated prefab".to_string(),
            category: "General".to_string(),
            tags: Vec::new(),
            version: "1.0".to_string(),
            author: "Sanji Engine".to_string(),
            icon_path: None,
        };

        Ok(PrefabData {
            metadata,
            root_entity: root_serialized,
            entities,
            resources: HashMap::new(),
        })
    }

    /// 反序列化预制件
    pub fn deserialize_prefab(&self, data: &PrefabData, world: &mut World) -> EngineResult<Entity> {
        // 创建实体映射
        let mut entity_mapping = HashMap::new();
        
        // 创建所有实体
        for serialized_entity in &data.entities {
            let new_entity = world.create_entity();
            entity_mapping.insert(serialized_entity.id, new_entity);
        }

        // 反序列化所有组件
        for serialized_entity in &data.entities {
            let entity = entity_mapping[&serialized_entity.id];
            
            for (component_type, component_data) in &serialized_entity.components {
                if let Some(serializer) = self.scene_serializer.component_serializers.get(component_type) {
                    serializer.deserialize_component(entity, component_data, world)?;
                }
            }
        }

        Ok(entity_mapping[&data.root_entity.id])
    }
}

impl Default for PrefabSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serializable for PrefabData {
    fn serialize(&self, context: &SerializationContext) -> EngineResult<Vec<u8>> {
        match context.format {
            SerializationFormat::Json => {
                if context.pretty_print {
                    Ok(serde_json::to_vec_pretty(self)?)
                } else {
                    Ok(serde_json::to_vec(self)?)
                }
            }
            SerializationFormat::Binary => {
                Ok(bincode::serialize(self)?)
            }
            SerializationFormat::MessagePack => {
                Ok(rmp_serde::to_vec(self)?)
            }
            SerializationFormat::YAML => {
                let yaml_string = serde_yaml::to_string(self)?;
                Ok(yaml_string.into_bytes())
            }
        }
    }

    fn deserialize(data: &[u8], context: &SerializationContext) -> EngineResult<Self> {
        match context.format {
            SerializationFormat::Json => {
                Ok(serde_json::from_slice(data)?)
            }
            SerializationFormat::Binary => {
                Ok(bincode::deserialize(data)?)
            }
            SerializationFormat::MessagePack => {
                Ok(rmp_serde::from_slice(data)?)
            }
            SerializationFormat::YAML => {
                let yaml_string = String::from_utf8(data.to_vec())?;
                Ok(serde_yaml::from_str(&yaml_string)?)
            }
        }
    }
}
