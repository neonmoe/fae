#![windows_subsystem = "windows"]
mod common;

use fae::{Alignment, Context, Font, GraphicsContext};

use sdl2::event::{Event, WindowEvent};

fn main() {
    let sdl = sdl2::init().unwrap();
    let sdl_video = sdl.video().unwrap();

    let window = sdl_video
        .window("SDL2 example", 640, 480)
        .opengl()
        .allow_highdpi()
        .resizable()
        .build()
        .unwrap();

    let gl_context = window.gl_create_context().unwrap();
    fae::gl::load_with(|name| sdl_video.gl_get_proc_address(name) as *const _);

    let mut fae_ctx: Context = Context::new();
    let font: Font = common::create_font(&mut fae_ctx);

    let mut event_pump = sdl.event_pump().unwrap();

    let mut fps_counter = common::FpsCounter::new();
    let mut tick = 0;

    'game_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'game_loop;
                }
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(_, _) => unsafe {
                        let (width, height) = window.drawable_size();
                        fae::gl::Viewport(0, 0, width as i32, height as i32);
                    },
                    _ => {}
                },
                _ => {}
            }
        }

        let (width, height) = (window.size().0 as f32, window.size().1 as f32);
        let physical_width = window.drawable_size().0 as f32;
        let dpi_factor = physical_width / width;

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
        window.gl_swap_window();
        fae_ctx.synchronize();
    }

    drop(gl_context);
}
