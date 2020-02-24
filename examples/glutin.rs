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
        .with_srgb(true)
        .with_gl(GlRequest::Latest)
        .build_windowed(window_builder, &event_loop)
        .unwrap();
    let context = unsafe {
        let context = context.make_current().unwrap();
        fae::gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
        context
    };

    let mut ctx: Context = Context::new();
    let font: Font = common::create_font(&mut ctx);

    let PhysicalSize { width, height } = context.window().inner_size();
    let mut dpi_factor = context.window().scale_factor() as f32;
    let (mut width, mut height) = (width as f32 / dpi_factor, height as f32 / dpi_factor);

    context.window().set_visible(true);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
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
                ctx.render(width, height);
                context.swap_buffers().unwrap();
                ctx.synchronize();
            }
            Event::MainEventsCleared => {
                fae::profiler::refresh();
                let mut ctx: GraphicsContext = ctx.start_frame(width, height, dpi_factor);
                font.draw(&mut ctx, "Hello, World!", 10.0, 10.0, 16.0)
                    .alignment(Alignment::Left)
                    .color((0.0, 0.5, 0.1, 1.0))
                    .clip_area((0.0, 0.0, width, height))
                    .finish();
                ctx.finish_frame();
                context.window().request_redraw();
            }
            _ => {}
        }
    });
}
