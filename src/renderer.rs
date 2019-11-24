// TODO: Audit all unsafes

use crate::gl;
use crate::gl::types::*;
use crate::gl_version::{self, OpenGlApi, OpenGlVersion};
use crate::image::Image;
use crate::sprite::Sprite;
use std::mem;
use std::ptr;

pub use crate::shaders::Shaders;

type TextureHandle = GLuint;
type VBOHandle = GLuint;
type VAOHandle = GLuint;

/// A handle to a draw call. Parameter for
/// [`Renderer::draw`](struct.Renderer.html#method.draw). Can be
/// cloned and shared, the clones will keep referring to the same draw
/// call.
///
/// Created with
/// [`Renderer::create_draw_call`](struct.Renderer.html#method.create_draw_call).
#[derive(Clone, Debug)]
pub struct DrawCallHandle(usize);

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
    texture_size: (i32, i32),
    program: ShaderProgram,
    attributes: Attributes,
    blend: bool,
    srgb: bool,
    highest_depth: f32,
}

#[derive(Clone, Debug)]
struct OpenGLState {
    legacy: bool,
    version: OpenGlVersion,
    // The fields below are settings set by other possible OpenGL
    // calls made in the surrounding program, because the point of
    // this crate is to behave well with other OpenGL code running
    // alongside it.
    pushed: bool,
    depth_test: bool,
    srgb: bool,
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
/// OpenGL.
#[derive(Debug)]
pub struct Renderer {
    calls: Vec<DrawCall>,
    gl_state: OpenGLState,
    pub(crate) dpi_factor: f32,
    /// Whether the Renderer should try to preserve the OpenGL
    /// state. If you're using OpenGL yourself, set this to `true` to
    /// avoid possible headaches. This is only best effort however,
    /// don't trust this to make the state clean.
    pub preserve_gl_state: bool,
}

/// Describes how textures are wrapped.
#[derive(Debug, Clone)]
pub enum TextureWrapping {
    /// Corresponds to `GL_CLAMP_TO_EDGE`.
    Clamp,
    /// Corresponds to `GL_REPEAT`.
    Repeat,
    /// Corresponds to `GL_MIRRORED_REPEAT`.
    RepeatMirrored,
}

/// Options which set capabilities, restrictions and resources for
/// draw calls.
///
/// Used in
/// [`Renderer::create_draw_call`](struct.Renderer.html#method.create_draw_call)
/// to create a [`DrawCallHandle`](struct.DrawCallHandle.html) which
/// can then be used to
/// [`Renderer::draw`](struct.Renderer.html#method.draw) with.
#[derive(Debug, Clone)]
pub struct DrawCallParameters {
    /// The texture used when drawing with this handle.
    pub image: Option<Image>,
    /// The shaders used when drawing with this handle.
    pub shaders: Shaders,
    /// Whether to blend with previously drawn pixels when drawing
    /// over them, or just replace the color. (In technical terms:
    /// whether `GL_BLEND` and back-to-front sorting are enabled.)
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
    /// Sets the texture's behavior when sampling coordinates under
    /// 0.0 or over 1.0, or smoothing over texture
    /// boundaries. (Corresponds to `GL_TEXTURE_WRAP_S` and
    /// `GL_TEXTURE_WRAP_T`, in that order.)
    pub wrap: (TextureWrapping, TextureWrapping),
    /// Controls whether or not `GL_FRAMEBUFFER_SRGB` is enabled when
    /// drawing with this handle. If you want to render in linear
    /// space, set this to false. You probably don't though, unless
    /// you know what you're doing.
    pub srgb: bool,
}

impl Default for DrawCallParameters {
    fn default() -> DrawCallParameters {
        DrawCallParameters {
            image: None,
            shaders: Default::default(),
            alpha_blending: true,
            minification_smoothing: true,
            magnification_smoothing: false,
            wrap: (TextureWrapping::Clamp, TextureWrapping::Clamp),
            srgb: true,
        }
    }
}

impl Renderer {
    /// Creates a new Renderer. **Requires** a valid OpenGL context.
    pub fn new(window: &crate::Window) -> Renderer {
        let version = gl_version::get_version();
        let legacy = match &version {
            OpenGlVersion::Available { api, major, minor } => {
                let legacy = match api {
                    OpenGlApi::Desktop => *major < 3 || (*major == 3 && *minor < 3),
                    OpenGlApi::ES => *major < 3,
                };
                log::info!(
                    "OpenGL version: {}.{}{}{}",
                    major,
                    minor,
                    "",
                    if legacy { " (legacy)" } else { "" },
                );
                legacy
            }
            OpenGlVersion::Unavailable { version_string } => {
                log::info!(
                    "Failed to parse OpenGL version string: '{}'",
                    version_string
                );
                true
            }
        };

        Renderer {
            calls: Vec::with_capacity(2),
            gl_state: OpenGLState {
                legacy,
                version,
                pushed: false,
                depth_test: false,
                srgb: false,
                depth_func: 0,
                blend: false,
                blend_func: (0, 0),
                program: 0,
                vao: 0,
                texture: 0,
                vbo: 0,
                element_buffer: 0,
            },
            dpi_factor: window.dpi_factor,
            preserve_gl_state: false,
        }
    }

