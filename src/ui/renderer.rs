//! UI渲染系统

use crate::math::{Vec2, Vec3, Mat4};
use crate::render::{RenderSystem, Mesh, Material, Texture, Shader};
use crate::ui::{UIStyle, Color};
use crate::ui::widgets::{Rect, UIRenderer};
use crate::ui::style::{BorderStyle, FontStyle};
use std::collections::HashMap;

/// UI顶点数据
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UIVertex {
    pub position: Vec3,
    pub uv: Vec2,
    pub color: [f32; 4],
}

impl UIVertex {
    pub fn new(position: Vec3, uv: Vec2, color: Color) -> Self {
        Self {
            position,
            uv,
            color: [color.r, color.g, color.b, color.a],
        }
    }
}

/// UI批次渲染数据
#[derive(Debug)]
pub struct UIBatch {
    pub vertices: Vec<UIVertex>,
    pub indices: Vec<u32>,
    pub texture: Option<String>, // 纹理路径
    pub shader_type: UIShaderType,
}

/// UI着色器类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UIShaderType {
    Solid,      // 纯色
    Textured,   // 纹理
    Text,       // 文本
    Gradient,   // 渐变
}

impl UIBatch {
    pub fn new(shader_type: UIShaderType) -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            texture: None,
            shader_type,
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.texture = None;
    }

    pub fn add_quad(&mut self, rect: Rect, color: Color, uv_rect: Option<Rect>) {
        let start_index = self.vertices.len() as u32;
        let uv = uv_rect.unwrap_or(Rect::new(0.0, 0.0, 1.0, 1.0));

        // 添加四个顶点
        self.vertices.extend_from_slice(&[
            UIVertex::new(
                Vec3::new(rect.x, rect.y, 0.0),
                Vec2::new(uv.x, uv.y),
                color
            ),
            UIVertex::new(
                Vec3::new(rect.x + rect.width, rect.y, 0.0),
                Vec2::new(uv.x + uv.width, uv.y),
                color
            ),
            UIVertex::new(
                Vec3::new(rect.x + rect.width, rect.y + rect.height, 0.0),
                Vec2::new(uv.x + uv.width, uv.y + uv.height),
                color
            ),
            UIVertex::new(
                Vec3::new(rect.x, rect.y + rect.height, 0.0),
                Vec2::new(uv.x, uv.y + uv.height),
                color
            ),
        ]);

        // 添加两个三角形的索引
        self.indices.extend_from_slice(&[
            start_index, start_index + 1, start_index + 2,
            start_index, start_index + 2, start_index + 3,
        ]);
    }

    pub fn add_rounded_rect(&mut self, rect: Rect, color: Color, radius: f32, segments: u32) {
        if radius <= 0.0 {
            self.add_quad(rect, color, None);
            return;
        }

        let clamped_radius = radius.min(rect.width * 0.5).min(rect.height * 0.5);
        let center = Vec2::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5);
        
        // 简化实现：使用多个四边形近似圆角
        let segment_angle = std::f32::consts::PI * 0.5 / segments as f32;
        
        // 添加中心矩形
        let inner_rect = Rect::new(
            rect.x + clamped_radius,
            rect.y + clamped_radius,
            rect.width - clamped_radius * 2.0,
            rect.height - clamped_radius * 2.0,
        );
        self.add_quad(inner_rect, color, None);

        // 添加四个边的矩形
        // 顶部
        self.add_quad(
            Rect::new(rect.x + clamped_radius, rect.y, rect.width - clamped_radius * 2.0, clamped_radius),
            color,
            None
        );
        // 底部
        self.add_quad(
            Rect::new(rect.x + clamped_radius, rect.y + rect.height - clamped_radius, rect.width - clamped_radius * 2.0, clamped_radius),
            color,
            None
        );
        // 左侧
        self.add_quad(
            Rect::new(rect.x, rect.y + clamped_radius, clamped_radius, rect.height - clamped_radius * 2.0),
            color,
            None
        );
        // 右侧
        self.add_quad(
            Rect::new(rect.x + rect.width - clamped_radius, rect.y + clamped_radius, clamped_radius, rect.height - clamped_radius * 2.0),
            color,
            None
        );

        // 添加四个圆角（简化为四边形）
        let corners = [
            Vec2::new(rect.x + clamped_radius, rect.y + clamped_radius), // 左上
            Vec2::new(rect.x + rect.width - clamped_radius, rect.y + clamped_radius), // 右上
            Vec2::new(rect.x + rect.width - clamped_radius, rect.y + rect.height - clamped_radius), // 右下
            Vec2::new(rect.x + clamped_radius, rect.y + rect.height - clamped_radius), // 左下
        ];

        for corner in corners {
            self.add_quad(
                Rect::new(corner.x - clamped_radius, corner.y - clamped_radius, clamped_radius * 2.0, clamped_radius * 2.0),
                color,
                None
            );
        }
    }
}

