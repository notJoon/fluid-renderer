use fluid_renderer::{
    velocity_field::VelocityField, FluidField, FrameBuffer, HeadlessBuffer, RendererCore,
};
use std::time::Instant;

/// Mock implementation of VelocityField for testing
struct MockVelocityField {
    width: usize,
    height: usize,
    velocity_x: Vec<f32>,
    velocity_y: Vec<f32>,
}

impl MockVelocityField {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            velocity_x: vec![0.0; width * height],
            velocity_y: vec![0.0; width * height],
        }
    }

    fn set_velocity(&mut self, i: usize, j: usize, vx: f32, vy: f32) {
        let index = j * self.width + i;
        self.velocity_x[index] = vx;
        self.velocity_y[index] = vy;
    }
}

impl VelocityField for MockVelocityField {
    fn grid_width(&self) -> usize {
        self.width
    }

    fn grid_height(&self) -> usize {
        self.height
    }

    fn velocity_x_at(&self, i: usize, j: usize) -> f32 {
        self.velocity_x[j * self.width + i]
    }

    fn velocity_y_at(&self, i: usize, j: usize) -> f32 {
        self.velocity_y[j * self.width + i]
    }
}

/// Mock density field for testing preservation
struct MockDensityField {
    width: usize,
    height: usize,
    density: Vec<f32>,
}

impl MockDensityField {
    fn new(width: usize, height: usize, value: f32) -> Self {
        Self {
            width,
            height,
            density: vec![value; width * height],
        }
    }
}

impl FluidField for MockDensityField {
    fn grid_width(&self) -> usize {
        self.width
    }

    fn grid_height(&self) -> usize {
        self.height
    }

    fn density_at(&self, i: usize, j: usize) -> f32 {
        self.density[j * self.width + i]
    }
}

/// Helper function to convert HSV to RGB
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
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
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

/// Test C-1: Hue mapping for canonical directions
#[test]
fn test_c1_hue_mapping() {
    let renderer = RendererCore::new(4);
    let mut buffer = HeadlessBuffer::new(8, 8);
    let mut velocity = MockVelocityField::new(2, 2);

    // Test canonical directions
    // Right (0°) → Red
    velocity.set_velocity(0, 0, 1.0, 0.0);
    // Up (90°) → Green-ish 
    velocity.set_velocity(1, 0, 0.0, 1.0);
    // Left (180°) → Cyan
    velocity.set_velocity(0, 1, -1.0, 0.0);
    // Down (270°) → Blue-ish
    velocity.set_velocity(1, 1, 0.0, -1.0);

    renderer.draw_velocity(&mut buffer, &velocity, 1.0);

    // Check pixel at center of each block (scale=4, so center is at +2)
    let pixel_00 = buffer.get_pixel(2, 2).unwrap();
    let pixel_10 = buffer.get_pixel(6, 2).unwrap();
    let pixel_01 = buffer.get_pixel(2, 6).unwrap();
    let pixel_11 = buffer.get_pixel(6, 6).unwrap();

    // Extract RGB components
    let r00 = ((pixel_00 >> 16) & 0xFF) as u8;
    let _r10 = ((pixel_10 >> 16) & 0xFF) as u8;
    let r01 = ((pixel_01 >> 16) & 0xFF) as u8;
    let _r11 = ((pixel_11 >> 16) & 0xFF) as u8;

    // Right (0°) should be red-dominant
    assert!(r00 > 200, "Right direction should be red-dominant");
    
    // Up (90°) should have high green
    let g10 = ((pixel_10 >> 8) & 0xFF) as u8;
    assert!(g10 > 100, "Up direction should have significant green");
    
    // Left (180°) should be cyan (low red, high green/blue)
    assert!(r01 < 100, "Left direction should have low red");
    
    // Down (270°) should have high blue
    let b11 = (pixel_11 & 0xFF) as u8;
    assert!(b11 > 100, "Down direction should have significant blue");
}

