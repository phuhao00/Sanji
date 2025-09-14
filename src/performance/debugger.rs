//! 调试器和可视化工具

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use serde::Serialize;

/// 调试器
pub struct Debugger {
    enabled: bool,
    debug_overlay: DebugOverlay,
    console: DebugConsole,
    profiler_visualizer: ProfilerVisualizer,
    memory_visualizer: MemoryVisualizer,
    frame_graph: FrameGraph,
    hot_reload: HotReloadManager,
}

/// 调试覆盖层
pub struct DebugOverlay {
    visible: bool,
    panels: HashMap<String, DebugPanel>,
    layout: OverlayLayout,
    style: OverlayStyle,
}

/// 调试面板
#[derive(Debug, Clone, serde::Serialize)]
pub struct DebugPanel {
    pub id: String,
    pub title: String,
    pub content: PanelContent,
    pub position: PanelPosition,
    pub size: PanelSize,
    pub visible: bool,
    pub collapsible: bool,
    pub collapsed: bool,
}

/// 面板内容
#[derive(Debug, Clone, serde::Serialize)]
pub enum PanelContent {
    Text(String),
    Metrics(HashMap<String, f64>),
    Graph(GraphData),
    Table(TableData),
    Tree(TreeData),
    Custom(String), // HTML或自定义格式
}

/// 图表数据
#[derive(Debug, Clone, Serialize)]
pub struct GraphData {
    pub series: Vec<GraphSeries>,
    pub x_axis: AxisConfig,
    pub y_axis: AxisConfig,
    pub legend: bool,
    pub grid: bool,
}

/// 图表系列
#[derive(Debug, Clone, Serialize)]
pub struct GraphSeries {
    pub name: String,
    pub data: Vec<(f64, f64)>, // (x, y) 坐标
    pub color: String,
    pub line_type: LineType,
}

/// 线条类型
#[derive(Debug, Clone, Serialize)]
pub enum LineType {
    Solid,
    Dashed,
    Dotted,
}

/// 轴配置
#[derive(Debug, Clone, Serialize)]
pub struct AxisConfig {
    pub label: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub auto_scale: bool,
}

/// 表格数据
#[derive(Debug, Clone, Serialize)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub sortable: bool,
    pub filterable: bool,
}

/// 树形数据
#[derive(Debug, Clone, Serialize)]
pub struct TreeData {
    pub nodes: Vec<TreeNode>,
}

/// 树节点
#[derive(Debug, Clone, Serialize)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
    pub data: HashMap<String, String>,
}

/// 面板位置
#[derive(Debug, Clone, serde::Serialize)]
pub struct PanelPosition {
    pub x: f32,
    pub y: f32,
    pub anchor: AnchorPoint,
}

