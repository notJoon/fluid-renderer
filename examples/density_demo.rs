use fluid_renderer::{FluidField, HeadlessBuffer, RendererCore, RendererMinifb};
use std::time::Instant;

struct SimpleFluidSim {
    width: usize,
    height: usize,
    density: Vec<f32>,
    time: f32,
}

impl SimpleFluidSim {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            density: vec![0.0; width * height],
            time: 0.0,
        }
    }

    fn update(&mut self, dt: f32) {
        self.time += dt;

        // Create a moving wave pattern
        for j in 0..self.height {
            for i in 0..self.width {
                let cx = self.width as f32 / 2.0;
                let cy = self.height as f32 / 2.0;
                let dx = i as f32 - cx;
                let dy = j as f32 - cy;
                let dist = ((dx * dx + dy * dy).sqrt() - self.time * 20.0).abs();

                // Create rings that fade out
                let value = (1.0 - dist / 50.0).max(0.0);
                let fade = (self.time * 0.5).sin() * 0.5 + 0.5;

                let idx = j * self.width + i;
                self.density[idx] = value * fade;
            }
        }
    }
}

impl FluidField for SimpleFluidSim {
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
    println!("Density Rendering Demo");
    println!("This demo shows animated density patterns");

    let grid_size = 128;
    let scale = 4;
    let window_width = grid_size * scale;
    let window_height = grid_size * scale;

    // Create simulation
    let mut sim = SimpleFluidSim::new(grid_size, grid_size);

    // Try to create window for visualization
    match RendererMinifb::new("Density Demo", window_width, window_height) {
        Ok(mut renderer_minifb) => {
            println!("Window created successfully!");

            let renderer_core = RendererCore::new(scale);
            let mut buffer = HeadlessBuffer::new(window_width, window_height);

            let mut frame_count = 0;
            let mut last_fps_time = Instant::now();

            while renderer_minifb.is_open() {
                // Update simulation
                sim.update(0.016); // ~60 FPS timestep

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
        }
        Err(e) => {
            println!("Could not create window: {}", e);
            println!("Running headless demo instead...");

            // Run headless for a few frames
            let renderer_core = RendererCore::new(scale);
            let mut buffer = HeadlessBuffer::new(window_width, window_height);

            for frame in 0..100 {
                sim.update(0.016);
                renderer_core.draw_density(&mut buffer, &sim);

                if frame % 20 == 0 {
                    println!("Frame {}: Rendered density field", frame);
                }
            }

            println!("Headless rendering complete!");
        }
    }
}
