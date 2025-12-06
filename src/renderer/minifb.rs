use crate::frame_buffer::FrameBuffer;
use minifb::{Key, Window, WindowOptions};

/// Window-based renderer using minifb
/// 
/// This is a thin wrapper around minifb::Window that presents frame buffers
/// to the screen. It handles window creation and event processing.
pub struct RendererMinifb {
    window: Window,
}

impl RendererMinifb {
    /// Creates a new window with the given title and dimensions
    /// 
    /// # Arguments
    /// * `title` - Window title
    /// * `width` - Window width in pixels
    /// * `height` - Window height in pixels
    /// 
    /// # Returns
    /// `Result` containing the renderer or a minifb error
    pub fn new(title: &str, width: usize, height: usize) -> Result<Self, minifb::Error> {
        let mut window = Window::new(title, width, height, WindowOptions::default())?;

        // Limit to ~60 FPS
        window.limit_update_rate(Some(std::time::Duration::from_micros(
            crate::constants::FRAME_TIME_60FPS_MICROS,
        )));

        Ok(Self { window })
    }

    /// Presents a frame buffer to the window
    /// 
    /// Returns true if successful, false if the window should close
    pub fn present(&mut self, fb: &impl FrameBuffer) -> bool {
        self.window
            .update_with_buffer(fb.buffer(), fb.width(), fb.height())
            .is_ok()
    }

    /// Checks if the window is still open and not requesting close
    pub fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }

    /// Checks if the window should close (closed or ESC pressed)
    pub fn should_close(&self) -> bool {
        !self.window.is_open() || self.window.is_key_down(Key::Escape)
    }

    /// Returns the current window size
    pub fn get_size(&self) -> (usize, usize) {
        self.window.get_size()
    }
}
