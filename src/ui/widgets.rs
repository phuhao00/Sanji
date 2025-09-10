//! UI组件系统

use crate::math::{Vec2, Vec3};
use crate::ui::{UIStyle, UIEvent, Color};
use crate::input::{KeyCode, MouseButton};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 组件ID类型
pub type WidgetId = u64;

/// 组件状态
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WidgetState {
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

/// 基础组件特征
pub trait Widget {
    /// 获取组件ID
    fn id(&self) -> WidgetId;
    
    /// 获取组件位置和大小
    fn bounds(&self) -> Rect;
    
    /// 设置位置
    fn set_position(&mut self, position: Vec2);
    
    /// 设置大小
    fn set_size(&mut self, size: Vec2);
    
    /// 获取样式
    fn style(&self) -> &UIStyle;
    
    /// 设置样式
    fn set_style(&mut self, style: UIStyle);
    
    /// 获取状态
    fn state(&self) -> WidgetState;
    
    /// 设置状态
    fn set_state(&mut self, state: WidgetState);
    
    /// 是否可见
    fn is_visible(&self) -> bool;
    
    /// 设置可见性
    fn set_visible(&mut self, visible: bool);
    
    /// 是否启用
    fn is_enabled(&self) -> bool;
    
    /// 设置启用状态
    fn set_enabled(&mut self, enabled: bool);
    
    /// 处理事件
    fn handle_event(&mut self, event: &UIEvent) -> bool;
    
    /// 更新组件
    fn update(&mut self, delta_time: f32);
    
    /// 渲染组件
    fn render(&self, renderer: &mut UIRenderer);
    
    /// 点击测试
    fn hit_test(&self, point: Vec2) -> bool {
        let bounds = self.bounds();
        point.x >= bounds.x && point.x <= bounds.x + bounds.width &&
        point.y >= bounds.y && point.y <= bounds.y + bounds.height
    }
}

/// 矩形区域
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x && point.x <= self.x + self.width &&
        point.y >= self.y && point.y <= self.y + self.height
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }
}

/// 基础组件数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseWidget {
    pub id: WidgetId,
    pub position: Vec2,
    pub size: Vec2,
    pub style: UIStyle,
    pub state: WidgetState,
    pub visible: bool,
    pub enabled: bool,
    pub parent: Option<WidgetId>,
    pub children: Vec<WidgetId>,
}

impl BaseWidget {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            position: Vec2::ZERO,
            size: Vec2::new(100.0, 30.0),
            style: UIStyle::default(),
            state: WidgetState::Normal,
            visible: true,
            enabled: true,
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, self.size.x, self.size.y)
    }
}

/// 文本组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextWidget {
    pub base: BaseWidget,
    pub text: String,
    pub word_wrap: bool,
    pub selectable: bool,
}

impl TextWidget {
    pub fn new(id: WidgetId, text: String) -> Self {
        let mut base = BaseWidget::new(id);
        base.style = crate::ui::style::StylePresets::label();
        
        Self {
            base,
            text,
            word_wrap: false,
            selectable: false,
        }
    }

    pub fn with_word_wrap(mut self, wrap: bool) -> Self {
        self.word_wrap = wrap;
        self
    }

    pub fn with_selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }
}

impl Widget for TextWidget {
    fn id(&self) -> WidgetId { self.base.id }
    fn bounds(&self) -> Rect { self.base.bounds() }
    fn set_position(&mut self, position: Vec2) { self.base.position = position; }
    fn set_size(&mut self, size: Vec2) { self.base.size = size; }
    fn style(&self) -> &UIStyle { &self.base.style }
    fn set_style(&mut self, style: UIStyle) { self.base.style = style; }
    fn state(&self) -> WidgetState { self.base.state }
    fn set_state(&mut self, state: WidgetState) { self.base.state = state; }
    fn is_visible(&self) -> bool { self.base.visible }
    fn set_visible(&mut self, visible: bool) { self.base.visible = visible; }
    fn is_enabled(&self) -> bool { self.base.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.base.enabled = enabled; }

