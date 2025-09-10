//! 帧分析器

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use serde::Serialize;

/// 帧分析器
pub struct FrameAnalyzer {
    frame_times: VecDeque<Duration>,
    frame_starts: VecDeque<Instant>,
    current_frame_start: Option<Instant>,
    target_fps: f32,
    max_history: usize,
    total_frames: u64,
    dropped_frames: u64,
    frame_budget: Duration,
    enabled: bool,
}

/// 帧统计数据
#[derive(Debug, Clone, Default, Serialize)]
pub struct FrameStats {
    pub fps: f32,
    pub average_frame_time: Duration,
    pub min_frame_time: Duration,
    pub max_frame_time: Duration,
    pub frame_time_variance: f32,
    pub dropped_frame_percentage: f32,
    pub frames_under_budget: u64,
    pub frames_over_budget: u64,
    pub total_frames: u64,
}

/// 帧分析结果
#[derive(Debug, Clone, Serialize)]
pub struct FrameAnalysis {
    pub stats: FrameStats,
    pub performance_grade: PerformanceGrade,
    pub frame_time_distribution: Vec<FrameTimeBucket>,
    pub spike_analysis: SpikeAnalysis,
    pub consistency_metrics: ConsistencyMetrics,
    pub recommendations: Vec<String>,
}

/// 性能等级
#[derive(Debug, Clone, Serialize)]
pub enum PerformanceGrade {
    Excellent, // > 55 FPS, 稳定
    Good,      // 45-55 FPS, 较稳定
    Fair,      // 30-45 FPS, 一般稳定
    Poor,      // 15-30 FPS, 不稳定
    Terrible,  // < 15 FPS, 很不稳定
}

/// 帧时间分布桶
#[derive(Debug, Clone, Serialize)]
pub struct FrameTimeBucket {
    pub min_time_ms: f32,
    pub max_time_ms: f32,
    pub count: usize,
    pub percentage: f32,
}

/// 峰值分析
#[derive(Debug, Clone, Serialize)]
pub struct SpikeAnalysis {
    pub spike_count: usize,
    pub average_spike_duration: Duration,
    pub max_spike_duration: Duration,
    pub spike_frequency: f32, // spikes per second
    pub spike_severity: SpikeSeverity,
}

/// 峰值严重程度
#[derive(Debug, Clone, Serialize)]
pub enum SpikeSeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// 一致性指标
#[derive(Debug, Clone, Serialize)]
pub struct ConsistencyMetrics {
    pub frame_time_std_dev: f32,
    pub coefficient_of_variation: f32,
    pub consistency_score: f32, // 0.0-1.0, 1.0 = 完全一致
    pub stability_index: f32,   // 0.0-1.0, 1.0 = 完全稳定
}

impl FrameAnalyzer {
    pub fn new(target_fps: f32) -> Self {
        let frame_budget = Duration::from_nanos((1_000_000_000.0 / target_fps) as u64);
        
        Self {
            frame_times: VecDeque::new(),
            frame_starts: VecDeque::new(),
            current_frame_start: None,
            target_fps,
            max_history: 300, // 保存最近300帧
            total_frames: 0,
            dropped_frames: 0,
            frame_budget,
            enabled: true,
        }
    }

    /// 启用/禁用分析器
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 设置目标FPS
    pub fn set_target_fps(&mut self, fps: f32) {
        self.target_fps = fps;
        self.frame_budget = Duration::from_nanos((1_000_000_000.0 / fps) as u64);
    }

    /// 设置历史记录大小
    pub fn set_max_history(&mut self, size: usize) {
        self.max_history = size;
        while self.frame_times.len() > size {
            self.frame_times.pop_front();
            self.frame_starts.pop_front();
        }
    }

    /// 开始帧
    pub fn begin_frame(&mut self) {
        if !self.enabled {
            return;
        }

        let now = Instant::now();
        
        // 检查是否有未结束的帧（可能表示丢帧）
        if self.current_frame_start.is_some() {
            self.dropped_frames += 1;
        }
        
        self.current_frame_start = Some(now);
    }

    /// 结束帧
    pub fn end_frame(&mut self) {
        if !self.enabled {
            return;
        }

        if let Some(start_time) = self.current_frame_start.take() {
            let frame_time = start_time.elapsed();
            
            // 记录帧时间
            self.frame_times.push_back(frame_time);
            self.frame_starts.push_back(start_time);
            self.total_frames += 1;

            // 保持历史记录大小
            while self.frame_times.len() > self.max_history {
                self.frame_times.pop_front();
                self.frame_starts.pop_front();
            }
        }
    }

