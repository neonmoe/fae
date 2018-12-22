//! This module does the OpenGL stuff.

use crate::gl;
use crate::gl::types::*;
use crate::image::Image;
use std::error::Error;
use std::mem;
use std::ptr;

type TextureHandle = GLuint;
type VBOHandle = GLuint;
type VAOHandle = GLuint;

/// Represents the shader code for a shader. Used in [`Renderer::create_draw_call`].
#[derive(Clone, Copy, Debug)]
pub struct Shaders {
    /// The GLSL 3.30 version of the vertex shader. Ensure that the
    /// first line is `#version 330`!
    pub vertex_shader_330: &'static str,
    /// The GLSL 3.30 version of the fragment shader. Ensure that the
    /// first line is `#version 330`!
    pub fragment_shader_330: &'static str,
    /// The GLSL 1.10 version of the vertex shader. Ensure that the
    /// first line is `#version 110`!
    pub vertex_shader_110: &'static str,
    /// The GLSL 1.10 version of the fragment shader. Ensure that the
    /// first line is `#version 110`!
    pub fragment_shader_110: &'static str,
}

#[derive(Clone, Copy, Debug)]
struct ShaderProgram {
    program: GLuint,
    vertex_shader: GLuint,
    fragment_shader: GLuint,
    projection_matrix_location: GLint,
    position_attrib_location: GLuint,
    texcoord_attrib_location: GLuint,
    color_attrib_location: GLuint,
    rotation_attrib_location: GLuint,
    depth_attrib_location: GLuint,
    shared_position_attrib_location: GLuint,
    shared_texcoord_attrib_location: GLuint,
}

#[derive(Clone, Debug)]
struct Attributes {
    vbo: VBOHandle,
    vbo_static: VBOHandle,
    element_buffer: VBOHandle,
    vao: VAOHandle,
    vbo_data: Vec<f32>,
    allocated_vbo_data_size: isize,
}

#[derive(Clone, Debug)]
struct DrawCall {
    texture: TextureHandle,
    program: ShaderProgram,
    attributes: Attributes,
}

#[derive(Clone, Copy, Debug)]
struct OpenGLState {
    legacy: bool,
    // The fields below are settings set by other possible OpenGL
    // calls made in the surrounding program, because the point of
    // this crate is to behave well with other OpenGL code running
    // alongside it.
    pushed: bool,
    depth_test: bool,
    blend: bool,
    blend_func: (GLint, GLint),
    program: GLint,
    vao: GLint,
    texture: GLint,
    vbo: GLint,
    element_buffer: GLint,
}

/// Contains the data and functionality needed to draw rectangles with
/// OpenGL. **Requires** a valid OpenGL context.
#[derive(Debug)]
pub struct Renderer {
    calls: Vec<DrawCall>,
    gl_state: OpenGLState,
}

#[derive(Debug, Clone)]
pub struct DrawCallParameters {
    pub image: Option<Image>,
    pub shaders: Option<Shaders>,
    pub minification_smoothing: bool,
    pub magnification_smoothing: bool,
}

impl Default for DrawCallParameters {
    fn default() -> DrawCallParameters {
        DrawCallParameters {
            image: None,
            shaders: None,
            minification_smoothing: true,
            magnification_smoothing: false,
        }
    }
}

impl Renderer {
    /// Create a new UI rendering system. This must be done after window
    /// and context creation, but before any drawing calls.
    ///
    /// `opengl21` disables some post-OpenGL 2.1 functionality, like
    /// VAOs. Ideally this should be `false` in all cases where the OpenGL
    /// version is >=3.0 (or OpenGL ES >=3) to allow for more optimized
    /// rendering.
    pub fn create(opengl21: bool) -> Result<Renderer, Box<Error>> {
        Ok(Renderer {
            calls: Vec::with_capacity(2),
            gl_state: OpenGLState {
                legacy: opengl21,
                pushed: false,
                depth_test: false,
                blend: false,
                blend_func: (0, 0),
                program: 0,
                vao: 0,
                texture: 0,
                vbo: 0,
                element_buffer: 0,
            },
        })
    }

