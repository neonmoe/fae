//! The rendering module.
//!
//! This module is responsible for the actual drawing procedures,
//! written in OpenGL. Usage in general terms:
//! - Create a window and load the OpenGL functions with
//!   `gl::load_with`.
//! - Create draw calls for each texture you want to use.
//! - Draw with [`Renderer::draw_quad`](struct.Renderer.html#method.draw_quad)
//!   and similar functions using the draw call index to use specific
//!   textures.
//!
//! ## Optimization tips
//! - Try to define different Z coordinates for your elements, and
//!   draw the ones in front first. This way you'll avoid rendering
//!   over already drawn pixels. If you're rendering *lots* of sprites,
//!   this is a good place to start optimizing.
//! - If possible, make your textures without using alpha values
//!   between 1 and 0 (ie. use only 100% and 0% opacity), and disable
//!   `alpha_blending` in your draw call.

use crate::gl;
use crate::gl::types::*;
use crate::image::Image;
use std::mem;
use std::ptr;

type TextureHandle = GLuint;
type VBOHandle = GLuint;
type VAOHandle = GLuint;

/// A handle with which you can draw during a specific draw
/// call. Created during [`Renderer::create_draw_call`], used during
/// [`Renderer::draw_quad`] and its variations.
pub struct DrawCallHandle(usize);

/// Represents the shader code for a shader. Used in
/// [`Renderer::create_draw_call`].
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
    blend: bool,
    lowest_depth: f32,
}

#[derive(Clone, Copy, Debug)]
struct OpenGLState {
    legacy: bool,
    version: Option<(u8, u8)>,
    // The fields below are settings set by other possible OpenGL
    // calls made in the surrounding program, because the point of
    // this crate is to behave well with other OpenGL code running
    // alongside it.
    pushed: bool,
    depth_test: bool,
    depth_func: GLint,
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
    profiler: Profiler,
    /// Whether the Renderer should try to preserve the OpenGL
    /// state. If you're using OpenGL yourself, set this to `true` to
    /// avoid possible headaches.
    pub preserve_gl_state: bool,
}

/// Describes how textures are wrapped.
#[derive(Debug, Clone)]
pub enum TextureWrapping {
    /// Corresponds to GL_CLAMP_TO_EDGE.
    Clamp,
    /// Corresponds to GL_REPEAT.
    Repeat,
    /// Corresponds to GL_MIRRORED_REPEAT.
    RepeatMirrored,
}

/// Options which set capabilities, restrictions and resources for
/// draw calls. Used in [`Renderer::create_draw_call`].
#[derive(Debug, Clone)]
pub struct DrawCallParameters {
    /// The texture used when drawing with this draw call.
    pub image: Option<Image>,
    /// The shaders used when drawing with this draw call.
    pub shaders: Option<Shaders>,
    /// Whether to blend with previously drawn pixels when drawing
    /// over them, or just replace the color. (In OpenGL terms:
    /// whether GL_BLEND is enabled.)
    pub alpha_blending: bool,
    /// When drawing quads that are smaller than the texture provided,
    /// use linear (true) or nearest neighbor (false) smoothing when
    /// scaling? (Linear is probably always better.)
    pub minification_smoothing: bool,
    /// When drawing quads that are larger than the texture provided,
    /// use linear (true) or nearest neighbor (false) smoothing when
    /// scaling? (Tip: for pixel art or other textures that don't
    /// suffer from jaggies, set this to false for the intended look.)
    pub magnification_smoothing: bool,
    /// Sets the texture's behavior when sampling under 0.0 or over
    /// 1.0, or smoothing over texture boundaries. (Corresponds to
    /// GL_TEXTURE_WRAP_S and GL_TEXTURE_WRAP_T, in that order.)
    pub wrap: (TextureWrapping, TextureWrapping),
}

impl Default for DrawCallParameters {
    fn default() -> DrawCallParameters {
        DrawCallParameters {
            image: None,
            shaders: None,
            alpha_blending: true,
            minification_smoothing: true,
            magnification_smoothing: false,
            wrap: (TextureWrapping::Clamp, TextureWrapping::Clamp),
        }
    }
}