    /// 获取当前统计
    pub fn get_stats(&self) -> FrameStats {
        if self.frame_times.is_empty() {
            return FrameStats::default();
        }

        let total_time: Duration = self.frame_times.iter().sum();
        let average_frame_time = total_time / self.frame_times.len() as u32;
        let fps = 1.0 / average_frame_time.as_secs_f32();

        let min_frame_time = *self.frame_times.iter().min().unwrap();
        let max_frame_time = *self.frame_times.iter().max().unwrap();

        // 计算方差
        let mean = average_frame_time.as_secs_f32();
        let variance: f32 = self.frame_times
            .iter()
            .map(|&t| (t.as_secs_f32() - mean).powi(2))
            .sum::<f32>() / self.frame_times.len() as f32;

        let frames_under_budget = self.frame_times
            .iter()
            .filter(|&&t| t <= self.frame_budget)
            .count() as u64;

        let frames_over_budget = self.frame_times.len() as u64 - frames_under_budget;

        let dropped_frame_percentage = if self.total_frames > 0 {
            (self.dropped_frames as f32 / self.total_frames as f32) * 100.0
        } else {
            0.0
        };

        FrameStats {
            fps,
            average_frame_time,
            min_frame_time,
            max_frame_time,
            frame_time_variance: variance,
            dropped_frame_percentage,
            frames_under_budget,
            frames_over_budget,
            total_frames: self.total_frames,
        }
    }

    /// 获取详细分析
    pub fn get_analysis(&self) -> FrameAnalysis {
        let stats = self.get_stats();
        
        FrameAnalysis {
            performance_grade: self.calculate_performance_grade(&stats),
            frame_time_distribution: self.analyze_frame_time_distribution(),
            spike_analysis: self.analyze_spikes(),
            consistency_metrics: self.calculate_consistency_metrics(&stats),
            recommendations: self.generate_recommendations(&stats),
            stats,
        }
    }

    /// 计算性能等级
    fn calculate_performance_grade(&self, stats: &FrameStats) -> PerformanceGrade {
        let fps = stats.fps;
        let consistency = self.calculate_consistency_score();

        match (fps, consistency) {
            (f, c) if f >= 55.0 && c >= 0.8 => PerformanceGrade::Excellent,
            (f, c) if f >= 45.0 && c >= 0.7 => PerformanceGrade::Good,
            (f, c) if f >= 30.0 && c >= 0.6 => PerformanceGrade::Fair,
            (f, c) if f >= 15.0 && c >= 0.4 => PerformanceGrade::Poor,
            _ => PerformanceGrade::Terrible,
        }
    }

    /// 分析帧时间分布
    fn analyze_frame_time_distribution(&self) -> Vec<FrameTimeBucket> {
        if self.frame_times.is_empty() {
            return Vec::new();
        }

        let buckets = [
            (0.0, 8.33),   // > 120 FPS
            (8.33, 16.67), // 60-120 FPS
            (16.67, 33.33), // 30-60 FPS
            (33.33, 50.0),  // 20-30 FPS
            (50.0, 100.0),  // 10-20 FPS
            (100.0, f32::MAX), // < 10 FPS
        ];

        let total_frames = self.frame_times.len();
        let mut result = Vec::new();

        for &(min_ms, max_ms) in &buckets {
            let count = self.frame_times
                .iter()
                .filter(|&&t| {
                    let ms = t.as_millis() as f32;
                    ms >= min_ms && ms < max_ms
                })
                .count();

            let percentage = (count as f32 / total_frames as f32) * 100.0;

            result.push(FrameTimeBucket {
                min_time_ms: min_ms,
                max_time_ms: if max_ms == f32::MAX { 1000.0 } else { max_ms },
                count,
                percentage,
            });
        }

        result
    }