/// 字体缓存
#[derive(Debug)]
pub struct FontCache {
    fonts: HashMap<String, Font>,
    text_textures: HashMap<String, Texture>, // 文本渲染缓存
}

/// 字体数据
#[derive(Debug, Clone)]
pub struct Font {
    pub family: String,
    pub size: f32,
    pub atlas_texture: Option<String>, // 字体图集纹理
    pub glyph_info: HashMap<char, GlyphInfo>,
}

/// 字形信息
#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    pub uv_rect: Rect,      // 在字体图集中的UV坐标
    pub size: Vec2,         // 字形大小
    pub offset: Vec2,       // 渲染偏移
    pub advance: f32,       // 字符进步距离
}

impl FontCache {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            text_textures: HashMap::new(),
        }
    }

    pub fn load_font(&mut self, family: &str, size: f32) -> Option<&Font> {
        let key = format!("{}_{}", family, size);
        
        if !self.fonts.contains_key(&key) {
            // TODO: 实际加载字体文件
            let font = Font {
                family: family.to_string(),
                size,
                atlas_texture: None,
                glyph_info: HashMap::new(),
            };
            self.fonts.insert(key.clone(), font);
        }

        self.fonts.get(&key)
    }

    pub fn get_text_size(&self, text: &str, font_style: &FontStyle) -> Vec2 {
        // 简化实现：基于字体大小估算
        let char_width = font_style.size * 0.6;
        let line_height = font_style.size * font_style.line_height;
        
        let lines: Vec<&str> = text.lines().collect();
        let max_width = lines.iter()
            .map(|line| line.len() as f32 * char_width)
            .fold(0.0, f32::max);
        let height = lines.len() as f32 * line_height;

        Vec2::new(max_width, height)
    }
}

/// UI渲染器实现
pub struct UIRendererImpl {
    batches: Vec<UIBatch>,
    current_batch: UIBatch,
    font_cache: FontCache,
    texture_cache: HashMap<String, Texture>,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    screen_size: Vec2,
}

impl UIRendererImpl {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let screen_size = Vec2::new(screen_width, screen_height);
        
        // UI使用正交投影
        let projection_matrix = Mat4::orthographic_rh(
            0.0, screen_width,
            screen_height, 0.0, // Y轴向下
            -1.0, 1.0
        );