    fn handle_event(&mut self, event: &UIEvent) -> bool {
        if !self.is_enabled() || !self.is_visible() {
            return false;
        }

        match event {
            UIEvent::MouseMove { position, .. } => {
                let was_hovered = self.state() == WidgetState::Hovered;
                let is_hovered = self.hit_test(*position);
                
                if is_hovered && !was_hovered {
                    self.set_state(WidgetState::Hovered);
                    return true;
                } else if !is_hovered && was_hovered {
                    self.set_state(WidgetState::Normal);
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    fn update(&mut self, _delta_time: f32) {
        // 文本组件通常不需要更新逻辑
    }

    fn render(&self, renderer: &mut UIRenderer) {
        if !self.is_visible() {
            return;
        }

        let bounds = self.bounds();
        
        // 渲染背景
        if self.style().background_color.a > 0.0 {
            renderer.draw_rect(bounds, self.style().background_color);
        }

        // 渲染边框
        if self.style().border.width > 0.0 {
            renderer.draw_border(bounds, &self.style().border);
        }

        // 渲染文本
        renderer.draw_text(&self.text, bounds, &self.style().font, self.style().text_color);
    }
}

/// 按钮组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonWidget {
    pub base: BaseWidget,
    pub text: String,
    pub icon: Option<String>, // 图标纹理路径
    pub on_click: Option<String>, // 回调函数名
}

impl ButtonWidget {
    pub fn new(id: WidgetId, text: String) -> Self {
        let mut base = BaseWidget::new(id);
        base.style = crate::ui::style::StylePresets::button();
        
        Self {
            base,
            text,
            icon: None,
            on_click: None,
        }
    }

    pub fn with_icon(mut self, icon_path: String) -> Self {
        self.icon = Some(icon_path);
        self
    }

    pub fn with_callback(mut self, callback: String) -> Self {
        self.on_click = Some(callback);
        self
    }
}

impl Widget for ButtonWidget {
    fn id(&self) -> WidgetId { self.base.id }
    fn bounds(&self) -> Rect { self.base.bounds() }
    fn set_position(&mut self, position: Vec2) { self.base.position = position; }
    fn set_size(&mut self, size: Vec2) { self.base.size = size; }
    fn style(&self) -> &UIStyle { &self.base.style }
    fn set_style(&mut self, style: UIStyle) { self.base.style = style; }
    fn state(&self) -> WidgetState { self.base.state }
    fn set_state(&mut self, state: WidgetState) { self.base.state = state; }
    fn is_visible(&self) -> bool { self.base.visible }
    fn set_visible(&mut self, visible: bool) { self.base.visible = visible; }
    fn is_enabled(&self) -> bool { self.base.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.base.enabled = enabled; }

    fn handle_event(&mut self, event: &UIEvent) -> bool {
        if !self.is_enabled() || !self.is_visible() {
            return false;
        }

        match event {
            UIEvent::MouseMove { position, .. } => {
                let was_hovered = self.state() == WidgetState::Hovered;
                let is_hovered = self.hit_test(*position);
                
                if is_hovered && !was_hovered && self.state() != WidgetState::Pressed {
                    self.set_state(WidgetState::Hovered);
                    return true;
                } else if !is_hovered && was_hovered {
                    self.set_state(WidgetState::Normal);
                    return true;
                }
            }
            UIEvent::MouseButtonDown { button: MouseButton::Left, position, .. } => {
                if self.hit_test(*position) {
                    self.set_state(WidgetState::Pressed);
                    return true;
                }
            }
            UIEvent::MouseButtonUp { button: MouseButton::Left, position, .. } => {
                if self.state() == WidgetState::Pressed {
                    self.set_state(if self.hit_test(*position) { WidgetState::Hovered } else { WidgetState::Normal });
                    
                    if self.hit_test(*position) {
                        // 触发点击事件
                        // TODO: 触发回调
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn update(&mut self, _delta_time: f32) {
        // 按钮可以在这里处理动画状态
    }

    fn render(&self, renderer: &mut UIRenderer) {
        if !self.is_visible() {
            return;
        }

        let bounds = self.bounds();
        let mut bg_color = self.style().background_color;

        // 根据状态调整外观
        match self.state() {
            WidgetState::Hovered => {
                bg_color = bg_color.mix(Color::WHITE, 0.1);
            }
            WidgetState::Pressed => {
                bg_color = bg_color.mix(Color::BLACK, 0.1);
            }
            WidgetState::Disabled => {
                bg_color = bg_color.with_alpha(0.5);
            }
            _ => {}
        }

        // 渲染背景
        renderer.draw_rect(bounds, bg_color);

        // 渲染边框
        if self.style().border.width > 0.0 {
            renderer.draw_border(bounds, &self.style().border);
        }

        // 渲染图标和文本
        if let Some(ref icon) = self.icon {
            // TODO: 渲染图标
            renderer.draw_icon(icon, bounds);
        }

        // 渲染文本
        renderer.draw_text(&self.text, bounds, &self.style().font, self.style().text_color);
    }
}

/// 输入框组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputWidget {
    pub base: BaseWidget,
    pub text: String,
    pub placeholder: String,
    pub password: bool,
    pub multiline: bool,
    pub cursor_position: usize,
    pub selection_start: usize,
    pub selection_end: usize,
    pub max_length: Option<usize>,
}

impl InputWidget {
    pub fn new(id: WidgetId) -> Self {
        let mut base = BaseWidget::new(id);
        base.style = crate::ui::style::StylePresets::input();
        
        Self {
            base,
            text: String::new(),
            placeholder: String::new(),
            password: false,
            multiline: false,
            cursor_position: 0,
            selection_start: 0,
            selection_end: 0,
            max_length: None,
        }
    }

    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn with_password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    pub fn with_multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    pub fn insert_text(&mut self, text: &str) {
        if let Some(max_len) = self.max_length {
            if self.text.len() + text.len() > max_len {
                return;
            }
        }

        self.text.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
        self.selection_start = self.cursor_position;
        self.selection_end = self.cursor_position;
    }

    pub fn delete_selection(&mut self) {
        if self.selection_start != self.selection_end {
            let start = self.selection_start.min(self.selection_end);
            let end = self.selection_start.max(self.selection_end);
            self.text.drain(start..end);
            self.cursor_position = start;
            self.selection_start = start;
            self.selection_end = start;
        }
    }

    pub fn backspace(&mut self) {
        if self.selection_start != self.selection_end {
            self.delete_selection();
        } else if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.text.remove(self.cursor_position);
            self.selection_start = self.cursor_position;
            self.selection_end = self.cursor_position;
        }
    }
}

impl Widget for InputWidget {
    fn id(&self) -> WidgetId { self.base.id }
    fn bounds(&self) -> Rect { self.base.bounds() }
    fn set_position(&mut self, position: Vec2) { self.base.position = position; }
    fn set_size(&mut self, size: Vec2) { self.base.size = size; }
    fn style(&self) -> &UIStyle { &self.base.style }
    fn set_style(&mut self, style: UIStyle) { self.base.style = style; }
    fn state(&self) -> WidgetState { self.base.state }
    fn set_state(&mut self, state: WidgetState) { self.base.state = state; }
    fn is_visible(&self) -> bool { self.base.visible }
    fn set_visible(&mut self, visible: bool) { self.base.visible = visible; }
    fn is_enabled(&self) -> bool { self.base.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.base.enabled = enabled; }

    fn handle_event(&mut self, event: &UIEvent) -> bool {
        if !self.is_enabled() || !self.is_visible() {
            return false;
        }

        match event {
            UIEvent::MouseButtonDown { button: MouseButton::Left, position, .. } => {
                if self.hit_test(*position) {
                    self.set_state(WidgetState::Focused);
                    // TODO: 计算光标位置
                    return true;
                } else if self.state() == WidgetState::Focused {
                    self.set_state(WidgetState::Normal);
                    return true;
                }
            }
            UIEvent::KeyDown { key, .. } => {
                if self.state() == WidgetState::Focused {
                    match key {
                        KeyCode::Backspace => {
                            self.backspace();
                            return true;
                        }
                        KeyCode::Delete => {
                            if self.cursor_position < self.text.len() {
                                self.text.remove(self.cursor_position);
                            }
                            return true;
                        }
                        KeyCode::Left => {
                            if self.cursor_position > 0 {
                                self.cursor_position -= 1;
                                self.selection_start = self.cursor_position;
                                self.selection_end = self.cursor_position;
                            }
                            return true;
                        }
                        KeyCode::Right => {
                            if self.cursor_position < self.text.len() {
                                self.cursor_position += 1;
                                self.selection_start = self.cursor_position;
                                self.selection_end = self.cursor_position;
                            }
                            return true;
                        }
                        _ => {}
                    }
                }
            }
            UIEvent::TextInput { text, .. } => {
                if self.state() == WidgetState::Focused {
                    self.insert_text(text);
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    fn update(&mut self, _delta_time: f32) {
        // 输入框可以在这里处理光标闪烁动画
    }

    fn render(&self, renderer: &mut UIRenderer) {
        if !self.is_visible() {
            return;
        }

        let bounds = self.bounds();
        let mut bg_color = self.style().background_color;
        let mut border_color = self.style().border.color;

        // 根据状态调整外观
        match self.state() {
            WidgetState::Focused => {
                border_color = Color::hex(0x007ACC);
            }
            WidgetState::Disabled => {
                bg_color = bg_color.mix(Color::BLACK, 0.1);
            }
            _ => {}
        }

        // 渲染背景
        renderer.draw_rect(bounds, bg_color);

        // 渲染边框
        let mut border_style = self.style().border;
        border_style.color = border_color;
        renderer.draw_border(bounds, &border_style);

        // 渲染文本或占位符
        let display_text = if self.text.is_empty() && !self.placeholder.is_empty() {
            &self.placeholder
        } else if self.password {
            &"*".repeat(self.text.len())
        } else {
            &self.text
        };

        let text_color = if self.text.is_empty() && !self.placeholder.is_empty() {
            Color::hex(0x999999)
        } else {
            self.style().text_color
        };

        renderer.draw_text(display_text, bounds, &self.style().font, text_color);

        // 渲染光标（如果聚焦）
        if self.state() == WidgetState::Focused {
            // TODO: 渲染光标和选择区域
        }
    }
}

/// 面板组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelWidget {
    pub base: BaseWidget,
    pub title: Option<String>,
    pub closable: bool,
    pub resizable: bool,
    pub draggable: bool,
}

impl PanelWidget {
    pub fn new(id: WidgetId) -> Self {
        let mut base = BaseWidget::new(id);
        base.style = crate::ui::style::StylePresets::panel();
        base.size = Vec2::new(300.0, 200.0);
        
        Self {
            base,
            title: None,
            closable: false,
            resizable: false,
            draggable: false,
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn with_draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }
}

impl Widget for PanelWidget {
    fn id(&self) -> WidgetId { self.base.id }
    fn bounds(&self) -> Rect { self.base.bounds() }
    fn set_position(&mut self, position: Vec2) { self.base.position = position; }
    fn set_size(&mut self, size: Vec2) { self.base.size = size; }
    fn style(&self) -> &UIStyle { &self.base.style }
    fn set_style(&mut self, style: UIStyle) { self.base.style = style; }
    fn state(&self) -> WidgetState { self.base.state }
    fn set_state(&mut self, state: WidgetState) { self.base.state = state; }
    fn is_visible(&self) -> bool { self.base.visible }
    fn set_visible(&mut self, visible: bool) { self.base.visible = visible; }
    fn is_enabled(&self) -> bool { self.base.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.base.enabled = enabled; }

    fn handle_event(&mut self, event: &UIEvent) -> bool {
        if !self.is_enabled() || !self.is_visible() {
            return false;
        }

        match event {
            UIEvent::MouseButtonDown { button: MouseButton::Left, position, .. } => {
                if self.hit_test(*position) {
                    // TODO: 处理拖拽和调整大小
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    fn update(&mut self, _delta_time: f32) {
        // 面板可以在这里处理动画
    }

    fn render(&self, renderer: &mut UIRenderer) {
        if !self.is_visible() {
            return;
        }

        let bounds = self.bounds();

        // 渲染背景
        renderer.draw_rect(bounds, self.style().background_color);

        // 渲染边框
        if self.style().border.width > 0.0 {
            renderer.draw_border(bounds, &self.style().border);
        }

        // 渲染标题栏
        if let Some(ref title) = self.title {
            let title_height = 30.0;
            let title_bounds = Rect::new(bounds.x, bounds.y, bounds.width, title_height);
            
            renderer.draw_rect(title_bounds, Color::hex(0xE0E0E0));
            renderer.draw_text(title, title_bounds, &self.style().font, Color::BLACK);

            // 渲染关闭按钮
            if self.closable {
                let close_size = 20.0;
                let close_bounds = Rect::new(
                    bounds.x + bounds.width - close_size - 5.0,
                    bounds.y + 5.0,
                    close_size,
                    close_size
                );
                renderer.draw_rect(close_bounds, Color::RED);
                renderer.draw_text("×", close_bounds, &self.style().font, Color::WHITE);
            }
        }
    }
}

/// UI渲染器接口
pub trait UIRenderer {
    fn draw_rect(&mut self, bounds: Rect, color: Color);
    fn draw_border(&mut self, bounds: Rect, border: &crate::ui::style::BorderStyle);
    fn draw_text(&mut self, text: &str, bounds: Rect, font: &crate::ui::style::FontStyle, color: Color);
    fn draw_icon(&mut self, icon_path: &str, bounds: Rect);
    fn draw_image(&mut self, image_path: &str, bounds: Rect);
}

/// 组件容器
#[derive(Debug)]
pub struct WidgetContainer {
    widgets: HashMap<WidgetId, Box<dyn Widget>>,
    root_widgets: Vec<WidgetId>,
    next_id: WidgetId,
}

impl WidgetContainer {
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
            root_widgets: Vec::new(),
            next_id: 1,
        }
    }

    pub fn generate_id(&mut self) -> WidgetId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_widget<W: Widget + 'static>(&mut self, widget: W) -> WidgetId {
        let id = widget.id();
        self.widgets.insert(id, Box::new(widget));
        self.root_widgets.push(id);
        id
    }

    pub fn remove_widget(&mut self, id: WidgetId) -> bool {
        if self.widgets.remove(&id).is_some() {
            self.root_widgets.retain(|&x| x != id);
            true
        } else {
            false
        }
    }

    pub fn get_widget(&self, id: WidgetId) -> Option<&dyn Widget> {
        self.widgets.get(&id).map(|w| w.as_ref())
    }

    pub fn get_widget_mut(&mut self, id: WidgetId) -> Option<&mut dyn Widget> {
        self.widgets.get_mut(&id).map(|w| w.as_mut())
    }

    pub fn handle_event(&mut self, event: &UIEvent) -> bool {
        for &id in &self.root_widgets {
            if let Some(widget) = self.widgets.get_mut(&id) {
                if widget.handle_event(event) {
                    return true;
                }
            }
        }
        false
    }

    pub fn update(&mut self, delta_time: f32) {
        for widget in self.widgets.values_mut() {
            widget.update(delta_time);
        }
    }

    pub fn render(&self, renderer: &mut dyn UIRenderer) {
        for &id in &self.root_widgets {
            if let Some(widget) = self.widgets.get(&id) {
                widget.render(renderer);
            }
        }
    }
}

impl Default for WidgetContainer {
    fn default() -> Self {
        Self::new()
    }
}