    /// 分析帧时间峰值
    fn analyze_spikes(&self) -> SpikeAnalysis {
        if self.frame_times.len() < 10 {
            return SpikeAnalysis {
                spike_count: 0,
                average_spike_duration: Duration::ZERO,
                max_spike_duration: Duration::ZERO,
                spike_frequency: 0.0,
                spike_severity: SpikeSeverity::None,
            };
        }

        // 计算平均帧时间
        let total_time: Duration = self.frame_times.iter().sum();
        let average_frame_time = total_time / self.frame_times.len() as u32;
        
        // 定义峰值阈值（比平均时间长2倍）
        let spike_threshold = average_frame_time * 2;

        let mut spikes = Vec::new();
        for &frame_time in &self.frame_times {
            if frame_time > spike_threshold {
                spikes.push(frame_time);
            }
        }

        let spike_count = spikes.len();
        let average_spike_duration = if spike_count > 0 {
            spikes.iter().sum::<Duration>() / spike_count as u32
        } else {
            Duration::ZERO
        };

        let max_spike_duration = spikes.iter().max().copied().unwrap_or(Duration::ZERO);

        // 计算峰值频率（每秒峰值数）
        let time_span = if self.frame_starts.len() > 1 {
            self.frame_starts[self.frame_starts.len() - 1]
                .duration_since(self.frame_starts[0])
                .as_secs_f32()
        } else {
            1.0
        };

        let spike_frequency = spike_count as f32 / time_span;

        let spike_severity = match spike_frequency {
            f if f < 0.1 => SpikeSeverity::None,
            f if f < 0.5 => SpikeSeverity::Low,
            f if f < 1.0 => SpikeSeverity::Medium,
            f if f < 2.0 => SpikeSeverity::High,
            _ => SpikeSeverity::Critical,
        };

        SpikeAnalysis {
            spike_count,
            average_spike_duration,
            max_spike_duration,
            spike_frequency,
            spike_severity,
        }
    }

    /// 计算一致性指标
    fn calculate_consistency_metrics(&self, stats: &FrameStats) -> ConsistencyMetrics {
        if self.frame_times.is_empty() {
            return ConsistencyMetrics {
                frame_time_std_dev: 0.0,
                coefficient_of_variation: 0.0,
                consistency_score: 1.0,
                stability_index: 1.0,
            };
        }

        let mean = stats.average_frame_time.as_secs_f32();
        let std_dev = stats.frame_time_variance.sqrt();
        
        let coefficient_of_variation = if mean > 0.0 {
            std_dev / mean
        } else {
            0.0
        };

        let consistency_score = (1.0 - coefficient_of_variation.min(1.0)).max(0.0);
        let stability_index = self.calculate_stability_index();

        ConsistencyMetrics {
            frame_time_std_dev: std_dev,
            coefficient_of_variation,
            consistency_score,
            stability_index,
        }
    }

    /// 计算一致性评分
    fn calculate_consistency_score(&self) -> f32 {
        if self.frame_times.len() < 10 {
            return 1.0;
        }

        let times: Vec<f32> = self.frame_times.iter()
            .map(|t| t.as_secs_f32())
            .collect();

        let mean = times.iter().sum::<f32>() / times.len() as f32;
        let variance = times.iter()
            .map(|&t| (t - mean).powi(2))
            .sum::<f32>() / times.len() as f32;
        let std_dev = variance.sqrt();

        let coefficient_of_variation = if mean > 0.0 {
            std_dev / mean
        } else {
            0.0
        };

        // 转换为一致性评分
        (1.0 - coefficient_of_variation.min(1.0)).max(0.0)
    }

    /// 计算稳定性指数
    fn calculate_stability_index(&self) -> f32 {
        if self.frame_times.len() < 20 {
            return 1.0;
        }

        // 计算连续帧时间差异的平均值
        let mut differences = Vec::new();
        for i in 1..self.frame_times.len() {
            let diff = (self.frame_times[i].as_secs_f32() - self.frame_times[i-1].as_secs_f32()).abs();
            differences.push(diff);
        }

        let mean_diff = differences.iter().sum::<f32>() / differences.len() as f32;
        let target_frame_time = 1.0 / self.target_fps;
        
        // 稳定性指数：差异越小，稳定性越高
        let stability = 1.0 - (mean_diff / target_frame_time).min(1.0);
        stability.max(0.0)
    }

