#![windows_subsystem = "windows"]
mod common;

use fae::{Alignment, Context, Font, GraphicsContext};

use glutin::dpi::{LogicalSize, PhysicalSize};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, GlRequest};

fn main() -> Result<(), fae::Error> {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("Glutin example")
        .with_inner_size(LogicalSize::new(f64::from(640.0), f64::from(480.0)))
        .with_visible(false);
    let context = ContextBuilder::new()
        .with_vsync(true)
        .with_srgb(true)
        .with_gl(GlRequest::Latest)
        .build_windowed(window_builder, &event_loop)
        .unwrap();
    let context = unsafe {
        let context = context.make_current().unwrap();
        fae::gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
        context
    };

    let mut fae_ctx: Context = Context::new();
    let font: Font = common::create_font(&mut fae_ctx);

    let PhysicalSize { width, height } = context.window().inner_size();
    let mut dpi_factor = context.window().scale_factor() as f32;
    let (mut width, mut height) = (width as f32 / dpi_factor, height as f32 / dpi_factor);

    context.window().set_visible(true);

    let mut fps_counter = common::FpsCounter::new();
    let mut tick = 0;

    event_loop.run(move |event, _, control_flow| match event {
        Event::LoopDestroyed => return,
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                new_inner_size,
            } => {
                dpi_factor = scale_factor as f32;
                width = new_inner_size.width as f32 / dpi_factor;
                height = new_inner_size.height as f32 / dpi_factor;
            }
            WindowEvent::Resized(physical_size) => {
                context.resize(physical_size);
                width = physical_size.width as f32 / dpi_factor;
                height = physical_size.height as f32 / dpi_factor;
                unsafe {
                    fae::gl::Viewport(
                        0,
                        0,
                        physical_size.width as i32,
                        physical_size.height as i32,
                    );
                }
            }
            WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        },
        Event::RedrawRequested(_) => {
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
            context.swap_buffers().unwrap();
        }
        Event::MainEventsCleared => {
            context.window().request_redraw();
        }
        _ => {}
    });
}
