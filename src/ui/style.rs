//! UI样式系统

use crate::math::Vec2;
use serde::{Deserialize, Serialize};

/// 颜色表示
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// 创建新颜色
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// 创建RGB颜色（不透明）
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    /// 创建RGBA颜色
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::new(r, g, b, a)
    }

    /// 从十六进制创建颜色
    pub fn hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as f32 / 255.0,
            g: ((hex >> 8) & 0xFF) as f32 / 255.0,
            b: (hex & 0xFF) as f32 / 255.0,
            a: 1.0,
        }
    }

    /// 从HSVA创建颜色
    pub fn hsva(h: f32, s: f32, v: f32, a: f32) -> Self {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match h {
            h if h < 60.0 => (c, x, 0.0),
            h if h < 120.0 => (x, c, 0.0),
            h if h < 180.0 => (0.0, c, x),
            h if h < 240.0 => (0.0, x, c),
            h if h < 300.0 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self::new(r + m, g + m, b + m, a)
    }

    /// 预定义颜色
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: Self = Self::new(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: Self = Self::new(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: Self = Self::new(1.0, 0.0, 1.0, 1.0);
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// 调整透明度
    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.a = alpha.clamp(0.0, 1.0);
        self
    }

    /// 颜色混合
    pub fn mix(self, other: Color, factor: f32) -> Self {
        let t = factor.clamp(0.0, 1.0);
        Self {
            r: self.r * (1.0 - t) + other.r * t,
            g: self.g * (1.0 - t) + other.g * t,
            b: self.b * (1.0 - t) + other.b * t,
            a: self.a * (1.0 - t) + other.a * t,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

/// 边距和填充
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Rect {
    /// 创建均匀边距
    pub fn all(value: f32) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    /// 创建水平和垂直边距
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }

    /// 创建自定义边距
    pub fn custom(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self { left, top, right, bottom }
    }

    /// 零边距
    pub const ZERO: Self = Self { left: 0.0, top: 0.0, right: 0.0, bottom: 0.0 };

    /// 获取水平总和
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// 获取垂直总和
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self::ZERO
    }
}

/// 边框样式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BorderStyle {
    /// 边框宽度
    pub width: f32,
    /// 边框颜色
    pub color: Color,
    /// 边框圆角半径
    pub radius: f32,
}

impl BorderStyle {
    pub fn new(width: f32, color: Color) -> Self {
        Self { width, color, radius: 0.0 }
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self {
            width: 0.0,
            color: Color::BLACK,
            radius: 0.0,
        }
    }
}

/// 阴影样式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ShadowStyle {
    /// 阴影偏移
    pub offset: Vec2,
    /// 阴影模糊半径
    pub blur_radius: f32,
    /// 阴影颜色
    pub color: Color,
}

impl Default for ShadowStyle {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            blur_radius: 0.0,
            color: Color::BLACK.with_alpha(0.5),
        }
    }
}

/// 字体样式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontStyle {
    /// 字体系列名称
    pub family: String,
    /// 字体大小
    pub size: f32,
    /// 字体粗细
    pub weight: FontWeight,
    /// 字体风格
    pub style: FontStyleType,
    /// 行间距
    pub line_height: f32,
    /// 字符间距
    pub letter_spacing: f32,
}

/// 字体粗细
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FontWeight {
    Thin,       // 100
    ExtraLight, // 200
    Light,      // 300
    Normal,     // 400
    Medium,     // 500
    SemiBold,   // 600
    Bold,       // 700
    ExtraBold,  // 800
    Black,      // 900
}

/// 字体风格类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FontStyleType {
    Normal,
    Italic,
    Oblique,
}

impl Default for FontStyle {
    fn default() -> Self {
        Self {
            family: "Arial".to_string(),
            size: 14.0,
            weight: FontWeight::Normal,
            style: FontStyleType::Normal,
            line_height: 1.2,
            letter_spacing: 0.0,
        }
    }
}

/// 文本对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

/// 垂直对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
    Baseline,
}

/// 显示类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Display {
    None,       // 不显示
    Block,      // 块级元素
    Inline,     // 内联元素
    InlineBlock, // 内联块级元素
    Flex,       // 弹性盒子
    Grid,       // 网格布局
}

/// 位置类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Position {
    Static,   // 正常布局流
    Relative, // 相对定位
    Absolute, // 绝对定位
    Fixed,    // 固定定位
    Sticky,   // 粘性定位
}

/// 溢出处理方式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Overflow {
    Visible, // 显示溢出内容
    Hidden,  // 隐藏溢出内容
    Scroll,  // 显示滚动条
    Auto,    // 自动显示滚动条
}

/// 光标样式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CursorStyle {
    Default,
    Pointer,
    Text,
    Move,
    Resize,
    NotAllowed,
    Wait,
    Crosshair,
    Help,
}