    /// 生成性能建议
    fn generate_recommendations(&self, stats: &FrameStats) -> Vec<String> {
        let mut recommendations = Vec::new();

        // FPS建议
        if stats.fps < self.target_fps {
            let deficit = self.target_fps - stats.fps;
            if deficit > 10.0 {
                recommendations.push(format!(
                    "FPS显著低于目标({:.1} vs {:.1})，考虑优化渲染管线",
                    stats.fps, self.target_fps
                ));
            } else {
                recommendations.push(format!(
                    "FPS略低于目标({:.1} vs {:.1})，可进行微调优化",
                    stats.fps, self.target_fps
                ));
            }
        }

        // 一致性建议
        let consistency = self.calculate_consistency_score();
        if consistency < 0.7 {
            recommendations.push("帧时间不够一致，检查是否存在间歇性性能问题".to_string());
        }

        // 峰值建议
        let spike_analysis = self.analyze_spikes();
        match spike_analysis.spike_severity {
            SpikeSeverity::High | SpikeSeverity::Critical => {
                recommendations.push("检测到频繁的帧时间峰值，可能存在GC暂停或阻塞操作".to_string());
            }
            SpikeSeverity::Medium => {
                recommendations.push("偶尔出现帧时间峰值，建议检查资源加载和异步操作".to_string());
            }
            _ => {}
        }

        // 丢帧建议
        if stats.dropped_frame_percentage > 1.0 {
            recommendations.push(format!(
                "丢帧率较高({:.1}%)，检查渲染循环实现",
                stats.dropped_frame_percentage
            ));
        }

        // 超预算帧建议
        let over_budget_percentage = (stats.frames_over_budget as f32 / stats.total_frames as f32) * 100.0;
        if over_budget_percentage > 10.0 {
            recommendations.push(format!(
                "{:.1}%的帧超出时间预算，考虑降低渲染质量或优化算法",
                over_budget_percentage
            ));
        }

        if recommendations.is_empty() {
            recommendations.push("性能表现良好，无需特别优化".to_string());
        }

        recommendations
    }

    /// 获取最近N帧的统计
    pub fn get_recent_stats(&self, frame_count: usize) -> FrameStats {
        if self.frame_times.is_empty() {
            return FrameStats::default();
        }

        let start_index = if self.frame_times.len() > frame_count {
            self.frame_times.len() - frame_count
        } else {
            0
        };

        let recent_times: Vec<Duration> = self.frame_times
            .range(start_index..)
            .copied()
            .collect();

        if recent_times.is_empty() {
            return FrameStats::default();
        }

        let total_time: Duration = recent_times.iter().sum();
        let average_frame_time = total_time / recent_times.len() as u32;
        let fps = 1.0 / average_frame_time.as_secs_f32();

        let min_frame_time = *recent_times.iter().min().unwrap();
        let max_frame_time = *recent_times.iter().max().unwrap();

        let mean = average_frame_time.as_secs_f32();
        let variance: f32 = recent_times
            .iter()
            .map(|&t| (t.as_secs_f32() - mean).powi(2))
            .sum::<f32>() / recent_times.len() as f32;

        FrameStats {
            fps,
            average_frame_time,
            min_frame_time,
            max_frame_time,
            frame_time_variance: variance,
            dropped_frame_percentage: 0.0, // 不在这里计算
            frames_under_budget: 0,         // 不在这里计算
            frames_over_budget: 0,          // 不在这里计算
            total_frames: recent_times.len() as u64,
        }
    }

    /// 重置分析器
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.frame_starts.clear();
        self.current_frame_start = None;
        self.total_frames = 0;
        self.dropped_frames = 0;
    }

    /// 导出帧时间数据为CSV
    pub fn export_frame_times_csv(&self) -> String {
        let mut csv = String::new();
        csv.push_str("Frame,Time(ms),FPS\n");

        for (i, &frame_time) in self.frame_times.iter().enumerate() {
            let ms = frame_time.as_millis();
            let fps = 1000.0 / ms as f32;
            csv.push_str(&format!("{},{},{:.2}\n", i, ms, fps));
        }

        csv
    }

    /// 检测性能回归
    pub fn detect_regression(&self, baseline_fps: f32, tolerance: f32) -> Option<RegressionReport> {
        let current_stats = self.get_stats();
        let fps_diff = baseline_fps - current_stats.fps;
        let relative_diff = fps_diff / baseline_fps;

        if relative_diff > tolerance {
            Some(RegressionReport {
                baseline_fps,
                current_fps: current_stats.fps,
                absolute_difference: fps_diff,
                relative_difference: relative_diff,
                severity: if relative_diff > 0.2 {
                    RegressionSeverity::Severe
                } else if relative_diff > 0.1 {
                    RegressionSeverity::Moderate
                } else {
                    RegressionSeverity::Minor
                },
            })
        } else {
            None
        }
    }
}

impl Default for FrameAnalyzer {
    fn default() -> Self {
        Self::new(60.0)
    }
}

/// 回归报告
#[derive(Debug, Clone, Serialize)]
pub struct RegressionReport {
    pub baseline_fps: f32,
    pub current_fps: f32,
    pub absolute_difference: f32,
    pub relative_difference: f32,
    pub severity: RegressionSeverity,
}

/// 回归严重程度
#[derive(Debug, Clone, Serialize)]
pub enum RegressionSeverity {
    Minor,    // < 10% 下降
    Moderate, // 10-20% 下降
    Severe,   // > 20% 下降
}
