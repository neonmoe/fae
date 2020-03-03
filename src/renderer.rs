use crate::gl;
use crate::gl::types::*;
use crate::gl_version::{self, OpenGlApi, OpenGlVersion};
use crate::image::Image;
use crate::sprite::Sprite;
use crate::types::RectPx;

use std::mem;
use std::ptr;

pub use crate::shaders::Shaders;

#[derive(Clone, Debug)]
#[repr(transparent)]
struct TextureHandle(GLuint);
#[derive(Clone, Debug)]
#[repr(transparent)]
struct VboHandle(GLuint);
#[derive(Clone, Debug)]
#[repr(transparent)]
struct VaoHandle(GLuint);

#[derive(Clone, Debug)]
pub(crate) struct DrawCallHandle {
    index: usize,
}

#[derive(Clone, Debug)]
struct ShaderProgram {
    program: GLuint,
    vertex_shader: GLuint,
    fragment_shader: GLuint,

    projection_matrix_location: Option<GLint>,
    gamma_correction_location: Option<GLint>,
    position_attrib_location: Option<GLuint>,
    texcoord_attrib_location: Option<GLuint>,
    color_attrib_location: Option<GLuint>,
    rotation_attrib_location: Option<GLuint>,
    depth_attrib_location: Option<GLuint>,
    shared_position_attrib_location: Option<GLuint>,
    shared_texcoord_attrib_location: Option<GLuint>,
}

