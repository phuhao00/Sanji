//! 场景图系统

use crate::{EngineResult, EngineError};
use specs::Entity;
use std::collections::{HashMap, HashSet};

/// 场景图节点
#[derive(Debug, Clone)]
pub struct SceneNode {
    /// 实体引用
    pub entity: Entity,
    /// 父节点
    pub parent: Option<Entity>,
    /// 子节点列表
    pub children: Vec<Entity>,
    /// 是否启用
    pub enabled: bool,
    /// 节点名称 (可选)
    pub name: Option<String>,
    /// 本地变换是否已修改
    pub transform_dirty: bool,
}

impl SceneNode {
    /// 创建新的场景节点
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            parent: None,
            children: Vec::new(),
            enabled: true,
            name: None,
            transform_dirty: true,
        }
    }

    /// 设置名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// 添加子节点
    pub fn add_child(&mut self, child: Entity) {
        if !self.children.contains(&child) {
            self.children.push(child);
        }
    }

    /// 移除子节点
    pub fn remove_child(&mut self, child: Entity) {
        self.children.retain(|&c| c != child);
    }

    /// 检查是否有指定子节点
    pub fn has_child(&self, child: Entity) -> bool {
        self.children.contains(&child)
    }

    /// 获取子节点数量
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// 标记变换为脏
    pub fn mark_transform_dirty(&mut self) {
        self.transform_dirty = true;
    }

    /// 清除变换脏标记
    pub fn clear_transform_dirty(&mut self) {
        self.transform_dirty = false;
    }
}

/// 场景图 - 管理实体的层次结构
#[derive(Debug)]
pub struct SceneGraph {
    /// 所有节点
    nodes: HashMap<Entity, SceneNode>,
    /// 根节点列表
    root_entities: Vec<Entity>,
    /// 启用状态缓存
    enabled_cache: HashMap<Entity, bool>,
    /// 缓存是否有效
    cache_valid: bool,
}

