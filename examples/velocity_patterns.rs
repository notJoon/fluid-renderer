use fluid_renderer::{
    velocity_field::VelocityField, FluidField, FrameBuffer, HeadlessBuffer, 
    RendererCore, RendererMinifb, FpsCounter,
};

/// Different velocity field patterns
enum VelocityPattern {
    Vortex,
    Source,
    Sink,
    Saddle,
    Shear,
}

struct PatternVelocityField {
    width: usize,
    height: usize,
    pattern: VelocityPattern,
    strength: f32,
}

impl PatternVelocityField {
    fn new(width: usize, height: usize, pattern: VelocityPattern) -> Self {
        Self { 
            width, 
            height, 
            pattern,
            strength: 1.0,
        }
    }
    
    fn next_pattern(&mut self) {
        self.pattern = match self.pattern {
            VelocityPattern::Vortex => VelocityPattern::Source,
            VelocityPattern::Source => VelocityPattern::Sink,
            VelocityPattern::Sink => VelocityPattern::Saddle,
            VelocityPattern::Saddle => VelocityPattern::Shear,
            VelocityPattern::Shear => VelocityPattern::Vortex,
        };
    }
    
    fn pattern_name(&self) -> &str {
        match self.pattern {
            VelocityPattern::Vortex => "Vortex (Circular flow)",
            VelocityPattern::Source => "Source (Outward flow)",
            VelocityPattern::Sink => "Sink (Inward flow)",
            VelocityPattern::Saddle => "Saddle Point",
            VelocityPattern::Shear => "Shear Flow",
        }
    }
}

impl VelocityField for PatternVelocityField {
    fn grid_width(&self) -> usize {
        self.width
    }

    fn grid_height(&self) -> usize {
        self.height
    }

    fn velocity_x_at(&self, i: usize, j: usize) -> f32 {
        let cx = self.width as f32 / 2.0;
        let cy = self.height as f32 / 2.0;
        let x = (i as f32 - cx) / cx; // Normalize to [-1, 1]
        let y = (j as f32 - cy) / cy;
        
        let vx = match self.pattern {
            VelocityPattern::Vortex => -y,
            VelocityPattern::Source => x,
            VelocityPattern::Sink => -x,
            VelocityPattern::Saddle => x,
            VelocityPattern::Shear => 1.0,
        };
        
        vx * self.strength * 0.5
    }

    fn velocity_y_at(&self, i: usize, j: usize) -> f32 {
        let cx = self.width as f32 / 2.0;
        let cy = self.height as f32 / 2.0;
        let x = (i as f32 - cx) / cx; // Normalize to [-1, 1]
        let y = (j as f32 - cy) / cy;
        
        let vy = match self.pattern {
            VelocityPattern::Vortex => x,
            VelocityPattern::Source => y,
            VelocityPattern::Sink => -y,
            VelocityPattern::Saddle => -y,
            VelocityPattern::Shear => y * 0.5,
        };
        
        vy * self.strength * 0.5
    }
}

/// Simple uniform density
struct UniformDensity {
    width: usize,
    height: usize,
    value: f32,
}

impl UniformDensity {
    fn new(width: usize, height: usize) -> Self {
        Self { width, height, value: 0.1 }
    }
}

impl FluidField for UniformDensity {
    fn grid_width(&self) -> usize {
        self.width
    }

    fn grid_height(&self) -> usize {
        self.height
    }

    fn density_at(&self, _i: usize, _j: usize) -> f32 {
        self.value
    }
}

fn main() {
    println!("Velocity Field Patterns Demo");
    println!("=============================");
    println!("Press ESC to exit");
    println!("The pattern changes every 3 seconds\n");

    let scale = 6;
    let grid_size = 48;
    let buffer_size = grid_size * scale;
    
    let mut buffer = HeadlessBuffer::new(buffer_size, buffer_size);
    let renderer = RendererCore::new(scale);
    
    match RendererMinifb::new("Velocity Patterns", buffer_size, buffer_size) {
        Ok(mut window) => {
            println!("Window created successfully!\n");
            
            let mut velocity = PatternVelocityField::new(grid_size, grid_size, VelocityPattern::Vortex);
            let density = UniformDensity::new(grid_size, grid_size);
            
            let mut fps_counter = FpsCounter::new();
            let mut pattern_timer = 0.0;
            let pattern_duration = 3.0; // seconds
            
            println!("Current pattern: {}", velocity.pattern_name());
            
            while !window.should_close() {
                // Clear and render
                buffer.clear(0xFF000000);
                renderer.draw_density(&mut buffer, &density);
                renderer.draw_velocity(&mut buffer, &velocity, 0.7);
                
                if !window.present(&buffer) {
                    break;
                }
                
                // Update timers
                pattern_timer += 0.016; // ~60 FPS
                
                // Change pattern periodically
                if pattern_timer >= pattern_duration {
                    pattern_timer = 0.0;
                    velocity.next_pattern();
                    println!("Current pattern: {}", velocity.pattern_name());
                }
                
                // Show FPS occasionally
                if let Some(fps) = fps_counter.update() {
                    println!("  FPS: {:.2}", fps);
                }
            }
            
            println!("\nDemo completed!");
            println!("Average FPS: {:.2}", fps_counter.average_fps());
        }
        Err(e) => {
            println!("Could not create window: {}", e);
            println!("Running headless version...\n");
            
            let mut velocity = PatternVelocityField::new(grid_size, grid_size, VelocityPattern::Vortex);
            let density = UniformDensity::new(grid_size, grid_size);
            
            for _pattern_idx in 0..5 {
                println!("Testing pattern: {}", velocity.pattern_name());
                
                // Render once
                buffer.clear(0xFF000000);
                renderer.draw_density(&mut buffer, &density);
                renderer.draw_velocity(&mut buffer, &velocity, 0.7);
                
                // Sample some velocities
                let samples = vec![(24, 24), (24, 12), (12, 24), (36, 24), (24, 36)];
                for (i, j) in samples {
                    let vx = velocity.velocity_x_at(i, j);
                    let vy = velocity.velocity_y_at(i, j);
                    let mag = (vx * vx + vy * vy).sqrt();
                    println!("  ({:2},{:2}): vx={:5.2}, vy={:5.2}, mag={:.2}", i, j, vx, vy, mag);
                }
                println!();
                
                velocity.next_pattern();
            }
            
            println!("Headless demo completed!");
        }
    }
}