    /// Creates a new draw call in the pipeline, and returns its
    /// index. Using the index, you can call [`Renderer::draw_quad`]
    /// to draw sprites from your image. As a rule of thumb, try to
    /// minimize the amount of draw calls.
    ///
    /// If you want to use your own GLSL shaders, you can provide them
    /// with the `shaders` parameter. Use `None` for defaults. Make
    /// sure to study the uniform variables and attributes of the
    /// default shaders before making your own.
    pub fn create_draw_call(&mut self, params: DrawCallParameters) -> usize {
        self.gl_push();

        let shaders = params.shaders.unwrap_or(DEFAULT_QUAD_SHADERS);
        let (vert, frag) = if self.gl_state.legacy {
            (shaders.vertex_shader_110, shaders.fragment_shader_110)
        } else {
            (shaders.vertex_shader_330, shaders.fragment_shader_330)
        };
        let index = self.calls.len();

        let program = create_program(&vert, &frag, self.gl_state.legacy);
        let attributes = create_attributes(self.gl_state.legacy, program);
        let filter = |smoothed| if smoothed { gl::LINEAR } else { gl::NEAREST } as i32;
        let texture = create_texture(
            filter(params.minification_smoothing),
            filter(params.magnification_smoothing),
        );
        self.calls.push(DrawCall {
            texture,
            program,
            attributes,
        });

        if let Some(image) = params.image {
            insert_texture(
                self.calls[index].texture,
                image.format,
                image.width,
                image.height,
                &image.pixels,
            );
        }

        self.gl_pop();
        index
    }

    /// Draws a rectangle with a ninepatch texture on the screen. This
    /// is very similar to [`Renderer::draw_quad`], except that
    /// stretching is handled differently.
    ///
    /// Specifically: The sprite is split into 3x3 tiles. Corner tiles
    /// are not stretched, the middle tiles of each side are stretched
    /// along one axis, and the middle-on-both-axis tile is stretched
    /// on both axis.
    ///
    /// - `ninepatch_dimensions`: Contains the widths and the heights
    /// of the tiles. Arrangement: ((left tile width, middle, right),
    /// (top tile height, middle, bottom))
    ///
    /// See [`Renderer::draw_quad`] for the rest of the parameters'
    /// docs.
    // TODO: Add rotation parameter
    pub fn draw_quad_ninepatch(
        &mut self,
        ninepatch_dimensions: ((f32, f32, f32), (f32, f32, f32)),
        coords: (f32, f32, f32, f32),
        texcoords: (f32, f32, f32, f32),
        color: (f32, f32, f32, f32),
        z: f32,
        call_index: usize,
    ) {
        let ((w0, w1, w2), (h0, h1, h2)) = ninepatch_dimensions;
        let (x0, y0, x1, y1) = coords;
        let (tx0, ty0, tx1, ty1) = texcoords;
        let (tex_w, tex_h) = (tx1 - tx0, ty1 - ty0);
        let (total_w, total_h) = (w0 + w1 + w2, h0 + h1 + h2);
        let ((tw0, th0), (tw2, th2)) = (
            (w0 / total_w * tex_w, h0 / total_h * tex_h),
            (w2 / total_w * tex_w, h2 / total_h * tex_h),
        );

        let create_tiles = |min: f32, max: f32, margin_min: f32, margin_max: f32| {
            let mins = [min, min + margin_min, max - margin_max];
            let maxes = [min + margin_min, max - margin_max, max];
            (mins, maxes)
        };
        let (x0, x1) = create_tiles(x0, x1, w0, w2);
        let (y0, y1) = create_tiles(y0, y1, h0, h2);
        let (tx0, tx1) = create_tiles(tx0, tx1, tw0, tw2);
        let (ty0, ty1) = create_tiles(ty0, ty1, th0, th2);

        for i in 0..9 {
            let xi = i % 3;
            let yi = i / 3;
            self.draw_quad(
                (x0[xi], y0[yi], x1[xi], y1[yi]),
                (tx0[xi], ty0[yi], tx1[xi], ty1[yi]),
                color,
                (0.0, 0.0, 0.0),
                z,
                call_index,
            );
        }
    }

