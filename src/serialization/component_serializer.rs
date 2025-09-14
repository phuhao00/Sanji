//! 组件序列化器

use super::scene_serializer::{ComponentSerializerTrait, GenericComponentSerializer};
use crate::ecs::{Entity, World, Component, VecStorage, HashMapStorage};
use crate::math::{Vec3, Quat, Mat4};
use crate::EngineResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use specs::{WorldExt, Builder};

/// 变换组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformComponent {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub local_matrix: Mat4,
    pub world_matrix: Mat4,
}

impl Component for TransformComponent {
    type Storage = VecStorage<Self>;
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            local_matrix: Mat4::IDENTITY,
            world_matrix: Mat4::IDENTITY,
        }
    }
}

/// 渲染组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderComponent {
    pub mesh_path: String,
    pub material_path: String,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
    pub visible: bool,
    pub layer: u32,
}

impl Component for RenderComponent {
    type Storage = VecStorage<Self>;
}

impl Default for RenderComponent {
    fn default() -> Self {
        Self {
            mesh_path: String::new(),
            material_path: String::new(),
            cast_shadows: true,
            receive_shadows: true,
            visible: true,
            layer: 0,
        }
    }
}

/// 光源组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightComponent {
    pub light_type: LightType,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub spot_angle: f32,
    pub cast_shadows: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LightType {
    Directional,
    Point,
    Spot,
}

impl Component for LightComponent {
    type Storage = VecStorage<Self>;
}

impl Default for LightComponent {
    fn default() -> Self {
        Self {
            light_type: LightType::Point,
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            range: 10.0,
            spot_angle: 45.0,
            cast_shadows: true,
        }
    }
}

/// 相机组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraComponent {
    pub is_active: bool,
    pub fov: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub orthographic: bool,
    pub orthographic_size: f32,
    pub clear_color: [f32; 4],
    pub clear_flags: ClearFlags,
    pub viewport: [f32; 4], // x, y, width, height (normalized)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClearFlags {
    SolidColor,
    Skybox,
    DepthOnly,
    Nothing,
}

impl Component for CameraComponent {
    type Storage = VecStorage<Self>;
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            is_active: true,
            fov: 60.0,
            near_plane: 0.1,
            far_plane: 1000.0,
            orthographic: false,
            orthographic_size: 10.0,
            clear_color: [0.2, 0.3, 0.4, 1.0],
            clear_flags: ClearFlags::SolidColor,
            viewport: [0.0, 0.0, 1.0, 1.0],
        }
    }
}

