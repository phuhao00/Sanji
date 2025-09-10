//! UI布局系统

use crate::math::Vec2;
use crate::ui::{UIStyle, WidgetId};
use crate::ui::widgets::Rect;
use crate::ui::style::{Display, Position, FlexDirection, JustifyContent, AlignItems, FlexWrap};
use std::collections::HashMap;

/// 布局约束
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutConstraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl LayoutConstraints {
    pub fn new(min_width: f32, max_width: f32, min_height: f32, max_height: f32) -> Self {
        Self {
            min_width,
            max_width,
            min_height,
            max_height,
        }
    }

    pub fn fixed(width: f32, height: f32) -> Self {
        Self {
            min_width: width,
            max_width: width,
            min_height: height,
            max_height: height,
        }
    }

    pub fn unbounded() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }

    pub fn constrain(&self, size: Vec2) -> Vec2 {
        Vec2::new(
            size.x.clamp(self.min_width, self.max_width),
            size.y.clamp(self.min_height, self.max_height),
        )
    }

    pub fn is_valid_size(&self, size: Vec2) -> bool {
        size.x >= self.min_width && size.x <= self.max_width &&
        size.y >= self.min_height && size.y <= self.max_height
    }
}

/// 布局结果
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutResult {
    pub position: Vec2,
    pub size: Vec2,
    pub content_size: Vec2, // 内容区域大小（排除padding）
}

impl LayoutResult {
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self {
            position,
            size,
            content_size: size,
        }
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, self.size.x, self.size.y)
    }

    pub fn content_bounds(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, self.content_size.x, self.content_size.y)
    }
}

/// 布局节点
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub widget_id: WidgetId,
    pub style: UIStyle,
    pub children: Vec<LayoutNode>,
    pub result: Option<LayoutResult>,
}

impl LayoutNode {
    pub fn new(widget_id: WidgetId, style: UIStyle) -> Self {
        Self {
            widget_id,
            style,
            children: Vec::new(),
            result: None,
        }
    }

    pub fn add_child(&mut self, child: LayoutNode) {
        self.children.push(child);
    }

    pub fn remove_child(&mut self, widget_id: WidgetId) -> bool {
        if let Some(pos) = self.children.iter().position(|child| child.widget_id == widget_id) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }
}

/// 弹性盒子项目信息
#[derive(Debug, Clone, Copy)]
struct FlexItem {
    pub main_size: f32,
    pub cross_size: f32,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: f32,
    pub margin_main_start: f32,
    pub margin_main_end: f32,
    pub margin_cross_start: f32,
    pub margin_cross_end: f32,
}

