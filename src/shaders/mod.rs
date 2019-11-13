use crate::gl_version::OpenGlApi;

/// Represents the shader code for a shader. Used in
/// [`Renderer::create_draw_call`](struct.Renderer.html#method.create_draw_call).
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

static DEFAULT_SHADERS: [&'static str; 4] = [
    include_str!("legacy/texquad.vert"),
    include_str!("legacy/texquad.frag"),
    include_str!("texquad.vert"),
    include_str!("texquad.frag"),
];
impl Default for Shaders {
    fn default() -> Self {
        Shaders {
            vertex_shader_110: DEFAULT_SHADERS[0],
            fragment_shader_110: DEFAULT_SHADERS[1],
            vertex_shader_330: DEFAULT_SHADERS[2],
            fragment_shader_330: DEFAULT_SHADERS[3],
        }
    }
}

impl Shaders {
    pub(crate) fn create_vert_string(&self, api: OpenGlApi, legacy: bool) -> String {
        create_string(api, legacy, self.vertex_shader_110, self.vertex_shader_330)
    }

    pub(crate) fn create_frag_string(&self, api: OpenGlApi, legacy: bool) -> String {
        create_string(
            api,
            legacy,
            self.fragment_shader_110,
            self.fragment_shader_330,
        )
    }
}

fn create_string(
    api: OpenGlApi,
    legacy: bool,
    legacy_str: &'static str,
    modern_str: &'static str,
) -> String {
    let base_string = if legacy { legacy_str } else { modern_str };

    match api {
        OpenGlApi::Desktop => base_string.to_string(),
        OpenGlApi::ES => base_string
            .to_string()
            .lines()
            .map(|line| replace_version(line, legacy))
            .fold(
                // The 28 here is the maximum amount of bytes added by replace_version
                String::with_capacity(base_string.len() + 28),
                |acc, line| acc + line + "\n",
            ),
    }
}

fn replace_version(line: &str, legacy: bool) -> &str {
    if line.starts_with("#version") {
        if legacy {
            "#version 100"
        } else {
            "#version 300 es\nprecision mediump float;"
        }
    } else {
        line
    }
}
