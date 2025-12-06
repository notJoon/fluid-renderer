use crate::frame_buffer::FrameBuffer;
use minifb::{Key, Window, WindowOptions};

pub struct RendererMinifb {
    window: Window,
}

impl RendererMinifb {
    pub fn new(title: &str, width: usize, height: usize) -> Result<Self, minifb::Error> {
        let mut window = Window::new(title, width, height, WindowOptions::default())?;

        window.limit_update_rate(Some(std::time::Duration::from_micros(16_666)));

        Ok(Self { window })
    }

    pub fn present(&mut self, fb: &impl FrameBuffer) -> bool {
        self.window
            .update_with_buffer(fb.buffer(), fb.width(), fb.height())
            .is_ok()
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }

    pub fn should_close(&self) -> bool {
        !self.window.is_open() || self.window.is_key_down(Key::Escape)
    }

    pub fn get_size(&self) -> (usize, usize) {
        self.window.get_size()
    }
}
