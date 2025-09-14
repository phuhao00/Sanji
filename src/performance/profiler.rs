//! 性能分析器

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::Serialize;

/// 性能分析器
pub struct Profiler {
    sections: HashMap<String, ProfileSection>,
    call_stack: Vec<String>,
    frame_data: Vec<FrameProfileData>,
    current_frame: FrameProfileData,
    enabled: bool,
}

/// 性能分析区域
#[derive(Debug, Clone)]
struct ProfileSection {
    name: String,
    total_time: Duration,
    call_count: u64,
    max_time: Duration,
    min_time: Duration,
    average_time: Duration,
    last_time: Duration,
    children: Vec<String>,
    parent: Option<String>,
}

/// 帧性能数据
#[derive(Debug, Clone)]
struct FrameProfileData {
    frame_number: u64,
    frame_start: Instant,
    sections: HashMap<String, SectionData>,
    total_frame_time: Duration,
}

impl Default for FrameProfileData {
    fn default() -> Self {
        Self {
            frame_number: 0,
            frame_start: Instant::now(),
            sections: HashMap::new(),
            total_frame_time: Duration::ZERO,
        }
    }
}

/// 区域数据
#[derive(Debug, Clone)]
struct SectionData {
    name: String,
    start_time: Instant,
    duration: Duration,
    call_count: u32,
    depth: u32,
}

/// 性能分析守卫
pub struct ProfilerGuard {
    profiler: *mut Profiler,
    section_name: String,
    start_time: Instant,
    enabled: bool,
}