impl SceneGraph {
    /// 创建新的场景图
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_entities: Vec::new(),
            enabled_cache: HashMap::new(),
            cache_valid: false,
        }
    }

    /// 添加实体到场景图
    pub fn add_entity(&mut self, entity: Entity, parent: Option<Entity>) -> EngineResult<()> {
        // 如果实体已存在，先移除
        if self.nodes.contains_key(&entity) {
            self.remove_entity(entity)?;
        }

        let mut node = SceneNode::new(entity);

        if let Some(parent_entity) = parent {
            // 检查父节点是否存在
            if !self.nodes.contains_key(&parent_entity) {
                return Err(EngineError::AssetError(format!("父节点不存在: {:?}", parent_entity)).into());
            }

            // 检查是否会造成循环引用
            if self.would_create_cycle(entity, parent_entity) {
                return Err(EngineError::AssetError("添加实体会造成循环引用".to_string()).into());
            }

            node.parent = Some(parent_entity);
            
            // 添加到父节点的子列表
            if let Some(parent_node) = self.nodes.get_mut(&parent_entity) {
                parent_node.add_child(entity);
            }
        } else {
            // 添加到根节点列表
            self.root_entities.push(entity);
        }

        self.nodes.insert(entity, node);
        self.invalidate_cache();
        
        Ok(())
    }

    /// 移除实体及其所有子节点
    pub fn remove_entity(&mut self, entity: Entity) -> EngineResult<()> {
        if let Some(node) = self.nodes.get(&entity).cloned() {
            // 递归移除所有子节点
            let children = node.children.clone();
            for child in children {
                self.remove_entity(child)?;
            }

            // 从父节点的子列表中移除
            if let Some(parent) = node.parent {
                if let Some(parent_node) = self.nodes.get_mut(&parent) {
                    parent_node.remove_child(entity);
                }
            } else {
                // 从根节点列表中移除
                self.root_entities.retain(|&e| e != entity);
            }

            // 移除节点
            self.nodes.remove(&entity);
            self.invalidate_cache();
        }

        Ok(())
    }

    /// 设置实体的父节点
    pub fn set_parent(&mut self, child: Entity, new_parent: Option<Entity>) -> EngineResult<()> {
        if !self.nodes.contains_key(&child) {
            return Err(EngineError::AssetError(format!("子节点不存在: {:?}", child)).into());
        }

        if let Some(parent) = new_parent {
            if !self.nodes.contains_key(&parent) {
                return Err(EngineError::AssetError(format!("父节点不存在: {:?}", parent)).into());
            }

            // 检查循环引用
            if self.would_create_cycle(child, parent) {
                return Err(EngineError::AssetError("设置父节点会造成循环引用".to_string()).into());
            }
        }

        // 获取当前父节点
        let current_parent = self.nodes.get(&child).unwrap().parent;

        // 从当前父节点移除
        if let Some(old_parent) = current_parent {
            if let Some(old_parent_node) = self.nodes.get_mut(&old_parent) {
                old_parent_node.remove_child(child);
            }
        } else {
            // 从根节点列表移除
            self.root_entities.retain(|&e| e != child);
        }

        // 设置新父节点
        if let Some(new_parent) = new_parent {
            // 添加到新父节点
            if let Some(parent_node) = self.nodes.get_mut(&new_parent) {
                parent_node.add_child(child);
            }
        } else {
            // 添加到根节点列表
            self.root_entities.push(child);
        }

        // 更新子节点的父引用
        if let Some(child_node) = self.nodes.get_mut(&child) {
            child_node.parent = new_parent;
            child_node.mark_transform_dirty();
        }

        self.invalidate_cache();
        Ok(())
    }

    /// 获取实体的父节点
    pub fn get_parent(&self, entity: Entity) -> Option<Entity> {
        self.nodes.get(&entity)?.parent
    }

    /// 获取实体的所有子节点
    pub fn get_children(&self, entity: Entity) -> Vec<Entity> {
        self.nodes.get(&entity)
            .map(|node| node.children.clone())
            .unwrap_or_default()
    }

    /// 获取实体的所有后代节点
    pub fn get_descendants(&self, entity: Entity) -> Vec<Entity> {
        let mut descendants = Vec::new();
        self.collect_descendants(entity, &mut descendants);
        descendants
    }

    /// 递归收集后代节点
    fn collect_descendants(&self, entity: Entity, descendants: &mut Vec<Entity>) {
        if let Some(node) = self.nodes.get(&entity) {
            for &child in &node.children {
                descendants.push(child);
                self.collect_descendants(child, descendants);
            }
        }
    }

    /// 获取实体的所有祖先节点
    pub fn get_ancestors(&self, entity: Entity) -> Vec<Entity> {
        let mut ancestors = Vec::new();
        let mut current = entity;
        
        while let Some(parent) = self.get_parent(current) {
            ancestors.push(parent);
            current = parent;
        }
        
        ancestors
    }

    /// 获取根节点列表
    pub fn get_root_entities(&self) -> Vec<Entity> {
        self.root_entities.clone()
    }

    /// 检查实体是否为根节点
    pub fn is_root(&self, entity: Entity) -> bool {
        self.root_entities.contains(&entity)
    }

    /// 检查实体是否启用 (考虑父节点状态)
    pub fn is_enabled(&self, entity: Entity) -> bool {
        if !self.cache_valid {
            self.update_enabled_cache();
        }
        
        self.enabled_cache.get(&entity).copied().unwrap_or(false)
    }

    /// 设置实体启用状态
    pub fn set_enabled(&mut self, entity: Entity, enabled: bool) {
        if let Some(node) = self.nodes.get_mut(&entity) {
            node.enabled = enabled;
            self.invalidate_cache();
        }
    }

    /// 检查是否会创建循环引用
    fn would_create_cycle(&self, child: Entity, potential_parent: Entity) -> bool {
        if child == potential_parent {
            return true;
        }

        let ancestors = self.get_ancestors(potential_parent);
        ancestors.contains(&child)
    }

    /// 更新启用状态缓存
    fn update_enabled_cache(&self) {
        // 这个方法应该是不可变的，但我们需要修改缓存
        // 在实际实现中，可能需要使用内部可变性或重新设计
    }

    /// 使缓存无效
    fn invalidate_cache(&mut self) {
        self.cache_valid = false;
        self.enabled_cache.clear();
    }

    /// 遍历场景图 (深度优先)
    pub fn traverse_depth_first<F>(&self, mut visitor: F) 
    where 
        F: FnMut(Entity, usize),
    {
        for &root in &self.root_entities {
            self.traverse_depth_first_recursive(root, 0, &mut visitor);
        }
    }

    /// 递归深度优先遍历
    fn traverse_depth_first_recursive<F>(&self, entity: Entity, depth: usize, visitor: &mut F) 
    where 
        F: FnMut(Entity, usize),
    {
        visitor(entity, depth);
        
        if let Some(node) = self.nodes.get(&entity) {
            for &child in &node.children {
                self.traverse_depth_first_recursive(child, depth + 1, visitor);
            }
        }
    }

    /// 遍历场景图 (广度优先)
    pub fn traverse_breadth_first<F>(&self, mut visitor: F) 
    where 
        F: FnMut(Entity, usize),
    {
        let mut queue = std::collections::VecDeque::new();
        
        // 添加所有根节点
        for &root in &self.root_entities {
            queue.push_back((root, 0));
        }
        
        while let Some((entity, depth)) = queue.pop_front() {
            visitor(entity, depth);
            
            // 添加子节点到队列
            if let Some(node) = self.nodes.get(&entity) {
                for &child in &node.children {
                    queue.push_back((child, depth + 1));
                }
            }
        }
    }

    /// 查找实体路径 (从根节点到实体的路径)
    pub fn get_entity_path(&self, entity: Entity) -> Vec<Entity> {
        let mut path = Vec::new();
        let mut current = entity;
        
        // 向上遍历到根节点
        loop {
            path.push(current);
            if let Some(parent) = self.get_parent(current) {
                current = parent;
            } else {
                break;
            }
        }
        
        // 反转路径，使其从根节点开始
        path.reverse();
        path
    }

    /// 通过路径查找实体
    pub fn find_entity_by_path(&self, path: &[Entity]) -> Option<Entity> {
        if path.is_empty() {
            return None;
        }
        
        // 检查路径是否有效
        for i in 1..path.len() {
            let parent = path[i - 1];
            let child = path[i];
            
            if !self.get_children(parent).contains(&child) {
                return None;
            }
        }
        
        path.last().copied()
    }

    /// 获取场景图统计信息
    pub fn stats(&self) -> SceneGraphStats {
        let mut max_depth = 0;
        let mut total_depth = 0;
        let mut node_count = 0;
        
        self.traverse_depth_first(|_, depth| {
            max_depth = max_depth.max(depth);
            total_depth += depth;
            node_count += 1;
        });
        
        SceneGraphStats {
            node_count,
            root_count: self.root_entities.len(),
            max_depth,
            average_depth: if node_count > 0 { total_depth as f32 / node_count as f32 } else { 0.0 },
        }
    }

    /// 清空场景图
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root_entities.clear();
        self.invalidate_cache();
    }

    /// 更新场景图 (每帧调用)
    pub fn update(&mut self, _delta_time: f32) {
        // 这里可以添加场景图的更新逻辑
        // 比如更新变换矩阵、检查脏标记等
    }

    /// 获取节点
    pub fn get_node(&self, entity: Entity) -> Option<&SceneNode> {
        self.nodes.get(&entity)
    }

    /// 获取可变节点
    pub fn get_node_mut(&mut self, entity: Entity) -> Option<&mut SceneNode> {
        self.nodes.get_mut(&entity)
    }

    /// 获取所有节点
    pub fn all_entities(&self) -> Vec<Entity> {
        self.nodes.keys().copied().collect()
    }
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// 场景图统计信息
#[derive(Debug, Clone)]
pub struct SceneGraphStats {
    pub node_count: usize,
    pub root_count: usize,
    pub max_depth: usize,
    pub average_depth: f32,
}
