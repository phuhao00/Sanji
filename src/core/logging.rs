//! 日志和调试系统

use log::{Level, LevelFilter};
use std::io::Write;
use std::sync::Mutex;
use chrono::{DateTime, Utc};

/// 日志配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogConfig {
    /// 日志级别
    pub level: String,
    /// 是否输出到控制台
    pub console_output: bool,
    /// 是否输出到文件
    pub file_output: bool,
    /// 日志文件路径
    pub file_path: String,
    /// 是否显示时间戳
    pub show_timestamp: bool,
    /// 是否显示线程ID
    pub show_thread_id: bool,
    /// 是否显示文件位置
    pub show_location: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            console_output: true,
            file_output: false,
            file_path: "sanji_engine.log".to_string(),
            show_timestamp: true,
            show_thread_id: false,
            show_location: false,
        }
    }
}

/// 自定义日志格式化器
struct SanjiLogger {
    config: LogConfig,
    file_writer: Option<Mutex<std::fs::File>>,
}

impl SanjiLogger {
    fn new(config: LogConfig) -> crate::EngineResult<Self> {
        let file_writer = if config.file_output {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&config.file_path)
                .map_err(|e| crate::EngineError::IoError(e))?;
            Some(Mutex::new(file))
        } else {
            None
        };

        Ok(Self {
            config,
            file_writer,
        })
    }

    fn format_message(&self, record: &log::Record) -> String {
        let mut message = String::new();

        // 时间戳
        if self.config.show_timestamp {
            let now: DateTime<Utc> = Utc::now();
            message.push_str(&format!("[{}] ", now.format("%Y-%m-%d %H:%M:%S%.3f")));
        }

        // 日志级别
        let level_color = match record.level() {
            Level::Error => "\x1b[31m", // 红色
            Level::Warn => "\x1b[33m",  // 黄色
            Level::Info => "\x1b[32m",  // 绿色
            Level::Debug => "\x1b[34m", // 蓝色
            Level::Trace => "\x1b[37m", // 白色
        };
        message.push_str(&format!("{}[{}]\x1b[0m ", level_color, record.level()));

        // 线程ID
        if self.config.show_thread_id {
            message.push_str(&format!("[{:?}] ", std::thread::current().id()));
        }

        // 模块路径
        if let Some(module) = record.module_path() {
            message.push_str(&format!("{}: ", module));
        }

        // 消息内容
        message.push_str(&record.args().to_string());

        // 文件位置
        if self.config.show_location {
            if let (Some(file), Some(line)) = (record.file(), record.line()) {
                message.push_str(&format!(" ({}:{})", file, line));
            }
        }

        message
    }
}

impl log::Log for SanjiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        let level_filter = match self.config.level.as_str() {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            "off" => LevelFilter::Off,
            _ => LevelFilter::Info,
        };
        metadata.level() <= level_filter
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let message = self.format_message(record);

            // 输出到控制台
            if self.config.console_output {
                println!("{}", message);
            }

            // 输出到文件
            if let Some(ref file_writer) = self.file_writer {
                if let Ok(mut file) = file_writer.lock() {
                    let file_message = format!("{}\n", message.replace("\x1b[31m", "").replace("\x1b[33m", "").replace("\x1b[32m", "").replace("\x1b[34m", "").replace("\x1b[37m", "").replace("\x1b[0m", ""));
                    let _ = file.write_all(file_message.as_bytes());
                    let _ = file.flush();
                }
            }
        }
    }

    fn flush(&self) {
        if let Some(ref file_writer) = self.file_writer {
            if let Ok(mut file) = file_writer.lock() {
                let _ = file.flush();
            }
        }
    }
}

/// 初始化日志系统
pub fn init_logging(config: LogConfig) -> crate::EngineResult<()> {
    let logger = SanjiLogger::new(config)?;
    let level_str = logger.config.level.clone();
    
    log::set_boxed_logger(Box::new(logger))
        .map_err(|e| crate::EngineError::RenderError(format!("初始化日志系统失败: {}", e)))?;
    
    let level_filter = match level_str.as_str() {
        "trace" => LevelFilter::Trace,
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        "off" => LevelFilter::Off,
        _ => LevelFilter::Info,
    };
    
    log::set_max_level(level_filter);
    
    log::info!("Sanji引擎日志系统已初始化");
    Ok(())
}

/// 调试统计信息
#[derive(Debug, Default)]
pub struct DebugStats {
    pub frame_count: u64,
    pub draw_calls: u32,
    pub vertices_rendered: u32,
    pub triangles_rendered: u32,
    pub textures_loaded: u32,
    pub memory_used_mb: f32,
    pub cpu_time_ms: f32,
    pub gpu_time_ms: f32,
}