    /// Draws a textured rectangle on the screen, but only the parts
    /// inside `clip_area`.
    ///
    /// - `clip_area`: The coordinates of the corners of the clipping
    /// area, in (logical) pixels. Arrangement: (left, top, right, bottom)
    ///
    /// See [`Renderer::draw_quad`] for the rest of the parameters'
    /// docs.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_quad_clipped(
        &mut self,
        coords: (f32, f32, f32, f32),
        texcoords: (f32, f32, f32, f32),
        color: (f32, f32, f32, f32),
        rotation: (f32, f32, f32),
        z: f32,
        call_index: usize,
        clip_area: (f32, f32, f32, f32),
    ) {
        let (cx0, cy0, cx1, cy1) = clip_area; // Clip coords
        let (ox0, oy0, ox1, oy1) = coords; // Original coords
        if ox0 > cx1 || ox1 < cx0 || oy0 > cy1 || oy1 < cy0 {
            return;
        }
        let (ow, oh) = (ox1 - ox0, oy1 - oy0);
        let (x0, y0, x1, y1) = (
            // Real coords
            ox0.max(cx0).min(cx1),
            oy0.max(cy0).min(cy1),
            ox1.max(cx0).min(cx1),
            oy1.max(cy0).min(cy1),
        );
        let (tx0, ty0, tx1, ty1) = texcoords;
        let (tw, th) = (tx1 - tx0, ty1 - ty0);
        let texcoords = (
            tx0.max(tx0 + tw * (x0 - ox0) / ow),
            ty0.max(ty0 + th * (y0 - oy0) / oh),
            tx1.min(tx1 + tw * (x1 - ox1) / ow),
            ty1.min(ty1 + th * (y1 - oy1) / oh),
        );

        self.draw_quad((x0, y0, x1, y1), texcoords, color, rotation, z, call_index);
    }

    /// Draws a tinted rectangle on the screen, without any texturing.
    ///
    /// Basically a shorthand for [`Renderer::draw_quad`] with the
    /// `texcoords` set to (-1.0, -1.0, -1.0, -1.0) which are a
    /// symbolic value for "don't use the texture".
    pub fn draw_quad_tinted(
        &mut self,
        coords: (f32, f32, f32, f32),
        color: (f32, f32, f32, f32),
        rotation: (f32, f32, f32),
        z: f32,
        call_index: usize,
    ) {
        self.draw_quad(
            coords,
            (-1.0, -1.0, -1.0, -1.0),
            color,
            rotation,
            z,
            call_index,
        );
    }

    /// Draws a textured rectangle on the screen.
    ///
    /// - `coords`: The coordinates of the corners of the quad, in
    /// (logical) pixels. Arrangement: (left, top, right, bottom)
    ///
    /// - `texcoords`: The texture coordinates (UVs) of the quad, in the
    /// range 0.0 - 1.0. Same arrangement as `coords`.
    ///
    /// - `color`: The color tint of the quad, in the range
    /// 0-255. Arrangement: (red, green, blue, alpha)
    ///
    /// - `rotation`: The rotation of the quad, in radians, and the
    /// point (relative to `coords` x and y, in logical pixels as well)
    /// around which the sprite pivots. Arrangement: (radians, x, y)
    ///
    /// - `z`: Used for ordering sprites on screen, in the range -1.0 -
    /// 1.0. Positive values are in front.
    ///
    /// - `call_index`: The index of the draw call to draw the quad
    /// in. This is the returned value from [`Renderer::create_draw_call`].
    #[inline]
    pub fn draw_quad(
        &mut self,
        coords: (f32, f32, f32, f32),
        texcoords: (f32, f32, f32, f32),
        color: (f32, f32, f32, f32),
        rotation: (f32, f32, f32),
        depth: f32,
        call_index: usize,
    ) {
        let (x0, y0, x1, y1) = coords;
        let (tx0, ty0, tx1, ty1) = texcoords;
        let (red, green, blue, alpha) = color;
        let (rads, pivot_x, pivot_y) = rotation;

        if self.gl_state.legacy {
            let (pivot_x, pivot_y) = (pivot_x + x0, pivot_y + y0);

            // 6 vertices, each of which consist of: position (x, y,
            // z), texcoord (x, y), colors (r, g, b, a), rotation
            // rads, rotation pivot (x, y)
            //
            // I apologize for this mess.
            let quad = [
                x0, y0, depth, tx0, ty0, red, green, blue, alpha, rads, pivot_x, pivot_y, x1, y0,
                depth, tx1, ty0, red, green, blue, alpha, rads, pivot_x, pivot_y, x1, y1, depth,
                tx1, ty1, red, green, blue, alpha, rads, pivot_x, pivot_y, x0, y0, depth, tx0, ty0,
                red, green, blue, alpha, rads, pivot_x, pivot_y, x1, y1, depth, tx1, ty1, red,
                green, blue, alpha, rads, pivot_x, pivot_y, x0, y1, depth, tx0, ty1, red, green,
                blue, alpha, rads, pivot_x, pivot_y,
            ];

            self.calls[call_index]
                .attributes
                .vbo_data
                .extend_from_slice(&quad);
        } else {
            let (width, height, tw, th) = (x1 - x0, y1 - y0, tx1 - tx0, ty1 - ty0);
            let quad = [
                x0, y0, width, height, tx0, ty0, tw, th, red, green, blue, alpha, rads, pivot_x,
                pivot_y, depth,
            ];
            self.calls[call_index]
                .attributes
                .vbo_data
                .extend_from_slice(&quad);
        }
    }

    pub fn render(&mut self, width: f32, height: f32) {
        let m00 = 2.0 / width;
        let m11 = -2.0 / height;
        let matrix = [
            m00, 0.0, 0.0, -1.0, 0.0, m11, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];

        unsafe {
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        self.gl_push();
        let legacy = self.gl_state.legacy;

        // The call order is reversed so that the text layer draws
        // last, for optimal text rendering quality.
        for (i, call) in self.calls.iter_mut().enumerate().rev() {
            if call.attributes.vbo_data.is_empty() {
                continue;
            }

            unsafe {
                gl::UseProgram(call.program.program);
                gl::UniformMatrix4fv(
                    call.program.projection_matrix_location,
                    1,
                    gl::FALSE,
                    matrix.as_ptr(),
                );
                if !legacy {
                    gl::BindVertexArray(call.attributes.vao);
                    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, call.attributes.element_buffer);
                }
                gl::BindTexture(gl::TEXTURE_2D, call.texture);
                gl::BindBuffer(gl::ARRAY_BUFFER, call.attributes.vbo);
            }
            print_gl_errors(&format!("after initializing draw call #{}", i));

            let len = (mem::size_of::<f32>() * call.attributes.vbo_data.len()) as isize;
            let ptr = call.attributes.vbo_data.as_ptr() as *const _;
            if len <= call.attributes.allocated_vbo_data_size {
                unsafe {
                    gl::BufferSubData(gl::ARRAY_BUFFER, 0, len, ptr);
                }
            } else {
                call.attributes.allocated_vbo_data_size = len;
                unsafe {
                    gl::BufferData(gl::ARRAY_BUFFER, len, ptr, gl::STREAM_DRAW);
                }
            }
            print_gl_errors(&format!("after pushing vertex buffer #{}", i));

            if legacy {
                // 12 floats (3 for pos + 2 tex + 4 col + 3 rot) per vertex
                let vertex_count = call.attributes.vbo_data.len() as i32 / 12;
                unsafe {
                    enable_vertex_attribs(&[
                        (call.program.position_attrib_location, 3),
                        (call.program.texcoord_attrib_location, 2),
                        (call.program.color_attrib_location, 4),
                        (call.program.rotation_attrib_location, 3),
                    ]);
                    gl::DrawArrays(gl::TRIANGLES, 0, vertex_count);
                    disable_vertex_attribs(&[
                        call.program.position_attrib_location,
                        call.program.texcoord_attrib_location,
                        call.program.color_attrib_location,
                        call.program.rotation_attrib_location,
                    ]);
                }
                print_gl_errors(&format!("[legacy] after drawing buffer #{}", i));
            } else {
                // 16 floats (4 for x,y,w,h + 4 tex xywh + 4 col + 3 rot + 1 z) per vertex
                let instance_count = call.attributes.vbo_data.len() as i32 / 16;
                unsafe {
                    gl::DrawElementsInstanced(
                        gl::TRIANGLES,
                        6,
                        gl::UNSIGNED_BYTE,
                        ptr::null(),
                        instance_count,
                    );
                }
                print_gl_errors(&format!("after drawing buffer #{}", i));
            }

            call.attributes.vbo_data.clear();

            print_gl_errors(&*format!("after render #{}", i));
        }
        self.gl_pop();
    }

    /// Returns the OpenGL texture handle for the texture used by draw
    /// call `index`.
    #[cfg(feature = "text")]
    pub(crate) fn get_texture(&self, index: usize) -> GLuint {
        self.calls[index].texture
    }

    /// Saves the current OpenGL state for [`Renderer::gl_pop`] and
    /// then sets some defaults used by this crate.
    fn gl_push(&mut self) {
        if !self.gl_state.pushed {
            unsafe {
                self.gl_state.depth_test = gl::IsEnabled(gl::DEPTH_TEST) != 0;
                self.gl_state.blend = gl::IsEnabled(gl::BLEND) != 0;
                let mut src = 0;
                let mut dst = 0;
                gl::GetIntegerv(gl::BLEND_SRC, &mut src);
                gl::GetIntegerv(gl::BLEND_DST, &mut dst);
                self.gl_state.blend_func = (src, dst);
                gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut self.gl_state.program);
                if !self.gl_state.legacy {
                    gl::GetIntegerv(gl::VERTEX_ARRAY_BINDING, &mut self.gl_state.vao);
                }
                gl::GetIntegerv(gl::TEXTURE_BINDING_2D, &mut self.gl_state.texture);
                gl::GetIntegerv(gl::ARRAY_BUFFER_BINDING, &mut self.gl_state.vbo);
                gl::GetIntegerv(
                    gl::ELEMENT_ARRAY_BUFFER_BINDING,
                    &mut self.gl_state.element_buffer,
                );
            }

            unsafe {
                gl::Enable(gl::DEPTH_TEST);
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            }

            self.gl_state.pushed = true;
            print_gl_errors("after glEnables");
        }
    }

    /// Restores the OpenGL state saved in [`Renderer::gl_push`].
    fn gl_pop(&mut self) {
        if self.gl_state.pushed {
            unsafe {
                if !self.gl_state.depth_test {
                    gl::Disable(gl::DEPTH_TEST);
                }
                if !self.gl_state.blend {
                    gl::Disable(gl::BLEND);
                }
                gl::BlendFunc(
                    self.gl_state.blend_func.0 as GLuint,
                    self.gl_state.blend_func.1 as GLuint,
                );
                gl::UseProgram(self.gl_state.program as GLuint);
                if !self.gl_state.legacy {
                    gl::BindVertexArray(self.gl_state.vao as GLuint);
                }
                gl::BindTexture(gl::TEXTURE_2D, self.gl_state.texture as GLuint);
                gl::BindBuffer(gl::ARRAY_BUFFER, self.gl_state.vbo as GLuint);
                gl::BindBuffer(
                    gl::ELEMENT_ARRAY_BUFFER,
                    self.gl_state.element_buffer as GLuint,
                );
            }
            self.gl_state.pushed = false;
            print_gl_errors("after restoring OpenGL state");
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let legacy = self.gl_state.legacy;
        for call in self.calls.iter() {
            let ShaderProgram {
                program,
                vertex_shader,
                fragment_shader,
                ..
            } = call.program;
            let Attributes {
                vbo,
                vbo_static,
                element_buffer,
                vao,
                ..
            } = call.attributes;
            unsafe {
                gl::DeleteShader(vertex_shader);
                gl::DeleteShader(fragment_shader);
                gl::DeleteProgram(program);
                gl::DeleteTextures(1, [call.texture].as_ptr());
                gl::DeleteBuffers(1, [vbo].as_ptr());
                if !legacy {
                    gl::DeleteBuffers(2, [vbo_static, element_buffer].as_ptr());
                    gl::DeleteVertexArrays(1, [vao].as_ptr());
                }
            }
        }
    }
}