/// Test C-2: Magnitude normalization and clamping
#[test]
fn test_c2_magnitude_normalization() {
    let renderer = RendererCore::new(2);
    let mut buffer = HeadlessBuffer::new(6, 2);
    let mut velocity = MockVelocityField::new(3, 1);

    // Zero magnitude
    velocity.set_velocity(0, 0, 0.0, 0.0);
    // Unit magnitude
    velocity.set_velocity(1, 0, 1.0, 0.0);
    // Over max (should be clamped)
    velocity.set_velocity(2, 0, 2.0, 0.0);

    renderer.draw_velocity(&mut buffer, &velocity, 1.0);

    // Zero magnitude should not write a pixel (or write black)
    let pixel_0 = buffer.get_pixel(1, 1).unwrap();
    assert!(
        pixel_0 == 0xFF000000 || pixel_0 == buffer.get_pixel(0, 0).unwrap(),
        "Zero magnitude should preserve background"
    );

    // Unit magnitude should have full brightness
    let pixel_1 = buffer.get_pixel(3, 1).unwrap();
    let brightness_1 = ((pixel_1 >> 16) & 0xFF) as u8; // Red channel for right direction
    assert!(brightness_1 > 250, "Unit magnitude should have full brightness");

    // Clamped magnitude should also have full brightness
    let pixel_2 = buffer.get_pixel(5, 1).unwrap();
    let brightness_2 = ((pixel_2 >> 16) & 0xFF) as u8;
    assert!(brightness_2 > 250, "Clamped magnitude should have full brightness");
}

/// Test C-3: NaN and Inf sanitization
#[test]
fn test_c3_nan_inf_sanitization() {
    let renderer = RendererCore::new(2);
    let mut buffer = HeadlessBuffer::new(8, 2);
    let mut velocity = MockVelocityField::new(4, 1);

    // NaN values
    velocity.set_velocity(0, 0, f32::NAN, f32::NAN);
    // +Inf
    velocity.set_velocity(1, 0, f32::INFINITY, 0.0);
    // -Inf
    velocity.set_velocity(2, 0, f32::NEG_INFINITY, 1.0);
    // Normal
    velocity.set_velocity(3, 0, 0.5, 0.5);

    // Should not panic
    renderer.draw_velocity(&mut buffer, &velocity, 1.0);

    // NaN should become zero (no pixel or black)
    let pixel_0 = buffer.get_pixel(1, 1).unwrap();
    assert!(
        pixel_0 == 0xFF000000 || pixel_0 == buffer.get_pixel(0, 0).unwrap(),
        "NaN should be treated as zero"
    );

    // +Inf should be clamped to max_magnitude
    let pixel_1 = buffer.get_pixel(3, 1).unwrap();
    assert!(pixel_1 != 0xFF000000, "+Inf should produce a visible pixel");

    // -Inf should be clamped
    let pixel_2 = buffer.get_pixel(5, 1).unwrap();
    assert!(pixel_2 != 0xFF000000, "-Inf should produce a visible pixel");

    // Normal should work
    let pixel_3 = buffer.get_pixel(7, 1).unwrap();
    assert!(pixel_3 != 0xFF000000, "Normal values should work");
}

/// Test C-4: Pixel position mapping
#[test]
fn test_c4_pixel_position_mapping() {
    let scale = 4;
    let renderer = RendererCore::new(scale);
    let mut buffer = HeadlessBuffer::new(20, 20);
    let mut velocity = MockVelocityField::new(5, 5);

    // Set velocity at grid position (2, 3)
    velocity.set_velocity(2, 3, 1.0, 0.0);

    renderer.draw_velocity(&mut buffer, &velocity, 1.0);

    // Expected pixel position: (2*4 + 4/2, 3*4 + 4/2) = (10, 14)
    let pixel = buffer.get_pixel(10, 14).unwrap();
    assert!(pixel != 0xFF000000, "Velocity should be rendered at correct position");

    // Check that surrounding pixels are not affected
    assert_eq!(buffer.get_pixel(9, 14).unwrap(), 0xFF000000);
    assert_eq!(buffer.get_pixel(11, 14).unwrap(), 0xFF000000);
    assert_eq!(buffer.get_pixel(10, 13).unwrap(), 0xFF000000);
    assert_eq!(buffer.get_pixel(10, 15).unwrap(), 0xFF000000);
}

