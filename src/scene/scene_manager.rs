//! 场景管理器

use crate::{EngineResult, EngineError};
use crate::scene::Scene;
use crate::ecs::ECSWorld;
use crate::events::{EventSystem, SceneLoadedEvent, SceneUnloadedEvent};

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::Path;

/// 场景管理器 - 管理多个场景的加载、切换和更新
pub struct SceneManager {
    /// 所有场景
    scenes: HashMap<String, Scene>,
    /// 当前激活的场景
    current_scene: Option<String>,
    /// 下一个要切换到的场景
    next_scene: Option<String>,
    /// 场景切换标志
    scene_transition: bool,
    /// 事件系统引用
    event_system: Option<Arc<RwLock<EventSystem>>>,
    /// 场景根目录
    scene_root: std::path::PathBuf,
}

impl SceneManager {
    /// 创建新的场景管理器
    pub fn new() -> Self {
        Self {
            scenes: HashMap::new(),
            current_scene: None,
            next_scene: None,
            scene_transition: false,
            event_system: None,
            scene_root: std::path::PathBuf::from("scenes"),
        }
    }

    /// 设置事件系统
    pub fn set_event_system(&mut self, event_system: Arc<RwLock<EventSystem>>) {
        self.event_system = Some(event_system);
    }

    /// 设置场景根目录
    pub fn set_scene_root(&mut self, path: impl Into<std::path::PathBuf>) {
        self.scene_root = path.into();
    }

    /// 添加场景
    pub fn add_scene(&mut self, scene: Scene) {
        let name = scene.name.clone();
        self.scenes.insert(name, scene);
    }

    /// 创建新场景
    pub fn create_scene(&mut self, name: impl Into<String>) -> &mut Scene {
        let name = name.into();
        let scene = Scene::new(name.clone());
        self.scenes.insert(name.clone(), scene);
        self.scenes.get_mut(&name).unwrap()
    }

    /// 创建默认场景
    pub fn create_default_scene(&mut self, name: impl Into<String>) -> &mut Scene {
        let name = name.into();
        let scene = Scene::create_default_scene(name.clone());
        self.scenes.insert(name.clone(), scene);
        self.scenes.get_mut(&name).unwrap()
    }

    /// 获取场景
    pub fn get_scene(&self, name: &str) -> Option<&Scene> {
        self.scenes.get(name)
    }

    /// 获取可变场景
    pub fn get_scene_mut(&mut self, name: &str) -> Option<&mut Scene> {
        self.scenes.get_mut(name)
    }

    /// 移除场景
    pub fn remove_scene(&mut self, name: &str) -> Option<Scene> {
        // 如果移除的是当前场景，先停用
        if self.current_scene.as_ref() == Some(&name.to_string()) {
            self.current_scene = None;
        }
        
        // 如果移除的是下一个场景，取消切换
        if self.next_scene.as_ref() == Some(&name.to_string()) {
            self.next_scene = None;
            self.scene_transition = false;
        }

        self.scenes.remove(name)
    }

    /// 切换到指定场景
    pub fn switch_to_scene(&mut self, name: impl Into<String>) -> EngineResult<()> {
        let name = name.into();
        
        if !self.scenes.contains_key(&name) {
            return Err(EngineError::AssetError(format!("场景不存在: {}", name)).into());
        }

        // 设置下一个场景和切换标志
        self.next_scene = Some(name);
        self.scene_transition = true;
        
        Ok(())
    }

    /// 立即切换场景
    pub fn switch_to_scene_immediately(&mut self, name: impl Into<String>, world: &mut ECSWorld) -> EngineResult<()> {
        let name = name.into();
        
        if !self.scenes.contains_key(&name) {
            return Err(EngineError::AssetError(format!("场景不存在: {}", name)).into());
        }

        // 停用当前场景
        if let Some(current_name) = &self.current_scene {
            if let Some(current_scene) = self.scenes.get_mut(current_name) {
                current_scene.deactivate();
                current_scene.clear(world)?;
                self.emit_scene_unloaded(current_name);
            }
        }

        // 激活新场景
        if let Some(new_scene) = self.scenes.get_mut(&name) {
            new_scene.activate();
            self.current_scene = Some(name.clone());
            self.emit_scene_loaded(&name);
        }

        Ok(())
    }

