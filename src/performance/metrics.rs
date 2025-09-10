//! 性能指标收集器

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::Serialize;

/// 指标收集器
pub struct MetricsCollector {
    metrics: HashMap<String, Metric>,
    counters: HashMap<String, Counter>,
    timers: HashMap<String, Timer>,
    histograms: HashMap<String, Histogram>,
    gauges: HashMap<String, Gauge>,
    enabled: bool,
}

/// 指标类型
#[derive(Debug, Clone, Serialize)]
pub enum MetricType {
    Counter,
    Timer,
    Histogram,
    Gauge,
}

/// 通用指标
#[derive(Debug, Clone, Serialize)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub unit: String,
    pub description: String,
    pub last_updated: u128, // 时间戳（毫秒）
    pub tags: HashMap<String, String>,
}

/// 计数器
#[derive(Debug, Clone)]
pub struct Counter {
    name: String,
    value: u64,
    description: String,
    tags: HashMap<String, String>,
    created_at: Instant,
}

/// 计时器
#[derive(Debug, Clone)]
pub struct Timer {
    name: String,
    total_duration: Duration,
    count: u64,
    min_duration: Duration,
    max_duration: Duration,
    description: String,
    tags: HashMap<String, String>,
    active_timings: HashMap<String, Instant>, // session_id -> start_time
}

/// 直方图
#[derive(Debug, Clone)]
pub struct Histogram {
    name: String,
    buckets: Vec<f64>,
    counts: Vec<u64>,
    total_count: u64,
    sum: f64,
    description: String,
    tags: HashMap<String, String>,
}

/// 仪表
#[derive(Debug, Clone)]
pub struct Gauge {
    name: String,
    value: f64,
    min_value: f64,
    max_value: f64,
    description: String,
    tags: HashMap<String, String>,
    history: Vec<(u128, f64)>, // (timestamp, value)
    max_history: usize,
}

/// 指标摘要
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSummary {
    pub total_metrics: usize,
    pub counters: usize,
    pub timers: usize,
    pub histograms: usize,
    pub gauges: usize,
    pub collection_period: Duration,
    pub last_collection: u128,
}

/// 定时器守卫
pub struct TimerGuard<'a> {
    timer: &'a mut Timer,
    session_id: String,
    start_time: Instant,
}

