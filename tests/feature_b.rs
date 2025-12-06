use fluid_renderer::{FrameBuffer, HeadlessBuffer, RendererCore, FluidField};
use std::time::Instant;

// Test-specific mock implementation
struct MockFluidSim {
    width: usize,
    height: usize,
    density: Vec<f32>,
}

impl MockFluidSim {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            density: vec![0.0; width * height],
        }
    }

    fn set_density(&mut self, i: usize, j: usize, value: f32) {
        if i < self.width && j < self.height {
            let idx = j * self.width + i;
            self.density[idx] = value;
        }
    }

    fn fill_density(&mut self, value: f32) {
        self.density.fill(value);
    }
}

impl FluidField for MockFluidSim {
    fn grid_width(&self) -> usize {
        self.width
    }

    fn grid_height(&self) -> usize {
        self.height
    }

    fn density_at(&self, i: usize, j: usize) -> f32 {
        if i < self.width && j < self.height {
            let idx = j * self.width + i;
            self.density[idx]
        } else {
            0.0
        }
    }
}

// Test B-1: Basic Grayscale Conversion Test
#[test]
fn test_b1_basic_grayscale_conversion() {
    let renderer = RendererCore::new(1);
    let mut buffer = HeadlessBuffer::new(3, 1);

    let mut sim = MockFluidSim::new(3, 1);
    sim.set_density(0, 0, 0.0);
    sim.set_density(1, 0, 0.5);
    sim.set_density(2, 0, 1.0);

    renderer.draw_density(&mut buffer, &sim);

    // Verify pixel values
    assert_eq!(
        buffer.get_pixel(0, 0),
        Some(0xFF000000),
        "density 0.0 should be black"
    );
    assert_eq!(
        buffer.get_pixel(1, 0),
        Some(0xFF808080),
        "density 0.5 should be gray 128"
    );
    assert_eq!(
        buffer.get_pixel(2, 0),
        Some(0xFFFFFFFF),
        "density 1.0 should be white"
    );
}

#[test]
fn test_b1_random_density_grayscale_accuracy() {
    let renderer = RendererCore::new(1);
    let mut buffer = HeadlessBuffer::new(10, 10);
    let mut sim = MockFluidSim::new(10, 10);

    // Test 100 random density values
    let test_values = vec![
        0.0, 0.1, 0.2, 0.25, 0.3, 0.4, 0.5, 0.6, 0.7, 0.75, 0.8, 0.9, 1.0, 0.001, 0.999, 0.333,
        0.666, 0.123, 0.456, 0.789,
    ];

    for (idx, &density) in test_values.iter().enumerate() {
        let i = idx % 10;
        let j = idx / 10;
        sim.set_density(i, j, density);
    }

    renderer.draw_density(&mut buffer, &sim);

    for (idx, &density) in test_values.iter().enumerate() {
        let i = idx % 10;
        let j = idx / 10;
        let pixel = buffer.get_pixel(i, j).unwrap();
        let gray = ((density * 255.0).round() as u8).clamp(0, 255);
        let expected = 0xFF000000 | ((gray as u32) << 16) | ((gray as u32) << 8) | (gray as u32);
        assert_eq!(
            pixel, expected,
            "density {} at ({}, {}) should map correctly",
            density, i, j
        );
    }
}

// Test B-2: Value Clamping and NaN/Inf Handling
#[test]
fn test_b2_value_clamping() {
    let renderer = RendererCore::new(1);
    let mut buffer = HeadlessBuffer::new(6, 1);
    let mut sim = MockFluidSim::new(6, 1);

    // Test extreme values
    sim.set_density(0, 0, -1.0); // Should clamp to 0
    sim.set_density(1, 0, 2.0); // Should clamp to 1
    sim.set_density(2, 0, 999.0); // Should clamp to 1
    sim.set_density(3, 0, f32::NAN); // Should become 0
    sim.set_density(4, 0, f32::INFINITY); // Should become 1
    sim.set_density(5, 0, f32::NEG_INFINITY); // Should become 0

    renderer.draw_density(&mut buffer, &sim);

    assert_eq!(
        buffer.get_pixel(0, 0),
        Some(0xFF000000),
        "negative density should clamp to 0"
    );
    assert_eq!(
        buffer.get_pixel(1, 0),
        Some(0xFFFFFFFF),
        "density > 1 should clamp to 1"
    );
    assert_eq!(
        buffer.get_pixel(2, 0),
        Some(0xFFFFFFFF),
        "large density should clamp to 1"
    );
    assert_eq!(
        buffer.get_pixel(3, 0),
        Some(0xFF000000),
        "NaN should become 0"
    );
    assert_eq!(
        buffer.get_pixel(4, 0),
        Some(0xFFFFFFFF),
        "+Inf should become 1"
    );
    assert_eq!(
        buffer.get_pixel(5, 0),
        Some(0xFF000000),
        "-Inf should become 0"
    );
}

