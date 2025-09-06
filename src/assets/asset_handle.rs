//! 资源句柄系统

use std::sync::{Arc, Weak};
use std::any::Any;
use std::fmt;

/// 资源ID类型
pub type AssetId = u64;

/// 资源句柄 - 用于安全地引用资源
#[derive(Clone)]
pub struct AssetHandle<T> {
    id: AssetId,
    inner: Weak<T>,
    path: String,
}

impl<T> AssetHandle<T> {
    /// 创建新的资源句柄
    pub fn new(id: AssetId, resource: &Arc<T>, path: impl Into<String>) -> Self {
        Self {
            id,
            inner: Arc::downgrade(resource),
            path: path.into(),
        }
    }

    /// 获取资源ID
    pub fn id(&self) -> AssetId {
        self.id
    }

    /// 获取资源路径
    pub fn path(&self) -> &str {
        &self.path
    }

    /// 尝试获取资源
    pub fn get(&self) -> Option<Arc<T>> {
        self.inner.upgrade()
    }

    /// 检查资源是否仍然有效
    pub fn is_valid(&self) -> bool {
        self.inner.strong_count() > 0
    }

    /// 获取弱引用计数
    pub fn weak_count(&self) -> usize {
        self.inner.weak_count()
    }

    /// 获取强引用计数
    pub fn strong_count(&self) -> usize {
        self.inner.strong_count()
    }
}

impl<T> fmt::Debug for AssetHandle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AssetHandle")
            .field("id", &self.id)
            .field("path", &self.path)
            .field("valid", &self.is_valid())
            .finish()
    }
}

impl<T> PartialEq for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for AssetHandle<T> {}

impl<T> std::hash::Hash for AssetHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// 无类型资源句柄
#[derive(Clone)]
pub struct UntypedAssetHandle {
    id: AssetId,
    inner: Weak<dyn Any + Send + Sync>,
    path: String,
    type_name: &'static str,
}

impl UntypedAssetHandle {
    /// 创建新的无类型资源句柄
    pub fn new<T: Send + Sync + 'static>(id: AssetId, resource: &Arc<T>, path: impl Into<String>) -> Self {
        Self {
            id,
            inner: Arc::downgrade(resource) as Weak<dyn Any + Send + Sync>,
            path: path.into(),
            type_name: std::any::type_name::<T>(),
        }
    }

    /// 获取资源ID
    pub fn id(&self) -> AssetId {
        self.id
    }

    /// 获取资源路径
    pub fn path(&self) -> &str {
        &self.path
    }

    /// 获取资源类型名
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// 尝试转换为类型化句柄
    pub fn typed<T: Send + Sync + 'static>(&self) -> Option<AssetHandle<T>> {
        if std::any::type_name::<T>() == self.type_name {
            if let Some(arc) = self.inner.upgrade() {
                if let Ok(typed_arc) = arc.downcast::<T>() {
                    return Some(AssetHandle::new(self.id, &typed_arc, &self.path));
                }
            }
        }
        None
    }

    /// 检查资源是否仍然有效
    pub fn is_valid(&self) -> bool {
        self.inner.strong_count() > 0
    }
}

impl fmt::Debug for UntypedAssetHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UntypedAssetHandle")
            .field("id", &self.id)
            .field("path", &self.path)
            .field("type_name", &self.type_name)
            .field("valid", &self.is_valid())
            .finish()
    }
}

/// 资源句柄管理器
pub struct AssetHandleManager {
    next_id: AssetId,
}

impl AssetHandleManager {
    /// 创建新的资源句柄管理器
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    /// 生成新的资源ID
    pub fn generate_id(&mut self) -> AssetId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// 创建类型化资源句柄
    pub fn create_handle<T: Send + Sync + 'static>(&mut self, resource: Arc<T>, path: impl Into<String>) -> AssetHandle<T> {
        let id = self.generate_id();
        AssetHandle::new(id, &resource, path)
    }

    /// 创建无类型资源句柄
    pub fn create_untyped_handle<T: Send + Sync + 'static>(&mut self, resource: Arc<T>, path: impl Into<String>) -> UntypedAssetHandle {
        let id = self.generate_id();
        UntypedAssetHandle::new(id, &resource, path)
    }
}

impl Default for AssetHandleManager {
    fn default() -> Self {
        Self::new()
    }
}