const DEFAULT_QUAD_SHADERS: Shaders = Shaders {
    vertex_shader_110: include_str!("shaders/legacy/texquad.vert"),
    fragment_shader_110: include_str!("shaders/legacy/texquad.frag"),
    vertex_shader_330: include_str!("shaders/texquad.vert"),
    fragment_shader_330: include_str!("shaders/texquad.frag"),
};

#[inline]
fn create_program(vert_source: &str, frag_source: &str, legacy: bool) -> ShaderProgram {
    let print_shader_error = |shader, shader_type| unsafe {
        let mut compilation_status = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compilation_status);
        if compilation_status as u8 != gl::TRUE {
            let mut info = [0; 1024];
            gl::GetShaderInfoLog(shader, 1024, ptr::null_mut(), info.as_mut_ptr());

            let error_msg = format!(
                "Shader ({}) compilation failed:\n{}",
                shader_type,
                String::from_utf8_lossy(&mem::transmute::<[i8; 1024], [u8; 1024]>(info)[..])
            );
            debug_assert!(false, "{}", error_msg);
            eprintln!("{}", error_msg);
        }
    };

    let program;
    let vertex_shader;
    let fragment_shader;

    unsafe {
        program = gl::CreateProgram();

        vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(
            vertex_shader,
            1,
            [vert_source.as_ptr() as *const _].as_ptr(),
            [vert_source.len() as GLint].as_ptr(),
        );
        gl::CompileShader(vertex_shader);
        print_shader_error(vertex_shader, "vertex");

        fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(
            fragment_shader,
            1,
            [frag_source.as_ptr() as *const _].as_ptr(),
            [frag_source.len() as GLint].as_ptr(),
        );
        gl::CompileShader(fragment_shader);
        print_shader_error(fragment_shader, "fragment");

        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);
        let mut link_status = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut link_status);
        if link_status as u8 != gl::TRUE {
            let mut info = [0; 1024];
            gl::GetProgramInfoLog(program, 1024, ptr::null_mut(), info.as_mut_ptr());

            let error_msg = format!(
                "Program linking failed:\n{}",
                String::from_utf8_lossy(&mem::transmute::<[i8; 1024], [u8; 1024]>(info)[..])
            );
            debug_assert!(false, "{}", error_msg);
            eprintln!("{}", error_msg);
        }
        print_gl_errors("after shader program creation");
    }

    let projection_matrix_location;
    let position_attrib_location;
    let texcoord_attrib_location;
    let color_attrib_location;
    let rotation_attrib_location;
    let mut depth_attrib_location = 0;
    let mut shared_position_attrib_location = 0;
    let mut shared_texcoord_attrib_location = 0;
    unsafe {
        gl::UseProgram(program);
        projection_matrix_location =
            gl::GetUniformLocation(program, "projection_matrix\0".as_ptr() as *const _);
        position_attrib_location =
            gl::GetAttribLocation(program, "position\0".as_ptr() as *const _) as GLuint;
        texcoord_attrib_location =
            gl::GetAttribLocation(program, "texcoord\0".as_ptr() as *const _) as GLuint;
        color_attrib_location =
            gl::GetAttribLocation(program, "color\0".as_ptr() as *const _) as GLuint;
        rotation_attrib_location =
            gl::GetAttribLocation(program, "rotation\0".as_ptr() as *const _) as GLuint;

        if !legacy {
            depth_attrib_location =
                gl::GetAttribLocation(program, "depth\0".as_ptr() as *const _) as GLuint;
            shared_position_attrib_location =
                gl::GetAttribLocation(program, "shared_position\0".as_ptr() as *const _) as GLuint;
            shared_texcoord_attrib_location =
                gl::GetAttribLocation(program, "shared_texcoord\0".as_ptr() as *const _) as GLuint;
        }

        print_gl_errors("after searching for attribute locations");
    }

    ShaderProgram {
        program,
        vertex_shader,
        fragment_shader,
        projection_matrix_location,
        position_attrib_location,
        texcoord_attrib_location,
        color_attrib_location,
        rotation_attrib_location,
        depth_attrib_location,
        shared_position_attrib_location,
        shared_texcoord_attrib_location,
    }
}