// Test B-3: Scale Magnification Test
#[test]
fn test_b3_scale_magnification() {
    let renderer = RendererCore::new(3); // scale = 3
    let mut buffer = HeadlessBuffer::new(12, 12); // 4x4 grid with scale 3 = 12x12 buffer
    let mut sim = MockFluidSim::new(4, 4);

    // Create diagonal pattern
    sim.set_density(0, 0, 1.0);
    sim.set_density(1, 1, 1.0);
    sim.set_density(2, 2, 1.0);
    sim.set_density(3, 3, 1.0);

    renderer.draw_density(&mut buffer, &sim);

    // Check that each 1.0 density creates a 3x3 white block
    for block in 0..4 {
        let base_x = block * 3;
        let base_y = block * 3;

        // Check the 3x3 block
        for dy in 0..3 {
            for dx in 0..3 {
                let pixel = buffer.get_pixel(base_x + dx, base_y + dy).unwrap();
                assert_eq!(
                    pixel,
                    0xFFFFFFFF,
                    "Block {} at ({}, {}) should be white",
                    block,
                    base_x + dx,
                    base_y + dy
                );
            }
        }
    }

    // Check that other areas are black
    assert_eq!(
        buffer.get_pixel(3, 0),
        Some(0xFF000000),
        "Off-diagonal should be black"
    );
    assert_eq!(
        buffer.get_pixel(0, 3),
        Some(0xFF000000),
        "Off-diagonal should be black"
    );
}

// Test B-4: Boundary Clipping Test
#[test]
fn test_b4_boundary_clipping() {
    let renderer = RendererCore::new(2); // scale = 2
    // Grid 4x4, scale 2 would need 8x8, but we use 7x7 buffer
    let mut buffer = HeadlessBuffer::new(7, 7);
    let mut sim = MockFluidSim::new(4, 4);

    // Fill entire grid with 0.5 density
    sim.fill_density(0.5);

    renderer.draw_density(&mut buffer, &sim);

    // Check that visible pixels are correct
    for y in 0..7 {
        for x in 0..7 {
            let pixel = buffer.get_pixel(x, y).unwrap();
            assert_eq!(pixel, 0xFF808080, "All visible pixels should be gray");
        }
    }

    // The function should not panic even though the last row/column is clipped
}

// Test B-5: Random Density Field Global Test
#[test]
fn test_b5_random_density_field() {
    let renderer = RendererCore::new(1);
    let mut buffer = HeadlessBuffer::new(128, 128);
    let mut sim = MockFluidSim::new(128, 128);

    // Fill with random values
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    let random = RandomState::new();
    for j in 0..128 {
        for i in 0..128 {
            let mut hasher = random.build_hasher();
            (i * 1000 + j).hash(&mut hasher);
            let hash = hasher.finish();
            let density = (hash % 1000) as f32 / 1000.0;
            sim.set_density(i, j, density);
        }
    }

    renderer.draw_density(&mut buffer, &sim);

    // Verify all pixels are valid grayscale
    for y in 0..128 {
        for x in 0..128 {
            let pixel = buffer.get_pixel(x, y).unwrap();
            let a = (pixel >> 24) & 0xFF;
            let r = (pixel >> 16) & 0xFF;
            let g = (pixel >> 8) & 0xFF;
            let b = pixel & 0xFF;

            assert_eq!(a, 0xFF, "Alpha should always be 255");
            assert_eq!(r, g, "R and G should be equal for grayscale");
            assert_eq!(g, b, "G and B should be equal for grayscale");
            assert!(r <= 255, "Grayscale value should be in 0-255 range");
        }
    }
}

// Test B-6: Performance Benchmark Test
#[test]
fn test_b6_performance_benchmark() {
    let renderer = RendererCore::new(2); // scale = 2
    let mut buffer = HeadlessBuffer::new(512, 512); // 256x256 grid with scale 2
    let mut sim = MockFluidSim::new(256, 256);

    // Fill with some density values
    for j in 0..256 {
        for i in 0..256 {
            let density = ((i + j) % 256) as f32 / 255.0;
            sim.set_density(i, j, density);
        }
    }

    // Measure performance
    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        renderer.draw_density(&mut buffer, &sim);
    }

    let elapsed = start.elapsed();
    let fps = iterations as f64 / elapsed.as_secs_f64();

    println!("Performance: {:.2} FPS for 256x256 grid at scale 2", fps);
    assert!(
        fps >= 150.0,
        "Should maintain at least 150 FPS, got {:.2}",
        fps
    );
}

