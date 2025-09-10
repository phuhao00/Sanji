//! 内存跟踪器

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::alloc::{GlobalAlloc, Layout, System};
use serde::Serialize;

/// 内存跟踪器
pub struct MemoryTracker {
    allocations: HashMap<*mut u8, AllocationInfo>,
    total_allocated: usize,
    peak_allocated: usize,
    current_allocated: usize,
    allocation_count: usize,
    deallocation_count: usize,
    history: Vec<MemorySnapshot>,
    sample_interval: Duration,
    last_sample: Instant,
    enabled: bool,
}

/// 分配信息
#[derive(Debug, Clone)]
struct AllocationInfo {
    size: usize,
    timestamp: Instant,
    location: String, // 调用位置信息
    tag: Option<String>, // 可选标签
}

/// 内存快照
#[derive(Debug, Clone, Serialize)]
pub struct MemorySnapshot {
    pub timestamp: u128, // 毫秒时间戳
    pub total_allocated: usize,
    pub current_allocated: usize,
    pub allocation_count: usize,
    pub fragmentation_ratio: f32,
    pub largest_free_block: usize,
}

/// 内存分析结果
#[derive(Debug, Clone, Serialize)]
pub struct MemoryAnalysis {
    pub summary: MemorySummary,
    pub allocations_by_size: Vec<AllocationBucket>,
    pub allocations_by_age: Vec<AllocationBucket>,
    pub potential_leaks: Vec<LeakCandidate>,
    pub fragmentation_analysis: FragmentationAnalysis,
    pub trends: MemoryTrends,
}

/// 内存摘要
#[derive(Debug, Clone, Default, Serialize)]
pub struct MemorySummary {
    pub total_allocated: usize,
    pub peak_allocated: usize,
    pub current_allocated: usize,
    pub allocation_count: usize,
    pub deallocation_count: usize,
    pub average_allocation_size: f32,
    pub allocation_rate: f32, // 每秒分配次数
    pub deallocation_rate: f32, // 每秒释放次数
}

/// 分配桶
#[derive(Debug, Clone, Serialize)]
pub struct AllocationBucket {
    pub range_start: usize,
    pub range_end: usize,
    pub count: usize,
    pub total_size: usize,
    pub percentage: f32,
}

/// 泄漏候选
#[derive(Debug, Clone, Serialize)]
pub struct LeakCandidate {
    pub address: u64,
    pub size: usize,
    pub age: Duration,
    pub location: String,
    pub likelihood: f32, // 泄漏可能性 0.0-1.0
}

/// 碎片化分析
#[derive(Debug, Clone, Serialize)]
pub struct FragmentationAnalysis {
    pub fragmentation_ratio: f32,
    pub largest_free_block: usize,
    pub total_free_space: usize,
    pub free_block_count: usize,
    pub average_free_block_size: f32,
}

/// 内存趋势
#[derive(Debug, Clone, Serialize)]
pub struct MemoryTrends {
    pub growth_rate: f32, // 字节/秒
    pub allocation_trend: TrendDirection,
    pub peak_frequency: f32, // 达到峰值的频率
    pub stability_score: f32, // 内存使用稳定性评分 0.0-1.0
}

