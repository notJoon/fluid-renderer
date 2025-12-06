use crate::frame_buffer::FrameBuffer;

pub struct RendererCore {
    pub scale: usize,
}

impl RendererCore {
    pub fn new(scale: usize) -> Self {
        if scale == 0 {
            panic!("Scale must be non-zero");
        }
        Self { scale }
    }

    pub fn draw_gradient(&self, fb: &mut impl FrameBuffer) {
        for y in 0..fb.height() {
            for x in 0..fb.width() {
                let r = (x * 255 / fb.width().max(1)) as u8;
                let g = (y * 255 / fb.height().max(1)) as u8;
                let b = 128;
                fb.set_pixel(x, y, rgb(r, g, b));
            }
        }
    }
}

pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (0xFF << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame_buffer::{FrameBuffer, HeadlessBuffer};

    #[test]
    fn test_renderer_core_creation() {
        let renderer = RendererCore::new(4);
        assert_eq!(renderer.scale, 4);
    }

    #[test]
    #[should_panic(expected = "Scale must be non-zero")]
    fn test_zero_scale_panic() {
        let _renderer = RendererCore::new(0);
    }

    #[test]
    fn test_gradient_rendering() {
        let renderer = RendererCore::new(1);
        let mut buffer = HeadlessBuffer::new(100, 100);

        buffer.clear(0xFF000000);
        renderer.draw_gradient(&mut buffer);

        assert_eq!(buffer.get_pixel(0, 0), Some(rgb(0, 0, 128)));
        assert_eq!(buffer.get_pixel(99, 99), Some(rgb(252, 252, 128)));
        assert_eq!(buffer.get_pixel(50, 50), Some(rgb(127, 127, 128)));
    }

    #[test]
    fn test_rgb_conversion() {
        assert_eq!(rgb(0, 0, 0), 0xFF000000);
        assert_eq!(rgb(255, 255, 255), 0xFFFFFFFF);
        assert_eq!(rgb(255, 0, 0), 0xFFFF0000);
        assert_eq!(rgb(0, 255, 0), 0xFF00FF00);
        assert_eq!(rgb(0, 0, 255), 0xFF0000FF);
    }
}
