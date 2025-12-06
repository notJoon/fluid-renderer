/// Trait for pixel buffer abstraction
/// 
/// This trait provides a common interface for both headless and window-backed buffers.
/// All pixels are in ARGB format (0xAARRGGBB).
pub trait FrameBuffer {
    /// Returns the width of the buffer in pixels
    fn width(&self) -> usize;
    
    /// Returns the height of the buffer in pixels
    fn height(&self) -> usize;

    /// Clears the entire buffer with the given color
    fn clear(&mut self, color: u32);
    
    /// Sets a pixel at (x, y) to the given color
    /// Does nothing if coordinates are out of bounds
    fn set_pixel(&mut self, x: usize, y: usize, color: u32);
    
    /// Gets the pixel color at (x, y)
    /// Returns None if coordinates are out of bounds
    fn get_pixel(&self, x: usize, y: usize) -> Option<u32>;

    /// Returns immutable reference to the underlying buffer
    fn buffer(&self) -> &[u32];
    
    /// Returns mutable reference to the underlying buffer
    fn buffer_mut(&mut self) -> &mut [u32];
}

/// Headless implementation of FrameBuffer for testing and CI environments
/// 
/// This buffer stores pixels in memory without any GUI dependencies,
/// making it safe to use in automated tests and headless environments.
pub struct HeadlessBuffer {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
}

impl HeadlessBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        if width == 0 || height == 0 {
            panic!("Buffer dimensions must be non-zero");
        }

        let pixels = vec![0xFF000000; width * height];
        Self {
            width,
            height,
            pixels,
        }
    }
}

impl FrameBuffer for HeadlessBuffer {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn clear(&mut self, color: u32) {
        self.pixels.fill(color);
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.pixels[index] = color;
        }
    }

    fn get_pixel(&self, x: usize, y: usize) -> Option<u32> {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            Some(self.pixels[index])
        } else {
            None
        }
    }

    fn buffer(&self) -> &[u32] {
        &self.pixels
    }

    fn buffer_mut(&mut self) -> &mut [u32] {
        &mut self.pixels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headless_buffer_creation() {
        let buffer = HeadlessBuffer::new(100, 100);
        assert_eq!(buffer.width(), 100);
        assert_eq!(buffer.height(), 100);
        assert_eq!(buffer.buffer().len(), 10000);
    }

    #[test]
    #[should_panic(expected = "Buffer dimensions must be non-zero")]
    fn test_zero_width_panic() {
        let _buffer = HeadlessBuffer::new(0, 100);
    }

    #[test]
    fn test_pixel_operations() {
        let mut buffer = HeadlessBuffer::new(10, 10);

        buffer.clear(0xFFFFFFFF);
        assert_eq!(buffer.get_pixel(5, 5), Some(0xFFFFFFFF));

        buffer.set_pixel(5, 5, 0xFF00FF00);
        assert_eq!(buffer.get_pixel(5, 5), Some(0xFF00FF00));

        buffer.set_pixel(10, 10, 0xFF000000);
        assert_eq!(buffer.get_pixel(10, 10), None);
    }
}
