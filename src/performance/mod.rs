//! 性能分析和调试工具模块

pub mod profiler;
pub mod debugger;
pub mod metrics;
pub mod memory_tracker;
pub mod frame_analyzer;

pub use profiler::*;
pub use debugger::*;
pub use metrics::*;
pub use memory_tracker::*;
pub use frame_analyzer::*;

use crate::EngineResult;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 性能统计数据
#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    pub frame_time: Duration,
    pub fps: f32,
    pub cpu_usage: f32,
    pub memory_usage: MemoryUsage,
    pub render_stats: RenderStats,
    pub physics_stats: PhysicsStats,
    pub audio_stats: AudioStats,
    pub custom_stats: HashMap<String, f64>,
}

/// 内存使用统计
#[derive(Debug, Clone, Default)]
pub struct MemoryUsage {
    pub total_allocated: usize,
    pub peak_allocated: usize,
    pub current_allocated: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
    pub heap_size: usize,
    pub stack_size: usize,
}

/// 渲染统计数据
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    pub draw_calls: u32,
    pub triangles: u32,
    pub vertices: u32,
    pub texture_switches: u32,
    pub render_targets_switches: u32,
    pub shader_switches: u32,
    pub gpu_memory_usage: usize,
    pub gpu_time: Duration,
}

/// 物理统计数据
#[derive(Debug, Clone, Default)]
pub struct PhysicsStats {
    pub rigid_bodies: usize,
    pub colliders: usize,
    pub active_bodies: usize,
    pub collision_pairs: usize,
    pub simulation_time: Duration,
    pub broad_phase_time: Duration,
    pub narrow_phase_time: Duration,
    pub solver_time: Duration,
}

/// 音频统计数据
#[derive(Debug, Clone, Default)]
pub struct AudioStats {
    pub active_sources: usize,
    pub total_sources: usize,
    pub streaming_sources: usize,
    pub buffer_underruns: usize,
    pub cpu_usage: f32,
    pub latency: Duration,
}

/// 性能监控器
pub struct PerformanceMonitor {
    profiler: Profiler,
    memory_tracker: MemoryTracker,
    frame_analyzer: FrameAnalyzer,
    metrics_collector: MetricsCollector,
    
    // 采样和历史记录
    sample_interval: Duration,
    last_sample_time: Instant,
    history_size: usize,
    stats_history: Vec<PerformanceStats>,
    
