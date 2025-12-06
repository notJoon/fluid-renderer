use crate::fluid_field::FluidField;
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

    /// Renders a density field as grayscale image
    /// 
    /// # Arguments
    /// * `fb` - Frame buffer to render to
    /// * `sim` - Fluid field containing density data
    /// 
    /// Density values are clamped to [0,1] and mapped to grayscale [0,255].
    /// NaN values become 0, ±Inf become 0 or 1 respectively.
    #[inline]
    pub fn draw_density(&self, fb: &mut impl FrameBuffer, sim: &impl FluidField) {
        // Clear buffer first (spec requirement B-5)
        fb.clear(0xFF000000);

        let grid_width = sim.grid_width();
        let grid_height = sim.grid_height();
        let buffer_width = fb.width();
        let buffer_height = fb.height();

        // Get direct access to buffer for better performance
        let buffer = fb.buffer_mut();

        // Iterate through the fluid grid
        for j in 0..grid_height {
            let buffer_y = j * self.scale;
            // Early boundary check for y
            if buffer_y >= buffer_height {
                break;
            }

            for i in 0..grid_width {
                let buffer_x = i * self.scale;
                // Early boundary check for x
                if buffer_x >= buffer_width {
                    break;
                }

                // Get density value at grid position
                let mut density = sim.density_at(i, j);

                // Handle NaN and Inf values (spec B-7) - optimized
                if !density.is_finite() {
                    density = if density.is_nan() || density < 0.0 {
                        0.0
                    } else {
                        1.0
                    };
                }

                // Clamp density to [0, 1] range (spec B-1-2)
                let d_clamped = density.clamp(0.0, 1.0);

                // Convert to grayscale (spec B-2)
                let gray = (d_clamped * 255.0).round() as u8;
                let pixel =
                    0xFF000000 | ((gray as u32) << 16) | ((gray as u32) << 8) | (gray as u32);

                // Draw scale x scale block (spec B-3-2)
                let max_dy = self.scale.min(buffer_height - buffer_y);
                let max_dx = self.scale.min(buffer_width - buffer_x);

                for dy in 0..max_dy {
                    let py = buffer_y + dy;
                    let row_start = py * buffer_width + buffer_x;

                    // Fill the row with the pixel value
                    for dx in 0..max_dx {
                        buffer[row_start + dx] = pixel;
                    }
                }
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
