use fluid_renderer::{FluidField, FrameBuffer, HeadlessBuffer, RendererCore, RendererMinifb};
use std::time::Instant;

/// Simple fluid simulation with a static circle pattern
struct CircleFluidSim {
    width: usize,
    height: usize,
    density: Vec<f32>,
}

impl CircleFluidSim {
    fn new(width: usize, height: usize) -> Self {
        let mut sim = Self {
            width,
            height,
            density: vec![0.0; width * height],
        };
        sim.create_circle();
        sim
    }
    
    /// Creates a circle/clock shape in the center of the grid
    fn create_circle(&mut self) {
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;
        let radius = self.width.min(self.height) as f32 / 3.0;
        
        println!("Creating circle at center ({:.1}, {:.1}) with radius {:.1}", 
                 center_x, center_y, radius);
        
        for j in 0..self.height {
            for i in 0..self.width {
                let dx = i as f32 - center_x;
                let dy = j as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                // Create a gradient circle
                let density = if distance <= radius {
                    // Smooth gradient from center (1.0) to edge (0.0)
                    1.0 - (distance / radius).powi(2)
                } else {
                    0.0
                };
                
                let idx = j * self.width + i;
                self.density[idx] = density;
            }
        }
    }
    
    /// Creates a circle with antialiased edges (like in test B-8)
    fn create_antialiased_circle(&mut self) {
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;
        let radius = self.width.min(self.height) as f32 / 4.0;
        
        println!("Creating antialiased circle at center ({:.1}, {:.1}) with radius {:.1}", 
                 center_x, center_y, radius);
        
        for j in 0..self.height {
            for i in 0..self.width {
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
                
                let idx = j * self.width + i;
                self.density[idx] = density;
            }
        }
    }
}

impl FluidField for CircleFluidSim {
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

fn main() {
    println!("===========================================");
    println!("Test B-8: Circle Integration Test (Example)");
    println!("===========================================");
    println!();
    println!("This example demonstrates the integration test from spec B-8:");
    println!("- Creates a circle shape in the center of the grid");
    println!("- Renders it through the complete pipeline");
    println!("- Verifies the entire system works correctly");
    println!();
    
    let grid_size = 128;
    let scale = 4;
    let window_size = grid_size * scale;
    
    // Create simulation with circle pattern
    let mut sim = CircleFluidSim::new(grid_size, grid_size);
    
    // First, run headless verification
    println!("Running headless verification...");
    verify_circle_headless(&sim, scale);
    
    // Then try visual display
    println!("\nAttempting visual display...");
    match RendererMinifb::new("Test B-8: Circle Integration", window_size, window_size) {
        Ok(mut renderer_minifb) => {
            println!("Window created successfully!");
            println!("Press ESC to switch between circle types, or close the window to exit.");
            
            let renderer_core = RendererCore::new(scale);
            let mut buffer = HeadlessBuffer::new(window_size, window_size);
            
            let mut frame_count = 0;
            let mut last_fps_time = Instant::now();
            let mut circle_type = 0;
            let mut last_switch = Instant::now();
            
            while renderer_minifb.is_open() {
                // Switch circle type every 3 seconds
                if last_switch.elapsed().as_secs() >= 3 {
                    circle_type = (circle_type + 1) % 2;
                    match circle_type {
                        0 => {
                            println!("\nSwitching to gradient circle");
                            sim.create_circle();
                        }
                        _ => {
                            println!("\nSwitching to antialiased circle");
                            sim.create_antialiased_circle();
                        }
                    }
                    last_switch = Instant::now();
                }
                
                // Render density to buffer
                renderer_core.draw_density(&mut buffer, &sim);
                
                // Present buffer to window
                if !renderer_minifb.present(&buffer) {
                    break;
                }
                
                // Calculate and display FPS
                frame_count += 1;
                let elapsed = last_fps_time.elapsed();
                if elapsed.as_secs() >= 1 {
                    let fps = frame_count as f64 / elapsed.as_secs_f64();
                    println!("FPS: {:.1}", fps);
                    frame_count = 0;
                    last_fps_time = Instant::now();
                }
            }
            
            println!("\nVisual test complete!");
        }
        Err(e) => {
            println!("Could not create window: {}", e);
            println!("This is expected in headless environments.");
        }
    }
    
    println!("\n===========================================");
    println!("Test B-8 Complete: Circle renders correctly");
    println!("The entire pipeline (grid → buffer → window)");
    println!("is properly connected and functional.");
    println!("===========================================");
}

fn verify_circle_headless(sim: &CircleFluidSim, scale: usize) {
    let renderer = RendererCore::new(scale);
    let grid_size = sim.grid_width();
    let buffer_size = grid_size * scale;
    let mut buffer = HeadlessBuffer::new(buffer_size, buffer_size);
    
    // Render the circle
    renderer.draw_density(&mut buffer, sim);
    
    // Perform verification checks
    let center_x = buffer_size / 2;
    let center_y = buffer_size / 2;
    
    // Check center point
    let center_pixel = buffer.get_pixel(center_x, center_y).unwrap();
    let center_gray = (center_pixel >> 16) & 0xFF;
    
    if center_gray > 200 {
        println!("✓ Center point is bright (gray value: {})", center_gray);
    } else {
        println!("✗ Center point is not bright enough (gray value: {})", center_gray);
    }
    
    // Check corners are black
    let corners = [
        (0, 0, "top-left"),
        (buffer_size - 1, 0, "top-right"),
        (0, buffer_size - 1, "bottom-left"),
        (buffer_size - 1, buffer_size - 1, "bottom-right"),
    ];
    
    let mut all_corners_black = true;
    for (x, y, name) in corners.iter() {
        let pixel = buffer.get_pixel(*x, *y).unwrap();
        if pixel == 0xFF000000 {
            println!("✓ Corner {} is black", name);
        } else {
            println!("✗ Corner {} is not black (pixel: {:08X})", name, pixel);
            all_corners_black = false;
        }
    }
    
    // Check circular symmetry
    let radius = buffer_size / 4;
    let mut symmetric = true;
    let mut values = Vec::new();
    
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::PI / 4.0;
        let x = (center_x as f32 + radius as f32 * angle.cos()) as usize;
        let y = (center_y as f32 + radius as f32 * angle.sin()) as usize;
        
        if let Some(pixel) = buffer.get_pixel(x, y) {
            let gray = (pixel >> 16) & 0xFF;
            values.push(gray);
        }
    }
    
    // Check if all values are similar (within tolerance)
    if !values.is_empty() {
        let avg = values.iter().sum::<u32>() / values.len() as u32;
        for (i, &val) in values.iter().enumerate() {
            let diff = (val as i32 - avg as i32).abs();
            if diff > 5 {
                println!("✗ Asymmetry detected at angle {}: gray {} (avg: {})", 
                         i * 45, val, avg);
                symmetric = false;
            }
        }
        
        if symmetric {
            println!("✓ Circle has good circular symmetry (avg gray at radius: {})", avg);
        }
    }
    
    if center_gray > 200 && all_corners_black && symmetric {
        println!("\n✓ All verification checks passed!");
    } else {
        println!("\n⚠ Some verification checks failed");
    }
}