#[inline]
fn create_attributes(opengl21: bool, program: ShaderProgram) -> Attributes {
    let mut vao = 0;
    if !opengl21 {
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        }
    }

    let mut vbo_static = 0;
    let mut element_buffer = 0;
    if !opengl21 {
        unsafe {
            gl::GenBuffers(1, &mut vbo_static);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo_static);
            enable_vertex_attribs(&[
                (program.shared_position_attrib_location, 2),
                (program.shared_texcoord_attrib_location, 2),
            ]);
            // This is the vertices of two triangles that form a quad,
            // interleaved in a (pos x, pos y, tex x, tex y)
            // arrangement.
            let static_quad_vertices: [f32; 16] = [
                0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0,
            ];
            let len = (mem::size_of::<f32>() * static_quad_vertices.len()) as isize;
            let ptr = static_quad_vertices.as_ptr() as *const _;
            gl::BufferData(gl::ARRAY_BUFFER, len, ptr, gl::STATIC_DRAW);

            gl::GenBuffers(1, &mut element_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer);
            let elements: [u8; 6] = [0, 1, 2, 0, 2, 3];
            let len = (mem::size_of::<f32>() * elements.len()) as isize;
            let ptr = elements.as_ptr() as *const _;
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, len, ptr, gl::STATIC_DRAW);
        }
    }

    let mut vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    }

    if !opengl21 {
        unsafe {
            enable_vertex_attribs(&[
                (program.position_attrib_location, 4),
                (program.texcoord_attrib_location, 4),
                (program.color_attrib_location, 4),
                (program.rotation_attrib_location, 3),
                (program.depth_attrib_location, 1),
            ]);
            gl::VertexAttribDivisor(program.position_attrib_location, 1);
            gl::VertexAttribDivisor(program.texcoord_attrib_location, 1);
            gl::VertexAttribDivisor(program.color_attrib_location, 1);
            gl::VertexAttribDivisor(program.rotation_attrib_location, 1);
            gl::VertexAttribDivisor(program.depth_attrib_location, 1);
        }
    }
    print_gl_errors("after attribute creation");

    Attributes {
        vao,
        vbo,
        vbo_static,
        element_buffer,
        vbo_data: Vec::new(),
        allocated_vbo_data_size: 0,
    }
}