impl<'a> Drop for TimerGuard<'a> {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        self.timer.record_duration(duration);
        self.timer.active_timings.remove(&self.session_id);
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            counters: HashMap::new(),
            timers: HashMap::new(),
            histograms: HashMap::new(),
            gauges: HashMap::new(),
            enabled: true,
        }
    }

    /// 启用/禁用收集器
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 创建计数器
    pub fn create_counter(&mut self, name: String, description: String) -> &mut Counter {
        let counter = Counter {
            name: name.clone(),
            value: 0,
            description,
            tags: HashMap::new(),
            created_at: Instant::now(),
        };
        
        self.counters.insert(name.clone(), counter);
        self.counters.get_mut(&name).unwrap()
    }

    /// 创建计时器
    pub fn create_timer(&mut self, name: String, description: String) -> &mut Timer {
        let timer = Timer {
            name: name.clone(),
            total_duration: Duration::ZERO,
            count: 0,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            description,
            tags: HashMap::new(),
            active_timings: HashMap::new(),
        };
        
        self.timers.insert(name.clone(), timer);
        self.timers.get_mut(&name).unwrap()
    }

    /// 创建直方图
    pub fn create_histogram(&mut self, name: String, buckets: Vec<f64>, description: String) -> &mut Histogram {
        let histogram = Histogram {
            name: name.clone(),
            buckets: buckets.clone(),
            counts: vec![0; buckets.len()],
            total_count: 0,
            sum: 0.0,
            description,
            tags: HashMap::new(),
        };
        
        self.histograms.insert(name.clone(), histogram);
        self.histograms.get_mut(&name).unwrap()
    }

    /// 创建仪表
    pub fn create_gauge(&mut self, name: String, description: String) -> &mut Gauge {
        let gauge = Gauge {
            name: name.clone(),
            value: 0.0,
            min_value: f64::MAX,
            max_value: f64::MIN,
            description,
            tags: HashMap::new(),
            history: Vec::new(),
            max_history: 1000,
        };
        
        self.gauges.insert(name.clone(), gauge);
        self.gauges.get_mut(&name).unwrap()
    }

    /// 增加计数器
    pub fn increment_counter(&mut self, name: &str, value: u64) {
        if !self.enabled {
            return;
        }

        if let Some(counter) = self.counters.get_mut(name) {
            counter.increment(value);
        }
    }

    /// 记录计时器
    pub fn record_timer(&mut self, name: &str, duration: Duration) {
        if !self.enabled {
            return;
        }

        if let Some(timer) = self.timers.get_mut(name) {
            timer.record_duration(duration);
        }
    }

    /// 开始计时
    pub fn start_timer(&mut self, name: &str) -> Option<TimerGuard> {
        if !self.enabled {
            return None;
        }

        if let Some(timer) = self.timers.get_mut(name) {
            let session_id = format!("{}_{}", name, timer.active_timings.len());
            let start_time = Instant::now();
            timer.active_timings.insert(session_id.clone(), start_time);
            
            Some(TimerGuard {
                timer,
                session_id,
                start_time,
            })
        } else {
            None
        }
    }

    /// 记录直方图值
    pub fn record_histogram(&mut self, name: &str, value: f64) {
        if !self.enabled {
            return;
        }

        if let Some(histogram) = self.histograms.get_mut(name) {
            histogram.record(value);
        }
    }

    /// 设置仪表值
    pub fn set_gauge(&mut self, name: &str, value: f64) {
        if !self.enabled {
            return;
        }

        if let Some(gauge) = self.gauges.get_mut(name) {
            gauge.set_value(value);
        }
    }

    /// 增加仪表值
    pub fn increment_gauge(&mut self, name: &str, value: f64) {
        if !self.enabled {
            return;
        }

        if let Some(gauge) = self.gauges.get_mut(name) {
            gauge.increment(value);
        }
    }

    /// 记录通用指标
    pub fn record(&mut self, name: &str, value: f64) {
        if !self.enabled {
            return;
        }

        let metric = Metric {
            name: name.to_string(),
            metric_type: MetricType::Gauge,
            value,
            unit: "".to_string(),
            description: "".to_string(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            tags: HashMap::new(),
        };

        self.metrics.insert(name.to_string(), metric);
    }

    /// 获取所有指标
    pub fn get_all_metrics(&self) -> HashMap<String, f64> {
        let mut result = HashMap::new();

        // 添加通用指标
        for (name, metric) in &self.metrics {
            result.insert(name.clone(), metric.value);
        }

        // 添加计数器
        for (name, counter) in &self.counters {
            result.insert(format!("{}_total", name), counter.value as f64);
        }

        // 添加计时器
        for (name, timer) in &self.timers {
            if timer.count > 0 {
                result.insert(format!("{}_avg_ms", name), timer.average_duration().as_millis() as f64);
                result.insert(format!("{}_min_ms", name), timer.min_duration.as_millis() as f64);
                result.insert(format!("{}_max_ms", name), timer.max_duration.as_millis() as f64);
                result.insert(format!("{}_count", name), timer.count as f64);
            }
        }

        // 添加仪表
        for (name, gauge) in &self.gauges {
            result.insert(name.clone(), gauge.value);
            result.insert(format!("{}_min", name), gauge.min_value);
            result.insert(format!("{}_max", name), gauge.max_value);
        }

        // 添加直方图
        for (name, histogram) in &self.histograms {
            result.insert(format!("{}_count", name), histogram.total_count as f64);
            result.insert(format!("{}_sum", name), histogram.sum);
            if histogram.total_count > 0 {
                result.insert(format!("{}_avg", name), histogram.sum / histogram.total_count as f64);
            }
        }

        result
    }

    /// 获取指标摘要
    pub fn get_summary(&self) -> MetricsSummary {
        MetricsSummary {
            total_metrics: self.metrics.len() + self.counters.len() + self.timers.len() + 
                          self.histograms.len() + self.gauges.len(),
            counters: self.counters.len(),
            timers: self.timers.len(),
            histograms: self.histograms.len(),
            gauges: self.gauges.len(),
            collection_period: Duration::from_millis(100), // 假设100ms收集周期
            last_collection: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        }
    }

    /// 导出Prometheus格式
    pub fn export_prometheus(&self) -> String {
        let mut output = String::new();

        // 导出计数器
        for (name, counter) in &self.counters {
            output.push_str(&format!("# HELP {} {}\n", name, counter.description));
            output.push_str(&format!("# TYPE {} counter\n", name));
            output.push_str(&format!("{} {}\n", name, counter.value));
        }

        // 导出仪表
        for (name, gauge) in &self.gauges {
            output.push_str(&format!("# HELP {} {}\n", name, gauge.description));
            output.push_str(&format!("# TYPE {} gauge\n", name));
            output.push_str(&format!("{} {}\n", name, gauge.value));
        }

        // 导出直方图
        for (name, histogram) in &self.histograms {
            output.push_str(&format!("# HELP {} {}\n", name, histogram.description));
            output.push_str(&format!("# TYPE {} histogram\n", name));
            
            for (i, &bucket) in histogram.buckets.iter().enumerate() {
                output.push_str(&format!("{}_bucket{{le=\"{}\"}} {}\n", name, bucket, histogram.counts[i]));
            }
            output.push_str(&format!("{}_bucket{{le=\"+Inf\"}} {}\n", name, histogram.total_count));
            output.push_str(&format!("{}_count {}\n", name, histogram.total_count));
            output.push_str(&format!("{}_sum {}\n", name, histogram.sum));
        }

        output
    }

    /// 重置所有指标
    pub fn reset(&mut self) {
        self.metrics.clear();
        
        for counter in self.counters.values_mut() {
            counter.reset();
        }
        
        for timer in self.timers.values_mut() {
            timer.reset();
        }
        
        for histogram in self.histograms.values_mut() {
            histogram.reset();
        }
        
        for gauge in self.gauges.values_mut() {
            gauge.reset();
        }
    }

    /// 移除指标
    pub fn remove_metric(&mut self, name: &str) {
        self.metrics.remove(name);
        self.counters.remove(name);
        self.timers.remove(name);
        self.histograms.remove(name);
        self.gauges.remove(name);
    }

    /// 获取内存使用量
    pub fn get_memory_usage(&self) -> usize {
        let metrics_size = self.metrics.len() * std::mem::size_of::<Metric>();
        let counters_size = self.counters.len() * std::mem::size_of::<Counter>();
        let timers_size = self.timers.len() * std::mem::size_of::<Timer>();
        let histograms_size = self.histograms.len() * std::mem::size_of::<Histogram>();
        let gauges_size = self.gauges.len() * std::mem::size_of::<Gauge>();
        
        metrics_size + counters_size + timers_size + histograms_size + gauges_size
    }
}