/// 趋势方向
#[derive(Debug, Clone, Serialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            total_allocated: 0,
            peak_allocated: 0,
            current_allocated: 0,
            allocation_count: 0,
            deallocation_count: 0,
            history: Vec::new(),
            sample_interval: Duration::from_millis(100),
            last_sample: Instant::now(),
            enabled: true,
        }
    }

    /// 启用/禁用跟踪
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 设置采样间隔
    pub fn set_sample_interval(&mut self, interval: Duration) {
        self.sample_interval = interval;
    }

    /// 记录分配
    pub fn record_allocation(&mut self, ptr: *mut u8, size: usize, location: String) {
        if !self.enabled {
            return;
        }

        let info = AllocationInfo {
            size,
            timestamp: Instant::now(),
            location,
            tag: None,
        };

        self.allocations.insert(ptr, info);
        self.total_allocated += size;
        self.current_allocated += size;
        self.allocation_count += 1;

        if self.current_allocated > self.peak_allocated {
            self.peak_allocated = self.current_allocated;
        }

        self.maybe_sample();
    }

    /// 记录释放
    pub fn record_deallocation(&mut self, ptr: *mut u8) {
        if !self.enabled {
            return;
        }

        if let Some(info) = self.allocations.remove(&ptr) {
            self.current_allocated -= info.size;
            self.deallocation_count += 1;
        }

        self.maybe_sample();
    }

    /// 标记分配
    pub fn tag_allocation(&mut self, ptr: *mut u8, tag: String) {
        if let Some(info) = self.allocations.get_mut(&ptr) {
            info.tag = Some(tag);
        }
    }

    /// 获取当前统计
    pub fn get_stats(&self) -> super::MemoryUsage {
        super::MemoryUsage {
            total_allocated: self.total_allocated,
            peak_allocated: self.peak_allocated,
            current_allocated: self.current_allocated,
            allocation_count: self.allocation_count,
            deallocation_count: self.deallocation_count,
            heap_size: self.get_heap_size(),
            stack_size: self.get_stack_size(),
        }
    }

    /// 获取详细分析
    pub fn get_analysis(&self) -> MemoryAnalysis {
        MemoryAnalysis {
            summary: self.get_summary(),
            allocations_by_size: self.analyze_allocations_by_size(),
            allocations_by_age: self.analyze_allocations_by_age(),
            potential_leaks: self.detect_potential_leaks(),
            fragmentation_analysis: self.analyze_fragmentation(),
            trends: self.analyze_trends(),
        }
    }

    /// 检测内存泄漏
    pub fn detect_leaks(&self) -> Vec<LeakCandidate> {
        let mut candidates = Vec::new();
        let now = Instant::now();

        for (addr, info) in &self.allocations {
            let age = now.duration_since(info.timestamp);
            
            // 简单的泄漏检测启发式
            let likelihood = self.calculate_leak_likelihood(info, age);
            
            if likelihood > 0.5 {
                candidates.push(LeakCandidate {
                    address: *addr as u64,
                    size: info.size,
                    age,
                    location: info.location.clone(),
                    likelihood,
                });
            }
        }

        // 按可能性排序
        candidates.sort_by(|a, b| b.likelihood.partial_cmp(&a.likelihood).unwrap());
        candidates
    }

    /// 获取内存热图
    pub fn get_memory_heatmap(&self, bucket_size: usize) -> Vec<(usize, usize)> {
        let mut heatmap = HashMap::new();
        
        for (addr, info) in &self.allocations {
            let bucket = (*addr as usize / bucket_size) * bucket_size;
            *heatmap.entry(bucket).or_insert(0) += info.size;
        }

        let mut result: Vec<_> = heatmap.into_iter().collect();
        result.sort_by_key(|&(addr, _)| addr);
        result
    }

    /// 重置跟踪器
    pub fn reset(&mut self) {
        self.allocations.clear();
        self.total_allocated = 0;
        self.peak_allocated = 0;
        self.current_allocated = 0;
        self.allocation_count = 0;
        self.deallocation_count = 0;
        self.history.clear();
        self.last_sample = Instant::now();
    }

    /// 可能采样
    fn maybe_sample(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_sample) >= self.sample_interval {
            self.sample();
            self.last_sample = now;
        }
    }

    /// 采样当前状态
    fn sample(&mut self) {
        let snapshot = MemorySnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            total_allocated: self.total_allocated,
            current_allocated: self.current_allocated,
            allocation_count: self.allocation_count,
            fragmentation_ratio: self.calculate_fragmentation_ratio(),
            largest_free_block: self.get_largest_free_block(),
        };

        self.history.push(snapshot);

        // 保持最近1000个快照
        if self.history.len() > 1000 {
            self.history.remove(0);
        }
    }

    /// 获取摘要
    fn get_summary(&self) -> MemorySummary {
        let allocation_rate = if !self.history.is_empty() {
            let time_span = self.history.last().unwrap().timestamp - self.history.first().unwrap().timestamp;
            if time_span > 0 {
                (self.allocation_count as f32) / (time_span as f32 / 1000.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        let deallocation_rate = if !self.history.is_empty() {
            let time_span = self.history.last().unwrap().timestamp - self.history.first().unwrap().timestamp;
            if time_span > 0 {
                (self.deallocation_count as f32) / (time_span as f32 / 1000.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        let average_allocation_size = if self.allocation_count > 0 {
            self.total_allocated as f32 / self.allocation_count as f32
        } else {
            0.0
        };

        MemorySummary {
            total_allocated: self.total_allocated,
            peak_allocated: self.peak_allocated,
            current_allocated: self.current_allocated,
            allocation_count: self.allocation_count,
            deallocation_count: self.deallocation_count,
            average_allocation_size,
            allocation_rate,
            deallocation_rate,
        }
    }

    /// 按大小分析分配
    fn analyze_allocations_by_size(&self) -> Vec<AllocationBucket> {
        let buckets = [
            (0, 64),
            (64, 256),
            (256, 1024),
            (1024, 4096),
            (4096, 16384),
            (16384, 65536),
            (65536, 262144),
            (262144, usize::MAX),
        ];

        let mut result = Vec::new();
        let total_allocations = self.allocations.len();

        for &(start, end) in &buckets {
            let mut count = 0;
            let mut total_size = 0;

            for info in self.allocations.values() {
                if info.size >= start && info.size < end {
                    count += 1;
                    total_size += info.size;
                }
            }

            let percentage = if total_allocations > 0 {
                (count as f32 / total_allocations as f32) * 100.0
            } else {
                0.0
            };

            result.push(AllocationBucket {
                range_start: start,
                range_end: end,
                count,
                total_size,
                percentage,
            });
        }

        result
    }

    /// 按年龄分析分配
    fn analyze_allocations_by_age(&self) -> Vec<AllocationBucket> {
        let now = Instant::now();
        let buckets = [
            (0, 1),      // < 1秒
            (1, 10),     // 1-10秒
            (10, 60),    // 10-60秒
            (60, 300),   // 1-5分钟
            (300, 1800), // 5-30分钟
            (1800, u64::MAX), // > 30分钟
        ];

        let mut result = Vec::new();
        let total_allocations = self.allocations.len();

        for &(start, end) in &buckets {
            let mut count = 0;
            let mut total_size = 0;

            for info in self.allocations.values() {
                let age_secs = now.duration_since(info.timestamp).as_secs();
                if age_secs >= start && age_secs < end {
                    count += 1;
                    total_size += info.size;
                }
            }

            let percentage = if total_allocations > 0 {
                (count as f32 / total_allocations as f32) * 100.0
            } else {
                0.0
            };

            result.push(AllocationBucket {
                range_start: start as usize,
                range_end: end as usize,
                count,
                total_size,
                percentage,
            });
        }

        result
    }

    /// 检测潜在泄漏
    fn detect_potential_leaks(&self) -> Vec<LeakCandidate> {
        self.detect_leaks()
    }

    /// 分析碎片化
    fn analyze_fragmentation(&self) -> FragmentationAnalysis {
        FragmentationAnalysis {
            fragmentation_ratio: self.calculate_fragmentation_ratio(),
            largest_free_block: self.get_largest_free_block(),
            total_free_space: self.get_total_free_space(),
            free_block_count: self.get_free_block_count(),
            average_free_block_size: self.get_average_free_block_size(),
        }
    }

    /// 分析趋势
    fn analyze_trends(&self) -> MemoryTrends {
        if self.history.len() < 2 {
            return MemoryTrends {
                growth_rate: 0.0,
                allocation_trend: TrendDirection::Stable,
                peak_frequency: 0.0,
                stability_score: 1.0,
            };
        }

        let growth_rate = self.calculate_growth_rate();
        let allocation_trend = self.determine_trend_direction();
        let peak_frequency = self.calculate_peak_frequency();
        let stability_score = self.calculate_stability_score();

        MemoryTrends {
            growth_rate,
            allocation_trend,
            peak_frequency,
            stability_score,
        }
    }

    /// 计算泄漏可能性
    fn calculate_leak_likelihood(&self, info: &AllocationInfo, age: Duration) -> f32 {
        let mut likelihood = 0.0;

        // 年龄因素
        let age_hours = age.as_secs_f32() / 3600.0;
        if age_hours > 1.0 {
            likelihood += 0.3;
        }
        if age_hours > 24.0 {
            likelihood += 0.3;
        }

        // 大小因素
        if info.size > 1024 * 1024 { // > 1MB
            likelihood += 0.2;
        }

        // 标签因素（如果有"temp"标签但存活很久）
        if let Some(ref tag) = info.tag {
            if tag.contains("temp") && age_hours > 0.1 {
                likelihood += 0.2;
            }
        }

        likelihood.min(1.0)
    }

    /// 计算碎片化比率
    fn calculate_fragmentation_ratio(&self) -> f32 {
        // 简化实现
        0.1
    }

    /// 获取最大自由块
    fn get_largest_free_block(&self) -> usize {
        // 简化实现
        1024 * 1024
    }

    /// 获取总自由空间
    fn get_total_free_space(&self) -> usize {
        // 简化实现
        self.get_heap_size() - self.current_allocated
    }

    /// 获取自由块数量
    fn get_free_block_count(&self) -> usize {
        // 简化实现
        100
    }

    /// 获取平均自由块大小
    fn get_average_free_block_size(&self) -> f32 {
        let total_free = self.get_total_free_space();
        let block_count = self.get_free_block_count();
        if block_count > 0 {
            total_free as f32 / block_count as f32
        } else {
            0.0
        }
    }

    /// 计算增长率
    fn calculate_growth_rate(&self) -> f32 {
        if self.history.len() < 2 {
            return 0.0;
        }

        let first = &self.history[0];
        let last = &self.history[self.history.len() - 1];
        let time_diff = (last.timestamp - first.timestamp) as f32 / 1000.0; // 转换为秒
        
        if time_diff > 0.0 {
            (last.current_allocated as f32 - first.current_allocated as f32) / time_diff
        } else {
            0.0
        }
    }

    /// 确定趋势方向
    fn determine_trend_direction(&self) -> TrendDirection {
        let growth_rate = self.calculate_growth_rate();
        
        if growth_rate.abs() < 100.0 { // < 100 bytes/second
            TrendDirection::Stable
        } else if growth_rate > 0.0 {
            TrendDirection::Increasing
        } else {
            TrendDirection::Decreasing
        }
    }

    /// 计算峰值频率
    fn calculate_peak_frequency(&self) -> f32 {
        // 简化实现
        0.0
    }

    /// 计算稳定性评分
    fn calculate_stability_score(&self) -> f32 {
        if self.history.len() < 10 {
            return 1.0;
        }

        // 计算内存使用的变异系数
        let values: Vec<f32> = self.history.iter()
            .map(|s| s.current_allocated as f32)
            .collect();

        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / values.len() as f32;
        let std_dev = variance.sqrt();

        let coefficient_of_variation = if mean > 0.0 {
            std_dev / mean
        } else {
            0.0
        };

        // 转换为稳定性评分（CV越小，稳定性越高）
        (1.0 - coefficient_of_variation.min(1.0)).max(0.0)
    }

    /// 获取堆大小
    fn get_heap_size(&self) -> usize {
        // 简化实现，实际应该查询系统
        1024 * 1024 * 100 // 100MB
    }

    /// 获取栈大小
    fn get_stack_size(&self) -> usize {
        // 简化实现，实际应该查询系统
        1024 * 8 // 8KB
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局内存分配器包装器（用于自动跟踪）
pub struct TrackedAllocator<A: GlobalAlloc> {
    inner: A,
    tracker: Arc<Mutex<MemoryTracker>>,
}

impl<A: GlobalAlloc> TrackedAllocator<A> {
    pub fn new(inner: A, tracker: Arc<Mutex<MemoryTracker>>) -> Self {
        Self { inner, tracker }
    }
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for TrackedAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        
        if !ptr.is_null() {
            if let Ok(mut tracker) = self.tracker.try_lock() {
                tracker.record_allocation(ptr, layout.size(), "unknown".to_string());
            }
        }
        
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Ok(mut tracker) = self.tracker.try_lock() {
            tracker.record_deallocation(ptr);
        }
        
        self.inner.dealloc(ptr, layout);
    }
}

/// 智能指针包装器（用于标记分配）
pub struct TrackedBox<T> {
    inner: Box<T>,
    ptr: *mut u8,
    tracker: Arc<Mutex<MemoryTracker>>,
}

impl<T> TrackedBox<T> {
    pub fn new_with_tag(value: T, tag: String, tracker: Arc<Mutex<MemoryTracker>>) -> Self {
        let inner = Box::new(value);
        let ptr = Box::as_ref(&inner) as *const T as *mut u8;
        
        if let Ok(mut t) = tracker.try_lock() {
            t.tag_allocation(ptr, tag);
        }
        
        Self { inner, ptr, tracker }
    }
}

impl<T> std::ops::Deref for TrackedBox<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for TrackedBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Drop for TrackedBox<T> {
    fn drop(&mut self) {
        if let Ok(mut tracker) = self.tracker.try_lock() {
            tracker.record_deallocation(self.ptr);
        }
    }
}
