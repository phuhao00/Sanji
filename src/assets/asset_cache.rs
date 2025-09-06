//! 资源缓存系统

use crate::assets::{AssetHandle, AssetId, UntypedAssetHandle};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::any::Any;

/// 缓存策略
#[derive(Debug, Clone, Copy)]
pub enum CacheStrategy {
    /// 永久缓存 - 资源永远不会被自动清理
    Permanent,
    /// LRU缓存 - 最近最少使用的资源会被清理
    LRU,
    /// 引用计数 - 当没有外部引用时自动清理
    RefCount,
}

/// 缓存条目
#[derive(Debug)]
struct CacheEntry {
    resource: Arc<dyn Any + Send + Sync>,
    access_count: u64,
    last_access: std::time::Instant,
    strategy: CacheStrategy,
    size_bytes: usize,
    path: String,
    type_name: &'static str,
}

impl CacheEntry {
    fn new<T: Send + Sync + 'static>(
        resource: Arc<T>, 
        path: String, 
        strategy: CacheStrategy,
        size_bytes: usize
    ) -> Self {
        Self {
            resource: resource as Arc<dyn Any + Send + Sync>,
            access_count: 0,
            last_access: std::time::Instant::now(),
            strategy,
            size_bytes,
            path,
            type_name: std::any::type_name::<T>(),
        }
    }

    fn access(&mut self) {
        self.access_count += 1;
        self.last_access = std::time::Instant::now();
    }

    fn get<T: Send + Sync + 'static>(&mut self) -> Option<Arc<T>> {
        self.access();
        self.resource.clone().downcast().ok()
    }

    fn should_cleanup(&self) -> bool {
        match self.strategy {
            CacheStrategy::Permanent => false,
            CacheStrategy::LRU => {
                // 如果超过5分钟没有访问，则可以清理
                self.last_access.elapsed().as_secs() > 300
            },
            CacheStrategy::RefCount => {
                // 如果只有缓存持有引用，则可以清理
                Arc::strong_count(&self.resource) <= 1
            },
        }
    }
}

/// 资源缓存
pub struct AssetCache {
    entries: RwLock<HashMap<AssetId, CacheEntry>>,
    path_to_id: RwLock<HashMap<String, AssetId>>,
    max_size_bytes: usize,
    current_size_bytes: RwLock<usize>,
    cleanup_threshold: f32,
}