/// 物理组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsComponent {
    pub body_type: PhysicsBodyType,
    pub mass: f32,
    pub friction: f32,
    pub restitution: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub gravity_scale: f32,
    pub freeze_rotation: [bool; 3],
    pub freeze_position: [bool; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhysicsBodyType {
    Static,
    Kinematic,
    Dynamic,
}

impl Component for PhysicsComponent {
    type Storage = VecStorage<Self>;
}

impl Default for PhysicsComponent {
    fn default() -> Self {
        Self {
            body_type: PhysicsBodyType::Dynamic,
            mass: 1.0,
            friction: 0.5,
            restitution: 0.0,
            linear_damping: 0.1,
            angular_damping: 0.05,
            gravity_scale: 1.0,
            freeze_rotation: [false; 3],
            freeze_position: [false; 3],
        }
    }
}

/// 碰撞体组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColliderComponent {
    pub shape: ColliderShape,
    pub is_trigger: bool,
    pub layer: u32,
    pub mask: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColliderShape {
    Box { size: [f32; 3] },
    Sphere { radius: f32 },
    Capsule { height: f32, radius: f32 },
    Mesh { path: String },
    ConvexMesh { path: String },
}

impl Component for ColliderComponent {
    type Storage = VecStorage<Self>;
}

impl Default for ColliderComponent {
    fn default() -> Self {
        Self {
            shape: ColliderShape::Box { size: [1.0, 1.0, 1.0] },
            is_trigger: false,
            layer: 0,
            mask: u32::MAX,
        }
    }
}

/// 音频源组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSourceComponent {
    pub clip_path: String,
    pub volume: f32,
    pub pitch: f32,
    pub loop_audio: bool,
    pub play_on_awake: bool,
    pub spatial: bool,
    pub min_distance: f32,
    pub max_distance: f32,
    pub rolloff: AudioRolloff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioRolloff {
    Linear,
    Logarithmic,
    Custom,
}

impl Component for AudioSourceComponent {
    type Storage = VecStorage<Self>;
}

impl Default for AudioSourceComponent {
    fn default() -> Self {
        Self {
            clip_path: String::new(),
            volume: 1.0,
            pitch: 1.0,
            loop_audio: false,
            play_on_awake: false,
            spatial: true,
            min_distance: 1.0,
            max_distance: 500.0,
            rolloff: AudioRolloff::Logarithmic,
        }
    }
}

/// 动画组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationComponent {
    pub controller_path: String,
    pub current_state: String,
    pub speed: f32,
    pub auto_play: bool,
    pub loop_mode: AnimationLoopMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationLoopMode {
    Once,
    Loop,
    PingPong,
}

impl Component for AnimationComponent {
    type Storage = VecStorage<Self>;
}

impl Default for AnimationComponent {
    fn default() -> Self {
        Self {
            controller_path: String::new(),
            current_state: "default".to_string(),
            speed: 1.0,
            auto_play: false,
            loop_mode: AnimationLoopMode::Loop,
        }
    }
}

/// 脚本组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptComponent {
    pub script_path: String,
    pub script_type: String,
    pub enabled: bool,
    pub parameters: HashMap<String, ScriptParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptParameter {
    Bool(bool),
    Int(i32),
    Float(f32),
    String(String),
    Vector3([f32; 3]),
    Color([f32; 4]),
}

impl Component for ScriptComponent {
    type Storage = VecStorage<Self>;
}

impl Default for ScriptComponent {
    fn default() -> Self {
        Self {
            script_path: String::new(),
            script_type: "rust".to_string(),
            enabled: true,
            parameters: HashMap::new(),
        }
    }
}

/// 粒子系统组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleSystemComponent {
    pub effect_name: String,
    pub auto_start: bool,
    pub loop_effect: bool,
    pub max_particles: u32,
    pub emission_rate: f32,
    pub start_lifetime: f32,
    pub start_speed: f32,
    pub start_size: f32,
    pub start_color: [f32; 4],
}

impl Component for ParticleSystemComponent {
    type Storage = VecStorage<Self>;
}

impl Default for ParticleSystemComponent {
    fn default() -> Self {
        Self {
            effect_name: String::new(),
            auto_start: true,
            loop_effect: true,
            max_particles: 100,
            emission_rate: 10.0,
            start_lifetime: 5.0,
            start_speed: 5.0,
            start_size: 1.0,
            start_color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// 标签组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagComponent {
    pub tags: Vec<String>,
}

impl Component for TagComponent {
    type Storage = HashMapStorage<Self>;
}

impl Default for TagComponent {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
        }
    }
}

/// 名称组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameComponent {
    pub name: String,
}

impl Component for NameComponent {
    type Storage = VecStorage<Self>;
}

impl Default for NameComponent {
    fn default() -> Self {
        Self {
            name: "Unnamed Entity".to_string(),
        }
    }
}

