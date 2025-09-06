//! 资源管理器

use crate::{EngineResult, EngineError};
use crate::assets::{AssetHandle, AssetLoader, AssetCache, AssetHandleManager, CacheStrategy};
use crate::render::{Texture, Mesh, Material, Shader};
use crate::events::{EventSystem, AssetLoadedEvent, AssetLoadFailedEvent};

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::path::{Path, PathBuf};

/// 资源管理器 - 统一管理所有游戏资源
pub struct AssetManager {
    /// 资源加载器
    loaders: HashMap<String, Box<dyn AssetLoader>>,
    /// 资源缓存
    cache: AssetCache,
    /// 句柄管理器
    handle_manager: AssetHandleManager,
    /// 资源根目录
    asset_root: PathBuf,
    /// 默认缓存策略
    default_cache_strategy: CacheStrategy,
    /// 事件系统引用
    event_system: Option<Arc<RwLock<EventSystem>>>,
}

impl AssetManager {
    /// 创建新的资源管理器
    pub fn new() -> EngineResult<Self> {
        let mut manager = Self {
            loaders: HashMap::new(),
            cache: AssetCache::default(),
            handle_manager: AssetHandleManager::new(),
            asset_root: PathBuf::from("assets"),
            default_cache_strategy: CacheStrategy::RefCount,
            event_system: None,
        };

        // 注册默认加载器
        manager.register_default_loaders()?;

        Ok(manager)
    }

    /// 设置资源根目录
    pub fn set_asset_root(&mut self, path: impl Into<PathBuf>) {
        self.asset_root = path.into();
    }

    /// 设置事件系统
    pub fn set_event_system(&mut self, event_system: Arc<RwLock<EventSystem>>) {
        self.event_system = Some(event_system);
    }

    /// 设置默认缓存策略
    pub fn set_default_cache_strategy(&mut self, strategy: CacheStrategy) {
        self.default_cache_strategy = strategy;
    }