        Self {
            batches: Vec::new(),
            current_batch: UIBatch::new(UIShaderType::Solid),
            font_cache: FontCache::new(),
            texture_cache: HashMap::new(),
            view_matrix: Mat4::IDENTITY,
            projection_matrix,
            screen_size,
        }
    }

    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.screen_size = Vec2::new(width, height);
        self.projection_matrix = Mat4::orthographic_rh(
            0.0, width,
            height, 0.0,
            -1.0, 1.0
        );
    }

    pub fn begin_frame(&mut self) {
        self.batches.clear();
        self.current_batch.clear();
    }

    pub fn end_frame(&mut self) {
        if !self.current_batch.vertices.is_empty() {
            self.batches.push(std::mem::replace(
                &mut self.current_batch,
                UIBatch::new(UIShaderType::Solid)
            ));
        }
    }

    fn flush_batch(&mut self) {
        if !self.current_batch.vertices.is_empty() {
            self.batches.push(std::mem::replace(
                &mut self.current_batch,
                UIBatch::new(UIShaderType::Solid)
            ));
        }
    }

    fn ensure_batch_type(&mut self, shader_type: UIShaderType, texture: Option<&str>) {
        if self.current_batch.shader_type != shader_type ||
           self.current_batch.texture.as_deref() != texture {
            self.flush_batch();
            self.current_batch = UIBatch::new(shader_type);
            self.current_batch.texture = texture.map(|s| s.to_string());
        }
    }

    pub fn render(&self, render_system: &mut RenderSystem) {
        // TODO: 实际渲染UI批次到GPU
        for batch in &self.batches {
            match batch.shader_type {
                UIShaderType::Solid => {
                    // 渲染纯色UI元素
                }
                UIShaderType::Textured => {
                    // 渲染纹理UI元素
                }
                UIShaderType::Text => {
                    // 渲染文本
                }
                UIShaderType::Gradient => {
                    // 渲染渐变
                }
            }
        }
    }

    /// 设置剪裁区域
    pub fn set_clip_rect(&mut self, rect: Option<Rect>) {
        // TODO: 实现剪裁功能
    }

    /// 推入剪裁区域栈
    pub fn push_clip_rect(&mut self, rect: Rect) {
        // TODO: 实现剪裁栈
    }

    /// 弹出剪裁区域栈
    pub fn pop_clip_rect(&mut self) {
        // TODO: 实现剪裁栈
    }
}

impl UIRenderer for UIRendererImpl {
    fn draw_rect(&mut self, bounds: Rect, color: Color) {
        self.ensure_batch_type(UIShaderType::Solid, None);
        self.current_batch.add_quad(bounds, color, None);
    }

    fn draw_border(&mut self, bounds: Rect, border: &BorderStyle) {
        if border.width <= 0.0 {
            return;
        }

        self.ensure_batch_type(UIShaderType::Solid, None);

        if border.radius > 0.0 {
            // 圆角边框（简化实现）
            let outer_rect = bounds;
            let inner_rect = Rect::new(
                bounds.x + border.width,
                bounds.y + border.width,
                bounds.width - border.width * 2.0,
                bounds.height - border.width * 2.0,
            );

            // 绘制外圆角矩形
            self.current_batch.add_rounded_rect(outer_rect, border.color, border.radius, 8);
            
            // 绘制内圆角矩形（用背景色"挖空"）
            if inner_rect.width > 0.0 && inner_rect.height > 0.0 {
                let inner_radius = (border.radius - border.width).max(0.0);
                self.current_batch.add_rounded_rect(inner_rect, Color::TRANSPARENT, inner_radius, 8);
            }
        } else {
            // 直角边框
            // 顶边
            self.current_batch.add_quad(
                Rect::new(bounds.x, bounds.y, bounds.width, border.width),
                border.color,
                None
            );
            // 底边
            self.current_batch.add_quad(
                Rect::new(bounds.x, bounds.y + bounds.height - border.width, bounds.width, border.width),
                border.color,
                None
            );
            // 左边
            self.current_batch.add_quad(
                Rect::new(bounds.x, bounds.y + border.width, border.width, bounds.height - border.width * 2.0),
                border.color,
                None
            );
            // 右边
            self.current_batch.add_quad(
                Rect::new(bounds.x + bounds.width - border.width, bounds.y + border.width, border.width, bounds.height - border.width * 2.0),
                border.color,
                None
            );
        }
    }