/// 锚点
#[derive(Debug, Clone, serde::Serialize)]
pub enum AnchorPoint {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

/// 面板大小
#[derive(Debug, Clone, serde::Serialize)]
pub struct PanelSize {
    pub width: f32,
    pub height: f32,
    pub min_width: f32,
    pub min_height: f32,
    pub resizable: bool,
}

/// 覆盖层布局
#[derive(Debug, Clone)]
pub struct OverlayLayout {
    pub dock_areas: Vec<DockArea>,
    pub floating_panels: Vec<String>,
}

/// 停靠区域
#[derive(Debug, Clone)]
pub struct DockArea {
    pub position: DockPosition,
    pub size: f32, // 百分比
    pub panels: Vec<String>,
}

/// 停靠位置
#[derive(Debug, Clone)]
pub enum DockPosition {
    Left,
    Right,
    Top,
    Bottom,
}

/// 覆盖层样式
#[derive(Debug, Clone)]
pub struct OverlayStyle {
    pub background_color: [f32; 4],
    pub text_color: [f32; 4],
    pub border_color: [f32; 4],
    pub font_size: f32,
    pub transparency: f32,
}

/// 调试控制台
pub struct DebugConsole {
    visible: bool,
    history: VecDeque<ConsoleMessage>,
    max_history: usize,
    input_buffer: String,
    command_handler: CommandHandler,
    filters: Vec<LogFilter>,
}

/// 控制台消息
#[derive(Debug, Clone, Serialize)]
pub struct ConsoleMessage {
    pub timestamp: u128,
    pub level: LogLevel,
    pub category: String,
    pub message: String,
    pub source: String,
    pub stack_trace: Option<String>,
}

/// 日志级别
#[derive(Debug, Clone, Serialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

/// 日志过滤器
#[derive(Debug, Clone)]
pub struct LogFilter {
    pub level: Option<LogLevel>,
    pub category: Option<String>,
    pub text_filter: Option<String>,
    pub enabled: bool,
}

/// 命令处理器
pub struct CommandHandler {
    commands: HashMap<String, Box<dyn Fn(&[String]) -> String + Send + Sync>>,
}

/// 性能分析器可视化器
pub struct ProfilerVisualizer {
    frame_time_graph: GraphData,
    call_tree_view: TreeData,
    hotspots_table: TableData,
    timeline_view: TimelineData,
}

/// 时间线数据
#[derive(Debug, Clone, Serialize)]
pub struct TimelineData {
    pub events: Vec<TimelineEvent>,
    pub duration: Duration,
    pub scale: f64,
}

/// 时间线事件
#[derive(Debug, Clone, Serialize)]
pub struct TimelineEvent {
    pub name: String,
    pub start_time: Duration,
    pub duration: Duration,
    pub category: String,
    pub color: String,
    pub level: u32,
}

/// 内存可视化器
pub struct MemoryVisualizer {
    usage_graph: GraphData,
    allocation_heatmap: HeatmapData,
    leak_candidates_table: TableData,
    memory_map: MemoryMapData,
}

/// 热力图数据
#[derive(Debug, Clone, Serialize)]
pub struct HeatmapData {
    pub data: Vec<Vec<f32>>,
    pub width: usize,
    pub height: usize,
    pub min_value: f32,
    pub max_value: f32,
    pub color_scheme: ColorScheme,
}

/// 颜色方案
#[derive(Debug, Clone, Serialize)]
pub enum ColorScheme {
    Heat,    // 红-黄渐变
    Cool,    // 蓝-绿渐变
    Grayscale, // 灰度
    Custom(Vec<[f32; 3]>), // 自定义颜色
}

/// 内存映射数据
#[derive(Debug, Clone, Serialize)]
pub struct MemoryMapData {
    pub regions: Vec<MemoryRegion>,
    pub total_size: usize,
    pub scale: f64,
}

/// 内存区域
#[derive(Debug, Clone, Serialize)]
pub struct MemoryRegion {
    pub start_address: usize,
    pub size: usize,
    pub region_type: MemoryRegionType,
    pub name: String,
    pub color: String,
}

/// 内存区域类型
#[derive(Debug, Clone, Serialize)]
pub enum MemoryRegionType {
    Allocated,
    Free,
    Reserved,
    System,
}

/// 帧图表
pub struct FrameGraph {
    nodes: Vec<FrameGraphNode>,
    edges: Vec<FrameGraphEdge>,
    layout: GraphLayout,
}

/// 帧图表节点
#[derive(Debug, Clone)]
pub struct FrameGraphNode {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub color: String,
    pub metadata: HashMap<String, String>,
}

/// 节点类型
#[derive(Debug, Clone)]
pub enum NodeType {
    RenderPass,
    ComputePass,
    Resource,
    Barrier,
    Present,
}

/// 帧图表边
#[derive(Debug, Clone)]
pub struct FrameGraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
    pub label: Option<String>,
    pub color: String,
}

/// 边类型
#[derive(Debug, Clone)]
pub enum EdgeType {
    DataDependency,
    ExecutionOrder,
    ResourceAccess,
}

/// 图表布局
#[derive(Debug, Clone)]
pub struct GraphLayout {
    pub algorithm: LayoutAlgorithm,
    pub spacing: f32,
    pub layers: bool,
}

/// 布局算法
#[derive(Debug, Clone)]
pub enum LayoutAlgorithm {
    Hierarchical,
    Force,
    Circular,
    Manual,
}

/// 热重载管理器
pub struct HotReloadManager {
    enabled: bool,
    watched_files: HashMap<String, FileWatcher>,
    reload_handlers: HashMap<String, Box<dyn Fn(&str) + Send + Sync>>,
}