    /// Returns whether or not running in legacy mode (OpenGL 3.3+
    /// optimizations off).
    pub fn is_legacy(&self) -> bool {
        self.gl_state.legacy
    }

    /// Returns the OpenGL version if it could be parsed.
    pub fn get_opengl_version(&self) -> &OpenGlVersion {
        &self.gl_state.version
    }

    /// Creates a new draw call in the pipeline, and returns a handle
    /// to use it with.
    ///
    /// Using the handle, you can call
    /// [`Renderer::draw`](struct.Renderer.html#method.draw) to draw
    /// sprites from your image. As a rule of thumb, try to minimize
    /// the amount of draw calls.
    ///
    /// If you want to use your own GLSL shaders, you can provide them
    /// with the `shaders` parameter. The `Shaders` struct implements
    /// `Default`, so you can replace only the shaders you want to
    /// replace, which usually means just the fragment shaders. Make
    /// sure to study the uniform variables and attributes of the
    /// default shaders before making your own.
    pub fn create_draw_call(&mut self, params: DrawCallParameters) -> DrawCallHandle {
        self.gl_push();

        let (api, legacy) = (
            match self.gl_state.version {
                OpenGlVersion::Available { api, .. } => api,
                _ => OpenGlApi::Desktop,
            },
            self.gl_state.legacy,
        );
        let vert = params.shaders.create_vert_string(api, legacy);
        let frag = params.shaders.create_frag_string(api, legacy);
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
            texture_size: (0, 0),
            program,
            attributes,
            blend: params.alpha_blending,
            srgb: params.srgb,
            highest_depth: -1.0,
        });

        if let Some(image) = params.image {
            insert_texture(
                self.calls[index].texture,
                image.format,
                image.pixel_type,
                image.width,
                image.height,
                if image.null_data {
                    None
                } else {
                    Some(&image.pixels)
                },
            );
            self.calls[index].texture_size = (image.width, image.height);
        }

