//! UI事件系统

use crate::math::Vec2;
use specs::Entity;
use serde::{Deserialize, Serialize};

/// UI事件类型
#[derive(Debug, Clone, PartialEq)]
pub enum UIEvent {
    /// 鼠标事件
    Mouse(MouseUIEvent),
    /// 键盘事件
    Keyboard(KeyboardUIEvent),
    /// 触摸事件
    Touch(TouchUIEvent),
    /// 焦点事件
    Focus(FocusUIEvent),
    /// 布局事件
    Layout(LayoutUIEvent),
    /// 自定义事件
    Custom(CustomUIEvent),
}

/// 鼠标UI事件
#[derive(Debug, Clone, PartialEq)]
pub struct MouseUIEvent {
    /// 事件类型
    pub event_type: MouseUIEventType,
    /// 鼠标位置
    pub position: Vec2,
    /// 鼠标按键
    pub button: Option<MouseButton>,
    /// 修饰键状态
    pub modifiers: KeyModifiers,
    /// 目标UI元素
    pub target: Option<Entity>,
    /// 点击次数（用于双击检测）
    pub click_count: u32,
}

/// 鼠标事件类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseUIEventType {
    /// 鼠标按下
    MouseDown,
    /// 鼠标释放
    MouseUp,
    /// 鼠标移动
    MouseMove,
    /// 鼠标进入
    MouseEnter,
    /// 鼠标离开
    MouseLeave,
    /// 鼠标悬停
    MouseOver,
    /// 滚轮滚动
    Scroll,
    /// 拖拽开始
    DragStart,
    /// 拖拽中
    Drag,
    /// 拖拽结束
    DragEnd,
    /// 点击
    Click,
    /// 双击
    DoubleClick,
}

/// 鼠标按键
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

/// 键盘UI事件
#[derive(Debug, Clone, PartialEq)]
pub struct KeyboardUIEvent {
    /// 事件类型
    pub event_type: KeyboardUIEventType,
    /// 按键代码
    pub key_code: KeyCode,
    /// 字符输入
    pub character: Option<char>,
    /// 修饰键状态
    pub modifiers: KeyModifiers,
    /// 目标UI元素
    pub target: Option<Entity>,
}

/// 键盘事件类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyboardUIEventType {
    /// 键盘按下
    KeyDown,
    /// 键盘释放
    KeyUp,
    /// 字符输入
    CharInput,
}

/// 按键代码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Escape,
    Tab,
    Enter,
    Space,
    Backspace,
    Delete,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
    PageUp,
    PageDown,
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Digit0, Digit1, Digit2, Digit3, Digit4,
    Digit5, Digit6, Digit7, Digit8, Digit9,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Unknown,
}

/// 修饰键状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool, // Windows键或Cmd键
}

/// 触摸UI事件
#[derive(Debug, Clone, PartialEq)]
pub struct TouchUIEvent {
    pub event_type: TouchUIEventType,
    pub touches: Vec<TouchPoint>,
    pub target: Option<Entity>,
}

/// 触摸事件类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TouchUIEventType {
    TouchStart,
    TouchMove,
    TouchEnd,
    TouchCancel,
}

/// 触摸点
#[derive(Debug, Clone, PartialEq)]
pub struct TouchPoint {
    pub id: u64,
    pub position: Vec2,
    pub pressure: f32,
}

/// 焦点事件
#[derive(Debug, Clone, PartialEq)]
pub struct FocusUIEvent {
    pub event_type: FocusUIEventType,
    pub target: Entity,
    pub related_target: Option<Entity>, // 失去或获得焦点的相关元素
}

/// 焦点事件类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusUIEventType {
    /// 获得焦点
    Focus,
    /// 失去焦点
    Blur,
    /// 焦点进入（包括子元素）
    FocusIn,
    /// 焦点离开（包括子元素）
    FocusOut,
}

/// 布局事件
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutUIEvent {
    pub event_type: LayoutUIEventType,
    pub target: Entity,
    pub old_size: Option<Vec2>,
    pub new_size: Option<Vec2>,
}

/// 布局事件类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutUIEventType {
    /// 大小改变
    Resize,
    /// 位置改变
    Move,
    /// 布局无效化
    InvalidateLayout,
    /// 布局更新
    LayoutUpdate,
}

/// 自定义事件
#[derive(Debug, Clone, PartialEq)]
pub struct CustomUIEvent {
    pub event_name: String,
    pub target: Entity,
    pub data: Option<String>, // 简化的数据传递，实际可以用更复杂的类型
}

/// UI事件监听器
pub trait UIEventListener {
    /// 处理UI事件
    fn handle_event(&mut self, event: &UIEvent) -> bool; // 返回true表示事件被消费
}

/// 函数式事件监听器
pub struct FunctionUIEventListener<F> 
where 
    F: Fn(&UIEvent) -> bool + Send + Sync,
{
    handler: F,
}

