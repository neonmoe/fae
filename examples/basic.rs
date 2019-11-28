mod common;

use common::WindowSettings;
use fae::glutin::event::{Event, WindowEvent};
use fae::glutin::event_loop::ControlFlow;
#[cfg(feature = "text")]
use fae::text::Alignment;
use fae::{DrawCallParameters, Image, Window};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut window = Window::create(WindowSettings::default().into()).unwrap();
    #[cfg(feature = "text")]
    let mut font = common::create_font(&mut window.ctx);
    let params = DrawCallParameters {
        image: {
            #[cfg(feature = "png")]
            let image = Image::from_png(include_bytes!("res/sprite.png"))?;
            #[cfg(not(feature = "png"))]
            let image = Image::from_color(16, 16, &[0xFF, 0xFF, 0x00, 0xFF]);
            Some(image)
        },
        alpha_blending: false,
        ..Default::default()
    };
    let sprite = window.ctx.create_draw_call(params);

    window.run(move |ctx, event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Some(mut ctx) = ctx {
            sprite
                .draw(&mut ctx, 0.5)
                .with_coordinates((0.0, 0.0, 640.0, 480.0))
                .with_texture_coordinates((0, 0, 1240, 920))
                .finish();
            #[cfg(feature = "text")]
            font.draw(&mut ctx, "Some cool text!", 10.0, 10.0, 0.6, 16.0)
                .with_alignment(Alignment::Left)
                .with_color((0.0, 0.5, 0.1, 1.0))
                .finish();
        } else {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                        *control_flow = ControlFlow::Exit
                    }
                    _ => {}
                }
            }
        }
    });
}
