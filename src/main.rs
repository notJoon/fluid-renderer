use fluid_renderer::{
    constants, FpsCounter, FrameBuffer, HeadlessBuffer, RendererCore, RendererMinifb,
};

fn main() {
    let width = constants::DEFAULT_WIDTH;
    let height = constants::DEFAULT_HEIGHT;
    let scale = constants::DEFAULT_SCALE;

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
    let mut fps_counter = FpsCounter::new();
    const MAX_FRAMES: u64 = 300;

    while !window.should_close() {
        buffer.clear(constants::BACKGROUND_COLOR);
        renderer.draw_gradient(buffer);

        if !window.present(buffer) {
            break;
        }

        if let Some(fps) = fps_counter.update() {
            println!("FPS: {:.2} (Frame {})", fps, fps_counter.frame_count());
        }

        if fps_counter.frame_count() >= MAX_FRAMES {
            break;
        }
    }

    println!("\nGUI test completed!");
    println!("Total frames: {}", fps_counter.frame_count());
    println!("Average FPS: {:.2}", fps_counter.average_fps());
}

fn run_headless(buffer: &mut impl FrameBuffer, renderer: &RendererCore) {
    let mut fps_counter = FpsCounter::with_interval(100);
    const ITERATIONS: u64 = 1000;

    for _ in 0..ITERATIONS {
        buffer.clear(constants::BACKGROUND_COLOR);
        renderer.draw_gradient(buffer);

        if fps_counter.update().is_some() {
            println!("Headless frame {}", fps_counter.frame_count());
        }
    }

    println!("\nHeadless test completed!");
    println!("Total frames: {}", fps_counter.frame_count());
    println!("Average FPS: {:.2}", fps_counter.average_fps());
}
