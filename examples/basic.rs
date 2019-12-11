mod common;

use common::WindowSettings;
use fae::glutin::event::{Event, WindowEvent};
use fae::glutin::event_loop::ControlFlow;
#[cfg(feature = "text")]
use fae::{Alignment, Font};
use fae::{Image, Spritesheet, SpritesheetBuilder, Window};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut window = Window::new(WindowSettings::default().into())?;

    #[cfg(feature = "text")]
    let mut font: Font = common::create_font(window.ctx());
    #[cfg(feature = "png")]
    let image = Image::with_png(include_bytes!("res/sprite.png"))?;
    #[cfg(not(feature = "png"))]
    let image = Image::with_color(16, 16, &[0xFF, 0xFF, 0x00, 0xFF]);

    let spritesheet: Spritesheet = SpritesheetBuilder::new()
        .image(image)
        .alpha_blending(false)
        .build(window.ctx());

    window.run(move |ctx, event, _, control_flow| {
        if let Some(mut ctx) = ctx {
            *control_flow = ControlFlow::Wait;
            spritesheet
                .draw(&mut ctx)
                .coordinates((0.0, 0.0, 640.0, 480.0))
                .texture_coordinates((0, 0, 1240, 920))
                .finish();
            #[cfg(feature = "text")]
            font.draw(&mut ctx, "Some cool text!", 10.0, 10.0, 16.0)
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