#[derive(Clone, Debug)]
struct Attributes {
    vbo: VboHandle,
    vbo_static: VboHandle,
    element_buffer: VboHandle,
    vao: VaoHandle,
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

/// Describes how textures are wrapped.
#[derive(Debug, Clone, Copy)]
pub enum TextureWrapping {
    /// Corresponds to `GL_CLAMP_TO_EDGE`.
    Clamp,
    /// Corresponds to `GL_REPEAT`.
    Repeat,
    /// Corresponds to `GL_MIRRORED_REPEAT`.
    RepeatMirrored,
}

/// Contains the data and functionality needed to draw rectangles with
/// OpenGL.
#[derive(Debug)]
pub(crate) struct Renderer {
    calls: Vec<DrawCall>,
    pub(crate) legacy: bool,
    pub(crate) version: OpenGlVersion,
    pub(crate) dpi_factor: f32,
}

impl Renderer {
    // TODO(0.6.0): Add a new renderer constructor that fails on legacy contexts.
    pub(crate) fn new() -> Renderer {
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
            calls: Vec::new(),
            legacy,
            version,
            dpi_factor: 1.0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_draw_call(
        &mut self,
        image: Option<&Image>,
        shaders: &Shaders,
        alpha_blending: bool,
        minification_smoothing: bool,
        magnification_smoothing: bool,
        wrap: (TextureWrapping, TextureWrapping),
        srgb: bool,
    ) -> DrawCallHandle {
        let (api, legacy) = (
            match self.version {
                OpenGlVersion::Available { api, .. } => api,
                _ => OpenGlApi::Desktop,
            },
            self.legacy,
        );
        let vert = shaders.create_vert_string(api, legacy);
        let frag = shaders.create_frag_string(api, legacy);
        let index = self.calls.len();

        let program = create_program(&vert, &frag);
        let attributes = create_attributes(legacy, &program);
        let filter = |smoothed| if smoothed { gl::LINEAR } else { gl::NEAREST } as i32;
        let get_wrap = |wrap_type| match wrap_type {
            TextureWrapping::Clamp => gl::CLAMP_TO_EDGE,
            TextureWrapping::Repeat => gl::REPEAT,
            TextureWrapping::RepeatMirrored => gl::MIRRORED_REPEAT,
        };
        let texture = create_texture(
            filter(minification_smoothing),
            filter(magnification_smoothing),
            get_wrap(wrap.0) as i32,
            get_wrap(wrap.1) as i32,
        );
        self.calls.push(DrawCall {
            texture,
            texture_size: (0, 0),
            program,
            attributes,
            blend: alpha_blending,
            srgb,
            highest_depth: -1.0,
        });

        if let Some(image) = image {
            insert_texture(
                &self.calls[index].texture,
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

        DrawCallHandle { index }
    }

    pub(crate) fn draw<'a, 'b>(&'a mut self, call: &'b DrawCallHandle) -> Sprite<'a, 'b> {
        Sprite::new(self, call)
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
        call: &DrawCallHandle,
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

        self.draw_quad((x0, y0, x1, y1), texcoords, color, rotation, z, call);
    }

    #[inline]
    pub(crate) fn draw_quad(
        &mut self,
        coords: (f32, f32, f32, f32),
        texcoords: (f32, f32, f32, f32),
        color: (f32, f32, f32, f32),
        rotation: (f32, f32, f32),
        depth: f32,
        call: &DrawCallHandle,
    ) {
        let (x0, y0, x1, y1) = coords;
        let (tx0, ty0, tx1, ty1) = texcoords;
        let (red, green, blue, alpha) = color;
        let (rads, pivot_x, pivot_y) = rotation;

        self.calls[call.index].highest_depth = self.calls[call.index].highest_depth.max(depth);
        if self.legacy {
            let (pivot_x, pivot_y) = (pivot_x + x0, pivot_y + y0);

            let quad = [
                x0, y0, depth, tx0, ty0, red, green, blue, alpha, rads, pivot_x,
                pivot_y, // Top-left vertex
                x1, y0, depth, tx1, ty0, red, green, blue, alpha, rads, pivot_x,
                pivot_y, // Top-right vertex
                x1, y1, depth, tx1, ty1, red, green, blue, alpha, rads, pivot_x,
                pivot_y, // Bottom-right vertex
                x0, y0, depth, tx0, ty0, red, green, blue, alpha, rads, pivot_x,
                pivot_y, // Top-left vertex
                x1, y1, depth, tx1, ty1, red, green, blue, alpha, rads, pivot_x,
                pivot_y, // Bottom-right vertex
                x0, y1, depth, tx0, ty1, red, green, blue, alpha, rads, pivot_x,
                pivot_y, // Bottom-left vertex
            ];

            self.calls[call.index]
                .attributes
                .vbo_data
                .extend_from_slice(&quad);
        } else {
            let (width, height, tw, th) = (x1 - x0, y1 - y0, tx1 - tx0, ty1 - ty0);
            let quad = [
                x0, y0, width, height, tx0, ty0, tw, th, red, green, blue, alpha, rads, pivot_x,
                pivot_y, depth,
            ];
            self.calls[call.index]
                .attributes
                .vbo_data
                .extend_from_slice(&quad);
        }
    }

    /// Renders the queued draws.
    pub(crate) fn render(&mut self, width: f32, height: f32, clear_color: (f32, f32, f32, f32)) {
        let m00 = 2.0 / width;
        let m11 = -2.0 / height;
        let matrix = [
            m00, 0.0, 0.0, -1.0, 0.0, m11, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        unsafe {
            gl::ClearColor(clear_color.0, clear_color.1, clear_color.2, clear_color.3);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        let legacy = self.legacy;

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
                gl::UseProgram(call.program.program);

                if !legacy {
                    if call.srgb {
                        gl::Enable(gl::FRAMEBUFFER_SRGB);
                    } else {
                        gl::Disable(gl::FRAMEBUFFER_SRGB);
                    }
                } else if let Some(gamma_correction_location) =
                    call.program.gamma_correction_location
                {
                    if call.srgb {
                        gl::Uniform1i(gamma_correction_location, 1);
                    } else {
                        gl::Uniform1i(gamma_correction_location, 0);
                    }
                }
                if let Some(location) = call.program.projection_matrix_location {
                    gl::UniformMatrix4fv(location, 1, gl::FALSE, matrix.as_ptr());
                }

                if !legacy {
                    gl::BindVertexArray(call.attributes.vao.0);
                    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, call.attributes.element_buffer.0);
                }
                gl::BindTexture(gl::TEXTURE_2D, call.texture.0);
                gl::BindBuffer(gl::ARRAY_BUFFER, call.attributes.vbo.0);
            }
            print_gl_errors(&format!("after initializing draw call #{}", i));

            if legacy {
                // 12 floats (3 for pos + 2 tex + 4 col + 3 rot) per vertex
                let vertex_count = call.attributes.vbo_data.len() as i32 / 12;
                enable_vertex_attribs(&[
                    (call.program.position_attrib_location, 3),
                    (call.program.texcoord_attrib_location, 2),
                    (call.program.color_attrib_location, 4),
                    (call.program.rotation_attrib_location, 3),
                ]);
                unsafe {
                    gl::DrawArrays(gl::TRIANGLES, 0, vertex_count);
                }
                disable_vertex_attribs(&[
                    call.program.position_attrib_location,
                    call.program.texcoord_attrib_location,
                    call.program.color_attrib_location,
                    call.program.rotation_attrib_location,
                ]);
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

            print_gl_errors(&*format!("after render #{}", i));
        }
    }

    /// Prepares the renderer for drawing.
    pub(crate) fn prepare_new_frame(&mut self, dpi_factor: f32) {
        self.dpi_factor = dpi_factor;
        for call in &mut self.calls {
            call.attributes.vbo_data.clear();
            call.highest_depth = -1.0;
        }
    }

    /// Renders all currently queued draws.
    ///
    /// First the non-alpha-blended calls, front to back, then the
    /// alpha-blended ones, back to front. Drawing from front to back
    /// is more efficient, as there is less overdraw because of depth
    /// testing, but proper blending requires back to front ordering.
    pub(crate) fn finish_frame(&mut self) {
        for call in &mut self.calls {
            unsafe {
                gl::BindBuffer(gl::ARRAY_BUFFER, call.attributes.vbo.0);
            }

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
            print_gl_errors("after pushing vertex buffer");
        }
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
    pub(crate) fn synchronize(&self) {
        use std::thread::sleep;
        use std::time::Duration;

        let mut synchronized = false;

        if !self.legacy {
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

    pub(crate) fn get_texture_size(&self, call: &DrawCallHandle) -> (i32, i32) {
        self.calls[call.index].texture_size
    }

    pub(crate) fn upload_texture_region(
        &self,
        call: &DrawCallHandle,
        region: RectPx,
        image: &Image,
    ) -> bool {
        let (tex_width, tex_height) = self.calls[call.index].texture_size;
        if region.width == image.width
            && region.height == image.height
            && region.x + region.width <= tex_width
            && region.y + region.height <= tex_height
            && region.x >= 0
            && region.y >= 0
        {
            insert_sub_texture(
                &self.calls[call.index].texture,
                region.x,
                region.y,
                region.width,
                region.height,
                image.format,
                &image.pixels,
            );
            true
        } else {
            false
        }
    }

    pub(crate) fn resize_texture(
        &mut self,
        call: &DrawCallHandle,
        new_width: i32,
        new_height: i32,
    ) -> bool {
        let (old_width, old_height) = self.calls[call.index].texture_size;
        if self.legacy {
            false
        } else if new_width >= old_width && new_height >= old_height {
            resize_texture(
                &self.calls[call.index].texture,
                old_width,
                old_height,
                new_width,
                new_height,
            );
            self.calls[call.index].texture_size = (new_width, new_height);
            true
        } else {
            false
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
        let legacy = self.legacy;
        for call in &self.calls {
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
            } = &call.attributes;
            unsafe {
                gl::DeleteShader(vertex_shader);
                gl::DeleteShader(fragment_shader);
                gl::DeleteProgram(program);
                gl::DeleteTextures(1, [call.texture.0].as_ptr());
                gl::DeleteBuffers(1, [vbo.0].as_ptr());
                if !legacy {
                    gl::DeleteBuffers(2, [vbo_static.0, element_buffer.0].as_ptr());
                    gl::DeleteVertexArrays(1, [vao.0].as_ptr());
                }
            }
        }
    }
}

#[inline]
fn create_program(vert_source: &str, frag_source: &str) -> ShaderProgram {
    let print_shader_error = |shader, shader_type| {
        let mut compilation_status = 0;
        unsafe {
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compilation_status);
        }
        if compilation_status as u8 != gl::TRUE {
            let mut info = vec![0; 1024];
            unsafe {
                gl::GetShaderInfoLog(shader, 1024, ptr::null_mut(), info.as_mut_ptr());
            }
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

    let program = unsafe { gl::CreateProgram() };

    let vertex_shader = unsafe { gl::CreateShader(gl::VERTEX_SHADER) };
    unsafe {
        gl::ShaderSource(
            vertex_shader,
            1,
            [vert_source.as_ptr() as *const _].as_ptr(),
            [vert_source.len() as GLint].as_ptr(),
        );
        gl::CompileShader(vertex_shader);
    }
    print_shader_error(vertex_shader, "vertex");

    let fragment_shader = unsafe { gl::CreateShader(gl::FRAGMENT_SHADER) };
    unsafe {
        gl::ShaderSource(
            fragment_shader,
            1,
            [frag_source.as_ptr() as *const _].as_ptr(),
            [frag_source.len() as GLint].as_ptr(),
        );
        gl::CompileShader(fragment_shader);
    }
    print_shader_error(fragment_shader, "fragment");

    unsafe {
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);
    }
    let mut link_status = 0;
    unsafe {
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut link_status);
    }
    if link_status as u8 != gl::TRUE {
        let mut info = vec![0; 1024];
        unsafe {
            gl::GetProgramInfoLog(program, 1024, ptr::null_mut(), info.as_mut_ptr());
        }
        log::error!(
            "Program linking failed:\n{}",
            error_buffer_into_string(info).trim()
        );
        if cfg!(debug_assertions) {
            panic!("program linking failed");
        }
    }
    print_gl_errors("after shader program creation");

    unsafe {
        gl::UseProgram(program);
    }
    let get_attrib_location = |name_ptr: &str| {
        let location = unsafe { gl::GetAttribLocation(program, name_ptr.as_ptr() as *const _) };
        match location {
            -1 => None,
            x => Some(x as GLuint),
        }
    };
    let get_uniform_location = |name_ptr: &str| {
        let location = unsafe { gl::GetUniformLocation(program, name_ptr.as_ptr() as *const _) };
        match location {
            -1 => None,
            x => Some(x),
        }
    };

    ShaderProgram {
        program,
        vertex_shader,
        fragment_shader,
        projection_matrix_location: get_uniform_location("projection_matrix\0"),
        gamma_correction_location: get_uniform_location("gamma_correct\0"),
        position_attrib_location: get_attrib_location("position\0"),
        texcoord_attrib_location: get_attrib_location("texcoord\0"),
        color_attrib_location: get_attrib_location("color\0"),
        rotation_attrib_location: get_attrib_location("rotation\0"),
        depth_attrib_location: get_attrib_location("depth\0"),
        shared_position_attrib_location: get_attrib_location("shared_position\0"),
        shared_texcoord_attrib_location: get_attrib_location("shared_texcoord\0"),
    }
}

#[inline]
fn create_attributes(legacy: bool, program: &ShaderProgram) -> Attributes {
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
        }
        enable_vertex_attribs(&[
            (program.shared_position_attrib_location, 2),
            (program.shared_texcoord_attrib_location, 2),
        ]);
        // The vertices of two triangles that form a quad, interleaved
        // in a (pos x, pos y, tex x, tex y) arrangement:
        let static_quad_vertices: [f32; 16] = [
            0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 1.0,
        ];
        let len = (mem::size_of::<f32>() * static_quad_vertices.len()) as isize;
        let ptr = static_quad_vertices.as_ptr() as *const _;
        unsafe {
            gl::BufferData(gl::ARRAY_BUFFER, len, ptr, gl::STATIC_DRAW);
        }

        unsafe {
            gl::GenBuffers(1, &mut element_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer);
        }
        let elements: [u8; 6] = [0, 1, 2, 0, 2, 3];
        let len = (mem::size_of::<f32>() * elements.len()) as isize;
        let ptr = elements.as_ptr() as *const _;
        unsafe {
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, len, ptr, gl::STATIC_DRAW);
        }
    }

    let mut vbo = 0;
    unsafe {
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
    }

    if !legacy {
        enable_vertex_attribs(&[
            (program.position_attrib_location, 4),
            (program.texcoord_attrib_location, 4),
            (program.color_attrib_location, 4),
            (program.rotation_attrib_location, 3),
            (program.depth_attrib_location, 1),
        ]);

        let setup_vertex_attrib_divisor = |location: Option<GLuint>| {
            if let Some(location) = location {
                unsafe {
                    gl::VertexAttribDivisor(location, 1);
                }
            }
        };
        setup_vertex_attrib_divisor(program.position_attrib_location);
        setup_vertex_attrib_divisor(program.texcoord_attrib_location);
        setup_vertex_attrib_divisor(program.color_attrib_location);
        setup_vertex_attrib_divisor(program.rotation_attrib_location);
        setup_vertex_attrib_divisor(program.depth_attrib_location);
    }
    print_gl_errors("after attribute creation");

    Attributes {
        vao: VaoHandle(vao),
        vbo: VboHandle(vbo),
        vbo_static: VboHandle(vbo_static),
        element_buffer: VboHandle(element_buffer),
        vbo_data: Vec::new(),
        allocated_vbo_data_size: 0,
    }
}

// (location, component_count)
type AttribArray = (Option<GLuint>, GLint);
fn enable_vertex_attribs(attribs: &[AttribArray]) {
    let total_components = attribs.iter().map(|attrib| attrib.1 * 4).sum();

    let mut offset = 0;
    for attrib in attribs {
        // Only enable the attributes that exist
        if let Some(location) = attrib.0 {
            unsafe {
                gl::VertexAttribPointer(
                    location,           /* Attrib location */
                    attrib.1,           /* Components */
                    gl::FLOAT,          /* Type */
                    gl::FALSE,          /* Normalize */
                    total_components,   /* Stride */
                    offset as *const _, /* Offset */
                );
                gl::EnableVertexAttribArray(location);
            }
        }
        let component_size = attrib.1 * 4;
        offset += component_size;
    }

    print_gl_errors("after enabling vertex attributes");
}

fn disable_vertex_attribs(attrib_locations: &[Option<GLuint>]) {
    for location in attrib_locations {
        if let Some(location) = location {
            unsafe {
                gl::DisableVertexAttribArray(*location);
            }
        }
    }

    print_gl_errors("after disabling vertex attributes");
}

#[inline]
fn create_texture(min: GLint, mag: GLint, wrap_s: GLint, wrap_t: GLint) -> TextureHandle {
    let mut tex = 0;
    unsafe {
        gl::GenTextures(1, &mut tex);
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_s);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_t);
    }
    print_gl_errors("after creating a texture");
    TextureHandle(tex)
}

#[inline]
fn insert_texture(
    texture: &TextureHandle,
    format: GLuint,
    pixel_type: GLuint,
    width: GLint,
    height: GLint,
    pixels: Option<&[u8]>,
) {
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, texture.0);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            format as GLint,
            width,
            height,
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
    texture: &TextureHandle,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    format: GLuint,
    data: &[u8],
) {
    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, texture.0);
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
    print_gl_errors("after insert_sub_texture");
}

fn resize_texture(
    texture: &TextureHandle,
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
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, texture.0, 0);
        // Create temp texture and copy over the texture from the framebuffer
        gl::GenTextures(1, &mut temp_tex);
        gl::BindTexture(gl::TEXTURE_2D, temp_tex);
        gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        gl::CopyTexImage2D(gl::TEXTURE_2D, 0, gl::RED, 0, 0, old_width, old_height, 0);
        // Re-create the original texture
        gl::BindTexture(gl::TEXTURE_2D, texture.0);
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
        gl::DeleteTextures(1, &temp_tex);
        gl::DeleteFramebuffers(1, &fbo);
    }
    print_gl_errors("after resize_texture");
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
    let original_len = raw.len();
    let len = raw
        .iter()
        .enumerate() // Enumerate: values are (index, &value)
        .find(|&(_, &v)| v == 0) // Find the nul byte
        .map(|(i, _)| i) // Get the index to use as the length
        .unwrap_or(original_len); // If no nul byte was found, assume the whole buffer len
    let info: Vec<u8> = raw[0..len]
        .iter()
        .map(|&i| unsafe { mem::transmute::<i8, u8>(i) })
        .collect();
    String::from_utf8_lossy(&info).to_string()
}