    /// 注册资源加载器
    pub fn register_loader<L: AssetLoader + 'static>(&mut self, extension: impl Into<String>, loader: L) {
        self.loaders.insert(extension.into(), Box::new(loader));
    }

    /// 注册默认加载器
    fn register_default_loaders(&mut self) -> EngineResult<()> {
        // 注册纹理加载器
        self.register_loader("png", TextureLoader);
        self.register_loader("jpg", TextureLoader);
        self.register_loader("jpeg", TextureLoader);
        self.register_loader("bmp", TextureLoader);
        self.register_loader("tga", TextureLoader);

        // 注册着色器加载器  
        self.register_loader("wgsl", ShaderLoader);
        
        // 注册网格加载器
        self.register_loader("obj", MeshLoader);
        
        // 注册材质加载器
        self.register_loader("json", MaterialLoader);

        Ok(())
    }

    /// 同步加载资源
    pub fn load<T: Send + Sync + 'static>(&mut self, path: impl AsRef<Path>) -> EngineResult<AssetHandle<T>> {
        let path = path.as_ref();
        let full_path = self.asset_root.join(path);
        let path_str = path.to_string_lossy().to_string();

        // 检查缓存
        if let Some(resource) = self.cache.get_by_path::<T>(&path_str) {
            return Ok(AssetHandle::new(
                self.handle_manager.generate_id(),
                &resource,
                path_str
            ));
        }

        // 获取文件扩展名
        let extension = full_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        // 查找对应的加载器
        let loader = self.loaders.get(&extension)
            .ok_or_else(|| EngineError::AssetError(format!("没有找到扩展名 '{}' 的加载器", extension)))?;

        // 加载资源
        match loader.load(&full_path) {
            Ok(resource_any) => {
                // 尝试转换为目标类型
                if let Ok(resource) = resource_any.downcast::<T>() {
                    // 计算资源大小 (简化估算)
                    let size_bytes = std::mem::size_of::<T>();
                    
                    // 插入缓存并获取句柄
                    let handle = self.cache.insert(
                        self.handle_manager.generate_id(),
                        resource,
                        path_str.clone(),
                        self.default_cache_strategy,
                        size_bytes
                    );

                    // 发送加载成功事件
                    self.emit_asset_loaded(&path_str, std::any::type_name::<T>());

                    Ok(handle)
                } else {
                    let error = format!("资源类型不匹配: {} -> {}", 
                        std::any::type_name::<T>(), 
                        "unknown");
                    self.emit_asset_load_failed(&path_str, &error);
                    Err(EngineError::AssetError(error))
                }
            }
            Err(e) => {
                let error = format!("加载资源失败: {}", e);
                self.emit_asset_load_failed(&path_str, &error);
                Err(e)
            }
        }
    }

    /// 异步加载资源 (简化版本，实际需要完整的异步实现)
    pub async fn load_async<T: Send + Sync + 'static>(&mut self, path: impl AsRef<Path>) -> EngineResult<AssetHandle<T>> {
        // 在实际实现中，这里应该使用异步文件IO和工作线程
        self.load(path)
    }

    /// 通过句柄获取资源
    pub fn get<T: Send + Sync + 'static>(&self, handle: &AssetHandle<T>) -> Option<Arc<T>> {
        handle.get()
    }

    /// 检查资源是否已加载
    pub fn is_loaded(&self, path: impl AsRef<Path>) -> bool {
        let path_str = path.as_ref().to_string_lossy().to_string();
        self.cache.contains_path(&path_str)
    }

    /// 卸载资源
    pub fn unload(&mut self, handle: &AssetHandle<impl Send + Sync>) -> bool {
        self.cache.remove(handle.id())
    }

    /// 通过路径卸载资源
    pub fn unload_by_path(&mut self, path: impl AsRef<Path>) -> bool {
        let path_str = path.as_ref().to_string_lossy().to_string();
        if let Some(resource) = self.cache.get_by_path::<()>(&path_str) {
            // 这里需要一个方法来获取资源ID，简化处理
            false // 临时返回false
        } else {
            false
        }
    }

    /// 清理缓存
    pub fn cleanup(&self) -> usize {
        self.cache.cleanup()
    }

    /// 清空所有缓存
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// 获取缓存统计
    pub fn cache_stats(&self) -> crate::assets::asset_cache::CacheStats {
        self.cache.stats()
    }

    /// 预热资源
    pub fn preheat(&self, paths: &[&str]) {
        self.cache.preheat(paths);
    }

    /// 重新加载资源
    pub fn reload<T: Send + Sync + 'static>(&mut self, path: impl AsRef<Path>) -> EngineResult<AssetHandle<T>> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        // 先从缓存中移除
        if let Some(_) = self.cache.get_by_path::<T>(&path_str) {
            // self.cache.remove_by_path(&path_str); // 需要实现这个方法
        }
        
        // 重新加载
        self.load(path)
    }

    /// 批量加载资源
    pub fn load_batch(&mut self, paths: &[&str]) -> Vec<EngineResult<()>> {
        let mut results = Vec::new();
        
        for &path in paths {
            // 根据扩展名确定类型并加载
            let extension = Path::new(path).extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_lowercase();
                
            let result = match extension.as_str() {
                "png" | "jpg" | "jpeg" | "bmp" | "tga" => {
                    self.load::<Texture>(path).map(|_| ())
                }
                "wgsl" => {
                    self.load::<Shader>(path).map(|_| ())
                }
                "obj" => {
                    self.load::<Mesh>(path).map(|_| ())
                }
                "json" => {
                    self.load::<Material>(path).map(|_| ())
                }
                _ => Err(EngineError::AssetError(format!("未知的文件类型: {}", extension)))
            };
            
            results.push(result);
        }
        
        results
    }

    /// 发送资源加载成功事件
    fn emit_asset_loaded(&self, path: &str, asset_type: &str) {
        if let Some(event_system) = &self.event_system {
            if let Ok(mut events) = event_system.write() {
                events.publish(AssetLoadedEvent {
                    asset_path: path.to_string(),
                    asset_type: asset_type.to_string(),
                });
            }
        }
    }

    /// 发送资源加载失败事件
    fn emit_asset_load_failed(&self, path: &str, error: &str) {
        if let Some(event_system) = &self.event_system {
            if let Ok(mut events) = event_system.write() {
                events.publish(AssetLoadFailedEvent {
                    asset_path: path.to_string(),
                    error: error.to_string(),
                });
            }
        }
    }

    /// 获取已加载资源的路径列表
    pub fn loaded_assets(&self) -> Vec<String> {
        self.cache.cached_paths()
    }
}

// 简单的资源加载器实现

/// 纹理加载器
struct TextureLoader;

impl AssetLoader for TextureLoader {
    fn load(&self, path: &Path) -> EngineResult<Arc<dyn std::any::Any + Send + Sync>> {
        let texture = Texture::from_file(path)?;
        Ok(Arc::new(texture))
    }

    fn extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "bmp", "tga"]
    }
}

/// 着色器加载器
struct ShaderLoader;

impl AssetLoader for ShaderLoader {
    fn load(&self, path: &Path) -> EngineResult<Arc<dyn std::any::Any + Send + Sync>> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| EngineError::IoError(e))?;
            
        let name = path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
            
        let shader = Shader::new(name).with_wgsl_source(source);
        Ok(Arc::new(shader))
    }

    fn extensions(&self) -> &[&str] {
        &["wgsl"]
    }
}

/// 网格加载器
struct MeshLoader;

impl AssetLoader for MeshLoader {
    fn load(&self, path: &Path) -> EngineResult<Arc<dyn std::any::Any + Send + Sync>> {
        // 简化的OBJ加载实现
        let mesh = Mesh::cube(); // 临时返回立方体
        Ok(Arc::new(mesh))
    }

    fn extensions(&self) -> &[&str] {
        &["obj"]
    }
}

/// 材质加载器
struct MaterialLoader;

impl AssetLoader for MaterialLoader {
    fn load(&self, path: &Path) -> EngineResult<Arc<dyn std::any::Any + Send + Sync>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| EngineError::IoError(e))?;
            
        // 简化的JSON材质加载
        let material = Material::default();
        Ok(Arc::new(material))
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