/// Test C-5: Underlying density preservation
#[test]
fn test_c5_density_preservation() {
    let renderer = RendererCore::new(4);
    let mut buffer = HeadlessBuffer::new(8, 8);
    let density = MockDensityField::new(2, 2, 0.5);
    let mut velocity = MockVelocityField::new(2, 2);

    // First draw density
    renderer.draw_density(&mut buffer, &density);

    // Save density pixel values
    let density_pixel = buffer.get_pixel(0, 0).unwrap();
    assert_ne!(density_pixel, 0xFF000000, "Density should be visible");

    // Set velocity only at (1, 0)
    velocity.set_velocity(1, 0, 1.0, 0.0);

    // Draw velocity over density
    renderer.draw_velocity(&mut buffer, &velocity, 1.0);

    // Check that density is preserved where velocity is zero
    let preserved_pixel = buffer.get_pixel(0, 0).unwrap();
    assert_eq!(preserved_pixel, density_pixel, "Density should be preserved where velocity is zero");

    // Check that velocity overwrote the center pixel at (1, 0)
    let velocity_pixel = buffer.get_pixel(6, 2).unwrap(); // Center of block at (1, 0)
    assert_ne!(velocity_pixel, density_pixel, "Velocity should overwrite center pixel");
}

/// Test C-6: HSV to RGB conversion accuracy
#[test]
fn test_c6_hsv_to_rgb_accuracy() {
    // Test canonical HSV to RGB conversions
    let (r, g, b) = hsv_to_rgb(0.0, 1.0, 1.0);
    assert_eq!((r, g, b), (255, 0, 0), "HSV(0,1,1) should be pure red");

    let (r, g, b) = hsv_to_rgb(120.0, 1.0, 1.0);
    assert_eq!((r, g, b), (0, 255, 0), "HSV(120,1,1) should be pure green");

    let (r, g, b) = hsv_to_rgb(240.0, 1.0, 1.0);
    assert_eq!((r, g, b), (0, 0, 255), "HSV(240,1,1) should be pure blue");

    // Test with different value (brightness)
    let (r, g, b) = hsv_to_rgb(0.0, 1.0, 0.5);
    assert_eq!((r, g, b), (127, 0, 0), "HSV(0,1,0.5) should be dark red");
}

/// Test C-7: Performance benchmark
#[test]
#[ignore] // Can be run with --ignored flag
fn test_c7_performance_benchmark() {
    let renderer = RendererCore::new(2);
    let mut buffer = HeadlessBuffer::new(512, 512);
    let mut velocity = MockVelocityField::new(256, 256);

    // Fill with varied velocities
    for j in 0..256 {
        for i in 0..256 {
            let vx = (i as f32 / 256.0) * 2.0 - 1.0;
            let vy = (j as f32 / 256.0) * 2.0 - 1.0;
            velocity.set_velocity(i, j, vx, vy);
        }
    }

    // Warm up
    for _ in 0..10 {
        renderer.draw_velocity(&mut buffer, &velocity, 1.0);
    }

    // Measure performance
    let iterations = 100;
    let start = Instant::now();
    
    for _ in 0..iterations {
        renderer.draw_velocity(&mut buffer, &velocity, 1.0);
    }
    
    let elapsed = start.elapsed();
    let fps = iterations as f64 / elapsed.as_secs_f64();
    
    println!("Velocity rendering performance: {:.2} FPS", fps);
    assert!(fps >= 100.0, "Performance should be >= 100 FPS, got {:.2}", fps);
}

/// Test C-8: Determinism test
#[test]
fn test_c8_determinism() {
    let renderer = RendererCore::new(3);
    let mut velocity = MockVelocityField::new(10, 10);

    // Set up a complex velocity field
    for j in 0..10 {
        for i in 0..10 {
            let angle = (i + j) as f32 * 0.5;
            velocity.set_velocity(i, j, angle.cos(), angle.sin());
        }
    }

    // Render multiple times
    let mut buffer1 = HeadlessBuffer::new(30, 30);
    let mut buffer2 = HeadlessBuffer::new(30, 30);
    let mut buffer3 = HeadlessBuffer::new(30, 30);

    renderer.draw_velocity(&mut buffer1, &velocity, 1.0);
    renderer.draw_velocity(&mut buffer2, &velocity, 1.0);
    renderer.draw_velocity(&mut buffer3, &velocity, 1.0);

    // Compare buffers
    for y in 0..30 {
        for x in 0..30 {
            let pixel1 = buffer1.get_pixel(x, y).unwrap();
            let pixel2 = buffer2.get_pixel(x, y).unwrap();
            let pixel3 = buffer3.get_pixel(x, y).unwrap();
            
            assert_eq!(pixel1, pixel2, "Rendering should be deterministic at ({}, {})", x, y);
            assert_eq!(pixel2, pixel3, "Rendering should be deterministic at ({}, {})", x, y);
        }
    }
}