/// 布局引擎
pub struct LayoutEngine {
    layout_cache: HashMap<WidgetId, LayoutResult>,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            layout_cache: HashMap::new(),
        }
    }

    /// 计算布局
    pub fn compute_layout(&mut self, node: &mut LayoutNode, constraints: LayoutConstraints) -> LayoutResult {
        // 检查缓存
        if let Some(cached) = self.layout_cache.get(&node.widget_id) {
            if constraints.is_valid_size(cached.size) {
                node.result = Some(*cached);
                return *cached;
            }
        }

        let result = match node.style.display {
            Display::None => {
                LayoutResult::new(Vec2::ZERO, Vec2::ZERO)
            }
            Display::Block => {
                self.compute_block_layout(node, constraints)
            }
            Display::Inline => {
                self.compute_inline_layout(node, constraints)
            }
            Display::InlineBlock => {
                self.compute_inline_block_layout(node, constraints)
            }
            Display::Flex => {
                self.compute_flex_layout(node, constraints)
            }
            Display::Grid => {
                self.compute_grid_layout(node, constraints)
            }
        };

        // 缓存结果
        self.layout_cache.insert(node.widget_id, result);
        node.result = Some(result);
        result
    }

    /// 计算块级布局
    fn compute_block_layout(&mut self, node: &mut LayoutNode, constraints: LayoutConstraints) -> LayoutResult {
        let style = &node.style;
        
        // 计算边距和填充
        let margin = &style.margin;
        let padding = &style.padding;
        
        // 计算可用空间
        let available_width = constraints.max_width - margin.horizontal() - padding.horizontal();
        let available_height = constraints.max_height - margin.vertical() - padding.vertical();
        
        // 计算自身大小
        let mut width = style.width.unwrap_or(available_width);
        let mut height = style.height.unwrap_or(0.0);
        
        // 应用min/max约束
        if let Some(min_width) = style.min_width {
            width = width.max(min_width);
        }
        if let Some(max_width) = style.max_width {
            width = width.min(max_width);
        }
        if let Some(min_height) = style.min_height {
            height = height.max(min_height);
        }
        if let Some(max_height) = style.max_height {
            height = height.min(max_height);
        }

        // 布局子元素
        let mut y_offset = padding.top;
        let content_width = width - padding.horizontal();
        
        for child in &mut node.children {
            if child.style.display == Display::None {
                continue;
            }

            let child_constraints = LayoutConstraints::new(
                0.0,
                content_width,
                0.0,
                available_height - y_offset,
            );

            let child_result = self.compute_layout(child, child_constraints);
            
            // 设置子元素位置
            if let Some(ref mut result) = child.result {
                result.position.x = padding.left + child.style.margin.left;
                result.position.y = y_offset + child.style.margin.top;
            }

            y_offset += child_result.size.y + child.style.margin.vertical();
        }

        // 如果高度未指定，根据内容计算
        if style.height.is_none() {
            height = y_offset + padding.bottom;
        }

        let total_width = width + margin.horizontal();
        let total_height = height + margin.vertical();

        LayoutResult {
            position: Vec2::new(margin.left, margin.top),
            size: Vec2::new(total_width, total_height),
            content_size: Vec2::new(width - padding.horizontal(), height - padding.vertical()),
        }
    }

    /// 计算内联布局
    fn compute_inline_layout(&mut self, node: &mut LayoutNode, constraints: LayoutConstraints) -> LayoutResult {
        // 简化实现：内联元素按照内容大小布局
        let style = &node.style;
        let width = style.width.unwrap_or(100.0);
        let height = style.height.unwrap_or(20.0);

        LayoutResult::new(Vec2::ZERO, Vec2::new(width, height))
    }

    /// 计算内联块级布局
    fn compute_inline_block_layout(&mut self, node: &mut LayoutNode, constraints: LayoutConstraints) -> LayoutResult {
        // 类似块级布局，但不独占一行
        self.compute_block_layout(node, constraints)
    }

    /// 计算弹性盒子布局
    fn compute_flex_layout(&mut self, node: &mut LayoutNode, constraints: LayoutConstraints) -> LayoutResult {
        let style = &node.style;
        let padding = &style.padding;
        let margin = &style.margin;

        // 计算容器大小
        let container_width = style.width.unwrap_or(constraints.max_width - margin.horizontal());
        let container_height = style.height.unwrap_or(constraints.max_height - margin.vertical());
        
        let content_width = container_width - padding.horizontal();
        let content_height = container_height - padding.vertical();

        // 确定主轴和交叉轴
        let is_row = matches!(style.flex_direction, FlexDirection::Row | FlexDirection::RowReverse);
        let main_size = if is_row { content_width } else { content_height };
        let cross_size = if is_row { content_height } else { content_width };

        // 收集弹性项目信息
        let mut flex_items = Vec::new();
        let mut total_flex_grow = 0.0;
        let mut total_flex_shrink = 0.0;
        let mut total_main_size = 0.0;

        for child in &mut node.children {
            if child.style.display == Display::None {
                continue;
            }

            let child_constraints = LayoutConstraints::new(0.0, content_width, 0.0, content_height);
            let child_result = self.compute_layout(child, child_constraints);

            let flex_basis = child.style.flex_basis.unwrap_or(
                if is_row { child_result.size.x } else { child_result.size.y }
            );

            let item = FlexItem {
                main_size: flex_basis,
                cross_size: if is_row { child_result.size.y } else { child_result.size.x },
                flex_grow: child.style.flex_grow,
                flex_shrink: child.style.flex_shrink,
                flex_basis,
                margin_main_start: if is_row { child.style.margin.left } else { child.style.margin.top },
                margin_main_end: if is_row { child.style.margin.right } else { child.style.margin.bottom },
                margin_cross_start: if is_row { child.style.margin.top } else { child.style.margin.left },
                margin_cross_end: if is_row { child.style.margin.bottom } else { child.style.margin.right },
            };

            total_flex_grow += item.flex_grow;
            total_flex_shrink += item.flex_shrink;
            total_main_size += item.main_size + item.margin_main_start + item.margin_main_end;

            flex_items.push(item);
        }

        // 计算剩余空间并分配
        let free_space = main_size - total_main_size;
        
        if free_space > 0.0 && total_flex_grow > 0.0 {
            // 分配剩余空间给可增长的项目
            let grow_factor = free_space / total_flex_grow;
            for (i, item) in flex_items.iter_mut().enumerate() {
                item.main_size += item.flex_grow * grow_factor;
            }
        } else if free_space < 0.0 && total_flex_shrink > 0.0 {
            // 收缩超出的项目
            let shrink_factor = -free_space / total_flex_shrink;
            for item in flex_items.iter_mut() {
                let shrink_amount = item.flex_shrink * shrink_factor;
                item.main_size = (item.main_size - shrink_amount).max(0.0);
            }
        }

        // 主轴对齐
        let mut main_position = padding.left;
        match style.justify_content {
            JustifyContent::FlexStart => {
                main_position = if is_row { padding.left } else { padding.top };
            }
            JustifyContent::FlexEnd => {
                let used_space: f32 = flex_items.iter().map(|item| 
                    item.main_size + item.margin_main_start + item.margin_main_end
                ).sum();
                main_position = main_size - used_space;
            }
            JustifyContent::Center => {
                let used_space: f32 = flex_items.iter().map(|item| 
                    item.main_size + item.margin_main_start + item.margin_main_end
                ).sum();
                main_position = (main_size - used_space) * 0.5;
            }
            JustifyContent::SpaceBetween => {
                main_position = 0.0;
            }
            JustifyContent::SpaceAround => {
                let used_space: f32 = flex_items.iter().map(|item| 
                    item.main_size + item.margin_main_start + item.margin_main_end
                ).sum();
                let space_per_item = (main_size - used_space) / flex_items.len() as f32;
                main_position = space_per_item * 0.5;
            }
            JustifyContent::SpaceEvenly => {
                let used_space: f32 = flex_items.iter().map(|item| 
                    item.main_size + item.margin_main_start + item.margin_main_end
                ).sum();
                let space_per_gap = (main_size - used_space) / (flex_items.len() + 1) as f32;
                main_position = space_per_gap;
            }
        }

        // 布局子元素
        for (i, child) in node.children.iter_mut().enumerate() {
            if child.style.display == Display::None {
                continue;
            }

            let item = &flex_items[i];
            
            // 主轴位置
            main_position += item.margin_main_start;

            // 交叉轴位置
            let cross_position = match style.align_items {
                AlignItems::FlexStart => {
                    item.margin_cross_start
                }
                AlignItems::FlexEnd => {
                    cross_size - item.cross_size - item.margin_cross_end
                }
                AlignItems::Center => {
                    (cross_size - item.cross_size) * 0.5
                }
                AlignItems::Baseline => {
                    // 简化实现：等同于FlexStart
                    item.margin_cross_start
                }
                AlignItems::Stretch => {
                    item.margin_cross_start
                }
            };

            // 设置子元素位置和大小
            if let Some(ref mut result) = child.result {
                if is_row {
                    result.position.x = main_position;
                    result.position.y = cross_position + padding.top;
                    result.size.x = item.main_size;
                    if matches!(style.align_items, AlignItems::Stretch) {
                        result.size.y = cross_size - item.margin_cross_start - item.margin_cross_end;
                    }
                } else {
                    result.position.x = cross_position + padding.left;
                    result.position.y = main_position;
                    result.size.y = item.main_size;
                    if matches!(style.align_items, AlignItems::Stretch) {
                        result.size.x = cross_size - item.margin_cross_start - item.margin_cross_end;
                    }
                }
            }

            main_position += item.main_size + item.margin_main_end;

            // SpaceBetween间距
            if matches!(style.justify_content, JustifyContent::SpaceBetween) && i < flex_items.len() - 1 {
                let used_space: f32 = flex_items.iter().map(|item| 
                    item.main_size + item.margin_main_start + item.margin_main_end
                ).sum();
                let gap = (main_size - used_space) / (flex_items.len() - 1) as f32;
                main_position += gap;
            }
        }

        LayoutResult {
            position: Vec2::new(margin.left, margin.top),
            size: Vec2::new(container_width + margin.horizontal(), container_height + margin.vertical()),
            content_size: Vec2::new(content_width, content_height),
        }
    }

    /// 计算网格布局
    fn compute_grid_layout(&mut self, node: &mut LayoutNode, constraints: LayoutConstraints) -> LayoutResult {
        // 简化实现：网格布局较复杂，这里提供基础框架
        let style = &node.style;
        let width = style.width.unwrap_or(constraints.max_width);
        let height = style.height.unwrap_or(constraints.max_height);

        // TODO: 实现真正的网格布局算法
        // 目前只是简单地按行排列子元素

        let padding = &style.padding;
        let mut y_offset = padding.top;
        let content_width = width - padding.horizontal();

        for child in &mut node.children {
            if child.style.display == Display::None {
                continue;
            }

            let child_constraints = LayoutConstraints::new(0.0, content_width, 0.0, f32::INFINITY);
            let child_result = self.compute_layout(child, child_constraints);

            if let Some(ref mut result) = child.result {
                result.position.x = padding.left;
                result.position.y = y_offset;
            }

            y_offset += child_result.size.y;
        }

        LayoutResult::new(Vec2::ZERO, Vec2::new(width, height))
    }

    /// 清除布局缓存
    pub fn clear_cache(&mut self) {
        self.layout_cache.clear();
    }

    /// 清除特定组件的布局缓存
    pub fn invalidate_layout(&mut self, widget_id: WidgetId) {
        self.layout_cache.remove(&widget_id);
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 布局管理器
pub struct LayoutManager {
    engine: LayoutEngine,
    root_nodes: Vec<LayoutNode>,
    viewport_size: Vec2,
}

impl LayoutManager {
    pub fn new() -> Self {
        Self {
            engine: LayoutEngine::new(),
            root_nodes: Vec::new(),
            viewport_size: Vec2::new(1920.0, 1080.0),
        }
    }

    /// 设置视口大小
    pub fn set_viewport_size(&mut self, size: Vec2) {
        self.viewport_size = size;
        self.engine.clear_cache();
    }

    /// 添加根节点
    pub fn add_root_node(&mut self, node: LayoutNode) {
        self.root_nodes.push(node);
    }

    /// 移除根节点
    pub fn remove_root_node(&mut self, widget_id: WidgetId) -> bool {
        if let Some(pos) = self.root_nodes.iter().position(|node| node.widget_id == widget_id) {
            self.root_nodes.remove(pos);
            true
        } else {
            false
        }
    }

    /// 查找节点
    pub fn find_node(&mut self, widget_id: WidgetId) -> Option<&mut LayoutNode> {
        for root in &mut self.root_nodes {
            if let Some(node) = Self::find_node_recursive(root, widget_id) {
                return Some(node);
            }
        }
        None
    }

    fn find_node_recursive(node: &mut LayoutNode, widget_id: WidgetId) -> Option<&mut LayoutNode> {
        if node.widget_id == widget_id {
            return Some(node);
        }

        for child in &mut node.children {
            if let Some(found) = Self::find_node_recursive(child, widget_id) {
                return Some(found);
            }
        }

        None
    }

    /// 更新布局
    pub fn update_layout(&mut self) {
        let constraints = LayoutConstraints::new(0.0, self.viewport_size.x, 0.0, self.viewport_size.y);
        
        for root in &mut self.root_nodes {
            self.engine.compute_layout(root, constraints);
        }
    }

    /// 获取组件布局结果
    pub fn get_layout_result(&self, widget_id: WidgetId) -> Option<LayoutResult> {
        for root in &self.root_nodes {
            if let Some(result) = Self::get_layout_result_recursive(root, widget_id) {
                return Some(result);
            }
        }
        None
    }

    fn get_layout_result_recursive(node: &LayoutNode, widget_id: WidgetId) -> Option<LayoutResult> {
        if node.widget_id == widget_id {
            return node.result;
        }

        for child in &node.children {
            if let Some(result) = Self::get_layout_result_recursive(child, widget_id) {
                return Some(result);
            }
        }

        None
    }

    /// 无效化布局
    pub fn invalidate_layout(&mut self, widget_id: WidgetId) {
        self.engine.invalidate_layout(widget_id);
    }

    /// 清除所有布局缓存
    pub fn clear_cache(&mut self) {
        self.engine.clear_cache();
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 布局工具函数
pub mod layout_utils {
    use super::*;

    /// 计算文本大小
    pub fn measure_text(text: &str, font: &crate::ui::style::FontStyle) -> Vec2 {
        // 简化实现：基于字体大小估算
        let char_width = font.size * 0.6; // 大致的字符宽度
        let line_height = font.size * font.line_height;
        
        let lines: Vec<&str> = text.lines().collect();
        let max_width = lines.iter()
            .map(|line| line.len() as f32 * char_width)
            .fold(0.0, f32::max);
        let height = lines.len() as f32 * line_height;

        Vec2::new(max_width, height)
    }

    /// 计算内容区域
    pub fn content_rect(bounds: Rect, padding: &crate::ui::style::Rect) -> Rect {
        Rect::new(
            bounds.x + padding.left,
            bounds.y + padding.top,
            bounds.width - padding.horizontal(),
            bounds.height - padding.vertical(),
        )
    }

    /// 在容器中居中对齐
    pub fn center_in_rect(content_size: Vec2, container: Rect) -> Vec2 {
        Vec2::new(
            container.x + (container.width - content_size.x) * 0.5,
            container.y + (container.height - content_size.y) * 0.5,
        )
    }

    /// 应用对齐方式
    pub fn apply_alignment(
        content_size: Vec2,
        container: Rect,
        horizontal_align: f32,  // 0.0 = 左对齐, 0.5 = 居中, 1.0 = 右对齐
        vertical_align: f32,    // 0.0 = 上对齐, 0.5 = 居中, 1.0 = 下对齐
    ) -> Vec2 {
        Vec2::new(
            container.x + (container.width - content_size.x) * horizontal_align,
            container.y + (container.height - content_size.y) * vertical_align,
        )
    }
}
