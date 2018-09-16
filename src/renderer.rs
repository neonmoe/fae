// FIXME: Get rid of as many unsafes as possible.

use gl;
use gl::types::*;
use image::load_image;
use std::mem;
use std::ptr;

static mut PROJECTION_MATRIX_LOCATION: GLint = -1;
const VERT_SHADER_SOURCE: &'static str = include_str!("shaders/texquad.vert");
const FRAG_SHADER_SOURCE: &'static str = include_str!("shaders/texquad.frag");

pub fn initialize() {
    let create_shader = |t: GLuint, source: &str| {
        let len = [source.len() as GLint].as_ptr();
        let source_ptr = [source.as_ptr() as *const _].as_ptr();
        let shader;
        unsafe {
            shader = gl::CreateShader(t);

            // FIXME: This doesn't actually upload the source when ran with --release.
            gl::ShaderSource(shader, 1, source_ptr, len);

            let mut uploaded = [0; 10];
            gl::GetShaderSource(shader, 10, ptr::null_mut(), uploaded.as_mut_ptr());
            println!("{:?}", uploaded);

            gl::CompileShader(shader);
        }
        shader
    };

    let vert_shader = create_shader(gl::VERTEX_SHADER, VERT_SHADER_SOURCE);
    let frag_shader = create_shader(gl::FRAGMENT_SHADER, FRAG_SHADER_SOURCE);

    let program;
    unsafe {
        program = gl::CreateProgram();
        gl::AttachShader(program, vert_shader);
        gl::AttachShader(program, frag_shader);
        gl::LinkProgram(program);
        let mut link_status = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut link_status);
        if link_status != gl::TRUE as GLint {
            let mut info = [0; 1024];
            gl::GetProgramInfoLog(program, 1024, ptr::null_mut(), info.as_mut_ptr());
            println!(
                "Program linking failed:\n{}",
                String::from_utf8_lossy(&mem::transmute::<[i8; 1024], [u8; 1024]>(info)[..])
            );
        }
        gl::UseProgram(program);

        PROJECTION_MATRIX_LOCATION =
            gl::GetUniformLocation(program, "projection_matrix\0".as_ptr() as *const _);
    }

    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        let mut buffer = 0;
        gl::GenBuffers(1, &mut buffer);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);

        /* Setup position attribute */
        gl::VertexAttribPointer(
            0,             /* Attrib location */
            3,             /* Components */
            gl::FLOAT,     /* Type */
            gl::FALSE,     /* Normalize */
            20,            /* Stride: sizeof(f32) * (Total component count)*/
            0 as *const _, /* Offset */
        );
        gl::EnableVertexAttribArray(0 /* Attribute location */);

        /* Setup texture coordinate attribute */
        gl::VertexAttribPointer(
            1,              /* Attrib location */
            2,              /* Components */
            gl::FLOAT,      /* Type */
            gl::FALSE,      /* Normalize */
            20,             /* Stride: sizeof(f32) * (Total component count)*/
            12 as *const _, /* Offset: sizeof(f32) * (Position's component count) */
        );
        gl::EnableVertexAttribArray(1 /* Attribute location */);
    }

    let mut tex = 0;
    unsafe {
        let image = load_image("images/gui.png").unwrap();

        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl::GenTextures(1, &mut tex);
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as GLint, /* Components in texture */
            image.width,
            image.height,
            0,
            gl::RGBA as GLuint, /* Format of the data */
            gl::UNSIGNED_BYTE,  /* Type of the data*/
            image.pixels.as_ptr() as *const _,
        );
    }

    print_gl_errors("after initialization");
}

static MAX_QUADS: usize = 800_000;
static mut CURRENT_QUAD_COUNT: usize = 0;
static mut VERTICES: [[f32; 30]; 800_000] = [[0.0; 30]; 800_000];

pub(crate) fn draw_quad(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    z: f32,
    tx: f32,
    ty: f32,
    tw: f32,
    th: f32,
) {
    let (x0, y0, x1, y1, tx0, ty0, tx1, ty1) = (x, y, x + w, y + h, tx, ty, tx + tw, ty + th);
    if unsafe { CURRENT_QUAD_COUNT } < MAX_QUADS {
        let quad: [f32; 30] = [
            x0, y0, z, tx0, ty0, x1, y0, z, tx1, ty0, x1, y1, z, tx1, ty1, x0, y0, z, tx0, ty0, x1,
            y1, z, tx1, ty1, x0, y1, z, tx0, ty1,
        ];
        unsafe {
            ptr::copy(
                quad.as_ptr(),
                VERTICES[CURRENT_QUAD_COUNT].as_mut_ptr(),
                mem::size_of::<[f32; 30]>(),
            );
            CURRENT_QUAD_COUNT += 1;
        }
    }
}

pub(crate) fn render(width: f64, height: f64) {
    let m00 = 2.0 / width as f32;
    let m11 = -2.0 / height as f32;
    let matrix = [
        m00, 0.0, 0.0, -1.0, 0.0, m11, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    unsafe {
        if CURRENT_QUAD_COUNT == MAX_QUADS {
            println!("Too many quads!");
        }
        gl::UniformMatrix4fv(PROJECTION_MATRIX_LOCATION, 1, gl::FALSE, matrix.as_ptr());
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (mem::size_of::<[f32; 30]>() * CURRENT_QUAD_COUNT) as isize,
            VERTICES.as_ptr() as *const _,
            gl::STREAM_DRAW,
        );
        gl::DrawArrays(gl::TRIANGLES, 0, CURRENT_QUAD_COUNT as i32 * 6);
        CURRENT_QUAD_COUNT = 0;
    }
}

fn print_gl_errors(context: &str) {
    let mut error = unsafe { gl::GetError() };
    while error != gl::NO_ERROR {
        println!("GL error @ {}: {}", context, gl_error_to_string(error));
        error = unsafe { gl::GetError() };
    }
}

fn gl_error_to_string(error: GLuint) -> &'static str {
    match error {
        0x0500 => "GL_INVALID_ENUM",
        0x0501 => "GL_INVALID_VALUE",
        0x0502 => "GL_INVALID_OPERATION",
        0x0503 => "GL_STACK_OVERFLOW",
        0x0504 => "GL_STACK_UNDERFLOW",
        0x0505 => "GL_OUT_OF_MEMORY",
        0x0506 => "GL_INVALID_FRAMEBUFFER_OPERATION",
        0x0507 => "GL_CONTEXT_LOST",
        0x0531 => "GL_TABLE_TOO_LARGE",
        _ => "unknown error",
    }
}