/// 组件注册器
pub struct ComponentRegistry {
    serializers: HashMap<String, Box<dyn ComponentSerializerTrait>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            serializers: HashMap::new(),
        };

        // 注册默认组件序列化器
        registry.register_default_components();
        registry
    }

    /// 注册组件序列化器
    pub fn register_component<T: Component + Serialize + for<'de> Deserialize<'de> + 'static + Send + Sync>(
        &mut self,
        name: String,
    ) {
        let serializer = Box::new(GenericComponentSerializer::<T>::new());
        self.serializers.insert(name, serializer);
    }

    /// 注册自定义组件序列化器
    pub fn register_custom_serializer(
        &mut self,
        name: String,
        serializer: Box<dyn ComponentSerializerTrait>,
    ) {
        self.serializers.insert(name, serializer);
    }

    /// 获取组件序列化器
    pub fn get_serializer(&self, name: &str) -> Option<&dyn ComponentSerializerTrait> {
        self.serializers.get(name).map(|s| s.as_ref())
    }

    /// 获取所有注册的组件类型
    pub fn get_registered_types(&self) -> Vec<&String> {
        self.serializers.keys().collect()
    }

    /// 注册默认组件
    fn register_default_components(&mut self) {
        self.register_component::<TransformComponent>("TransformComponent".to_string());
        self.register_component::<RenderComponent>("RenderComponent".to_string());
        self.register_component::<LightComponent>("LightComponent".to_string());
        self.register_component::<CameraComponent>("CameraComponent".to_string());
        self.register_component::<PhysicsComponent>("PhysicsComponent".to_string());
        self.register_component::<ColliderComponent>("ColliderComponent".to_string());
        self.register_component::<AudioSourceComponent>("AudioSourceComponent".to_string());
        self.register_component::<AnimationComponent>("AnimationComponent".to_string());
        self.register_component::<ScriptComponent>("ScriptComponent".to_string());
        self.register_component::<ParticleSystemComponent>("ParticleSystemComponent".to_string());
        self.register_component::<TagComponent>("TagComponent".to_string());
        self.register_component::<NameComponent>("NameComponent".to_string());
    }

    /// 序列化实体的所有组件
    pub fn serialize_entity_components(&self, entity: Entity, world: &World) -> EngineResult<HashMap<String, serde_json::Value>> {
        let mut components = HashMap::new();

        for (name, serializer) in &self.serializers {
            if let Some(component_data) = serializer.serialize_component(entity, world)? {
                components.insert(name.clone(), component_data);
            }
        }

        Ok(components)
    }

    /// 反序列化实体的所有组件
    pub fn deserialize_entity_components(
        &self,
        entity: Entity,
        components_data: &HashMap<String, serde_json::Value>,
        world: &mut World,
    ) -> EngineResult<()> {
        for (component_type, component_data) in components_data {
            if let Some(serializer) = self.get_serializer(component_type) {
                serializer.deserialize_component(entity, component_data, world)?;
            } else {
                log::warn!("Unknown component type: {}", component_type);
            }
        }

        Ok(())
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 组件序列化工具
pub mod component_utils {
    use super::*;

    /// 克隆实体的所有组件到新实体
    pub fn clone_entity_components(
        source_entity: Entity,
        target_entity: Entity,
        world: &mut World,
        registry: &ComponentRegistry,
    ) -> EngineResult<()> {
        // 序列化源实体的组件
        let components_data = registry.serialize_entity_components(source_entity, world)?;
        
        // 反序列化到目标实体
        registry.deserialize_entity_components(target_entity, &components_data, world)?;

        Ok(())
    }

    /// 比较两个实体的组件
    pub fn compare_entity_components(
        entity1: Entity,
        entity2: Entity,
        world: &World,
        registry: &ComponentRegistry,
    ) -> EngineResult<Vec<String>> {
        let components1 = registry.serialize_entity_components(entity1, world)?;
        let components2 = registry.serialize_entity_components(entity2, world)?;

        let mut differences = Vec::new();

        // 检查组件1中存在但组件2中不存在的
        for (component_type, data1) in &components1 {
            if let Some(data2) = components2.get(component_type) {
                if data1 != data2 {
                    differences.push(format!("Component {} has different values", component_type));
                }
            } else {
                differences.push(format!("Component {} exists in entity1 but not in entity2", component_type));
            }
        }

        // 检查组件2中存在但组件1中不存在的
        for component_type in components2.keys() {
            if !components1.contains_key(component_type) {
                differences.push(format!("Component {} exists in entity2 but not in entity1", component_type));
            }
        }

        Ok(differences)
    }

    /// 导出实体为JSON
    pub fn export_entity_to_json(
        entity: Entity,
        world: &World,
        registry: &ComponentRegistry,
        pretty: bool,
    ) -> EngineResult<String> {
        let components = registry.serialize_entity_components(entity, world)?;
        
        let entity_data = serde_json::json!({
            "entity_id": entity.id(),
            "components": components
        });

        if pretty {
            Ok(serde_json::to_string_pretty(&entity_data)?)
        } else {
            Ok(serde_json::to_string(&entity_data)?)
        }
    }

    /// 从JSON导入实体
    pub fn import_entity_from_json(
        json: &str,
        world: &mut World,
        registry: &ComponentRegistry,
    ) -> EngineResult<Entity> {
        let entity_data: serde_json::Value = serde_json::from_str(json)?;
        
        let entity = world.create_entity().build();
        
        if let Some(components) = entity_data.get("components").and_then(|c| c.as_object()) {
            let components_map: HashMap<String, serde_json::Value> = components
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            
            // registry.deserialize_entity_components(entity, &components_map, world)?; // TODO: Fix entity type
        }

        Ok(entity)
    }
}
