#![windows_subsystem = "windows"]
mod common;

use fae::{Alignment, Font, GraphicsContext};

use glfw::{Context, WindowHint};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(WindowHint::SRgbCapable(true));
    let (mut window, events) = glfw
        .create_window(640, 480, "GLFW example", glfw::WindowMode::Windowed)
        .unwrap();

    window.set_size_polling(true);

    window.make_current();
    fae::gl::load_with(|name| window.get_proc_address(name) as *const _);

    let mut fae_ctx: fae::Context = fae::Context::new();
    let font: Font = common::create_font(&mut fae_ctx);

    let mut fps_counter = common::FpsCounter::new();
    let mut tick = 0;

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            use glfw::WindowEvent;
            match event {
                WindowEvent::Size(width, height) => unsafe {
                    fae::gl::Viewport(0, 0, width, height);
                },
                _ => {}
            }
        }

        let (width, height) = (window.get_size().0 as f32, window.get_size().1 as f32);
        let dpi_factor = 1.0;

        fps_counter.record_frame();
        fae::profiler::refresh();
        let mut ctx: GraphicsContext = fae_ctx.start_frame(width, height, dpi_factor);

        font.draw(&mut ctx, "Hello, World!", 10.0, 10.0, 16.0)
            .alignment(Alignment::Left)
            .color((0.0, 0.5, 0.1, 1.0))
            .clip_area((0.0, 0.0, width, height))
            .finish();

        let fps_text = format!("FPS: {}", fps_counter.get_fps());
        font.draw(&mut ctx, &fps_text, width - 110.0, 10.0, 16.0)
            .alignment(Alignment::Right)
            .max_width(100.0)
            .color((0.0, 0.5, 0.1, 1.0))
            .clip_area((0.0, 0.0, width, height))
            .finish();

        tick += 1;
        font.draw(&mut ctx, "wheee", width / 2.0, height / 2.0, 16.0)
            .alignment(Alignment::Center)
            .rotation(tick as f32 / 20.0, 0.0, 8.0)
            .finish();

        ctx.finish_frame();
        fae_ctx.render(width, height);
        window.swap_buffers();
        fae_ctx.synchronize();
    }
}