impl<F> FunctionUIEventListener<F> 
where 
    F: Fn(&UIEvent) -> bool + Send + Sync,
{
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<F> UIEventListener for FunctionUIEventListener<F>
where 
    F: Fn(&UIEvent) -> bool + Send + Sync,
{
    fn handle_event(&mut self, event: &UIEvent) -> bool {
        (self.handler)(event)
    }
}

/// UI事件管理器
pub struct UIEventManager {
    /// 事件监听器
    listeners: Vec<Box<dyn UIEventListener + Send + Sync>>,
    /// 事件队列
    event_queue: Vec<UIEvent>,
    /// 当前焦点元素
    focused_element: Option<Entity>,
    /// 当前悬停元素
    hovered_element: Option<Entity>,
    /// 拖拽状态
    drag_state: Option<DragState>,
    /// 双击检测
    last_click: Option<(Entity, std::time::Instant, Vec2)>,
    /// 双击时间阈值（毫秒）
    double_click_time: u64,
    /// 双击距离阈值
    double_click_distance: f32,
}

/// 拖拽状态
#[derive(Debug, Clone)]
struct DragState {
    target: Entity,
    start_position: Vec2,
    current_position: Vec2,
    button: MouseButton,
}

impl UIEventManager {
    /// 创建新的UI事件管理器
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
            event_queue: Vec::new(),
            focused_element: None,
            hovered_element: None,
            drag_state: None,
            last_click: None,
            double_click_time: 500, // 500ms
            double_click_distance: 5.0, // 5像素
        }
    }

    /// 添加事件监听器
    pub fn add_listener<L>(&mut self, listener: L) 
    where 
        L: UIEventListener + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(listener));
    }

    /// 添加函数式监听器
    pub fn add_function_listener<F>(&mut self, handler: F)
    where 
        F: Fn(&UIEvent) -> bool + Send + Sync + 'static,
    {
        self.add_listener(FunctionUIEventListener::new(handler));
    }

    /// 发布事件
    pub fn emit_event(&mut self, event: UIEvent) {
        self.event_queue.push(event);
    }

    /// 处理鼠标输入
    pub fn handle_mouse_input(&mut self, event_type: MouseUIEventType, position: Vec2, button: Option<MouseButton>) {
        // 这里应该进行命中测试以确定目标元素
        let target = self.hit_test(position);
        
        // 处理悬停状态
        if event_type == MouseUIEventType::MouseMove {
            if self.hovered_element != target {
                // 发送鼠标离开事件
                if let Some(old_target) = self.hovered_element {
                    self.emit_event(UIEvent::Mouse(MouseUIEvent {
                        event_type: MouseUIEventType::MouseLeave,
                        position,
                        button: None,
                        modifiers: KeyModifiers::default(),
                        target: Some(old_target),
                        click_count: 0,
                    }));
                }
                
                // 发送鼠标进入事件
                if let Some(new_target) = target {
                    self.emit_event(UIEvent::Mouse(MouseUIEvent {
                        event_type: MouseUIEventType::MouseEnter,
                        position,
                        button: None,
                        modifiers: KeyModifiers::default(),
                        target: Some(new_target),
                        click_count: 0,
                    }));
                }
                
                self.hovered_element = target;
            }
        }
        
        // 处理拖拽
        match event_type {
            MouseUIEventType::MouseDown if button.is_some() => {
                if let Some(target) = target {
                    self.drag_state = Some(DragState {
                        target,
                        start_position: position,
                        current_position: position,
                        button: button.unwrap(),
                    });
                }
            }
            MouseUIEventType::MouseMove => {
                if let Some(ref mut drag_state) = self.drag_state {
                    drag_state.current_position = position;
                    
                    // 发送拖拽事件
                    let drag_distance = (position - drag_state.start_position).length();
                    if drag_distance > 3.0 { // 3像素的拖拽阈值
                        self.emit_event(UIEvent::Mouse(MouseUIEvent {
                            event_type: MouseUIEventType::Drag,
                            position,
                            button: Some(drag_state.button),
                            modifiers: KeyModifiers::default(),
                            target: Some(drag_state.target),
                            click_count: 0,
                        }));
                    }
                }
            }
            MouseUIEventType::MouseUp => {
                if let Some(drag_state) = self.drag_state.take() {
                    let drag_distance = (position - drag_state.start_position).length();
                    
                    if drag_distance <= 3.0 {
                        // 这是一个点击，不是拖拽
                        let click_count = self.detect_double_click(drag_state.target, position);
                        
                        self.emit_event(UIEvent::Mouse(MouseUIEvent {
                            event_type: if click_count > 1 { MouseUIEventType::DoubleClick } else { MouseUIEventType::Click },
                            position,
                            button: Some(drag_state.button),
                            modifiers: KeyModifiers::default(),
                            target: Some(drag_state.target),
                            click_count,
                        }));
                    } else {
                        // 发送拖拽结束事件
                        self.emit_event(UIEvent::Mouse(MouseUIEvent {
                            event_type: MouseUIEventType::DragEnd,
                            position,
                            button: Some(drag_state.button),
                            modifiers: KeyModifiers::default(),
                            target: Some(drag_state.target),
                            click_count: 0,
                        }));
                    }
                }
            }
            _ => {}
        }
        
        // 发送原始鼠标事件
        self.emit_event(UIEvent::Mouse(MouseUIEvent {
            event_type,
            position,
            button,
            modifiers: KeyModifiers::default(), // 这里应该从实际输入系统获取
            target,
            click_count: 0,
        }));
    }

    /// 处理键盘输入
    pub fn handle_keyboard_input(&mut self, event_type: KeyboardUIEventType, key_code: KeyCode, character: Option<char>) {
        self.emit_event(UIEvent::Keyboard(KeyboardUIEvent {
            event_type,
            key_code,
            character,
            modifiers: KeyModifiers::default(),
            target: self.focused_element,
        }));
    }

    /// 设置焦点
    pub fn set_focus(&mut self, element: Option<Entity>) {
        if self.focused_element != element {
            // 发送失去焦点事件
            if let Some(old_focus) = self.focused_element {
                self.emit_event(UIEvent::Focus(FocusUIEvent {
                    event_type: FocusUIEventType::Blur,
                    target: old_focus,
                    related_target: element,
                }));
            }
            
            // 发送获得焦点事件
            if let Some(new_focus) = element {
                self.emit_event(UIEvent::Focus(FocusUIEvent {
                    event_type: FocusUIEventType::Focus,
                    target: new_focus,
                    related_target: self.focused_element,
                }));
            }
            
            self.focused_element = element;
        }
    }

    /// 处理事件队列
    pub fn process_events(&mut self) {
        let events = std::mem::take(&mut self.event_queue);
        
        for event in events {
            let mut consumed = false;
            
            // 将事件分发给监听器
            for listener in &mut self.listeners {
                if listener.handle_event(&event) {
                    consumed = true;
                    break; // 事件被消费，停止分发
                }
            }
            
            // 如果事件没有被消费，可以在这里进行默认处理
            if !consumed {
                self.handle_default_event(&event);
            }
        }
    }

    /// 命中测试 - 确定鼠标位置下的UI元素
    fn hit_test(&self, position: Vec2) -> Option<Entity> {
        // 这里应该实现实际的命中测试逻辑
        // 需要与UI布局系统集成
        None
    }

    /// 检测双击
    fn detect_double_click(&mut self, target: Entity, position: Vec2) -> u32 {
        let now = std::time::Instant::now();
        
        if let Some((last_target, last_time, last_pos)) = self.last_click {
            if last_target == target 
                && now.duration_since(last_time).as_millis() < self.double_click_time as u128
                && (position - last_pos).length() < self.double_click_distance {
                self.last_click = None; // 重置双击检测
                return 2; // 双击
            }
        }
        
        self.last_click = Some((target, now, position));
        1 // 单击
    }

    /// 处理默认事件
    fn handle_default_event(&self, event: &UIEvent) {
        match event {
            UIEvent::Keyboard(keyboard_event) => {
                // 默认键盘处理（如Tab键焦点切换）
                if keyboard_event.key_code == KeyCode::Tab 
                    && keyboard_event.event_type == KeyboardUIEventType::KeyDown {
                    // 实现Tab键焦点切换逻辑
                }
            }
            _ => {}
        }
    }

    /// 清空事件队列
    pub fn clear_events(&mut self) {
        self.event_queue.clear();
    }

    /// 获取当前焦点元素
    pub fn focused_element(&self) -> Option<Entity> {
        self.focused_element
    }

    /// 获取当前悬停元素
    pub fn hovered_element(&self) -> Option<Entity> {
        self.hovered_element
    }

    /// 检查是否在拖拽中
    pub fn is_dragging(&self) -> bool {
        self.drag_state.is_some()
    }
}

impl Default for UIEventManager {
    fn default() -> Self {
        Self::new()
    }
}

/// UI事件宏，用于简化事件创建
#[macro_export]
macro_rules! ui_mouse_event {
    ($event_type:expr, $pos:expr) => {
        UIEvent::Mouse(MouseUIEvent {
            event_type: $event_type,
            position: $pos,
            button: None,
            modifiers: KeyModifiers::default(),
            target: None,
            click_count: 0,
        })
    };
    ($event_type:expr, $pos:expr, $button:expr) => {
        UIEvent::Mouse(MouseUIEvent {
            event_type: $event_type,
            position: $pos,
            button: Some($button),
            modifiers: KeyModifiers::default(),
            target: None,
            click_count: 0,
        })
    };
}

#[macro_export]
macro_rules! ui_keyboard_event {
    ($event_type:expr, $key:expr) => {
        UIEvent::Keyboard(KeyboardUIEvent {
            event_type: $event_type,
            key_code: $key,
            character: None,
            modifiers: KeyModifiers::default(),
            target: None,
        })
    };
}