    /// 获取当前场景
    pub fn current_scene(&self) -> Option<&Scene> {
        self.current_scene.as_ref()
            .and_then(|name| self.scenes.get(name))
    }

    /// 获取当前场景的可变引用
    pub fn current_scene_mut(&mut self) -> Option<&mut Scene> {
        let current_name = self.current_scene.clone();
        current_name.as_ref()
            .and_then(move |name| self.scenes.get_mut(name))
    }

    /// 获取当前场景名称
    pub fn current_scene_name(&self) -> Option<&String> {
        self.current_scene.as_ref()
    }

    /// 更新场景管理器
    pub fn update(&mut self, delta_time: f32) -> EngineResult<()> {
        // 处理场景切换
        if self.scene_transition {
            // 实际的场景切换逻辑应该在这里实现
            // 可能涉及加载屏幕、资源预加载等
            self.scene_transition = false;
            
            if let Some(next_name) = self.next_scene.take() {
                // 这里应该调用更安全的切换方法
                log::info!("准备切换到场景: {}", next_name);
            }
        }

        // 更新当前场景
        if let Some(current_scene) = self.current_scene_mut() {
            current_scene.update(delta_time)?;
        }

        Ok(())
    }

    /// 安全的场景切换 (带ECS世界参数)
    pub fn process_scene_transition(&mut self, world: &mut ECSWorld) -> EngineResult<()> {
        if self.scene_transition && self.next_scene.is_some() {
            let next_name = self.next_scene.take().unwrap();
            self.switch_to_scene_immediately(next_name, world)?;
            self.scene_transition = false;
        }
        Ok(())
    }

    /// 预载场景
    pub fn preload_scene(&mut self, name: &str) -> EngineResult<()> {
        if let Some(scene) = self.scenes.get_mut(name) {
            // 这里可以实现场景预加载逻辑
            // 比如预加载资源、初始化对象等
            log::info!("预载场景: {}", name);
        } else {
            return Err(EngineError::AssetError(format!("场景不存在: {}", name)).into());
        }
        Ok(())
    }

    /// 卸载场景
    pub fn unload_scene(&mut self, name: &str, world: &mut ECSWorld) -> EngineResult<()> {
        if let Some(scene) = self.scenes.get_mut(name) {
            scene.clear(world)?;
            scene.deactivate();
            self.emit_scene_unloaded(name);
        }
        Ok(())
    }

    /// 从文件加载场景
    pub fn load_scene_from_file(&mut self, path: impl AsRef<Path>) -> EngineResult<String> {
        let path = path.as_ref();
        let full_path = self.scene_root.join(path);
        
        // 读取场景文件
        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| EngineError::IoError(e))?;
        
        // 解析场景数据 (简化实现)
        let scene_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
            
        let scene = Scene::new(scene_name.clone());
        self.scenes.insert(scene_name.clone(), scene);
        