/// 完整的UI样式
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UIStyle {
    // 盒模型
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    
    // 边距和填充
    pub margin: Rect,
    pub padding: Rect,
    
    // 位置
    pub position: Position,
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub z_index: i32,
    
    // 显示
    pub display: Display,
    pub visibility: bool,
    pub opacity: f32,
    pub overflow: Overflow,
    
    // 背景
    pub background_color: Color,
    pub background_image: Option<String>, // 纹理路径
    
    // 边框
    pub border: BorderStyle,
    
    // 阴影
    pub box_shadow: Option<ShadowStyle>,
    
    // 文本
    pub font: FontStyle,
    pub text_color: Color,
    pub text_align: TextAlign,
    pub vertical_align: VerticalAlign,
    
    // 交互
    pub cursor: CursorStyle,
    pub pointer_events: bool,
    
    // 动画
    pub transition_duration: f32, // 过渡动画时长（秒）
    pub transition_property: Vec<String>, // 要过渡的属性
    
    // 弹性盒子属性
    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub flex_wrap: FlexWrap,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Option<f32>,
}

/// 弹性盒子方向
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

/// 主轴对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// 交叉轴对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

/// 弹性换行
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

impl Default for UIStyle {
    fn default() -> Self {
        Self {
            // 盒模型
            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            
            // 边距和填充
            margin: Rect::ZERO,
            padding: Rect::ZERO,
            
            // 位置
            position: Position::Static,
            left: None,
            top: None,
            right: None,
            bottom: None,
            z_index: 0,
            
            // 显示
            display: Display::Block,
            visibility: true,
            opacity: 1.0,
            overflow: Overflow::Visible,
            
            // 背景
            background_color: Color::TRANSPARENT,
            background_image: None,
            
            // 边框
            border: BorderStyle::default(),
            
            // 阴影
            box_shadow: None,
            
            // 文本
            font: FontStyle::default(),
            text_color: Color::BLACK,
            text_align: TextAlign::Left,
            vertical_align: VerticalAlign::Top,
            
            // 交互
            cursor: CursorStyle::Default,
            pointer_events: true,
            
            // 动画
            transition_duration: 0.0,
            transition_property: Vec::new(),
            
            // 弹性盒子属性
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            flex_wrap: FlexWrap::NoWrap,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: None,
        }
    }
}

/// 样式构建器
pub struct StyleBuilder {
    style: UIStyle,
}

impl StyleBuilder {
    pub fn new() -> Self {
        Self {
            style: UIStyle::default(),
        }
    }

    /// 设置宽高
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.style.width = Some(width);
        self.style.height = Some(height);
        self
    }

    /// 设置背景色
    pub fn background_color(mut self, color: Color) -> Self {
        self.style.background_color = color;
        self
    }

    /// 设置填充
    pub fn padding(mut self, padding: Rect) -> Self {
        self.style.padding = padding;
        self
    }

    /// 设置边距
    pub fn margin(mut self, margin: Rect) -> Self {
        self.style.margin = margin;
        self
    }

    /// 设置边框
    pub fn border(mut self, border: BorderStyle) -> Self {
        self.style.border = border;
        self
    }

    /// 设置字体
    pub fn font(mut self, font: FontStyle) -> Self {
        self.style.font = font;
        self
    }

    /// 设置文本颜色
    pub fn text_color(mut self, color: Color) -> Self {
        self.style.text_color = color;
        self
    }

    /// 设置弹性盒子
    pub fn flex(mut self) -> Self {
        self.style.display = Display::Flex;
        self
    }

    /// 设置弹性方向
    pub fn flex_direction(mut self, direction: FlexDirection) -> Self {
        self.style.flex_direction = direction;
        self
    }

    /// 设置主轴对齐
    pub fn justify_content(mut self, justify: JustifyContent) -> Self {
        self.style.justify_content = justify;
        self
    }

    /// 设置交叉轴对齐
    pub fn align_items(mut self, align: AlignItems) -> Self {
        self.style.align_items = align;
        self
    }

    /// 构建样式
    pub fn build(self) -> UIStyle {
        self.style
    }
}

impl Default for StyleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 样式预设
pub struct StylePresets;

impl StylePresets {
    /// 按钮样式
    pub fn button() -> UIStyle {
        StyleBuilder::new()
            .size(120.0, 40.0)
            .background_color(Color::hex(0x007ACC))
            .padding(Rect::symmetric(16.0, 8.0))
            .border(BorderStyle::new(1.0, Color::hex(0x005A9F)).with_radius(4.0))
            .text_color(Color::WHITE)
            .build()
    }

    /// 输入框样式
    pub fn input() -> UIStyle {
        StyleBuilder::new()
            .size(200.0, 32.0)
            .background_color(Color::WHITE)
            .padding(Rect::symmetric(8.0, 6.0))
            .border(BorderStyle::new(1.0, Color::hex(0xCCCCCC)))
            .text_color(Color::BLACK)
            .build()
    }

    /// 面板样式
    pub fn panel() -> UIStyle {
        StyleBuilder::new()
            .background_color(Color::hex(0xF5F5F5))
            .padding(Rect::all(16.0))
            .border(BorderStyle::new(1.0, Color::hex(0xDDDDDD)).with_radius(8.0))
            .build()
    }

    /// 标题样式
    pub fn title() -> UIStyle {
        let mut style = UIStyle::default();
        style.font.size = 24.0;
        style.font.weight = FontWeight::Bold;
        style.text_color = Color::hex(0x333333);
        style.margin.bottom = 16.0;
        style
    }

    /// 标签样式
    pub fn label() -> UIStyle {
        let mut style = UIStyle::default();
        style.font.size = 14.0;
        style.text_color = Color::hex(0x666666);
        style.margin.bottom = 4.0;
        style
    }
}