impl DebugStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset_frame_stats(&mut self) {
        self.draw_calls = 0;
        self.vertices_rendered = 0;
        self.triangles_rendered = 0;
        self.cpu_time_ms = 0.0;
        self.gpu_time_ms = 0.0;
    }

    pub fn add_draw_call(&mut self, vertices: u32) {
        self.draw_calls += 1;
        self.vertices_rendered += vertices;
        self.triangles_rendered += vertices / 3;
    }

    pub fn print_stats(&self) {
        log::debug!("=== 引擎统计 ===");
        log::debug!("帧数: {}", self.frame_count);
        log::debug!("绘制调用: {}", self.draw_calls);
        log::debug!("顶点数: {}", self.vertices_rendered);
        log::debug!("三角形数: {}", self.triangles_rendered);
        log::debug!("已加载纹理: {}", self.textures_loaded);
        log::debug!("内存使用: {:.2} MB", self.memory_used_mb);
        log::debug!("CPU时间: {:.2} ms", self.cpu_time_ms);
        log::debug!("GPU时间: {:.2} ms", self.gpu_time_ms);
        log::debug!("==================");
    }
}

/// 性能分析器
pub struct Profiler {
    start_time: std::time::Instant,
    samples: Vec<f32>,
    current_sample: usize,
    max_samples: usize,
}

impl Profiler {
    pub fn new(max_samples: usize) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            samples: Vec::with_capacity(max_samples),
            current_sample: 0,
            max_samples,
        }
    }

    pub fn begin_sample(&mut self) {
        self.start_time = std::time::Instant::now();
    }

    pub fn end_sample(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f32() * 1000.0; // 转换为毫秒
        
        if self.samples.len() < self.max_samples {
            self.samples.push(elapsed);
        } else {
            self.samples[self.current_sample] = elapsed;
            self.current_sample = (self.current_sample + 1) % self.max_samples;
        }
    }

    pub fn average_time(&self) -> f32 {
        if self.samples.is_empty() {
            0.0
        } else {
            self.samples.iter().sum::<f32>() / self.samples.len() as f32
        }
    }

    pub fn min_time(&self) -> f32 {
        self.samples.iter().fold(f32::INFINITY, |a, &b| a.min(b))
    }

    pub fn max_time(&self) -> f32 {
        self.samples.iter().fold(0.0, |a, &b| a.max(b))
    }

    pub fn print_stats(&self, name: &str) {
        if !self.samples.is_empty() {
            log::debug!("{}: 平均 {:.2}ms, 最小 {:.2}ms, 最大 {:.2}ms", 
                name, self.average_time(), self.min_time(), self.max_time());
        }
    }
}

/// 日志宏扩展
#[macro_export]
macro_rules! engine_trace {
    ($($arg:tt)*) => {
        log::trace!(target: "sanji_engine", $($arg)*);
    };
}

#[macro_export]
macro_rules! engine_debug {
    ($($arg:tt)*) => {
        log::debug!(target: "sanji_engine", $($arg)*);
    };
}

#[macro_export]
macro_rules! engine_info {
    ($($arg:tt)*) => {
        log::info!(target: "sanji_engine", $($arg)*);
    };
}

#[macro_export]
macro_rules! engine_warn {
    ($($arg:tt)*) => {
        log::warn!(target: "sanji_engine", $($arg)*);
    };
}

#[macro_export]
macro_rules! engine_error {
    ($($arg:tt)*) => {
        log::error!(target: "sanji_engine", $($arg)*);
    };
}

/// 性能监控宏
#[macro_export]
macro_rules! profile_scope {
    ($profiler:expr, $name:expr, $code:block) => {
        $profiler.begin_sample();
        let result = $code;
        $profiler.end_sample();
        $profiler.print_stats($name);
        result
    };
}

/// 断言宏（仅在debug模式下有效）
#[macro_export]
macro_rules! engine_assert {
    ($condition:expr) => {
        debug_assert!($condition, "引擎断言失败: {}", stringify!($condition));
    };
    ($condition:expr, $($arg:tt)*) => {
        debug_assert!($condition, $($arg)*);
    };
}

/// 调试信息显示
pub struct DebugOverlay {
    pub show_fps: bool,
    pub show_stats: bool,
    pub show_profiler: bool,
    pub show_memory: bool,
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self {
            show_fps: true,
            show_stats: false,
            show_profiler: false,
            show_memory: false,
        }
    }
}

impl DebugOverlay {
    pub fn toggle_fps(&mut self) {
        self.show_fps = !self.show_fps;
        log::info!("FPS显示: {}", if self.show_fps { "开启" } else { "关闭" });
    }

    pub fn toggle_stats(&mut self) {
        self.show_stats = !self.show_stats;
        log::info!("统计信息显示: {}", if self.show_stats { "开启" } else { "关闭" });
    }

    pub fn toggle_profiler(&mut self) {
        self.show_profiler = !self.show_profiler;
        log::info!("性能分析显示: {}", if self.show_profiler { "开启" } else { "关闭" });
    }
}