impl Renderer {
    /// Creates a new Renderer.
    ///
    /// Takes a Window as a parameter to ensure that a valid OpenGL
    /// context exists.
    pub fn new(_: &crate::Window) -> Renderer {
        let version = get_version();
        let legacy = if let Some((major, minor)) = &version {
            *major < 3 || (*major == 3 && *minor < 3)
        } else {
            true // Fallback to legacy if parsing fails
        };

        Renderer {
            calls: Vec::with_capacity(2),
            gl_state: OpenGLState {
                legacy,
                version,
                pushed: false,
                depth_test: false,
                depth_func: 0,
                blend: false,
                blend_func: (0, 0),
                program: 0,
                vao: 0,
                texture: 0,
                vbo: 0,
                element_buffer: 0,
            },
            profiler: Profiler::new(),
            preserve_gl_state: false,
        }
    }

    /// Toggles whether profiling is enabled.
    ///
    /// If the `profiler` feature is enabled, the renderer will send
    /// performance data to the profiler.
    pub fn set_profiling(&mut self, should_profile: bool) {
        self.profiler.toggle(should_profile);
    }

    /// Returns whether or not running in legacy mode (OpenGL 3.3+
    /// optimizations off).
    pub fn is_legacy(&self) -> bool {
        self.gl_state.legacy
    }

    /// Returns the OpenGL version (major, minor) if it could be
    /// parsed.
    pub fn get_opengl_version(&self) -> Option<(u8, u8)> {
        self.gl_state.version
    }