// (location, component_count)
type AttribArray = (GLuint, GLint);
unsafe fn enable_vertex_attribs(attribs: &[AttribArray]) {
    let total_components = attribs.iter().map(|attrib| attrib.1 * 4).sum();
    let mut offset = 0;
    for attrib in attribs {
        gl::VertexAttribPointer(
            attrib.0,           /* Attrib location */
            attrib.1,           /* Components */
            gl::FLOAT,          /* Type */
            gl::FALSE,          /* Normalize */
            total_components,   /* Stride */
            offset as *const _, /* Offset */
        );
        gl::EnableVertexAttribArray(attrib.0);
        let component_size = attrib.1 * 4;
        offset += component_size;
    }

    print_gl_errors("after enabling vertex attributes");
}

unsafe fn disable_vertex_attribs(attrib_locations: &[GLuint]) {
    for location in attrib_locations {
        gl::DisableVertexAttribArray(*location);
    }

    print_gl_errors("after disabling vertex attributes");
}

#[inline]
fn create_texture(min_filter: GLint, mag_filter: GLint) -> GLuint {
    let mut tex = 0;
    unsafe {
        gl::GenTextures(1, &mut tex);
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter as GLint);
    }
    print_gl_errors("after creating a texture");
    tex
}

#[inline]
fn insert_texture(tex: GLuint, format: GLuint, w: GLint, h: GLint, pixels: &[u8]) {
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            format as i32,
            w,
            h,
            0,
            format,
            gl::UNSIGNED_BYTE,
            pixels.as_ptr() as *const _,
        );
    }
    print_gl_errors("after inserting a texture");
}