        self.gl_pop();
        DrawCallHandle(index)
    }

    /// Creates a Sprite struct, which you can render after specifying
    /// your parameters by modifying it.
    ///
    /// Higher Z sprites are drawn over the lower ones (with the
    /// exception of the case described below).
    ///
    /// ## Weird Z-coordinate behavior note
    ///
    /// Try to constrain your z-coordinates to small ranges within
    /// individual draw calls; draw call rendering order is decided by
    /// the highest z-coordinate that each draw call has to draw. This
    /// can even cause visual glitches in alpha-blended draw calls, if
    /// their sprites overlap and have overlapping ranges of
    /// z-coordinates. For an example of this, see the `drawing_order`
    /// example.
    ///
    /// ## Optimization tips
    /// - Draw the sprites in front first. This way you'll avoid
    ///   rendering over already drawn pixels. If you're rendering
    ///   *lots* of sprites, this is a good place to start optimizing.
    /// - If possible, make your textures without using alpha values
    ///   between 1 and 0 (ie. use only 100% and 0% opacity), and
    ///   disable `alpha_blending` in your draw call. These kinds of
    ///   sprites can be drawn much more efficiently when it comes to
    ///   overdraw.
    ///
    /// # Usage
    /// ```ignore
    /// renderer.draw(&call, 0.0)
    ///     .with_coordinates((100.0, 100.0, 16.0, 16.0))
    ///     .with_texture_coordinates((0, 0, 16, 16))
    ///     .finish();
    /// ```
    #[inline]
    pub fn draw<'a, 'b>(&'a mut self, call: &'b DrawCallHandle, z: f32) -> Sprite<'a, 'b> {
        Sprite::create(self, call, z)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_quad_clipped(
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

    #[inline]
    pub(crate) fn draw_quad(
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

        self.calls[call_handle.0].highest_depth =
            self.calls[call_handle.0].highest_depth.max(depth);
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

    /// Updates the DPI multiplication factor of the screen.
    pub fn set_dpi_factor(&mut self, dpi_factor: f32) {
        self.dpi_factor = dpi_factor;
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
        self.gl_push();

        let m00 = 2.0 / width;
        let m11 = -2.0 / height;
        let matrix = [
            m00, 0.0, 0.0, -1.0, 0.0, m11, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];

        unsafe {
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let legacy = self.gl_state.legacy;

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        let mut call_indices: Vec<usize> = (0..self.calls.len()).collect();
        call_indices.sort_unstable_by(|a, b| {
            let call_a = &self.calls[*a];
            let call_b = &self.calls[*b];
            let a = call_a.highest_depth;
            let a = if call_a.blend { a } else { -2.0 - a };
            let b = call_b.highest_depth;
            let b = if call_b.blend { b } else { -2.0 - b };
            a.partial_cmp(&b).unwrap()
        });

        for i in call_indices {
            let call = &mut self.calls[i];

            if call.attributes.vbo_data.is_empty() {
                continue;
            }

            unsafe {
                if call.blend {
                    gl::Enable(gl::BLEND);
                    gl::DepthFunc(gl::LEQUAL);
                } else {
                    gl::Disable(gl::BLEND);
                    gl::DepthFunc(gl::LESS);
                }
                if call.srgb {
                    gl::Enable(gl::FRAMEBUFFER_SRGB);
                } else {
                    gl::Disable(gl::FRAMEBUFFER_SRGB);
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
                crate::profiler::write(|p| p.quads_drawn += vertex_count as u32 / 6);
                print_gl_errors(&format!("[legacy] after drawing buffer #{}", i));
            } else {
                // 16 floats (4 for x,y,w,h + 4 tex xywh + 4 col + 3 rot + 1 z) per vertex
                let count = call.attributes.vbo_data.len() as i32 / 16;
                let mode = gl::TRIANGLES;
                let val_type = gl::UNSIGNED_BYTE;
                unsafe {
                    gl::DrawElementsInstanced(mode, 6, val_type, ptr::null(), count);
                }
                crate::profiler::write(|p| p.quads_drawn += count as u32);
                print_gl_errors(&format!("after drawing buffer #{}", i));
            }

            call.attributes.vbo_data.clear();
            call.highest_depth = -1.0;

            print_gl_errors(&*format!("after render #{}", i));
        }

        self.gl_pop();
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
    }

    pub(crate) fn get_texture_size(&self, call_handle: &DrawCallHandle) -> (i32, i32) {
        self.calls[call_handle.0].texture_size
    }

    pub(crate) fn upload_texture_region(
        &self,
        call_handle: &DrawCallHandle,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: GLuint,
        data: Vec<u8>,
    ) {
        insert_sub_texture(
            self.calls[call_handle.0].texture,
            x,
            y,
            width,
            height,
            format,
            data,
        );
    }

    pub(crate) fn resize_texture(
        &mut self,
        call_handle: &DrawCallHandle,
        new_width: i32,
        new_height: i32,
    ) {
        resize_texture(
            self.calls[call_handle.0].texture,
            self.calls[call_handle.0].texture_size.0,
            self.calls[call_handle.0].texture_size.1,
            new_width,
            new_height,
        );
        self.calls[call_handle.0].texture_size = (new_width, new_height);
    }

    /// Saves the current OpenGL state for [`Renderer::gl_pop`] and
    /// then sets some defaults used by this crate.
    fn gl_push(&mut self) {
        if !self.gl_state.pushed {
            unsafe {
                self.gl_state.depth_test = gl::IsEnabled(gl::DEPTH_TEST) != 0;
                self.gl_state.blend = gl::IsEnabled(gl::BLEND) != 0;
                self.gl_state.srgb = gl::IsEnabled(gl::FRAMEBUFFER_SRGB) != 0;
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
    }

    /// Restores the OpenGL state saved in [`Renderer::gl_push`].
    fn gl_pop(&mut self) {
        if !self.preserve_gl_state {
            return;
        }

        if self.gl_state.pushed {
            unsafe {
                if !self.gl_state.depth_test {
                    gl::Disable(gl::DEPTH_TEST);
                } else {
                    gl::Enable(gl::DEPTH_TEST);
                }
                if !self.gl_state.blend {
                    gl::Disable(gl::BLEND);
                } else {
                    gl::Enable(gl::BLEND);
                }
                if !self.gl_state.srgb {
                    gl::Disable(gl::FRAMEBUFFER_SRGB);
                } else {
                    gl::Enable(gl::FRAMEBUFFER_SRGB);
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

#[inline]
fn create_program(vert_source: &str, frag_source: &str, legacy: bool) -> ShaderProgram {
    let print_shader_error = |shader, shader_type| unsafe {
        let mut compilation_status = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compilation_status);
        if compilation_status as u8 != gl::TRUE {
            let mut info = vec![0; 1024];
            gl::GetShaderInfoLog(shader, 1024, ptr::null_mut(), info.as_mut_ptr());
            log::error!(
                "Shader ({}) compilation failed:\n{}",
                shader_type,
                error_buffer_into_string(info).trim()
            );
            if cfg!(debug_assertions) {
                panic!("shader compilation failed");
            }
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
            let mut info = vec![0; 1024];
            gl::GetProgramInfoLog(program, 1024, ptr::null_mut(), info.as_mut_ptr());
            log::error!(
                "Program linking failed:\n{}",
                error_buffer_into_string(info).trim()
            );
            if cfg!(debug_assertions) {
                panic!("program linking failed");
            }
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
fn insert_texture(
    tex: GLuint,
    format: GLuint,
    pixel_type: GLuint,
    w: GLint,
    h: GLint,
    pixels: Option<&[u8]>,
) {
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            format as GLint,
            w,
            h,
            0,
            match format {
                gl::SRGB => gl::RGB,
                gl::SRGB_ALPHA => gl::RGBA,
                format => format,
            },
            pixel_type,
            if let Some(pixels) = pixels {
                pixels.as_ptr() as *const _
            } else {
                ptr::null()
            },
        );
    }
    print_gl_errors("after inserting a texture");
}

fn insert_sub_texture(
    texture: GLuint,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    format: GLuint,
    data: Vec<u8>,
) {
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::TexSubImage2D(
            gl::TEXTURE_2D,            // target
            0,                         // level
            x,                         // xoffset
            y,                         // yoffset
            width,                     // width
            height,                    // height
            format,                    // format
            gl::UNSIGNED_BYTE,         // type
            data.as_ptr() as *const _, // pixels
        );
    }
    crate::renderer::print_gl_errors("after insert_sub_texture");
}

fn resize_texture(
    texture: GLuint,
    old_width: i32,
    old_height: i32,
    new_width: i32,
    new_height: i32,
) {
    let mut fbo = 0;
    let mut temp_tex = 0;
    unsafe {
        // Create framebuffer and attach the original texture onto it
        gl::GenFramebuffers(1, &mut fbo);
        gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, texture, 0);
        // Create temp texture and copy over the texture from the framebuffer
        gl::GenTextures(1, &mut temp_tex);
        gl::BindTexture(gl::TEXTURE_2D, temp_tex);
        gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        gl::CopyTexImage2D(gl::TEXTURE_2D, 0, gl::RED, 0, 0, old_width, old_height, 0);
        // Re-create the original texture
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RED as GLint,
            new_width,
            new_height,
            0,
            gl::RED,
            gl::UNSIGNED_BYTE,
            std::ptr::null(),
        );
        // Attach the temp texture to the framebuffer and draw it onto the original texture
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, temp_tex, 0);
        gl::CopyTexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, 0, 0, old_width, old_height);
        // Cleanup
        gl::DeleteTextures(1, &mut temp_tex);
        gl::DeleteFramebuffers(1, &mut fbo);
    }
    crate::renderer::print_gl_errors("after resize_texture");
}

#[cfg(not(debug_assertions))]
pub(crate) fn print_gl_errors(_context: &str) {}

#[cfg(debug_assertions)]
pub(crate) fn print_gl_errors(context: &str) {
    let error = unsafe { gl::GetError() };
    if error != gl::NO_ERROR {
        panic!("GL error {}: {}", context, gl_error_to_string(error));
    }
}

#[cfg(debug_assertions)]
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

fn error_buffer_into_string(raw: Vec<i8>) -> String {
    let len = raw
        .iter()
        .enumerate() // Enumerate: values are (index, &value)
        .find(|&(_, &v)| v == 0) // Find the nul byte
        .map(|(i, _)| i) // Get the index to use as the length
        .unwrap_or(1024); // If no nul byte was found, assume 1024 for length
    let info: Vec<u8> = raw[0..len]
        .iter()
        .map(|&i| unsafe { mem::transmute::<i8, u8>(i) })
        .collect();
    String::from_utf8_lossy(&info).to_string()
}