        log::info!("从文件加载场景: {:?}", full_path);
        Ok(scene_name)
    }

    /// 保存场景到文件
    pub fn save_scene_to_file(&self, scene_name: &str, path: impl AsRef<Path>) -> EngineResult<()> {
        let scene = self.scenes.get(scene_name)
            .ok_or_else(|| EngineError::AssetError(format!("场景不存在: {}", scene_name)))?;
        
        let full_path = self.scene_root.join(path);
        
        // 确保目录存在
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| EngineError::IoError(e))?;
        }
        
        // 序列化场景
        let scene_data = scene.serialize()?;
        
        // 写入文件
        std::fs::write(&full_path, scene_data)
            .map_err(|e| EngineError::IoError(e))?;
            
        log::info!("保存场景到文件: {:?}", full_path);
        Ok(())
    }

    /// 获取所有场景名称
    pub fn scene_names(&self) -> Vec<&String> {
        self.scenes.keys().collect()
    }

    /// 检查场景是否存在
    pub fn has_scene(&self, name: &str) -> bool {
        self.scenes.contains_key(name)
    }

    /// 获取场景数量
    pub fn scene_count(&self) -> usize {
        self.scenes.len()
    }

    /// 检查是否有激活的场景
    pub fn has_active_scene(&self) -> bool {
        self.current_scene.is_some()
    }

    /// 检查是否正在进行场景切换
    pub fn is_transitioning(&self) -> bool {
        self.scene_transition
    }

    /// 清空所有场景
    pub fn clear_all_scenes(&mut self, world: &mut ECSWorld) -> EngineResult<()> {
        // 清理所有场景
        for (_, scene) in self.scenes.iter_mut() {
            scene.clear(world)?;
        }
        
        self.scenes.clear();
        self.current_scene = None;
        self.next_scene = None;
        self.scene_transition = false;
        
        Ok(())
    }

    /// 复制场景
    pub fn duplicate_scene(&mut self, source_name: &str, new_name: impl Into<String>) -> EngineResult<()> {
        let new_name = new_name.into();
        
        if !self.scenes.contains_key(source_name) {
            return Err(EngineError::AssetError(format!("源场景不存在: {}", source_name)).into());
        }
        
        if self.scenes.contains_key(&new_name) {
            return Err(EngineError::AssetError(format!("目标场景已存在: {}", new_name)).into());
        }
        
        // 简化的场景复制 - 创建新的空场景
        let new_scene = Scene::new(new_name.clone());
        self.scenes.insert(new_name, new_scene);
        
        Ok(())
    }

    /// 重命名场景
    pub fn rename_scene(&mut self, old_name: &str, new_name: impl Into<String>) -> EngineResult<()> {
        let new_name = new_name.into();
        
        if let Some(mut scene) = self.scenes.remove(old_name) {
            scene.name = new_name.clone();
            self.scenes.insert(new_name.clone(), scene);
            
            // 更新当前场景引用
            if self.current_scene.as_ref() == Some(&old_name.to_string()) {
                self.current_scene = Some(new_name.clone());
            }
            
            // 更新下一个场景引用
            if self.next_scene.as_ref() == Some(&old_name.to_string()) {
                self.next_scene = Some(new_name);
            }
            
            Ok(())
        } else {
            Err(EngineError::AssetError(format!("场景不存在: {}", old_name)).into())
        }
    }

    /// 发送场景加载事件
    fn emit_scene_loaded(&self, scene_name: &str) {
        if let Some(event_system) = &self.event_system {
            if let Ok(mut events) = event_system.write() {
                events.publish(SceneLoadedEvent {
                    scene_name: scene_name.to_string(),
                });
            }
        }
    }

    /// 发送场景卸载事件
    fn emit_scene_unloaded(&self, scene_name: &str) {
        if let Some(event_system) = &self.event_system {
            if let Ok(mut events) = event_system.write() {
                events.publish(SceneUnloadedEvent {
                    scene_name: scene_name.to_string(),
                });
            }
        }
    }

    /// 获取场景管理器统计信息
    pub fn stats(&self) -> SceneManagerStats {
        SceneManagerStats {
            total_scenes: self.scenes.len(),
            active_scene: self.current_scene.clone(),
            is_transitioning: self.scene_transition,
            scene_names: self.scenes.keys().cloned().collect(),
        }
    }
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 场景管理器统计信息
#[derive(Debug, Clone)]
pub struct SceneManagerStats {
    pub total_scenes: usize,
    pub active_scene: Option<String>,
    pub is_transitioning: bool,
    pub scene_names: Vec<String>,
}