/// 文件监视器
#[derive(Debug)]
pub struct FileWatcher {
    pub path: String,
    pub last_modified: std::time::SystemTime,
    pub file_type: FileType,
}

/// 文件类型
#[derive(Debug, Clone)]
pub enum FileType {
    Shader,
    Texture,
    Model,
    Script,
    Config,
    Scene,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            enabled: true,
            debug_overlay: DebugOverlay::new(),
            console: DebugConsole::new(),
            profiler_visualizer: ProfilerVisualizer::new(),
            memory_visualizer: MemoryVisualizer::new(),
            frame_graph: FrameGraph::new(),
            hot_reload: HotReloadManager::new(),
        }
    }

    /// 启用/禁用调试器
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 切换调试覆盖层
    pub fn toggle_overlay(&mut self) {
        self.debug_overlay.visible = !self.debug_overlay.visible;
    }

    /// 添加调试面板
    pub fn add_panel(&mut self, panel: DebugPanel) {
        self.debug_overlay.panels.insert(panel.id.clone(), panel);
    }

    /// 更新面板内容
    pub fn update_panel(&mut self, panel_id: &str, content: PanelContent) {
        if let Some(panel) = self.debug_overlay.panels.get_mut(panel_id) {
            panel.content = content;
        }
    }

    /// 记录控制台消息
    pub fn log(&mut self, level: LogLevel, category: String, message: String, source: String) {
        let console_message = ConsoleMessage {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            level,
            category,
            message,
            source,
            stack_trace: None,
        };

        self.console.add_message(console_message);
    }

    /// 执行控制台命令
    pub fn execute_command(&mut self, command: &str) -> String {
        self.console.execute_command(command)
    }

    /// 更新性能分析可视化
    pub fn update_profiler_visualization(&mut self, profiler_data: &crate::performance::ProfilerExportData) {
        self.profiler_visualizer.update(profiler_data);
    }

    /// 更新内存可视化
    pub fn update_memory_visualization(&mut self, memory_data: &crate::performance::MemoryAnalysis) {
        self.memory_visualizer.update(memory_data);
    }

    /// 注册热重载处理器
    pub fn register_hot_reload_handler<F>(&mut self, file_pattern: String, handler: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.hot_reload.register_handler(file_pattern, Box::new(handler));
    }

    /// 检查热重载
    pub fn check_hot_reload(&mut self) {
        self.hot_reload.check_and_reload();
    }

    /// 渲染调试界面
    pub fn render(&mut self, ui_system: &mut crate::ui::UISystem) {
        if !self.enabled || !self.debug_overlay.visible {
            return;
        }

        self.render_overlay(ui_system);
        self.render_console(ui_system);
    }

    /// 渲染覆盖层
    fn render_overlay(&mut self, _ui_system: &mut crate::ui::UISystem) {
        // TODO: 实现覆盖层渲染
        // 这里需要使用UI系统来渲染各种调试面板
    }

    /// 渲染控制台
    fn render_console(&mut self, _ui_system: &mut crate::ui::UISystem) {
        // TODO: 实现控制台渲染
        // 这里需要使用UI系统来渲染控制台界面
    }

    /// 导出调试信息
    pub fn export_debug_info(&self) -> DebugExport {
        DebugExport {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            console_messages: self.console.history.clone().into(),
            panel_states: self.debug_overlay.panels.clone(),
            performance_data: self.profiler_visualizer.export_data(),
            memory_data: self.memory_visualizer.export_data(),
        }
    }
}

impl DebugOverlay {
    pub fn new() -> Self {
        Self {
            visible: false,
            panels: HashMap::new(),
            layout: OverlayLayout::default(),
            style: OverlayStyle::default(),
        }
    }
}

impl DebugConsole {
    pub fn new() -> Self {
        let mut console = Self {
            visible: false,
            history: VecDeque::new(),
            max_history: 1000,
            input_buffer: String::new(),
            command_handler: CommandHandler::new(),
            filters: Vec::new(),
        };

        console.register_default_commands();
        console
    }

