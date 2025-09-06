//! 时间管理系统

use instant::Instant;

/// 时间管理器
#[derive(Debug)]
pub struct TimeManager {
    start_time: Instant,
    last_frame_time: Instant,
    delta_time: f32,
    total_time: f32,
    frame_count: u64,
    fps: f32,
    fps_timer: f32,
    fps_frame_count: u32,
}

impl TimeManager {
    /// 创建新的时间管理器
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame_time: now,
            delta_time: 0.0,
            total_time: 0.0,
            frame_count: 0,
            fps: 0.0,
            fps_timer: 0.0,
            fps_frame_count: 0,
        }
    }

    /// 更新时间管理器 (每帧调用)
    pub fn update(&mut self) {
        let now = Instant::now();
        
        // 计算帧时间
        let frame_duration = now.duration_since(self.last_frame_time);
        self.delta_time = frame_duration.as_secs_f32();
        self.last_frame_time = now;
        
        // 更新总时间
        let total_duration = now.duration_since(self.start_time);
        self.total_time = total_duration.as_secs_f32();
        
        // 更新帧计数
        self.frame_count += 1;
        
        // 更新FPS计算
        self.fps_timer += self.delta_time;
        self.fps_frame_count += 1;
        
        if self.fps_timer >= 1.0 {
            self.fps = self.fps_frame_count as f32 / self.fps_timer;
            self.fps_timer = 0.0;
            self.fps_frame_count = 0;
        }
    }

    /// 获取帧时间 (秒)
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    /// 获取总运行时间 (秒)
    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    /// 获取帧计数
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// 获取当前FPS
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// 获取帧时间 (毫秒)
    pub fn delta_time_ms(&self) -> f32 {
        self.delta_time * 1000.0
    }

    /// 重置时间管理器
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.start_time = now;
        self.last_frame_time = now;
        self.delta_time = 0.0;
        self.total_time = 0.0;
        self.frame_count = 0;
        self.fps = 0.0;
        self.fps_timer = 0.0;
        self.fps_frame_count = 0;
    }

    /// 获取平均FPS
    pub fn average_fps(&self) -> f32 {
        if self.total_time > 0.0 {
            self.frame_count as f32 / self.total_time
        } else {
            0.0
        }
    }
}

impl Default for TimeManager {
    fn default() -> Self {
        Self::new()
    }
}
