use fluid_renderer::{FrameBuffer, HeadlessBuffer, RendererCore, rgb};
use std::time::Instant;

#[test]
fn test_headless_buffer_operations() {
    let mut buffer = HeadlessBuffer::new(256, 256);

    buffer.clear(0xFF000000);
    assert!(buffer.buffer().iter().all(|&p| p == 0xFF000000));

    buffer.clear(0xFFFFFFFF);
    assert!(buffer.buffer().iter().all(|&p| p == 0xFFFFFFFF));
}

#[test]
fn test_headless_pixel_operations() {
    let mut buffer = HeadlessBuffer::new(100, 100);

    buffer.set_pixel(50, 50, 0xFF00FF00);
    assert_eq!(buffer.get_pixel(50, 50), Some(0xFF00FF00));

    buffer.set_pixel(99, 99, 0xFFFF0000);
    assert_eq!(buffer.get_pixel(99, 99), Some(0xFFFF0000));

    buffer.set_pixel(100, 100, 0xFF0000FF);
    assert_eq!(buffer.get_pixel(100, 100), None);
}

#[test]
fn test_renderer_core_gradient() {
    let renderer = RendererCore::new(1);
    let mut buffer = HeadlessBuffer::new(256, 256);

    buffer.clear(0xFF000000);
    renderer.draw_gradient(&mut buffer);

    assert_eq!(buffer.get_pixel(0, 0), Some(rgb(0, 0, 128)));
    assert_ne!(buffer.get_pixel(128, 128), Some(0xFF000000));
}

#[test]
fn test_headless_performance() {
    let renderer = RendererCore::new(1);
    let mut buffer = HeadlessBuffer::new(512, 512);

    let start = Instant::now();
    let iterations = 1000;

    for _ in 0..iterations {
        buffer.clear(0xFF000000);
        renderer.draw_gradient(&mut buffer);
    }

    let elapsed = start.elapsed();
    let fps = iterations as f64 / elapsed.as_secs_f64();

    println!("Headless performance: {:.2} FPS", fps);
    assert!(fps > 100.0, "Headless rendering should exceed 100 FPS");
}

#[test]
fn test_buffer_size_consistency() {
    let sizes = vec![(256, 256), (512, 512), (1024, 768), (1920, 1080)];

    for (width, height) in sizes {
        let buffer = HeadlessBuffer::new(width, height);
        assert_eq!(buffer.width(), width);
        assert_eq!(buffer.height(), height);
        assert_eq!(buffer.buffer().len(), width * height);
    }
}

#[test]
fn test_clear_performance() {
    let mut buffer = HeadlessBuffer::new(1024, 1024);

    let start = Instant::now();
    for i in 0..10000 {
        buffer.clear(rgb((i % 256) as u8, 0, 0));
    }
    let elapsed = start.elapsed();

    let clears_per_second = 10000.0 / elapsed.as_secs_f64();
    println!("Clear operations per second: {:.0}", clears_per_second);

    assert!(
        clears_per_second > 100.0,
        "Clear operations should exceed 100 per second"
    );
}

#[test]
fn test_pixel_boundary_conditions() {
    let mut buffer = HeadlessBuffer::new(10, 10);

    buffer.set_pixel(0, 0, 0xFFFFFFFF);
    buffer.set_pixel(9, 9, 0xFFFFFFFF);
    buffer.set_pixel(10, 10, 0xFFFFFFFF);
    buffer.set_pixel(usize::MAX, usize::MAX, 0xFFFFFFFF);

    assert_eq!(buffer.get_pixel(0, 0), Some(0xFFFFFFFF));
    assert_eq!(buffer.get_pixel(9, 9), Some(0xFFFFFFFF));
    assert_eq!(buffer.get_pixel(10, 10), None);
    assert_eq!(buffer.get_pixel(usize::MAX, usize::MAX), None);
}
