use gl;
use gl::types::*;
use image::load_image;
use std::error::Error;
use std::mem;
use std::ptr;
use std::sync::Mutex;
use text;

const TEXTURE_COUNT: usize = 2; // UI elements, glyph cache

pub(crate) const DRAW_CALL_INDEX_UI: usize = 0;
pub(crate) const DRAW_CALL_INDEX_TEXT: usize = 1;

type TexQuad = [f32; 30];
type Texture = GLuint;
type VertexBufferObject = GLuint;
type VertexArrayObject = GLuint;
type ShaderProgram = GLuint;

#[derive(Clone, Debug)]
struct DrawCall {
    texture: Texture,
    vbo: VertexBufferObject,
    vao: VertexArrayObject,
    program: ShaderProgram,
    vbo_data: Vec<TexQuad>,
    allocated_vbo_data_size: isize,
}

#[derive(Debug)]
struct DrawState {
    calls: Vec<DrawCall>,
}

lazy_static! {
    static ref DRAW_STATE: Mutex<DrawState> = Mutex::new(DrawState {
        calls: vec![
            DrawCall {
                texture: 0,
                vbo: 0,
                vao: 0,
                program: 0,
                vbo_data: Vec::new(),
                allocated_vbo_data_size: 0,
            };
            TEXTURE_COUNT
        ]
    });
}

static mut PROJECTION_MATRIX_LOCATION: GLint = -1;
const VERTEX_SHADER_SOURCE: [&str; TEXTURE_COUNT] = [
    include_str!("shaders/texquad.vert"),
    include_str!("shaders/text.vert"),
];
const FRAGMENT_SHADER_SOURCE: [&str; TEXTURE_COUNT] = [
    include_str!("shaders/texquad.frag"),
    include_str!("shaders/text.frag"),
];

/// Initialize the UI rendering system. Handled by
/// `window_bootstrap`. This must be done after window and context
/// creation, but before any drawing calls.
///
/// `ui_spritesheet_image` should a Vec of the bytes of a .png file
/// with an alpha channel. To load the image at compile-time, you
/// could run the following (of course, with your own path):
/// ```no_run
/// fungui::initialize_renderer(include_bytes!("resources/gui.png"));
/// ```
pub fn initialize_renderer(ui_spritesheet_image: &[u8]) -> Result<(), Box<Error>> {
    let mut draw_state = DRAW_STATE.lock().unwrap();

    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    for (i, call) in draw_state.calls.iter_mut().enumerate() {
        call.program = create_program(VERTEX_SHADER_SOURCE[i], FRAGMENT_SHADER_SOURCE[i]);
    }

    for call in draw_state.calls.iter_mut() {
        let (vao, vbo) = create_vao();
        call.vao = vao;
        call.vbo = vbo;
    }

    for call in draw_state.calls.iter_mut() {
        call.texture = create_texture();
    }

    let image = load_image(ui_spritesheet_image).unwrap();
    insert_texture(
        draw_state.calls[DRAW_CALL_INDEX_UI].texture,
        gl::RGBA as GLint,
        image.width,
        image.height,
        image.pixels,
    );

    // This creates the glyph cache texture
    insert_texture(
        draw_state.calls[DRAW_CALL_INDEX_TEXT].texture,
        gl::RED as GLint,
        text::GLYPH_CACHE_WIDTH as GLint,
        text::GLYPH_CACHE_HEIGHT as GLint,
        vec![0; (text::GLYPH_CACHE_WIDTH * text::GLYPH_CACHE_HEIGHT) as usize],
    );

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
        if link_status as u8 != gl::TRUE {
            let mut info = [0; 1024];
            gl::GetProgramInfoLog(program, 1024, ptr::null_mut(), info.as_mut_ptr());
            println!(
                "Program linking failed:\n{}",
                String::from_utf8_lossy(&mem::transmute::<[i8; 1024], [u8; 1024]>(info)[..])
            );
        }
        gl::UseProgram(program);

        // FIXME: Convert projection matrix location to an array
        // because there's more than one program now
        PROJECTION_MATRIX_LOCATION =
            gl::GetUniformLocation(program, "projection_matrix\0".as_ptr() as *const _);
    }

    program
}