impl Counter {
    /// 增加计数
    pub fn increment(&mut self, value: u64) {
        self.value += value;
    }

    /// 重置计数器
    pub fn reset(&mut self) {
        self.value = 0;
        self.created_at = Instant::now();
    }

    /// 获取每秒速率
    pub fn rate_per_second(&self) -> f64 {
        let elapsed = self.created_at.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.value as f64 / elapsed
        } else {
            0.0
        }
    }

    /// 添加标签
    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }
}

impl Timer {
    /// 记录持续时间
    pub fn record_duration(&mut self, duration: Duration) {
        self.total_duration += duration;
        self.count += 1;
        self.min_duration = self.min_duration.min(duration);
        self.max_duration = self.max_duration.max(duration);
    }

    /// 获取平均持续时间
    pub fn average_duration(&self) -> Duration {
        if self.count > 0 {
            self.total_duration / self.count as u32
        } else {
            Duration::ZERO
        }
    }

    /// 重置计时器
    pub fn reset(&mut self) {
        self.total_duration = Duration::ZERO;
        self.count = 0;
        self.min_duration = Duration::MAX;
        self.max_duration = Duration::ZERO;
        self.active_timings.clear();
    }

    /// 添加标签
    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }
}

impl Histogram {
    /// 记录值
    pub fn record(&mut self, value: f64) {
        self.sum += value;
        self.total_count += 1;

        // 找到合适的桶
        for (i, &bucket) in self.buckets.iter().enumerate() {
            if value <= bucket {
                self.counts[i] += 1;
            }
        }
    }