    /// 添加消息
    pub fn add_message(&mut self, message: ConsoleMessage) {
        self.history.push_back(message);
        
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    /// 执行命令
    pub fn execute_command(&mut self, command: &str) -> String {
        self.command_handler.execute(command)
    }

    /// 注册默认命令
    fn register_default_commands(&mut self) {
        self.command_handler.register("help", Box::new(|_: &[String]| -> String {
            "Available commands:\n\
             - help: Show this help\n\
             - clear: Clear console\n\
             - fps: Show current FPS\n\
             - memory: Show memory usage\n\
             - gc: Force garbage collection\n\
             - reload: Reload resources".to_string()
        }));

        self.command_handler.register("clear", Box::new(|_: &[String]| -> String {
            "Console cleared".to_string()
        }));

        self.command_handler.register("fps", Box::new(|_: &[String]| -> String {
            // TODO: 获取实际FPS
            "Current FPS: 60.0".to_string()
        }));
    }
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// 注册命令
    pub fn register<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(&[String]) -> String + Send + Sync + 'static,
    {
        self.commands.insert(name.to_string(), Box::new(handler));
    }

    /// 执行命令
    pub fn execute(&self, command_line: &str) -> String {
        let parts: Vec<String> = command_line.split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if parts.is_empty() {
            return "Empty command".to_string();
        }

        let command_name = &parts[0];
        let args = &parts[1..];

        if let Some(handler) = self.commands.get(command_name) {
            handler(args)
        } else {
            format!("Unknown command: {}", command_name)
        }
    }
}

impl ProfilerVisualizer {
    pub fn new() -> Self {
        Self {
            frame_time_graph: GraphData::default(),
            call_tree_view: TreeData { nodes: Vec::new() },
            hotspots_table: TableData::default(),
            timeline_view: TimelineData::default(),
        }
    }

    /// 更新可视化数据
    pub fn update(&mut self, profiler_data: &crate::performance::ProfilerExportData) {
        self.update_frame_time_graph(profiler_data);
        self.update_call_tree(profiler_data);
        self.update_hotspots_table(profiler_data);
        self.update_timeline(profiler_data);
    }

    fn update_frame_time_graph(&mut self, _data: &crate::performance::ProfilerExportData) {
        // TODO: 实现帧时间图表更新
    }

    fn update_call_tree(&mut self, _data: &crate::performance::ProfilerExportData) {
        // TODO: 实现调用树更新
    }

    fn update_hotspots_table(&mut self, _data: &crate::performance::ProfilerExportData) {
        // TODO: 实现热点表格更新
    }

    fn update_timeline(&mut self, _data: &crate::performance::ProfilerExportData) {
        // TODO: 实现时间线更新
    }

    /// 导出数据
    pub fn export_data(&self) -> String {
        serde_json::to_string_pretty(&self.frame_time_graph).unwrap_or_default()
    }
}

impl MemoryVisualizer {
    pub fn new() -> Self {
        Self {
            usage_graph: GraphData::default(),
            allocation_heatmap: HeatmapData::default(),
            leak_candidates_table: TableData::default(),
            memory_map: MemoryMapData::default(),
        }
    }

    /// 更新可视化数据
    pub fn update(&mut self, memory_data: &crate::performance::MemoryAnalysis) {
        self.update_usage_graph(memory_data);
        self.update_allocation_heatmap(memory_data);
        self.update_leak_candidates(memory_data);
        self.update_memory_map(memory_data);
    }

    fn update_usage_graph(&mut self, _data: &crate::performance::MemoryAnalysis) {
        // TODO: 实现内存使用图表更新
    }

    fn update_allocation_heatmap(&mut self, _data: &crate::performance::MemoryAnalysis) {
        // TODO: 实现分配热力图更新
    }

    fn update_leak_candidates(&mut self, _data: &crate::performance::MemoryAnalysis) {
        // TODO: 实现泄漏候选表格更新
    }

    fn update_memory_map(&mut self, _data: &crate::performance::MemoryAnalysis) {
        // TODO: 实现内存映射更新
    }