#[inline]
fn create_vao() -> (VertexArrayObject, VertexBufferObject) {
    let mut vao = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }

    let mut vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    }

    /* Setup position attribute */
    unsafe {
        gl::VertexAttribPointer(
            0,           /* Attrib location */
            3,           /* Components */
            gl::FLOAT,   /* Type */
            gl::FALSE,   /* Normalize */
            20,          /* Stride: sizeof(f32) * (Total component count)*/
            ptr::null(), /* Offset */
        );
        gl::EnableVertexAttribArray(0 /* Attribute location */);
    }

    /* Setup texture coordinate attribute */
    unsafe {
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

    (vao, vbo)
}

#[inline]
fn create_texture() -> GLuint {
    let mut tex = 0;
    unsafe {
        gl::GenTextures(1, &mut tex);
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
    }
    tex
}

#[inline]
fn insert_texture(tex: GLuint, components: GLint, w: GLint, h: GLint, pixels: Vec<u8>) {
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            components,
            w,
            h,
            0,
            components as GLuint,
            gl::UNSIGNED_BYTE,
            pixels.as_ptr() as *const _,
        );
    }
}

pub(crate) fn draw_quad(
    coords: (f32, f32, f32, f32),
    texcoords: (f32, f32, f32, f32),
    z: f32,
    tex_index: usize,
) {
    let (x0, y0, x1, y1) = coords;
    let (tx0, ty0, tx1, ty1) = texcoords;
    let mut draw_state = DRAW_STATE.lock().unwrap();
    let call = &mut draw_state.calls[tex_index];

    let quad: TexQuad = [
        x0, y0, z, tx0, ty0, x1, y0, z, tx1, ty0, x1, y1, z, tx1, ty1, x0, y0, z, tx0, ty0, x1, y1,
        z, tx1, ty1, x0, y1, z, tx0, ty1,
    ];
    call.vbo_data.push(quad);
}

pub(crate) fn render(width: f64, height: f64) {
    let m00 = 2.0 / width as f32;
    let m11 = -2.0 / height as f32;
    let matrix = [
        m00, 0.0, 0.0, -1.0, 0.0, m11, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];

    text::draw_text();

    let mut draw_state = DRAW_STATE.lock().unwrap();
    for (i, call) in draw_state.calls.iter_mut().enumerate() {
        if call.vbo_data.len() == 0 {
            continue;
        }

        unsafe {
            gl::UseProgram(call.program);
            gl::UniformMatrix4fv(PROJECTION_MATRIX_LOCATION, 1, gl::FALSE, matrix.as_ptr());
            gl::BindVertexArray(call.vao);
            gl::BindTexture(gl::TEXTURE_2D, call.texture);
            gl::BindBuffer(gl::ARRAY_BUFFER, call.vbo);
        }

        let buffer_length = (mem::size_of::<TexQuad>() * call.vbo_data.len()) as isize;
        let buffer_ptr = call.vbo_data.as_ptr() as *const _;

        if buffer_length < call.allocated_vbo_data_size {
            unsafe {
                gl::BufferSubData(gl::ARRAY_BUFFER, 0, buffer_length, buffer_ptr);
            }
        } else {
            call.allocated_vbo_data_size = buffer_length;
            unsafe {
                gl::BufferData(gl::ARRAY_BUFFER, buffer_length, buffer_ptr, gl::STREAM_DRAW);
            }
        }

        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, call.vbo_data.len() as i32 * 6);
        }
        call.vbo_data.clear();
        print_gl_errors(&*format!("after render #{}", i));
    }
}

pub(crate) fn get_texture(index: usize) -> GLuint {
    let draw_state = DRAW_STATE.lock().unwrap();
    draw_state.calls[index].texture
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