    // 配置
    enabled: bool,
    detailed_profiling: bool,
    memory_tracking: bool,
    gpu_profiling: bool,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            profiler: Profiler::new(),
            memory_tracker: MemoryTracker::new(),
            frame_analyzer: FrameAnalyzer::new(),
            metrics_collector: MetricsCollector::new(),
            sample_interval: Duration::from_millis(16), // ~60 FPS
            last_sample_time: Instant::now(),
            history_size: 300, // 5秒历史 @ 60 FPS
            stats_history: Vec::new(),
            enabled: true,
            detailed_profiling: false,
            memory_tracking: true,
            gpu_profiling: false,
        }
    }

    /// 设置采样间隔
    pub fn set_sample_interval(&mut self, interval: Duration) {
        self.sample_interval = interval;
    }

    /// 设置历史记录大小
    pub fn set_history_size(&mut self, size: usize) {
        self.history_size = size;
        if self.stats_history.len() > size {
            self.stats_history.truncate(size);
        }
    }

    /// 启用/禁用监控
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            self.profiler.reset();
            self.memory_tracker.reset();
            self.frame_analyzer.reset();
        }
    }

    /// 启用/禁用详细性能分析
    pub fn set_detailed_profiling(&mut self, enabled: bool) {
        self.detailed_profiling = enabled;
    }

    /// 启用/禁用内存跟踪
    pub fn set_memory_tracking(&mut self, enabled: bool) {
        self.memory_tracking = enabled;
    }

    /// 启用/禁用GPU性能分析
    pub fn set_gpu_profiling(&mut self, enabled: bool) {
        self.gpu_profiling = enabled;
    }

    /// 开始帧性能分析
    pub fn begin_frame(&mut self) {
        if !self.enabled {
            return;
        }

        self.frame_analyzer.begin_frame();
        
        if self.detailed_profiling {
            self.profiler.begin_frame();
        }
    }

    /// 结束帧性能分析
    pub fn end_frame(&mut self) {
        if !self.enabled {
            return;
        }

        self.frame_analyzer.end_frame();

        // 检查是否需要采样
        let now = Instant::now();
        if now.duration_since(self.last_sample_time) >= self.sample_interval {
            self.sample_performance();
            self.last_sample_time = now;
        }
    }

    /// 开始性能区域分析
    pub fn begin_section(&mut self, name: &str) -> ProfilerGuard {
        if self.enabled && self.detailed_profiling {
            self.profiler.begin_section(name)
        } else {
            ProfilerGuard::disabled()
        }
    }

    /// 记录自定义指标
    pub fn record_metric(&mut self, name: &str, value: f64) {
        if self.enabled {
            self.metrics_collector.record(name, value);
        }
    }

    /// 获取当前性能统计
    pub fn get_current_stats(&self) -> PerformanceStats {
        if !self.enabled {
            return PerformanceStats::default();
        }

        let frame_stats = self.frame_analyzer.get_stats();
        let memory_stats = if self.memory_tracking {
            self.memory_tracker.get_stats()
        } else {
            MemoryUsage::default()
        };

        PerformanceStats {
            frame_time: frame_stats.average_frame_time,
            fps: frame_stats.fps,
            cpu_usage: self.get_cpu_usage(),
            memory_usage: memory_stats,
            render_stats: RenderStats::default(), // TODO: 从渲染系统获取
            physics_stats: PhysicsStats::default(), // TODO: 从物理系统获取
            audio_stats: AudioStats::default(), // TODO: 从音频系统获取
            custom_stats: self.metrics_collector.get_all_metrics(),
        }
    }

    /// 获取历史统计数据
    pub fn get_stats_history(&self) -> &[PerformanceStats] {
        &self.stats_history
    }

    /// 获取性能报告
    pub fn generate_report(&self) -> PerformanceReport {
        PerformanceReport {
            summary: self.get_performance_summary(),
            detailed_breakdown: if self.detailed_profiling {
                Some(self.profiler.get_detailed_breakdown())
            } else {
                None
            },
            memory_analysis: if self.memory_tracking {
                Some(self.memory_tracker.get_analysis())
            } else {
                None
            },
            frame_analysis: self.frame_analyzer.get_analysis(),
            recommendations: self.generate_recommendations(),
        }
    }

    /// 导出性能数据
    pub fn export_data(&self, format: ExportFormat) -> EngineResult<String> {
        let report = self.generate_report();
        
        match format {
            ExportFormat::Json => {
                Ok(serde_json::to_string_pretty(&report)?)
            }
            ExportFormat::Csv => {
                self.export_csv(&report)
            }
            ExportFormat::Html => {
                self.export_html(&report)
            }
        }
    }

    /// 重置所有统计数据
    pub fn reset(&mut self) {
        self.profiler.reset();
        self.memory_tracker.reset();
        self.frame_analyzer.reset();
        self.metrics_collector.reset();
        self.stats_history.clear();
        self.last_sample_time = Instant::now();
    }

    /// 采样性能数据
    fn sample_performance(&mut self) {
        let stats = self.get_current_stats();
        
        // 添加到历史记录
        self.stats_history.push(stats);
        
        // 保持历史记录大小
        if self.stats_history.len() > self.history_size {
            self.stats_history.remove(0);
        }
    }

    /// 获取CPU使用率
    fn get_cpu_usage(&self) -> f32 {
        // TODO: 实现CPU使用率监控
        // 这需要平台特定的实现
        0.0
    }

    /// 获取性能摘要
    fn get_performance_summary(&self) -> PerformanceSummary {
        if self.stats_history.is_empty() {
            return PerformanceSummary::default();
        }

        let mut min_fps = f32::MAX;
        let mut max_fps = f32::MIN;
        let mut total_fps = 0.0;
        let mut frame_time_sum = Duration::ZERO;

        for stats in &self.stats_history {
            min_fps = min_fps.min(stats.fps);
            max_fps = max_fps.max(stats.fps);
            total_fps += stats.fps;
            frame_time_sum += stats.frame_time;
        }

        let count = self.stats_history.len() as f32;
        
        PerformanceSummary {
            average_fps: total_fps / count,
            min_fps,
            max_fps,
            average_frame_time: frame_time_sum / self.stats_history.len() as u32,
            total_samples: self.stats_history.len(),
            peak_memory: self.stats_history.iter()
                .map(|s| s.memory_usage.peak_allocated)
                .max()
                .unwrap_or(0),
        }
    }

    /// 生成性能优化建议
    fn generate_recommendations(&self) -> Vec<PerformanceRecommendation> {
        let mut recommendations = Vec::new();
        let summary = self.get_performance_summary();

        // FPS相关建议
        if summary.average_fps < 30.0 {
            recommendations.push(PerformanceRecommendation {
                category: "FPS".to_string(),
                severity: Severity::High,
                title: "低帧率检测".to_string(),
                description: format!("平均FPS为{:.1}，低于30FPS阈值", summary.average_fps),
                suggestions: vec![
                    "检查渲染管线瓶颈".to_string(),
                    "优化着色器复杂度".to_string(),
                    "减少绘制调用数量".to_string(),
                    "启用遮挡剔除".to_string(),
                ],
            });
        }

        // 内存相关建议
        if summary.peak_memory > 1024 * 1024 * 1024 { // 1GB
            recommendations.push(PerformanceRecommendation {
                category: "Memory".to_string(),
                severity: Severity::Medium,
                title: "高内存使用".to_string(),
                description: format!("峰值内存使用达到{:.1}MB", summary.peak_memory as f64 / (1024.0 * 1024.0)),
                suggestions: vec![
                    "检查内存泄漏".to_string(),
                    "优化纹理大小和格式".to_string(),
                    "实现对象池".to_string(),
                    "使用流式加载".to_string(),
                ],
            });
        }

        recommendations
    }

    /// 导出CSV格式
    fn export_csv(&self, report: &PerformanceReport) -> EngineResult<String> {
        let mut csv = String::new();
        csv.push_str("Timestamp,FPS,FrameTime(ms),MemoryUsage(MB),CPUUsage(%)\n");
        
        for (i, stats) in self.stats_history.iter().enumerate() {
            csv.push_str(&format!(
                "{},{:.1},{:.2},{:.1},{:.1}\n",
                i,
                stats.fps,
                stats.frame_time.as_millis(),
                stats.memory_usage.current_allocated as f64 / (1024.0 * 1024.0),
                stats.cpu_usage
            ));
        }

        Ok(csv)
    }

    /// 导出HTML格式
    fn export_html(&self, report: &PerformanceReport) -> EngineResult<String> {
        let html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Sanji Engine Performance Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .summary {{ background: #f0f0f0; padding: 10px; border-radius: 5px; }}
        .metric {{ margin: 5px 0; }}
        .recommendation {{ background: #fff3cd; padding: 10px; margin: 10px 0; border-radius: 5px; }}
        .severity-high {{ border-left: 5px solid #dc3545; }}
        .severity-medium {{ border-left: 5px solid #ffc107; }}
        .severity-low {{ border-left: 5px solid #28a745; }}
    </style>
</head>
<body>
    <h1>Sanji Engine Performance Report</h1>
    
    <div class="summary">
        <h2>Performance Summary</h2>
        <div class="metric">Average FPS: {:.1}</div>
        <div class="metric">Min FPS: {:.1}</div>
        <div class="metric">Max FPS: {:.1}</div>
        <div class="metric">Average Frame Time: {:.2}ms</div>
        <div class="metric">Peak Memory: {:.1}MB</div>
    </div>
    
    <h2>Recommendations</h2>
    {}
    
</body>
</html>
            "#,
            report.summary.average_fps,
            report.summary.min_fps,
            report.summary.max_fps,
            report.summary.average_frame_time.as_millis(),
            report.summary.peak_memory as f64 / (1024.0 * 1024.0),
            report.recommendations.iter()
                .map(|r| format!(
                    r#"<div class="recommendation severity-{}">
                        <h3>{}</h3>
                        <p>{}</p>
                        <ul>{}</ul>
                    </div>"#,
                    match r.severity {
                        Severity::High => "high",
                        Severity::Medium => "medium",
                        Severity::Low => "low",
                    },
                    r.title,
                    r.description,
                    r.suggestions.iter()
                        .map(|s| format!("<li>{}</li>", s))
                        .collect::<String>()
                ))
                .collect::<String>()
        );

        Ok(html)
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 导出格式
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Json,
    Csv,
    Html,
}

/// 性能报告
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceReport {
    pub summary: PerformanceSummary,
    pub detailed_breakdown: Option<DetailedBreakdown>,
    pub memory_analysis: Option<MemoryAnalysis>,
    pub frame_analysis: FrameAnalysis,
    pub recommendations: Vec<PerformanceRecommendation>,
}

/// 性能摘要
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct PerformanceSummary {
    pub average_fps: f32,
    pub min_fps: f32,
    pub max_fps: f32,
    pub average_frame_time: Duration,
    pub total_samples: usize,
    pub peak_memory: usize,
}

/// 性能建议
#[derive(Debug, Clone, serde::Serialize)]
pub struct PerformanceRecommendation {
    pub category: String,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub suggestions: Vec<String>,
}

/// 严重程度
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum Severity {
    Low,
    Medium,
    High,
}

/// 全局性能监控实例
static mut GLOBAL_MONITOR: Option<PerformanceMonitor> = None;
static MONITOR_INIT: std::sync::Once = std::sync::Once::new();

/// 获取全局性能监控器
pub fn get_global_monitor() -> &'static mut PerformanceMonitor {
    unsafe {
        MONITOR_INIT.call_once(|| {
            GLOBAL_MONITOR = Some(PerformanceMonitor::new());
        });
        GLOBAL_MONITOR.as_mut().unwrap()
    }
}

/// 性能分析宏
#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        let _guard = $crate::performance::get_global_monitor().begin_section($name);
    };
}

#[macro_export]
macro_rules! profile_function {
    () => {
        profile_scope!(function_name!());
    };
}

#[macro_export]
macro_rules! record_metric {
    ($name:expr, $value:expr) => {
        $crate::performance::get_global_monitor().record_metric($name, $value);
    };
}