impl ProfilerGuard {
    pub fn new(profiler: &mut Profiler, section_name: String) -> Self {
        let start_time = Instant::now();
        profiler.push_section(&section_name);
        
        Self {
            profiler: profiler as *mut Profiler,
            section_name,
            start_time,
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        Self {
            profiler: std::ptr::null_mut(),
            section_name: String::new(),
            start_time: Instant::now(),
            enabled: false,
        }
    }
}

impl Drop for ProfilerGuard {
    fn drop(&mut self) {
        if self.enabled && !self.profiler.is_null() {
            unsafe {
                let profiler = &mut *self.profiler;
                let duration = self.start_time.elapsed();
                profiler.pop_section(&self.section_name, duration);
            }
        }
    }
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            sections: HashMap::new(),
            call_stack: Vec::new(),
            frame_data: Vec::new(),
            current_frame: FrameProfileData::default(),
            enabled: true,
        }
    }

    /// 启用/禁用分析器
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 开始帧分析
    pub fn begin_frame(&mut self) {
        if !self.enabled {
            return;
        }

        self.current_frame = FrameProfileData {
            frame_number: self.frame_data.len() as u64 + 1,
            frame_start: Instant::now(),
            sections: HashMap::new(),
            total_frame_time: Duration::ZERO,
        };
    }

    /// 结束帧分析
    pub fn end_frame(&mut self) {
        if !self.enabled {
            return;
        }

        self.current_frame.total_frame_time = self.current_frame.frame_start.elapsed();
        self.frame_data.push(self.current_frame.clone());

        // 保持最近的1000帧数据
        if self.frame_data.len() > 1000 {
            self.frame_data.remove(0);
        }
    }

    /// 开始分析区域
    pub fn begin_section(&mut self, name: &str) -> ProfilerGuard {
        if self.enabled {
            ProfilerGuard::new(self, name.to_string())
        } else {
            ProfilerGuard::disabled()
        }
    }

    /// 推入分析区域到栈
    fn push_section(&mut self, name: &str) {
        self.call_stack.push(name.to_string());
        
        let section_data = SectionData {
            name: name.to_string(),
            start_time: Instant::now(),
            duration: Duration::ZERO,
            call_count: 1,
            depth: self.call_stack.len() as u32 - 1,
        };

        self.current_frame.sections.insert(name.to_string(), section_data);
    }

    /// 弹出分析区域
    fn pop_section(&mut self, name: &str, duration: Duration) {
        if let Some(pos) = self.call_stack.iter().position(|n| n == name) {
            self.call_stack.remove(pos);
        }

        // 更新当前帧数据
        if let Some(section_data) = self.current_frame.sections.get_mut(name) {
            section_data.duration = duration;
        }

        // 获取父级名称
        let parent_name = self.call_stack.last().cloned();
        let section_name = name.to_string();

        // 更新总体统计
        let section = self.sections.entry(section_name.clone()).or_insert_with(|| ProfileSection {
            name: section_name.clone(),
            total_time: Duration::ZERO,
            call_count: 0,
            max_time: Duration::ZERO,
            min_time: Duration::MAX,
            average_time: Duration::ZERO,
            last_time: duration,
            children: Vec::new(),
            parent: parent_name.clone(),
        });

        section.total_time += duration;
        section.call_count += 1;
        section.max_time = section.max_time.max(duration);
        section.min_time = section.min_time.min(duration);
        section.average_time = section.total_time / section.call_count as u32;
        section.last_time = duration;

        // 分离更新父子关系
        if let Some(parent_name) = parent_name {
            if let Some(parent_section) = self.sections.get_mut(&parent_name) {
                if !parent_section.children.contains(&section_name) {
                    parent_section.children.push(section_name);
                }
            }
        }
    }

    /// 获取详细分析结果
    pub fn get_detailed_breakdown(&self) -> DetailedBreakdown {
        DetailedBreakdown {
            sections: self.sections.values().cloned().collect(),
            frame_data: self.get_recent_frame_analysis(60), // 最近60帧
            call_tree: self.build_call_tree(),
        }
    }

    /// 获取最近帧分析
    fn get_recent_frame_analysis(&self, frame_count: usize) -> Vec<FrameAnalysisData> {
        let start_index = if self.frame_data.len() > frame_count {
            self.frame_data.len() - frame_count
        } else {
            0
        };

        self.frame_data[start_index..]
            .iter()
            .map(|frame| FrameAnalysisData {
                frame_number: frame.frame_number,
                total_time: frame.total_frame_time,
                sections: frame.sections.values().cloned().collect(),
            })
            .collect()
    }

    /// 构建调用树
    fn build_call_tree(&self) -> Vec<CallTreeNode> {
        let mut roots = Vec::new();
        
        for section in self.sections.values() {
            if section.parent.is_none() {
                let node = self.build_call_tree_node(section);
                roots.push(node);
            }
        }

        roots
    }

    /// 构建调用树节点
    fn build_call_tree_node(&self, section: &ProfileSection) -> CallTreeNode {
        let mut children = Vec::new();
        
        for child_name in &section.children {
            if let Some(child_section) = self.sections.get(child_name) {
                children.push(self.build_call_tree_node(child_section));
            }
        }

        CallTreeNode {
            name: section.name.clone(),
            total_time: section.total_time,
            call_count: section.call_count,
            average_time: section.average_time,
            percentage: self.calculate_percentage(section.total_time),
            children,
        }
    }

    /// 计算时间百分比
    fn calculate_percentage(&self, time: Duration) -> f32 {
        if let Some(last_frame) = self.frame_data.last() {
            if last_frame.total_frame_time.as_nanos() > 0 {
                (time.as_nanos() as f32 / last_frame.total_frame_time.as_nanos() as f32) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// 获取性能热点
    pub fn get_hotspots(&self, limit: usize) -> Vec<PerformanceHotspot> {
        let mut hotspots: Vec<_> = self.sections
            .values()
            .map(|section| PerformanceHotspot {
                name: section.name.clone(),
                total_time: section.total_time,
                average_time: section.average_time,
                call_count: section.call_count,
                percentage: self.calculate_percentage(section.total_time),
            })
            .collect();

        // 按总时间排序
        hotspots.sort_by(|a, b| b.total_time.cmp(&a.total_time));
        hotspots.truncate(limit);
        hotspots
    }

    /// 重置分析器
    pub fn reset(&mut self) {
        self.sections.clear();
        self.call_stack.clear();
        self.frame_data.clear();
        self.current_frame = FrameProfileData::default();
    }

    /// 获取分析摘要
    pub fn get_summary(&self) -> ProfilerSummary {
        let total_sections = self.sections.len();
        let total_calls = self.sections.values().map(|s| s.call_count).sum();
        let total_time: Duration = self.sections.values().map(|s| s.total_time).sum();
        
        let average_frame_time = if !self.frame_data.is_empty() {
            self.frame_data.iter()
                .map(|f| f.total_frame_time)
                .sum::<Duration>() / self.frame_data.len() as u32
        } else {
            Duration::ZERO
        };

        ProfilerSummary {
            total_sections,
            total_calls,
            total_time,
            average_frame_time,
            frames_analyzed: self.frame_data.len(),
        }
    }

    /// 导出分析数据为JSON
    pub fn export_json(&self) -> serde_json::Result<String> {
        let data = ProfilerExportData {
            summary: self.get_summary(),
            sections: self.sections.values().cloned().collect(),
            hotspots: self.get_hotspots(10),
            call_tree: self.build_call_tree(),
        };
        
        serde_json::to_string_pretty(&data)
    }

    /// 获取内存使用情况
    pub fn get_memory_usage(&self) -> ProfilerMemoryUsage {
        let sections_memory = self.sections.len() * std::mem::size_of::<ProfileSection>();
        let frame_data_memory = self.frame_data.len() * std::mem::size_of::<FrameProfileData>();
        let call_stack_memory = self.call_stack.capacity() * std::mem::size_of::<String>();

        ProfilerMemoryUsage {
            total_bytes: sections_memory + frame_data_memory + call_stack_memory,
            sections_bytes: sections_memory,
            frame_data_bytes: frame_data_memory,
            call_stack_bytes: call_stack_memory,
        }
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

/// 详细分析结果
#[derive(Debug, Clone, Serialize)]
pub struct DetailedBreakdown {
    pub sections: Vec<ProfileSection>,
    pub frame_data: Vec<FrameAnalysisData>,
    pub call_tree: Vec<CallTreeNode>,
}

/// 帧分析数据
#[derive(Debug, Clone, Serialize)]
pub struct FrameAnalysisData {
    pub frame_number: u64,
    pub total_time: Duration,
    pub sections: Vec<SectionData>,
}

/// 调用树节点
#[derive(Debug, Clone, Serialize)]
pub struct CallTreeNode {
    pub name: String,
    pub total_time: Duration,
    pub call_count: u64,
    pub average_time: Duration,
    pub percentage: f32,
    pub children: Vec<CallTreeNode>,
}

/// 性能热点
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceHotspot {
    pub name: String,
    pub total_time: Duration,
    pub average_time: Duration,
    pub call_count: u64,
    pub percentage: f32,
}

/// 分析器摘要
#[derive(Debug, Clone, Serialize)]
pub struct ProfilerSummary {
    pub total_sections: usize,
    pub total_calls: u64,
    pub total_time: Duration,
    pub average_frame_time: Duration,
    pub frames_analyzed: usize,
}

/// 分析器导出数据
#[derive(Debug, Clone, Serialize)]
pub struct ProfilerExportData {
    pub summary: ProfilerSummary,
    pub sections: Vec<ProfileSection>,
    pub hotspots: Vec<PerformanceHotspot>,
    pub call_tree: Vec<CallTreeNode>,
}

/// 分析器内存使用
#[derive(Debug, Clone, Serialize)]
pub struct ProfilerMemoryUsage {
    pub total_bytes: usize,
    pub sections_bytes: usize,
    pub frame_data_bytes: usize,
    pub call_stack_bytes: usize,
}

// 为需要序列化的类型实现Serialize
impl Serialize for ProfileSection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ProfileSection", 8)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("total_time_ms", &self.total_time.as_millis())?;
        state.serialize_field("call_count", &self.call_count)?;
        state.serialize_field("max_time_ms", &self.max_time.as_millis())?;
        state.serialize_field("min_time_ms", &self.min_time.as_millis())?;
        state.serialize_field("average_time_ms", &self.average_time.as_millis())?;
        state.serialize_field("last_time_ms", &self.last_time.as_millis())?;
        state.serialize_field("children", &self.children)?;
        state.end()
    }
}

impl Serialize for SectionData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SectionData", 4)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("duration_ms", &self.duration.as_millis())?;
        state.serialize_field("call_count", &self.call_count)?;
        state.serialize_field("depth", &self.depth)?;
        state.end()
    }
}

/// 自动性能分析宏
#[macro_export]
macro_rules! profile_scope_auto {
    ($profiler:expr, $name:expr, $code:block) => {{
        let _guard = $profiler.begin_section($name);
        $code
    }};
}

/// 条件性能分析宏
#[macro_export]
macro_rules! profile_scope_if {
    ($condition:expr, $profiler:expr, $name:expr, $code:block) => {{
        if $condition {
            let _guard = $profiler.begin_section($name);
            $code
        } else {
            $code
        }
    }};
}
