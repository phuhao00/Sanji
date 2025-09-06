//! 事件系统

use std::collections::{HashMap, VecDeque};
use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};

/// 事件trait - 所有事件都必须实现此trait
pub trait Event: Any + Send + Sync {
    /// 获取事件名称
    fn event_name(&self) -> &'static str;
}

/// 事件处理器trait
pub trait EventHandler<T: Event>: Send + Sync {
    /// 处理事件
    fn handle(&mut self, event: &T);
}

/// 函数式事件处理器
pub struct FunctionHandler<T: Event> {
    handler: Box<dyn Fn(&T) + Send + Sync>,
}

impl<T: Event> FunctionHandler<T> {
    pub fn new<F>(handler: F) -> Self 
    where 
        F: Fn(&T) + Send + Sync + 'static,
    {
        Self {
            handler: Box::new(handler),
        }
    }
}

impl<T: Event> EventHandler<T> for FunctionHandler<T> {
    fn handle(&mut self, event: &T) {
        (self.handler)(event);
    }
}

/// 事件监听器
type EventListener = Box<dyn Fn(&dyn Any) + Send + Sync>;

/// 事件系统
pub struct EventSystem {
    /// 事件监听器
    listeners: HashMap<TypeId, Vec<EventListener>>,
    /// 事件队列
    event_queue: Arc<Mutex<VecDeque<Box<dyn Any + Send + Sync>>>>,
    /// 是否启用即时模式
    immediate_mode: bool,
}

impl EventSystem {
    /// 创建新的事件系统
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
            immediate_mode: false,
        }
    }

    /// 设置即时模式
    pub fn set_immediate_mode(&mut self, immediate: bool) {
        self.immediate_mode = immediate;
    }

    /// 订阅事件
    pub fn subscribe<T: Event + 'static, F>(&mut self, handler: F)
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let listener = Box::new(move |event: &dyn Any| {
            if let Some(typed_event) = event.downcast_ref::<T>() {
                handler(typed_event);
            }
        });
        
        self.listeners.entry(type_id).or_insert_with(Vec::new).push(listener);
    }

    /// 发布事件
    pub fn publish<T: Event + 'static>(&mut self, event: T) {
        if self.immediate_mode {
            self.handle_event_immediate(&event);
        } else {
            let mut queue = self.event_queue.lock().unwrap();
            queue.push_back(Box::new(event));
        }
    }

    /// 立即处理事件
    fn handle_event_immediate<T: Event + 'static>(&self, event: &T) {
        let type_id = TypeId::of::<T>();
        if let Some(listeners) = self.listeners.get(&type_id) {
            for listener in listeners {
                listener(event);
            }
        }
    }

    /// 处理事件队列
    pub fn process_events(&mut self) {
        let events: Vec<Box<dyn Any + Send + Sync>> = {
            let mut queue = self.event_queue.lock().unwrap();
            queue.drain(..).collect()
        };

        for event in events {
            // 获取事件类型ID
            let type_id = (*event).type_id();
            
            // 调用对应的监听器
            if let Some(listeners) = self.listeners.get(&type_id) {
                for listener in listeners {
                    listener(event.as_ref());
                }
            }
        }
    }

    /// 清空事件队列
    pub fn clear_queue(&mut self) {
        let mut queue = self.event_queue.lock().unwrap();
        queue.clear();
    }

    /// 获取队列中的事件数量
    pub fn queue_size(&self) -> usize {
        let queue = self.event_queue.lock().unwrap();
        queue.len()
    }

    /// 取消订阅所有事件
    pub fn unsubscribe_all<T: Event + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.listeners.remove(&type_id);
    }

    /// 清空所有监听器
    pub fn clear_listeners(&mut self) {
        self.listeners.clear();
    }
}

impl Default for EventSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// 常见的引擎事件

/// 窗口事件
#[derive(Debug, Clone)]
pub struct WindowResizedEvent {
    pub width: u32,
    pub height: u32,
}

impl Event for WindowResizedEvent {
    fn event_name(&self) -> &'static str {
        "WindowResized"
    }
}

#[derive(Debug, Clone)]
pub struct WindowClosedEvent;

