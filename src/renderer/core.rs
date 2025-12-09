use crate::fluid_field::FluidField;
use crate::frame_buffer::FrameBuffer;
use crate::velocity_field::VelocityField;
use std::f32::consts::PI;

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

    /// Renders a velocity field with hue encoding for direction and brightness for magnitude
    /// 
    /// # Arguments
    /// * `fb` - Frame buffer to render to
    /// * `velocity` - Velocity field containing x and y components
    /// * `max_magnitude` - Maximum magnitude for clamping and normalization
    /// 
    /// Velocity vectors are visualized at the center of each grid cell block.
    /// Direction is encoded as hue (0-360°), magnitude as brightness (0-1).
    #[inline]
    pub fn draw_velocity(
        &self,
        fb: &mut impl FrameBuffer,
        velocity: &impl VelocityField,
        max_magnitude: f32,
    ) {
        let grid_width = velocity.grid_width();
        let grid_height = velocity.grid_height();
        let buffer_width = fb.width();
        let buffer_height = fb.height();

        for j in 0..grid_height {
            // Calculate center pixel y-coordinate for this grid cell
            let pixel_y = j * self.scale + self.scale / 2;
            if pixel_y >= buffer_height {
                break;
            }

            for i in 0..grid_width {
                // Calculate center pixel x-coordinate for this grid cell
                let pixel_x = i * self.scale + self.scale / 2;
                if pixel_x >= buffer_width {
                    break;
                }

                // Get velocity components
                let mut vx = velocity.velocity_x_at(i, j);
                let mut vy = velocity.velocity_y_at(i, j);

                // Sanitize NaN and Inf values (spec C-2-2)
                if vx.is_nan() {
                    vx = 0.0;
                } else if vx.is_infinite() {
                    vx = if vx > 0.0 { max_magnitude } else { -max_magnitude };
                }

                if vy.is_nan() {
                    vy = 0.0;
                } else if vy.is_infinite() {
                    vy = if vy > 0.0 { max_magnitude } else { -max_magnitude };
                }

                // Calculate magnitude
                let mag = (vx * vx + vy * vy).sqrt();

                // Skip if magnitude is zero (spec C-4-3)
                if mag == 0.0 {
                    continue;
                }

                // Clamp magnitude (spec C-2-3)
                let mag_clamped = mag.min(max_magnitude);

                // Calculate angle and convert to hue (spec C-3-1)
                let theta = vy.atan2(vx); // Range [-pi, pi]
                let mut hue = theta * 180.0 / PI; // Convert to degrees
                if hue < 0.0 {
                    hue += 360.0; // Normalize to [0, 360)
                }

                // Calculate brightness from magnitude (spec C-3-2)
                let value = mag_clamped / max_magnitude;

                // Convert HSV to RGB (spec C-3-3)
                let (r, g, b) = hsv_to_rgb(hue, 1.0, value);
                let pixel = rgb(r, g, b);

                // Set the pixel at the center of the block (spec C-4-1)
                fb.set_pixel(pixel_x, pixel_y, pixel);
            }
        }
    }
}

pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (0xFF << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Converts HSV color space to RGB
/// 
/// # Arguments
/// * `h` - Hue in degrees [0, 360)
/// * `s` - Saturation [0, 1]
/// * `v` - Value/Brightness [0, 1]
/// 
/// # Returns
/// RGB tuple with each component in range [0, 255]
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    )
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