    /// Creates a new draw call in the pipeline, and returns its
    /// index.
    ///
    /// Using the index, you can call [`Renderer::draw_quad`] to draw
    /// sprites from your image. As a rule of thumb, try to minimize
    /// the amount of draw calls.
    ///
    /// If you want to use your own GLSL shaders, you can provide them
    /// with the `shaders` parameter. Use `None` for defaults. Make
    /// sure to study the uniform variables and attributes of the
    /// default shaders before making your own.
    pub fn create_draw_call(&mut self, params: DrawCallParameters) -> DrawCallHandle {
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
        let wrap = |wrap_type| match wrap_type {
            TextureWrapping::Clamp => gl::CLAMP_TO_EDGE,
            TextureWrapping::Repeat => gl::REPEAT,
            TextureWrapping::RepeatMirrored => gl::MIRRORED_REPEAT,
        };
        let texture = create_texture(
            filter(params.minification_smoothing),
            filter(params.magnification_smoothing),
            wrap(params.wrap.0) as i32,
            wrap(params.wrap.1) as i32,
        );
        self.calls.push(DrawCall {
            texture,
            program,
            attributes,
            blend: params.alpha_blending,
            lowest_depth: 1.0,
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
        DrawCallHandle(index)
    }

    #[allow(dead_code)]
    pub(crate) fn create_dummy_draw_call(&mut self) -> DrawCallHandle {
        let index = self.calls.len();
        self.calls.push(DrawCall {
            texture: 0,
            program: ShaderProgram {
                program: 0,
                vertex_shader: 0,
                fragment_shader: 0,
                projection_matrix_location: 0,
                position_attrib_location: 0,
                texcoord_attrib_location: 0,
                color_attrib_location: 0,
                rotation_attrib_location: 0,
                depth_attrib_location: 0,
                shared_position_attrib_location: 0,
                shared_texcoord_attrib_location: 0,
            },
            attributes: Attributes {
                vbo: 0,
                vbo_static: 0,
                element_buffer: 0,
                vao: 0,
                vbo_data: Vec::new(),
                allocated_vbo_data_size: 0,
            },
            blend: false,
            lowest_depth: 1.0,
        });
        DrawCallHandle(index)
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
        clip_area: (f32, f32, f32, f32),
        coords: (f32, f32, f32, f32),
        texcoords: (f32, f32, f32, f32),
        color: (f32, f32, f32, f32),
        rotation: (f32, f32, f32),
        z: f32,
        call_handle: &DrawCallHandle,
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

        self.draw_quad((x0, y0, x1, y1), texcoords, color, rotation, z, call_handle);
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
        call_handle: &DrawCallHandle,
    ) {
        self.draw_quad(
            coords,
            (-1.0, -1.0, -1.0, -1.0),
            color,
            rotation,
            z,
            call_handle,
        );
    }

    /// Draws a textured rectangle on the screen.
    ///
    /// - `coords`: The coordinates of the corners of the quad, in
    /// (logical) pixels. Arrangement: (left, top, right, bottom)
    ///
    /// - `texcoords`: The texture coordinates (UVs) of the quad, in
    /// the range `0.0 - 1.0`. The shader will not use the texture at
    /// all if the texcoords are all `-1.0`. Same arrangement as
    /// `coords`.
    ///
    /// - `color`: The color tint of the quad, in the range
    /// `0-255`. Arrangement: (red, green, blue, alpha)
    ///
    /// - `rotation`: The rotation of the quad, in radians, and the
    /// point (relative to `coords` x and y, in logical pixels as well)
    /// around which the sprite pivots. Arrangement: (radians, x, y)
    ///
    /// - `depth`: Used for ordering sprites on screen, in the range
    /// `-1.0 - 1.0`. Negative values are in front.
    ///
    /// - `call_handle`: The index of the draw call to draw the quad
    /// in. This is the returned value from [`Renderer::create_draw_call`].
    #[inline]
    pub fn draw_quad(
        &mut self,
        coords: (f32, f32, f32, f32),
        texcoords: (f32, f32, f32, f32),
        color: (f32, f32, f32, f32),
        rotation: (f32, f32, f32),
        depth: f32,
        call_handle: &DrawCallHandle,
    ) {
        let (x0, y0, x1, y1) = coords;
        let (tx0, ty0, tx1, ty1) = texcoords;
        let (red, green, blue, alpha) = color;
        let (rads, pivot_x, pivot_y) = rotation;

        self.calls[call_handle.0].lowest_depth = self.calls[call_handle.0].lowest_depth.min(depth);
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

            self.calls[call_handle.0]
                .attributes
                .vbo_data
                .extend_from_slice(&quad);
        } else {
            let (width, height, tw, th) = (x1 - x0, y1 - y0, tx1 - tx0, ty1 - ty0);
            let quad = [
                x0, y0, width, height, tx0, ty0, tw, th, red, green, blue, alpha, rads, pivot_x,
                pivot_y, depth,
            ];
            self.calls[call_handle.0]
                .attributes
                .vbo_data
                .extend_from_slice(&quad);
        }
    }

    /// Clears all queued draws. Like a dummy-version of [`Renderer::render`].
    pub fn flush(&mut self) {
        for call in self.calls.iter_mut() {
            call.attributes.vbo_data.clear();
        }
    }

    // TODO: Add a re-render function for fast re-rendering in resize events
    // Rationale:
    // - Clears need to be made when resizing, otherwise there will be flickering when expanding the window
    // - render() might be clearer if split into two anyway: uploading the buffers, and drawing them
    // - Possibility for optimization: the uploading action could only be ran when needed

    /// Renders all currently queued draws.
    ///
    /// First the non-alpha-blended calls, front to back, then the
    /// alpha-blended ones, back to front. Drawing from front to back
    /// is more efficient, as there is less overdraw because of depth
    /// testing, but proper blending requires back to front ordering.
    pub fn render(&mut self, width: f32, height: f32) {
        self.profiler.start("render");
        self.gl_push();

        let m00 = 2.0 / width;
        let m11 = -2.0 / height;
        let matrix = [
            m00, 0.0, 0.0, -1.0, 0.0, m11, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];

        let profiler = &self.profiler;
        profiler.start("clear");
        unsafe {
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        profiler.end("clear");

        let legacy = self.gl_state.legacy;

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        let mut call_indices: Vec<usize> = (0..self.calls.len()).collect();
        call_indices.sort_unstable_by(|a, b| {
            let call_a = &self.calls[*a];
            let call_b = &self.calls[*b];
            let a = call_a.lowest_depth;
            let a = if call_a.blend { a } else { 2.0 - a };
            let b = call_b.lowest_depth;
            let b = if call_b.blend { b } else { 2.0 - b };
            b.partial_cmp(&a).unwrap()
        });

        for i in call_indices {
            let call = &mut self.calls[i];

            profiler.start(format!("call {}", i));
            if call.attributes.vbo_data.is_empty() {
                profiler.end(format!("call {}", i));
                continue;
            }

            profiler.start("setting state");
            unsafe {
                if call.blend {
                    gl::Enable(gl::BLEND);
                    gl::DepthFunc(gl::LEQUAL);
                } else {
                    gl::Disable(gl::BLEND);
                    gl::DepthFunc(gl::LESS);
                }
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
            profiler.end("setting state");
            print_gl_errors(&format!("after initializing draw call #{}", i));

            let len = (mem::size_of::<f32>() * call.attributes.vbo_data.len()) as isize;
            let ptr = call.attributes.vbo_data.as_ptr() as *const _;
            if len <= call.attributes.allocated_vbo_data_size {
                unsafe {
                    profiler.start("bufferSubData");
                    gl::BufferSubData(gl::ARRAY_BUFFER, 0, len, ptr);
                    profiler.end("bufferSubData");
                }
            } else {
                call.attributes.allocated_vbo_data_size = len;
                unsafe {
                    profiler.start("bufferData");
                    gl::BufferData(gl::ARRAY_BUFFER, len, ptr, gl::STREAM_DRAW);
                    profiler.end("bufferData");
                }
            }
            print_gl_errors(&format!("after pushing vertex buffer #{}", i));

            if legacy {
                // 12 floats (3 for pos + 2 tex + 4 col + 3 rot) per vertex
                let vertex_count = call.attributes.vbo_data.len() as i32 / 12;
                unsafe {
                    profiler.start("enable vertex attribs");
                    enable_vertex_attribs(&[
                        (call.program.position_attrib_location, 3),
                        (call.program.texcoord_attrib_location, 2),
                        (call.program.color_attrib_location, 4),
                        (call.program.rotation_attrib_location, 3),
                    ]);
                    profiler.end("enable vertex attribs");
                    profiler.start("drawArrays");
                    gl::DrawArrays(gl::TRIANGLES, 0, vertex_count);
                    profiler.end("drawArrays");
                    profiler.start("disable vertex attribs");
                    disable_vertex_attribs(&[
                        call.program.position_attrib_location,
                        call.program.texcoord_attrib_location,
                        call.program.color_attrib_location,
                        call.program.rotation_attrib_location,
                    ]);
                    profiler.end("disable vertex attribs");
                }
                print_gl_errors(&format!("[legacy] after drawing buffer #{}", i));
            } else {
                // 16 floats (4 for x,y,w,h + 4 tex xywh + 4 col + 3 rot + 1 z) per vertex
                let count = call.attributes.vbo_data.len() as i32 / 16;
                let mode = gl::TRIANGLES;
                let val_type = gl::UNSIGNED_BYTE;
                profiler.start("drawElementsInstanced");
                unsafe {
                    gl::DrawElementsInstanced(mode, 6, val_type, ptr::null(), count);
                }
                profiler.end("drawElementsInstanced");
                print_gl_errors(&format!("after drawing buffer #{}", i));
            }

            call.attributes.vbo_data.clear();
            call.lowest_depth = 1.0;

            print_gl_errors(&*format!("after render #{}", i));
            profiler.end(format!("call {}", i));
        }

        self.gl_pop();
        self.profiler.end("render");
    }

    /// Synchronizes the GPU and CPU state, ensuring that all OpenGL
    /// calls made so far have been executed. One use case would be
    /// after swapping buffers, to sleep until the buffers really have
    /// been swapped.
    ///
    /// If running with modern OpenGL, this is implemented with
    /// glClientWaitSync calls, 2ms thread::sleeps in between.
    ///
    /// If running with legacy (2.1 or 2.0 ES) OpenGL, this is
    /// equivalent to glFinish.
    pub fn synchronize(&self) {
        use std::thread::sleep;
        use std::time::Duration;

        self.profiler.start("synchronization");
        let mut synchronized = false;

        if !self.gl_state.legacy {
            let fence = unsafe { gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0) };
            while {
                let status = unsafe { gl::ClientWaitSync(fence, gl::SYNC_FLUSH_COMMANDS_BIT, 0) };
                match status {
                    gl::ALREADY_SIGNALED | gl::CONDITION_SATISFIED => {
                        synchronized = true;
                        false
                    }
                    gl::WAIT_FAILED => {
                        print_gl_errors("glClientWaitSync");
                        false
                    }
                    _ => true,
                }
            } {
                sleep(Duration::from_micros(2000));
            }

            unsafe {
                gl::DeleteSync(fence);
            }
        }

        if !synchronized {
            unsafe {
                gl::Finish();
            }
        }
        self.profiler.end("synchronization");
    }

    /// Returns the OpenGL texture handle for the texture used by the
    /// draw call.
    #[cfg(all(feature = "text", feature = "font8x8" /* or font-kit, in the future */))]
    pub(crate) fn get_texture(&self, call_handle: &DrawCallHandle) -> GLuint {
        self.calls[call_handle.0].texture
    }

    /// Saves the current OpenGL state for [`Renderer::gl_pop`] and
    /// then sets some defaults used by this crate.
    fn gl_push(&mut self) {
        self.profiler.start("gl state push");
        if !self.gl_state.pushed {
            unsafe {
                self.gl_state.depth_test = gl::IsEnabled(gl::DEPTH_TEST) != 0;
                self.gl_state.blend = gl::IsEnabled(gl::BLEND) != 0;
                gl::GetIntegerv(gl::DEPTH_FUNC, &mut self.gl_state.depth_func);
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

            self.gl_state.pushed = true;
            print_gl_errors("after glEnables");
        }
        self.profiler.end("gl state push");
    }

    /// Restores the OpenGL state saved in [`Renderer::gl_push`].
    fn gl_pop(&mut self) {
        if !self.preserve_gl_state {
            return;
        }

        self.profiler.start("gl state pop");
        if self.gl_state.pushed {
            unsafe {
                if !self.gl_state.depth_test {
                    gl::Disable(gl::DEPTH_TEST);
                }
                if !self.gl_state.blend {
                    gl::Disable(gl::BLEND);
                }
                gl::DepthFunc(self.gl_state.depth_func as GLuint);
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
        self.profiler.end("gl state pop");
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        if !gl::Viewport::is_loaded() {
            // Running without a valid gl context, no need to clean up
            // gl resources (because they can't have been allocated)
            return;
        }
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
            if cfg!(debug_assertions) {
                panic!("{}", error_msg);
            }
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
            if cfg!(debug_assertions) {
                panic!("{}", error_msg);
            }
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
fn create_attributes(legacy: bool, program: ShaderProgram) -> Attributes {
    let mut vao = 0;
    if !legacy {
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        }
    }

    let mut vbo_static = 0;
    let mut element_buffer = 0;
    if !legacy {
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

    if !legacy {
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
fn create_texture(min_filter: GLint, mag_filter: GLint, wrap_s: GLint, wrap_t: GLint) -> GLuint {
    let mut tex = 0;
    unsafe {
        gl::GenTextures(1, &mut tex);
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_s);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_t);
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
pub(crate) fn print_gl_errors(context: &str) {
    let mut error = unsafe { gl::GetError() };
    while error != gl::NO_ERROR {
        let error_msg = format!("GL error {}: {}", context, gl_error_to_string(error));
        if cfg!(debug_assertions) {
            panic!("{}", error_msg);
        }
        eprintln!("{}", error_msg);
        error = unsafe { gl::GetError() };
    }
}

// Sorry for the mess, but OpenGL version strings are unreliable, and
// I'm not sure *how* unreliable. Here's my attempt at a robust way of
// parsing the version.
fn get_version() -> Option<(u8, u8)> {
    let version_str = unsafe { std::ffi::CStr::from_ptr(gl::GetString(gl::VERSION) as *const _) };
    let version_str = version_str.to_string_lossy();

    let mut split = version_str.split('.'); // Split at .
    let major_str = &split.next()?; // Major version is the first part before the first .
    let major = u8::from_str_radix(major_str, 10).ok()?; // Parse the version

    let rest_of_version = split.next()?; // Find the next part after the first .
    let end_of_version_num = rest_of_version
        .find(|c: char| !c.is_digit(10))
        .unwrap_or(rest_of_version.len()); // Find where the minor version ends
    let minor_str = &rest_of_version[0..end_of_version_num]; // Minor version as str
    let minor = u8::from_str_radix(minor_str, 10).ok()?; // Parse minor version

    Some((major, minor))
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

use renderer_profiler::Profiler;

#[cfg(feature = "profiler")]
mod renderer_profiler {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::time::Instant;

    #[derive(Clone, Debug)]
    pub struct Profiler {
        should_profile: bool,
        starts: RefCell<HashMap<String, Instant>>,
    }

    impl Profiler {
        pub fn new() -> Profiler {
            Profiler {
                should_profile: false,
                starts: RefCell::new(HashMap::new()),
            }
        }

        pub fn toggle(&mut self, should_profile: bool) {
            self.should_profile = should_profile;
        }

        pub fn start<S: Into<String>>(&self, name: S) {
            if self.should_profile {
                let mut starts = self.starts.borrow_mut();
                starts.insert(name.into(), Instant::now());
            }
        }

        pub fn end<S: Into<String>>(&self, name: S) {
            if self.should_profile {
                let end_time = Instant::now();
                let mut starts = self.starts.borrow_mut();
                let name = name.into();
                if let Some(start_time) = starts.remove(&name) {
                    crate::profiler::insert_profiling_data(
                        name.to_string(),
                        format!("{:?}", end_time - start_time),
                    );
                }
            }
        }
    }
}

#[cfg(not(feature = "profiler"))]
mod renderer_profiler {
    #[derive(Clone, Debug)]
    pub struct Profiler {}

    impl Profiler {
        pub fn new() -> Profiler {
            Profiler {}
        }

        pub fn toggle(&mut self, _should_profile: bool) {}
        pub fn start<S: Into<String>>(&self, _name: S) {}
        pub fn end<S: Into<String>>(&self, _name: S) {}
    }
}
