// FIXME: Consider cleaning up some unnecessary unsafes

mod text;
pub(crate) use self::text::queue_text;
pub use self::text::{initialize_font, update_dpi};

use gl;
use gl::types::*;
use image::load_image;
use std::error::Error;
use std::mem;
use std::ptr;

const MAX_QUADS: usize = 16_000_000 / mem::size_of::<TexQuad>(); // 16 MB vertex buffers
const TEXTURE_COUNT: usize = 2; // UI elements, glyph cache

type TexQuad = [f32; 30];
type VertexBufferData = [TexQuad; MAX_QUADS];
type Texture = GLuint;
type VertexBufferObject = GLuint;
type VertexArrayObject = GLuint;
type ShaderProgram = GLuint;

static mut QUAD_COUNTS: [usize; TEXTURE_COUNT] = [0; TEXTURE_COUNT];
/// The textures are always in the same order:
/// [GUI elements spritesheet, Glyph Cache]
static mut TEXTURES: [Texture; TEXTURE_COUNT] = [0; TEXTURE_COUNT];
static mut VBOS: [VertexBufferObject; TEXTURE_COUNT] = [0; TEXTURE_COUNT];
static mut VAOS: [VertexArrayObject; TEXTURE_COUNT] = [0; TEXTURE_COUNT];
static mut SHADER_PROGRAMS: [ShaderProgram; TEXTURE_COUNT] = [0; TEXTURE_COUNT];
static mut VERTEX_BUFFERS: [VertexBufferData; TEXTURE_COUNT] =
    [[[0.0; 30]; MAX_QUADS]; TEXTURE_COUNT];

static mut PROJECTION_MATRIX_LOCATION: GLint = -1;
const VERTEX_SHADER_SOURCE: [&'static str; TEXTURE_COUNT] = [
    include_str!("../shaders/texquad.vert"),
    include_str!("../shaders/text.vert"),
];
const FRAGMENT_SHADER_SOURCE: [&'static str; TEXTURE_COUNT] = [
    include_str!("../shaders/texquad.frag"),
    include_str!("../shaders/text.frag"),
];

pub fn initialize(ui_spritesheet_image: Vec<u8>) -> Result<(), Box<Error>> {
    unsafe {
        //gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    unsafe {
        for i in 0..TEXTURE_COUNT {
            let program = create_program(VERTEX_SHADER_SOURCE[i], FRAGMENT_SHADER_SOURCE[i]);
            SHADER_PROGRAMS[i] = program;
        }
    }

    unsafe {
        for i in 0..TEXTURE_COUNT {
            let (vao, vbo) = create_vao();
            VAOS[i] = vao;
            VBOS[i] = vbo;
        }
    }

    unsafe {
        for i in 0..TEXTURE_COUNT {
            TEXTURES[i] = create_texture();
        }

        let image = load_image(&*ui_spritesheet_image).unwrap();
        gl::BindTexture(gl::TEXTURE_2D, TEXTURES[0]);
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

        // This creates the glyph cache texture
        gl::BindTexture(gl::TEXTURE_2D, TEXTURES[1]);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RED as GLint, /* Components in texture */
            text::GLYPH_CACHE_WIDTH as GLint,
            text::GLYPH_CACHE_HEIGHT as GLint,
            0,
            gl::RED as GLuint, /* Format of the data */
            gl::UNSIGNED_BYTE, /* Type of the data */
            vec![0; (text::GLYPH_CACHE_WIDTH * text::GLYPH_CACHE_HEIGHT) as usize].as_ptr()
                as *const _,
        );
    }

    print_gl_errors("after initialization");
    Ok(())
}

#[inline]
fn create_program(vert_source: &str, frag_source: &str) -> ShaderProgram {
    let program;
    unsafe {
        program = gl::CreateProgram();

        let vert_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(
            vert_shader,
            1,
            [vert_source.as_ptr() as *const _].as_ptr(),
            [vert_source.len() as GLint].as_ptr(),
        );
        gl::CompileShader(vert_shader);
        let frag_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(
            frag_shader,
            1,
            [frag_source.as_ptr() as *const _].as_ptr(),
            [frag_source.len() as GLint].as_ptr(),
        );
        gl::CompileShader(frag_shader);

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

    program
}

#[inline]
unsafe fn create_vao() -> (VertexArrayObject, VertexBufferObject) {
    let mut vao = 0;
    gl::GenVertexArrays(1, &mut vao);
    gl::BindVertexArray(vao);

    let mut vbo = 0;
    gl::GenBuffers(1, &mut vbo);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

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

    (vao, vbo)
}

#[inline]
unsafe fn create_texture() -> GLuint {
    let mut tex = 0;
    gl::GenTextures(1, &mut tex);
    gl::BindTexture(gl::TEXTURE_2D, tex);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
    tex
}

pub(crate) fn draw_quad(
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    z: f32,
    tx0: f32,
    ty0: f32,
    tx1: f32,
    ty1: f32,
    tex_index: usize,
) {
    if unsafe { QUAD_COUNTS[tex_index] } < MAX_QUADS {
        let quad: TexQuad = [
            x0, y0, z, tx0, ty0, x1, y0, z, tx1, ty0, x1, y1, z, tx1, ty1, x0, y0, z, tx0, ty0, x1,
            y1, z, tx1, ty1, x0, y1, z, tx0, ty1,
        ];
        unsafe {
            ptr::copy(
                quad.as_ptr(),
                VERTEX_BUFFERS[tex_index][QUAD_COUNTS[tex_index]].as_mut_ptr(),
                mem::size_of::<TexQuad>(),
            );
            QUAD_COUNTS[tex_index] += 1;
        }
    } else {
        println!("Too many quads!");
    }
}

pub(crate) fn render(width: f64, height: f64) {
    let m00 = 2.0 / width as f32;
    let m11 = -2.0 / height as f32;
    let matrix = [
        m00, 0.0, 0.0, -1.0, 0.0, m11, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];

    text::draw_text();

    for tex_index in 0..TEXTURE_COUNT {
        unsafe {
            gl::UseProgram(SHADER_PROGRAMS[tex_index]);
            gl::UniformMatrix4fv(PROJECTION_MATRIX_LOCATION, 1, gl::FALSE, matrix.as_ptr());

            gl::BindVertexArray(VAOS[tex_index]);
            gl::BindTexture(gl::TEXTURE_2D, TEXTURES[tex_index]);
            gl::BindBuffer(gl::ARRAY_BUFFER, VBOS[tex_index]);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mem::size_of::<TexQuad>() * QUAD_COUNTS[tex_index]) as isize,
                VERTEX_BUFFERS[tex_index].as_ptr() as *const _,
                gl::STREAM_DRAW,
            );
            gl::DrawArrays(gl::TRIANGLES, 0, QUAD_COUNTS[tex_index] as i32 * 6);
            QUAD_COUNTS[tex_index] = 0;
        }
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