    /// 获取百分位数
    pub fn percentile(&self, p: f64) -> Option<f64> {
        if self.total_count == 0 {
            return None;
        }

        let target_count = (self.total_count as f64 * p / 100.0) as u64;
        let mut accumulated = 0;

        for (i, &count) in self.counts.iter().enumerate() {
            accumulated += count;
            if accumulated >= target_count {
                return Some(self.buckets[i]);
            }
        }

        self.buckets.last().copied()
    }

    /// 重置直方图
    pub fn reset(&mut self) {
        self.counts.fill(0);
        self.total_count = 0;
        self.sum = 0.0;
    }

    /// 添加标签
    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }
}

impl Gauge {
    /// 设置值
    pub fn set_value(&mut self, value: f64) {
        self.value = value;
        self.min_value = self.min_value.min(value);
        self.max_value = self.max_value.max(value);
        
        // 记录历史
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        self.history.push((timestamp, value));
        
        // 保持历史大小
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// 增加值
    pub fn increment(&mut self, delta: f64) {
        self.set_value(self.value + delta);
    }

    /// 减少值
    pub fn decrement(&mut self, delta: f64) {
        self.set_value(self.value - delta);
    }

    /// 重置仪表
    pub fn reset(&mut self) {
        self.value = 0.0;
        self.min_value = f64::MAX;
        self.max_value = f64::MIN;
        self.history.clear();
    }

    /// 获取趋势
    pub fn get_trend(&self) -> TrendDirection {
        if self.history.len() < 2 {
            return TrendDirection::Stable;
        }

        let recent_count = (self.history.len() / 4).max(2);
        let recent_values: Vec<f64> = self.history
            .iter()
            .rev()
            .take(recent_count)
            .map(|(_, v)| *v)
            .collect();

        let first_half: f64 = recent_values[recent_count/2..].iter().sum::<f64>() / (recent_count - recent_count/2) as f64;
        let second_half: f64 = recent_values[..recent_count/2].iter().sum::<f64>() / (recent_count/2) as f64;

        let change_rate = (second_half - first_half) / first_half.abs().max(1.0);

        if change_rate > 0.05 {
            TrendDirection::Increasing
        } else if change_rate < -0.05 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }

    /// 添加标签
    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }
}

/// 趋势方向
#[derive(Debug, Clone, Serialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// 指标宏
#[macro_export]
macro_rules! increment_counter {
    ($collector:expr, $name:expr) => {
        $collector.increment_counter($name, 1);
    };
    ($collector:expr, $name:expr, $value:expr) => {
        $collector.increment_counter($name, $value);
    };
}

#[macro_export]
macro_rules! time_block {
    ($collector:expr, $name:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        $collector.record_timer($name, start.elapsed());
        result
    }};
}

#[macro_export]
macro_rules! record_gauge {
    ($collector:expr, $name:expr, $value:expr) => {
        $collector.set_gauge($name, $value);
    };
}
