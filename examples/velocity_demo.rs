use fluid_renderer::{
    velocity_field::VelocityField, FluidField, FrameBuffer, HeadlessBuffer, 
    RendererCore, RendererMinifb, FpsCounter,
};
use std::time::Instant;

/// Time-varying velocity field that creates interesting patterns
struct AnimatedVelocityField {
    width: usize,
    height: usize,
    time: f32,
}

impl AnimatedVelocityField {
    fn new(width: usize, height: usize) -> Self {
        Self { width, height, time: 0.0 }
    }
    
    fn update(&mut self, dt: f32) {
        self.time += dt;
    }
}

impl VelocityField for AnimatedVelocityField {
    fn grid_width(&self) -> usize {
        self.width
    }

    fn grid_height(&self) -> usize {
        self.height
    }

    fn velocity_x_at(&self, i: usize, j: usize) -> f32 {
        let cx = self.width as f32 / 2.0;
        let cy = self.height as f32 / 2.0;
        let x = i as f32 - cx;
        let y = j as f32 - cy;
        
        // Create a rotating vortex pattern
        let dist = (x * x + y * y).sqrt();
        let angle = self.time * 0.5;
        
        if dist > 0.0 {
            // Tangential velocity with rotation
            let base_vx = -y / dist;
            let rot_vx = base_vx * angle.cos() - (x / dist) * angle.sin();
            rot_vx * (1.0 - dist / (self.width as f32 * 0.5)).max(0.0)
        } else {
            0.0
        }
    }

    fn velocity_y_at(&self, i: usize, j: usize) -> f32 {
        let cx = self.width as f32 / 2.0;
        let cy = self.height as f32 / 2.0;
        let x = i as f32 - cx;
        let y = j as f32 - cy;
        
        // Create a rotating vortex pattern
        let dist = (x * x + y * y).sqrt();
        let angle = self.time * 0.5;
        
        if dist > 0.0 {
            // Tangential velocity with rotation
            let base_vy = x / dist;
            let rot_vy = base_vy * angle.cos() + (y / dist) * angle.sin();
            rot_vy * (1.0 - dist / (self.width as f32 * 0.5)).max(0.0)
        } else {
            0.0
        }
    }
}

/// Background density field with circular gradient
struct CircularDensity {
    width: usize,
    height: usize,
}

impl CircularDensity {
    fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

impl FluidField for CircularDensity {
    fn grid_width(&self) -> usize {
        self.width
    }

    fn grid_height(&self) -> usize {
        self.height
    }

    fn density_at(&self, i: usize, j: usize) -> f32 {
        let cx = self.width as f32 / 2.0;
        let cy = self.height as f32 / 2.0;
        let x = i as f32 - cx;
        let y = j as f32 - cy;
        let dist = ((x * x + y * y).sqrt() / (self.width as f32 * 0.5)).min(1.0);
        
        // Create a dark circular gradient
        (1.0 - dist) * 0.2
    }
}

fn main() {
    println!("Velocity Field Visualization Demo");
    println!("==================================");
    println!("Press ESC to exit\n");

    let scale = 4;
    let grid_size = 64;
    let buffer_size = grid_size * scale;
    
    let mut buffer = HeadlessBuffer::new(buffer_size, buffer_size);
    let renderer = RendererCore::new(scale);
    
    // Create window
    match RendererMinifb::new("Velocity Field Demo", buffer_size, buffer_size) {
        Ok(mut window) => {
            println!("Window created successfully!");
            
            // Create velocity and density fields
            let mut velocity = AnimatedVelocityField::new(grid_size, grid_size);
            let density = CircularDensity::new(grid_size, grid_size);
            
            let mut fps_counter = FpsCounter::new();
            let mut last_time = Instant::now();
            
            while !window.should_close() {
                // Calculate delta time
                let now = Instant::now();
                let dt = now.duration_since(last_time).as_secs_f32();
                last_time = now;
                
                // Update velocity field
                velocity.update(dt);
                
                // Clear buffer
                buffer.clear(0xFF000000);
                
                // Draw density as background
                renderer.draw_density(&mut buffer, &density);
                
                // Draw velocity field overlay
                renderer.draw_velocity(&mut buffer, &velocity, 1.0);
                
                // Present to window
                if !window.present(&buffer) {
                    break;
                }
                
                // Update FPS counter
                if let Some(fps) = fps_counter.update() {
                    println!("FPS: {:.2}", fps);
                }
            }
            
            println!("\nDemo completed!");
            println!("Total frames: {}", fps_counter.frame_count());
            println!("Average FPS: {:.2}", fps_counter.average_fps());
        }
        Err(e) => {
            println!("Could not create window: {}", e);
            println!("Running headless demo instead...\n");
            
            // Run headless version
            let mut velocity = AnimatedVelocityField::new(grid_size, grid_size);
            let density = CircularDensity::new(grid_size, grid_size);
            let mut fps_counter = FpsCounter::new();
            
            for frame in 0..300 {
                velocity.update(0.016); // ~60 FPS timing
                
                buffer.clear(0xFF000000);
                renderer.draw_density(&mut buffer, &density);
                renderer.draw_velocity(&mut buffer, &velocity, 1.0);
                
                if frame % 60 == 0 {
                    println!("Frame {}: Rendered velocity field", frame);
                    
                    // Sample center velocity
                    let center = grid_size / 2;
                    let vx = velocity.velocity_x_at(center, center);
                    let vy = velocity.velocity_y_at(center, center);
                    println!("  Center velocity: ({:.3}, {:.3})", vx, vy);
                }
                
                fps_counter.update();
            }
            
            println!("\nHeadless demo completed!");
            println!("Average FPS: {:.2}", fps_counter.average_fps());
        }
    }
}