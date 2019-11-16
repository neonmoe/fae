// SDF testing to try out if it'd be plausible to render very big
// glyphs with them (to avoid eating all of the VRAM).

// TODO: Test out the signed-distance-field crate

use fae::{DrawCallParameters, Image, Rect, Renderer, Shaders, Window, WindowSettings};
use std::error::Error;

static SHADER: &'static str = include_str!("res/sdf_shader.frag");

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let image = generate_sdf(Image::from_png(include_bytes!("res/letter.png"))?);
    let mut window = Window::create(&WindowSettings::default()).unwrap();
    let mut renderer = Renderer::new(&window);
    let params = DrawCallParameters {
        image: { Some(image) },
        magnification_smoothing: true,
        shaders: Shaders {
            //fragment_shader_330: include_str!("../src/shaders/texquad.frag"),
            fragment_shader_330: SHADER,
            ..Default::default()
        },
        ..Default::default()
    };
    let call = renderer.create_draw_call(params);
    while window.refresh() {
        renderer.set_dpi_factor(window.dpi_factor);

        let size = 512.0;
        renderer
            .draw(&call, 0.0)
            .with_coordinates(Rect {
                x: (window.width - size) / 2.0,
                y: (window.height - size) / 2.0,
                width: size,
                height: size,
            })
            .with_texture_coordinates((0, 0, 64, 64))
            .finish();

        let size = 256.0;
        renderer
            .draw(&call, 0.0)
            .with_coordinates(Rect {
                x: (window.width - size) / 2.0 - (size + 20.0) * 1.0,
                y: (window.height - size) / 2.0,
                width: size,
                height: size,
            })
            .with_texture_coordinates((0, 0, 64, 64))
            .finish();

        let size = 128.0;
        renderer
            .draw(&call, 0.0)
            .with_coordinates(Rect {
                x: (window.width - size) / 2.0 - (size + 20.0) * 1.0,
                y: (window.height - size) / 2.0,
                width: size,
                height: size,
            })
            .with_texture_coordinates((0, 0, 64, 64))
            .finish();

        renderer.render(window.width, window.height);
        window.swap_buffers(Some(&renderer));
    }
    Ok(())
}

fn generate_sdf(image: Image) -> Image {
    use std::time::Instant;
    let start = Instant::now();

    let mut result = Image::from_color(64, 64, &[0, 0, 0, 0xFF]);
    let scale = image.width / result.width;
    for y in 0..result.height {
        for x in 0..result.width {
            let r = 8;
            let color = {
                let x = x as i32 * scale + scale / 2;
                let y = y as i32 * scale + scale / 2;
                if image.pixels[((x + y * image.width) * 4 + 3) as usize] == 0xFF {
                    let distance = find_distance(x, y, &image, r, 0);
                    let distance = 0.5 + (distance / r as f32).min(1.0).max(0.0) * 0.5;
                    (0xFF as f32 * distance) as u8
                } else {
                    let distance = find_distance(x, y, &image, r, 0xFF);
                    let distance = 0.5 - (distance / r as f32).min(1.0).max(0.0) * 0.5;
                    (0xFF as f32 * distance) as u8
                }
            };
            let index = ((x + y * result.width) * 4 + 3) as usize;
            result.pixels[index] = color;
        }
    }

    println!("Time: {:?}", Instant::now() - start);
    result
}

fn find_distance(x: i32, y: i32, image: &Image, r: i32, target: u8) -> f32 {
    let mut distance = std::f32::INFINITY;
    for y_ in (y - r).max(0)..(y + r).min(image.height - 1) {
        for x_ in (x - r).max(0)..(x + r).min(image.width - 1) {
            let index = ((x_ + y_ * image.width) * 4 + 3) as usize;
            if image.pixels[index] == target {
                let curr_dist = (((x_ - x) as f32).powf(2.0) + ((y_ - y) as f32).powf(2.0)).sqrt();
                if curr_dist < distance {
                    distance = curr_dist;
                }
            }
        }
    }
    distance
}