// TODO: Change this to print out to env_logger or such, not stderr
fn print_gl_errors(context: &str) {
    let mut error = unsafe { gl::GetError() };
    while error != gl::NO_ERROR {
        let error_msg = format!("GL error {}: {}", context, gl_error_to_string(error));
        debug_assert!(false, "{}", error_msg);
        eprintln!("{}", error_msg);
        error = unsafe { gl::GetError() };
    }
}

fn gl_error_to_string(error: GLuint) -> String {
    match error {
        0x0500 => "GL_INVALID_ENUM (0x0500)".to_owned(),
        0x0501 => "GL_INVALID_VALUE (0x0501)".to_owned(),
        0x0502 => "GL_INVALID_OPERATION (0x0502)".to_owned(),
        0x0503 => "GL_STACK_OVERFLOW (0x0503)".to_owned(),
        0x0504 => "GL_STACK_UNDERFLOW (0x0504)".to_owned(),
        0x0505 => "GL_OUT_OF_MEMORY (0x0505)".to_owned(),
        0x0506 => "GL_INVALID_FRAMEBUFFER_OPERATION (0x0506)".to_owned(),
        0x0507 => "GL_CONTEXT_LOST (0x0507)".to_owned(),
        0x0531 => "GL_TABLE_TOO_LARGE (0x0531)".to_owned(),
        _ => format!("unknown error ({:#06x})", error),
    }
}