// Test B-8: Integration Test (Engine) - Circle/Clock Shape
#[test]
fn test_b8_integration_circle_rendering() {
    let renderer = RendererCore::new(4); // Use scale 4 for better visibility
    let grid_size = 64;
    let buffer_size = grid_size * 4; // 256x256 buffer
    let mut buffer = HeadlessBuffer::new(buffer_size, buffer_size);
    let mut sim = MockFluidSim::new(grid_size, grid_size);
    
    // Create a circle/clock shape in the center
    let center_x = grid_size as f32 / 2.0;
    let center_y = grid_size as f32 / 2.0;
    let radius = grid_size as f32 / 4.0; // Circle radius is 1/4 of grid size
    
    for j in 0..grid_size {
        for i in 0..grid_size {
            let dx = i as f32 - center_x;
            let dy = j as f32 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            
            // Create a smooth circle with antialiased edges
            let density = if distance <= radius - 1.0 {
                1.0 // Solid inside
            } else if distance <= radius + 1.0 {
                // Smooth edge transition
                1.0 - (distance - radius + 1.0) / 2.0
            } else {
                0.0 // Outside circle
            };
            
            sim.set_density(i, j, density);
        }
    }
    
    // Render the circle
    renderer.draw_density(&mut buffer, &sim);
    
    // Verify the circle is rendered correctly
    // Check center point (should be white)
    let center_buffer_x = (center_x * 4.0) as usize;
    let center_buffer_y = (center_y * 4.0) as usize;
    let center_pixel = buffer.get_pixel(center_buffer_x, center_buffer_y).unwrap();
    assert_eq!(center_pixel, 0xFFFFFFFF, "Center of circle should be white");
    
    // Check points on the circle perimeter (should be partially filled)
    let test_angles = [0.0, std::f32::consts::PI / 2.0, std::f32::consts::PI, 3.0 * std::f32::consts::PI / 2.0];
    for angle in test_angles.iter() {
        let edge_x = center_x + radius * angle.cos();
        let edge_y = center_y + radius * angle.sin();
        let edge_buffer_x = (edge_x * 4.0) as usize;
        let edge_buffer_y = (edge_y * 4.0) as usize;
        
        let edge_pixel = buffer.get_pixel(edge_buffer_x, edge_buffer_y).unwrap();
        let gray_value = (edge_pixel >> 16) & 0xFF;
        
        // Edge should be partially filled (not completely black or white)
        assert!(gray_value > 0 && gray_value < 255, 
            "Circle edge at angle {} should be antialiased (gray value: {})", angle, gray_value);
    }
    
    // Check corners (should be black)
    assert_eq!(buffer.get_pixel(0, 0), Some(0xFF000000), "Top-left corner should be black");
    assert_eq!(buffer.get_pixel(buffer_size - 1, 0), Some(0xFF000000), "Top-right corner should be black");
    assert_eq!(buffer.get_pixel(0, buffer_size - 1), Some(0xFF000000), "Bottom-left corner should be black");
    assert_eq!(buffer.get_pixel(buffer_size - 1, buffer_size - 1), Some(0xFF000000), "Bottom-right corner should be black");
    
    // Verify circular symmetry by checking multiple points at same radius
    let test_radius = radius / 2.0; // Test at half radius
    let mut pixel_values = Vec::new();
    
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::PI / 4.0;
        let x = center_x + test_radius * angle.cos();
        let y = center_y + test_radius * angle.sin();
        let buffer_x = (x * 4.0) as usize;
        let buffer_y = (y * 4.0) as usize;
        
        if let Some(pixel) = buffer.get_pixel(buffer_x, buffer_y) {
            pixel_values.push(pixel);
        }
    }
    
    // All points at the same radius should have the same value (circular symmetry)
    let first_value = pixel_values[0];
    for (i, &value) in pixel_values.iter().enumerate() {
        assert_eq!(value, first_value, 
            "Circular symmetry check failed at point {}: expected {:08X}, got {:08X}", 
            i, first_value, value);
    }
    
    println!("Test B-8: Circle rendered successfully with proper centering and symmetry");
}
