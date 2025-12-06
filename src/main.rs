use fluid_renderer::{FrameBuffer, HeadlessBuffer, RendererCore, RendererMinifb};
use std::time::Instant;

fn main() {
    let width = 512;
    let height = 512;
    let scale = 1;

    let mut buffer = HeadlessBuffer::new(width, height);
    let renderer_core = RendererCore::new(scale);

    match RendererMinifb::new("Fluid Renderer", width, height) {
        Ok(mut window) => {
            println!("Window created successfully, running GUI mode");
            run_with_window(&mut buffer, &renderer_core, &mut window);
        }
        Err(e) => {
            println!("Cannot create window: {}. Running headless mode only", e);
            run_headless(&mut buffer, &renderer_core);
        }
    }
}

fn run_with_window(
    buffer: &mut impl FrameBuffer,
    renderer: &RendererCore,
    window: &mut RendererMinifb,
) {
    let start = Instant::now();
    let mut frame_count = 0;

    while !window.should_close() {
        buffer.clear(0xFF000000);
        renderer.draw_gradient(buffer);

        if !window.present(buffer) {
            break;
        }

        frame_count += 1;

        if frame_count % 60 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let fps = frame_count as f64 / elapsed;
            println!("FPS: {:.2} (Frame {})", fps, frame_count);
        }

        if frame_count >= 300 {
            break;
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    let average_fps = frame_count as f64 / elapsed;
    println!("\nGUI test completed!");
    println!("Total frames: {}", frame_count);
    println!("Average FPS: {:.2}", average_fps);
}

fn run_headless(buffer: &mut impl FrameBuffer, renderer: &RendererCore) {
    let start = Instant::now();
    let iterations = 1000;

    for i in 0..iterations {
        buffer.clear(0xFF000000);
        renderer.draw_gradient(buffer);

        if i % 100 == 0 {
            println!("Headless frame {}", i);
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    let fps = iterations as f64 / elapsed;

    println!("\nHeadless test completed!");
    println!("Total frames: {}", iterations);
    println!("Average FPS: {:.2}", fps);
}