    /// 导出数据
    pub fn export_data(&self) -> String {
        serde_json::to_string_pretty(&self.usage_graph).unwrap_or_default()
    }
}

impl FrameGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            layout: GraphLayout::default(),
        }
    }

    /// 添加节点
    pub fn add_node(&mut self, node: FrameGraphNode) {
        self.nodes.push(node);
    }

    /// 添加边
    pub fn add_edge(&mut self, edge: FrameGraphEdge) {
        self.edges.push(edge);
    }

    /// 更新布局
    pub fn update_layout(&mut self) {
        match self.layout.algorithm {
            LayoutAlgorithm::Hierarchical => self.hierarchical_layout(),
            LayoutAlgorithm::Force => self.force_layout(),
            LayoutAlgorithm::Circular => self.circular_layout(),
            LayoutAlgorithm::Manual => {}, // 手动布局不需要更新
        }
    }

    fn hierarchical_layout(&mut self) {
        // TODO: 实现层次化布局算法
    }

    fn force_layout(&mut self) {
        // TODO: 实现力导向布局算法
    }

    fn circular_layout(&mut self) {
        // TODO: 实现圆形布局算法
    }
}

impl HotReloadManager {
    pub fn new() -> Self {
        Self {
            enabled: true,
            watched_files: HashMap::new(),
            reload_handlers: HashMap::new(),
        }
    }

    /// 注册处理器
    pub fn register_handler<F>(&mut self, pattern: String, handler: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.reload_handlers.insert(pattern, Box::new(handler));
    }

    /// 添加监视文件
    pub fn watch_file(&mut self, path: String, file_type: FileType) {
        let metadata = std::fs::metadata(&path).ok();
        let last_modified = metadata
            .and_then(|m| m.modified().ok())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

        let watcher = FileWatcher {
            path: path.clone(),
            last_modified,
            file_type,
        };

        self.watched_files.insert(path, watcher);
    }

    /// 检查并重载
    pub fn check_and_reload(&mut self) {
        if !self.enabled {
            return;
        }

        let mut changed_files = Vec::new();

        for (path, watcher) in &mut self.watched_files {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    if modified > watcher.last_modified {
                        watcher.last_modified = modified;
                        changed_files.push(path.clone());
                    }
                }
            }
        }

        for file_path in changed_files {
            for (pattern, handler) in &self.reload_handlers {
                if file_path.contains(pattern) {
                    handler(&file_path);
                    break;
                }
            }
        }
    }
}

/// 调试导出数据
#[derive(Debug, Clone, Serialize)]
pub struct DebugExport {
    pub timestamp: u128,
    pub console_messages: Vec<ConsoleMessage>,
    pub panel_states: HashMap<String, DebugPanel>,
    pub performance_data: String,
    pub memory_data: String,
}

// 默认实现
impl Default for OverlayLayout {
    fn default() -> Self {
        Self {
            dock_areas: Vec::new(),
            floating_panels: Vec::new(),
        }
    }
}

impl Default for OverlayStyle {
    fn default() -> Self {
        Self {
            background_color: [0.0, 0.0, 0.0, 0.8],
            text_color: [1.0, 1.0, 1.0, 1.0],
            border_color: [0.5, 0.5, 0.5, 1.0],
            font_size: 14.0,
            transparency: 0.9,
        }
    }
}

impl Default for GraphData {
    fn default() -> Self {
        Self {
            series: Vec::new(),
            x_axis: AxisConfig {
                label: "X".to_string(),
                min: None,
                max: None,
                auto_scale: true,
            },
            y_axis: AxisConfig {
                label: "Y".to_string(),
                min: None,
                max: None,
                auto_scale: true,
            },
            legend: true,
            grid: true,
        }
    }
}

impl Default for TableData {
    fn default() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
            sortable: true,
            filterable: true,
        }
    }
}

impl Default for HeatmapData {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            width: 0,
            height: 0,
            min_value: 0.0,
            max_value: 1.0,
            color_scheme: ColorScheme::Heat,
        }
    }
}

impl Default for MemoryMapData {
    fn default() -> Self {
        Self {
            regions: Vec::new(),
            total_size: 0,
            scale: 1.0,
        }
    }
}

impl Default for TimelineData {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            duration: Duration::ZERO,
            scale: 1.0,
        }
    }
}

impl Default for GraphLayout {
    fn default() -> Self {
        Self {
            algorithm: LayoutAlgorithm::Hierarchical,
            spacing: 50.0,
            layers: true,
        }
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}
