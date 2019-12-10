mod common;

use common::WindowSettings;
use fae::glutin::event::{Event, WindowEvent};
use fae::glutin::event_loop::ControlFlow;
#[cfg(feature = "text")]
use fae::Alignment;
use fae::{DrawCallHandle, DrawCallParameters, Image, Window};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut window = Window::create(WindowSettings::default().into()).unwrap();

    #[cfg(feature = "text")]
    let mut font = common::create_font(&mut window.ctx);
    #[cfg(feature = "png")]
    let image = Image::with_png(include_bytes!("res/sprite.png"))?;
    #[cfg(not(feature = "png"))]
    let image = Image::with_color(16, 16, &[0xFF, 0xFF, 0x00, 0xFF]);

    let sprite = DrawCallHandle::new(
        &mut window.ctx,
        DrawCallParameters {
            image: Some(image),
            alpha_blending: false,
            ..Default::default()
        },
    );

    window.run(move |ctx, event, _, control_flow| {
        if let Some(mut ctx) = ctx {
            *control_flow = ControlFlow::Wait;
            sprite
                .draw(&mut ctx)
                .coordinates((0.0, 0.0, 640.0, 480.0))
                .texture_coordinates((0, 0, 1240, 920))
                .finish();
            #[cfg(feature = "text")]
            font.draw(&mut ctx, "Some cool text!", 10.0, 10.0, 0.6, 16.0)
                .alignment(Alignment::Left)
                .color((0.0, 0.5, 0.1, 1.0))
                .finish();
        } else if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        }
    });
}