impl Event for WindowClosedEvent {
    fn event_name(&self) -> &'static str {
        "WindowClosed"
    }
}

/// 输入事件
#[derive(Debug, Clone)]
pub struct KeyPressedEvent {
    pub key_code: winit::keyboard::KeyCode,
    pub repeat: bool,
}

impl Event for KeyPressedEvent {
    fn event_name(&self) -> &'static str {
        "KeyPressed"
    }
}

#[derive(Debug, Clone)]
pub struct KeyReleasedEvent {
    pub key_code: winit::keyboard::KeyCode,
}

impl Event for KeyReleasedEvent {
    fn event_name(&self) -> &'static str {
        "KeyReleased"
    }
}

#[derive(Debug, Clone)]
pub struct MouseButtonPressedEvent {
    pub button: winit::event::MouseButton,
    pub position: glam::Vec2,
}

impl Event for MouseButtonPressedEvent {
    fn event_name(&self) -> &'static str {
        "MouseButtonPressed"
    }
}

#[derive(Debug, Clone)]
pub struct MouseButtonReleasedEvent {
    pub button: winit::event::MouseButton,
    pub position: glam::Vec2,
}

impl Event for MouseButtonReleasedEvent {
    fn event_name(&self) -> &'static str {
        "MouseButtonReleased"
    }
}

#[derive(Debug, Clone)]
pub struct MouseMovedEvent {
    pub position: glam::Vec2,
    pub delta: glam::Vec2,
}

impl Event for MouseMovedEvent {
    fn event_name(&self) -> &'static str {
        "MouseMoved"
    }
}

/// 场景事件
#[derive(Debug, Clone)]
pub struct SceneLoadedEvent {
    pub scene_name: String,
}

impl Event for SceneLoadedEvent {
    fn event_name(&self) -> &'static str {
        "SceneLoaded"
    }
}

#[derive(Debug, Clone)]
pub struct SceneUnloadedEvent {
    pub scene_name: String,
}

impl Event for SceneUnloadedEvent {
    fn event_name(&self) -> &'static str {
        "SceneUnloaded"
    }
}

/// 资源事件
#[derive(Debug, Clone)]
pub struct AssetLoadedEvent {
    pub asset_path: String,
    pub asset_type: String,
}

impl Event for AssetLoadedEvent {
    fn event_name(&self) -> &'static str {
        "AssetLoaded"
    }
}

#[derive(Debug, Clone)]
pub struct AssetLoadFailedEvent {
    pub asset_path: String,
    pub error: String,
}

impl Event for AssetLoadFailedEvent {
    fn event_name(&self) -> &'static str {
        "AssetLoadFailed"
    }
}

/// 事件发送器 - 线程安全的事件发送接口
#[derive(Clone)]
pub struct EventSender {
    event_queue: Arc<Mutex<VecDeque<Box<dyn Any + Send + Sync>>>>,
}

impl EventSender {
    pub fn new(event_queue: Arc<Mutex<VecDeque<Box<dyn Any + Send + Sync>>>>) -> Self {
        Self { event_queue }
    }

    /// 发送事件
    pub fn send<T: Event + 'static>(&self, event: T) {
        let mut queue = self.event_queue.lock().unwrap();
        queue.push_back(Box::new(event));
    }
}

/// 事件系统扩展，提供便捷方法
impl EventSystem {
    /// 获取事件发送器
    pub fn sender(&self) -> EventSender {
        EventSender::new(Arc::clone(&self.event_queue))
    }

    /// 发布窗口调整大小事件
    pub fn publish_window_resized(&mut self, width: u32, height: u32) {
        self.publish(WindowResizedEvent { width, height });
    }

    /// 发布窗口关闭事件
    pub fn publish_window_closed(&mut self) {
        self.publish(WindowClosedEvent);
    }

    /// 发布按键事件
    pub fn publish_key_pressed(&mut self, key_code: winit::keyboard::KeyCode, repeat: bool) {
        self.publish(KeyPressedEvent { key_code, repeat });
    }

    /// 发布鼠标事件
    pub fn publish_mouse_moved(&mut self, position: glam::Vec2, delta: glam::Vec2) {
        self.publish(MouseMovedEvent { position, delta });
    }
}