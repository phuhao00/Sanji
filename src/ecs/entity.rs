//! 实体管理工具

use crate::ecs::component::*;
use specs::{World, Entity, WorldExt, Builder};
use glam::{Vec3, Quat};

/// 实体构建器辅助类
pub struct EntityBuilder<'a> {
    builder: specs::EntityBuilder,
    world: &'a mut World,
}

impl<'a> EntityBuilder<'a> {
    /// 创建新的实体构建器
    pub fn new(world: &'a mut World) -> Self {
        let builder = world.create_entity();
        Self { builder, world }
    }

    /// 添加变换组件
    pub fn with_transform(mut self) -> Self {
        self.builder = self.builder.with(Transform::new());
        self
    }

    /// 添加带有位置的变换组件
    pub fn with_transform_at(mut self, position: Vec3) -> Self {
        let mut transform = Transform::new();
        transform.set_position(position);
        self.builder = self.builder.with(transform);
        self
    }

    /// 添加完整的变换组件
    pub fn with_full_transform(mut self, position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        let mut transform = Transform::new();
        transform.set_position(position);
        transform.set_rotation(rotation);
        transform.set_scale(scale);
        self.builder = self.builder.with(transform);
        self
    }

    /// 添加网格渲染器
    pub fn with_mesh_renderer(mut self, mesh_name: impl Into<String>, material_name: impl Into<String>) -> Self {
        self.builder = self.builder.with(MeshRenderer::new(mesh_name, material_name));
        self
    }

    /// 添加相机
    pub fn with_camera(mut self) -> Self {
        self.builder = self.builder.with(Camera::default());
        self
    }

    /// 添加主相机
    pub fn with_main_camera(mut self) -> Self {
        let mut camera = Camera::default();
        camera.camera.is_main = true;
        self.builder = self.builder.with(camera);
        self
    }

    /// 添加光源
    pub fn with_light(mut self, light: Light) -> Self {
        self.builder = self.builder.with(light);
        self
    }

    /// 添加方向光
    pub fn with_directional_light(mut self, color: Vec3, intensity: f32) -> Self {
        let mut light = Light::default();
        light.color = color;
        light.intensity = intensity;
        self.builder = self.builder.with(light);
        self
    }

    /// 添加点光源
    pub fn with_point_light(mut self, color: Vec3, intensity: f32, range: f32) -> Self {
        let mut light = Light::default();
        light.light_type = crate::ecs::component::LightType::Point;
        light.color = color;
        light.intensity = intensity;
        light.range = range;
        self.builder = self.builder.with(light);
        self
    }

    /// 添加刚体
    pub fn with_rigidbody(mut self) -> Self {
        self.builder = self.builder.with(RigidBody::default());
        self
    }

    /// 添加名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.builder = self.builder.with(Name::new(name));
        self
    }

    /// 添加标签
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        let tags = Tag::new().with_tag(tag);
        self.builder = self.builder.with(tags);
        self
    }

    /// 添加多个标签
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        let mut tag_component = Tag::new();
        for tag in tags {
            tag_component.add_tag(tag);
        }
        self.builder = self.builder.with(tag_component);
        self
    }

    /// 构建实体
    pub fn build(self) -> Entity {
        self.builder.build()
    }
}

/// 预制件系统 - 创建常用的游戏对象
pub struct Prefabs;

impl Prefabs {
    /// 创建空的游戏对象
    pub fn empty_game_object(world: &mut World, name: impl Into<String>) -> Entity {
        EntityBuilder::new(world)
            .with_transform()
            .with_name(name)
            .build()
    }

    /// 创建立方体
    pub fn cube(world: &mut World, position: Vec3) -> Entity {
        EntityBuilder::new(world)
            .with_transform_at(position)
            .with_mesh_renderer("cube", "default")
            .with_name("立方体")
            .build()
    }

    /// 创建球体
    pub fn sphere(world: &mut World, position: Vec3) -> Entity {
        EntityBuilder::new(world)
            .with_transform_at(position)
            .with_mesh_renderer("sphere", "default")
            .with_name("球体")
            .build()
    }

    /// 创建平面
    pub fn plane(world: &mut World, position: Vec3) -> Entity {
        EntityBuilder::new(world)
            .with_transform_at(position)
            .with_mesh_renderer("plane", "default")
            .with_name("平面")
            .build()
    }

    /// 创建主相机
    pub fn main_camera(world: &mut World, position: Vec3) -> Entity {
        EntityBuilder::new(world)
            .with_transform_at(position)
            .with_main_camera()
            .with_name("主相机")
            .build()
    }

    /// 创建方向光
    pub fn directional_light(world: &mut World, color: Vec3, intensity: f32) -> Entity {
        EntityBuilder::new(world)
            .with_transform()
            .with_directional_light(color, intensity)
            .with_name("方向光")
            .build()
    }

    /// 创建点光源
    pub fn point_light(world: &mut World, position: Vec3, color: Vec3, intensity: f32, range: f32) -> Entity {
        EntityBuilder::new(world)
            .with_transform_at(position)
            .with_point_light(color, intensity, range)
            .with_name("点光源")
            .build()
    }

    /// 创建带物理的球体
    pub fn physics_sphere(world: &mut World, position: Vec3) -> Entity {
        EntityBuilder::new(world)
            .with_transform_at(position)
            .with_mesh_renderer("sphere", "default")
            .with_rigidbody()
            .with_name("物理球体")
            .build()
    }
}

/// 实体查询辅助工具
pub struct EntityQuery;

impl EntityQuery {
    /// 查找带有指定名称的实体
    pub fn find_by_name(world: &World, name: &str) -> Option<Entity> {
        use specs::Join;
        
        let entities = world.entities();
        let names = world.read_storage::<Name>();
        
        for (entity, name_comp) in (&entities, &names).join() {
            if name_comp.name == name {
                return Some(entity);
            }
        }
        None
    }

    /// 查找带有指定标签的所有实体
    pub fn find_by_tag(world: &World, tag: &str) -> Vec<Entity> {
        use specs::Join;
        
        let entities = world.entities();
        let tags = world.read_storage::<Tag>();
        
        (&entities, &tags).join()
            .filter(|(_, tag_comp)| tag_comp.has_tag(tag))
            .map(|(entity, _)| entity)
            .collect()
    }

    /// 查找主相机
    pub fn find_main_camera(world: &World) -> Option<Entity> {
        use specs::Join;
        
        let entities = world.entities();
        let cameras = world.read_storage::<Camera>();
        
        for (entity, camera) in (&entities, &cameras).join() {
            if camera.camera.is_main {
                return Some(entity);
            }
        }
        None
    }
}
