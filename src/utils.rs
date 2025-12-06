use std::time::Instant;

/// FPS counter for performance monitoring
pub struct FpsCounter {
    start_time: Instant,
    last_report_time: Instant,
    frame_count: u64,
    report_interval: u64,
}

impl FpsCounter {
    /// Create a new FPS counter
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_report_time: now,
            frame_count: 0,
            report_interval: 60, // Report every 60 frames by default
        }
    }
    
    /// Create FPS counter with custom report interval
    pub fn with_interval(report_interval: u64) -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_report_time: now,
            frame_count: 0,
            report_interval,
        }
    }
    
    /// Update frame count and optionally print FPS
    pub fn update(&mut self) -> Option<f64> {
        self.frame_count += 1;
        
        if self.frame_count % self.report_interval == 0 {
            let elapsed = self.last_report_time.elapsed().as_secs_f64();
            let fps = self.report_interval as f64 / elapsed;
            self.last_report_time = Instant::now();
            Some(fps)
        } else {
            None
        }
    }
    
    /// Get average FPS since start
    pub fn average_fps(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.frame_count as f64 / elapsed
        } else {
            0.0
        }
    }
    
    /// Get total frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Common constants for the renderer
pub mod constants {
    /// Default background color (black)
    pub const BACKGROUND_COLOR: u32 = 0xFF000000;
    
    /// Default white color
    pub const WHITE_COLOR: u32 = 0xFFFFFFFF;
    
    /// Default window width
    pub const DEFAULT_WIDTH: usize = 512;
    
    /// Default window height
    pub const DEFAULT_HEIGHT: usize = 512;
    
    /// Default scale factor
    pub const DEFAULT_SCALE: usize = 1;
    
    /// Frame time for 60 FPS in microseconds
    pub const FRAME_TIME_60FPS_MICROS: u64 = 16_666;
}