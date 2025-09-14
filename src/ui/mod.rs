//! UI系统模块

pub mod events;
pub mod style;
pub mod widgets;
pub mod layout;
pub mod renderer;

pub use events::*;
pub use style::*;
pub use widgets::*;
pub use layout::*;
pub use renderer::*;

/// UI系统主接口
pub struct UISystem {
    pub container: WidgetContainer,
    pub layout_manager: LayoutManager,
    pub render_context: UIRenderContext,
    pub event_dispatcher: events::UIEventManager,
}

impl UISystem {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            container: WidgetContainer::new(),
            layout_manager: LayoutManager::new(),
            render_context: UIRenderContext::new(screen_width, screen_height),
            event_dispatcher: events::UIEventManager::new(),
        }
    }

    /// 更新UI系统
    pub fn update(&mut self, delta_time: f32) {
        // 处理事件
        while let Some(event) = self.event_dispatcher.poll_event() {
            self.container.handle_event(&event);
        }

        // 更新组件
        self.container.update(delta_time);

        // 更新布局
        self.layout_manager.set_viewport_size(crate::math::Vec2::new(
            1024.0, // 默认宽度
            768.0   // 默认高度
        ));
        self.layout_manager.update_layout();
    }

    /// 渲染UI
    pub fn render(&mut self, render_system: &mut crate::render::RenderSystem) {
        self.render_context.begin_frame();
        self.container.render(&mut self.render_context.renderer);
        self.render_context.end_frame();
        self.render_context.render(render_system);
    }

    /// 设置屏幕大小
    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.render_context.renderer.set_screen_size(width, height);
        self.layout_manager.set_viewport_size(crate::math::Vec2::new(width, height));
    }

    /// 添加组件
    pub fn add_widget<W: Widget + 'static>(&mut self, widget: W) -> WidgetId {
        self.container.add_widget(widget)
    }

    /// 移除组件
    pub fn remove_widget(&mut self, id: WidgetId) -> bool {
        self.container.remove_widget(id)
    }

    /// 获取组件
    pub fn get_widget(&self, id: WidgetId) -> Option<&dyn Widget> {
        self.container.get_widget(id)
    }

    /// 获取可变组件
    pub fn get_widget_mut(&mut self, id: WidgetId) -> Option<&mut dyn Widget> {
        self.container.get_widget_mut(id)
    }

    /// 发送事件
    pub fn send_event(&mut self, event: UIEvent) {
        self.event_dispatcher.send_event(event);
    }
}