    fn draw_text(&mut self, text: &str, bounds: Rect, font: &FontStyle, color: Color) {
        if text.is_empty() {
            return;
        }

        self.ensure_batch_type(UIShaderType::Text, None);

        // 加载字体
        if let Some(_font) = self.font_cache.load_font(&font.family, font.size) {
            // 计算文本位置
            let text_size = self.font_cache.get_text_size(text, font);
            let mut position = Vec2::new(bounds.x, bounds.y);

            // 水平对齐
            match font.family.as_str() {
                _ => {
                    // 简化实现：左对齐
                }
            }

            // 渲染每个字符
            let char_width = font.size * 0.6;
            let line_height = font.size * font.line_height;
            
            let mut current_pos = position;
            
            for line in text.lines() {
                current_pos.x = position.x;
                
                for ch in line.chars() {
                    if ch == ' ' {
                        current_pos.x += char_width;
                        continue;
                    }

                    // TODO: 使用真实的字形数据
                    let char_rect = Rect::new(
                        current_pos.x,
                        current_pos.y,
                        char_width,
                        font.size
                    );

                    // 添加字符四边形
                    self.current_batch.add_quad(char_rect, color, None);
                    current_pos.x += char_width + font.letter_spacing;
                }
                
                current_pos.y += line_height;
            }
        }
    }

    fn draw_icon(&mut self, icon_path: &str, bounds: Rect) {
        self.ensure_batch_type(UIShaderType::Textured, Some(icon_path));
        self.current_batch.add_quad(bounds, Color::WHITE, None);
    }

    fn draw_image(&mut self, image_path: &str, bounds: Rect) {
        self.ensure_batch_type(UIShaderType::Textured, Some(image_path));
        self.current_batch.add_quad(bounds, Color::WHITE, None);
    }
}

/// UI渲染统计信息
#[derive(Debug, Default)]
pub struct UIRenderStats {
    pub draw_calls: u32,
    pub triangles: u32,
    pub vertices: u32,
    pub text_draws: u32,
    pub batches: u32,
}

impl UIRenderStats {
    pub fn reset(&mut self) {
        *self = Default::default();
    }

    pub fn add_batch(&mut self, batch: &UIBatch) {
        self.batches += 1;
        self.vertices += batch.vertices.len() as u32;
        self.triangles += batch.indices.len() as u32 / 3;
        self.draw_calls += 1;
        
        if batch.shader_type == UIShaderType::Text {
            self.text_draws += 1;
        }
    }
}

/// UI着色器管理器
pub struct UIShaderManager {
    shaders: HashMap<UIShaderType, Shader>,
}

impl UIShaderManager {
    pub fn new() -> Self {
        Self {
            shaders: HashMap::new(),
        }
    }

    pub fn load_shaders(&mut self) {
        // TODO: 加载UI着色器
        // self.shaders.insert(UIShaderType::Solid, load_solid_shader());
        // self.shaders.insert(UIShaderType::Textured, load_textured_shader());
        // self.shaders.insert(UIShaderType::Text, load_text_shader());
        // self.shaders.insert(UIShaderType::Gradient, load_gradient_shader());
    }

    pub fn get_shader(&self, shader_type: UIShaderType) -> Option<&Shader> {
        self.shaders.get(&shader_type)
    }
}

/// UI渲染上下文
pub struct UIRenderContext {
    pub renderer: UIRendererImpl,
    pub shader_manager: UIShaderManager,
    pub stats: UIRenderStats,
}

impl UIRenderContext {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            renderer: UIRendererImpl::new(screen_width, screen_height),
            shader_manager: UIShaderManager::new(),
            stats: UIRenderStats::default(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.renderer.begin_frame();
        self.stats.reset();
    }

    pub fn end_frame(&mut self) {
        self.renderer.end_frame();
        
        // 更新统计信息
        for batch in &self.renderer.batches {
            self.stats.add_batch(batch);
        }
    }

    pub fn render(&self, render_system: &mut RenderSystem) {
        self.renderer.render(render_system);
    }
}