impl AssetCache {
    /// 创建新的资源缓存
    pub fn new(max_size_bytes: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            path_to_id: RwLock::new(HashMap::new()),
            max_size_bytes,
            current_size_bytes: RwLock::new(0),
            cleanup_threshold: 0.8, // 当达到80%容量时开始清理
        }
    }

    /// 设置清理阈值
    pub fn set_cleanup_threshold(&mut self, threshold: f32) {
        self.cleanup_threshold = threshold.clamp(0.0, 1.0);
    }

    /// 插入资源到缓存
    pub fn insert<T: Send + Sync + 'static>(
        &self, 
        id: AssetId, 
        resource: Arc<T>, 
        path: impl Into<String>,
        strategy: CacheStrategy,
        size_bytes: usize
    ) -> AssetHandle<T> {
        let path = path.into();
        let handle = AssetHandle::new(id, &resource, &path);
        
        let entry = CacheEntry::new(resource, path.clone(), strategy, size_bytes);
        
        {
            let mut entries = self.entries.write().unwrap();
            let mut path_to_id = self.path_to_id.write().unwrap();
            let mut current_size = self.current_size_bytes.write().unwrap();
            
            // 如果已存在，先移除旧的
            if let Some(old_entry) = entries.get(&id) {
                *current_size -= old_entry.size_bytes;
            }
            
            entries.insert(id, entry);
            path_to_id.insert(path, id);
            *current_size += size_bytes;
        }
        
        // 检查是否需要清理
        self.maybe_cleanup();
        
        handle
    }

    /// 通过ID获取资源
    pub fn get<T: Send + Sync + 'static>(&self, id: AssetId) -> Option<Arc<T>> {
        let mut entries = self.entries.write().unwrap();
        if let Some(entry) = entries.get_mut(&id) {
            entry.get()
        } else {
            None
        }
    }

    /// 通过路径获取资源
    pub fn get_by_path<T: Send + Sync + 'static>(&self, path: &str) -> Option<Arc<T>> {
        let path_to_id = self.path_to_id.read().unwrap();
        if let Some(&id) = path_to_id.get(path) {
            drop(path_to_id);
            self.get(id)
        } else {
            None
        }
    }

    /// 检查资源是否在缓存中
    pub fn contains(&self, id: AssetId) -> bool {
        let entries = self.entries.read().unwrap();
        entries.contains_key(&id)
    }

    /// 通过路径检查资源是否在缓存中
    pub fn contains_path(&self, path: &str) -> bool {
        let path_to_id = self.path_to_id.read().unwrap();
        path_to_id.contains_key(path)
    }

    /// 移除资源
    pub fn remove(&self, id: AssetId) -> bool {
        let mut entries = self.entries.write().unwrap();
        let mut path_to_id = self.path_to_id.write().unwrap();
        let mut current_size = self.current_size_bytes.write().unwrap();
        
        if let Some(entry) = entries.remove(&id) {
            path_to_id.remove(&entry.path);
            *current_size -= entry.size_bytes;
            true
        } else {
            false
        }
    }

    /// 清理缓存
    pub fn cleanup(&self) -> usize {
        let mut entries = self.entries.write().unwrap();
        let mut path_to_id = self.path_to_id.write().unwrap();
        let mut current_size = self.current_size_bytes.write().unwrap();
        
        let mut to_remove = Vec::new();
        
        // 收集需要清理的资源
        for (&id, entry) in entries.iter() {
            if entry.should_cleanup() {
                to_remove.push((id, entry.path.clone(), entry.size_bytes));
            }
        }
        
        // 移除资源
        let removed_count = to_remove.len();
        for (id, path, size) in to_remove {
            entries.remove(&id);
            path_to_id.remove(&path);
            *current_size -= size;
        }
        
        removed_count
    }

    /// 如果需要则清理缓存
    fn maybe_cleanup(&self) {
        let current_size = *self.current_size_bytes.read().unwrap();
        let threshold_size = (self.max_size_bytes as f32 * self.cleanup_threshold) as usize;
        
        if current_size > threshold_size {
            self.cleanup();
        }
    }

    /// 强制清理所有可清理的资源
    pub fn force_cleanup(&self) {
        self.cleanup();
    }

    /// 清空所有缓存
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        let mut path_to_id = self.path_to_id.write().unwrap();
        let mut current_size = self.current_size_bytes.write().unwrap();
        
        entries.clear();
        path_to_id.clear();
        *current_size = 0;
    }

    /// 获取缓存统计信息
    pub fn stats(&self) -> CacheStats {
        let entries = self.entries.read().unwrap();
        let current_size = *self.current_size_bytes.read().unwrap();
        
        let entry_count = entries.len();
        let mut type_counts = HashMap::new();
        
        for entry in entries.values() {
            *type_counts.entry(entry.type_name).or_insert(0) += 1;
        }
        
        CacheStats {
            entry_count,
            total_size_bytes: current_size,
            max_size_bytes: self.max_size_bytes,
            usage_ratio: current_size as f32 / self.max_size_bytes as f32,
            type_counts,
        }
    }

    /// 获取所有缓存的路径
    pub fn cached_paths(&self) -> Vec<String> {
        let path_to_id = self.path_to_id.read().unwrap();
        path_to_id.keys().cloned().collect()
    }

    /// 获取指定类型的所有资源
    pub fn get_all_of_type<T: Send + Sync + 'static>(&self) -> Vec<Arc<T>> {
        let entries = self.entries.read().unwrap();
        let target_type = std::any::type_name::<T>();
        
        entries.values()
            .filter(|entry| entry.type_name == target_type)
            .filter_map(|entry| entry.resource.clone().downcast().ok())
            .collect()
    }

    /// 预热缓存 - 预先加载指定的资源
    pub fn preheat(&self, paths: &[&str]) {
        // 这里可以实现预热逻辑
        // 实际实现需要结合资源加载器
        log::info!("预热缓存: {:?}", paths);
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entry_count: usize,
    pub total_size_bytes: usize,
    pub max_size_bytes: usize,
    pub usage_ratio: f32,
    pub type_counts: HashMap<&'static str, usize>,
}

impl CacheStats {
    /// 是否接近容量限制
    pub fn is_near_capacity(&self) -> bool {
        self.usage_ratio > 0.8
    }

    /// 获取最常见的资源类型
    pub fn most_common_type(&self) -> Option<(&'static str, usize)> {
        self.type_counts.iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&name, &count)| (name, count))
    }

    /// 格式化为可读的字符串
    pub fn format(&self) -> String {
        format!(
            "缓存统计: {} 项, {:.1} MB / {:.1} MB ({:.1}%)",
            self.entry_count,
            self.total_size_bytes as f32 / 1024.0 / 1024.0,
            self.max_size_bytes as f32 / 1024.0 / 1024.0,
            self.usage_ratio * 100.0
        )
    }
}

impl Default for AssetCache {
    fn default() -> Self {
        // 默认512MB缓存
        Self::new(512 * 1024 * 1024)
    }
}
