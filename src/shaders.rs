use crate::gl_version::OpenGlApi;

/// Contains the shader code for a spritesheet.
///
/// Passed to the renderer with
/// [`SpritesheetBuilder::shaders`](struct.SpritesheetBuilder.html#method.shaders).
///
/// # The GLSL versions
///
/// As you might need to include tweaks for every GLSL version, you
/// can provide versions of your shader code for each. However,
/// usually the `#version 100`/`#version 110` and `#version 300
/// es`/`#version 330` shaders are identical aside from the version
/// string. To make this use-case more ergonomic, you can just leave
/// the version preprocessor line out, and the relevant one is
/// inserted during runtime. Then you can use the same
/// [`ShaderPair`](struct.ShaderPair.html) for `shader_110` and
/// `shader_100`, for example.
///
/// Additionally: if you don't include a version preprocessor,
/// `precision mediump float;` is added to the shader's OpenGL ES
/// version (in addition to the automatically inserted version
/// preprocessor), as it's required in OpenGL ES shaders but not
/// desktop OpenGL ones.
///
/// As an example, the following `shader_300_es` code:
/// ```glsl, ignore
/// void main() {}
/// ```
/// Will be modified into the following in an OpenGL ES 3.0 context:
/// ```glsl, ignore
/// #version 300 es
/// precision mediump float;
/// void main() {}
/// ```
///
/// # Example
/// ```no_run
#[doc = "# let mut ctx = fae::GraphicsContext::dummy();
# let fragment_shader_code_330 = String::new();
# let fragment_shader_code_110 = String::new();
use fae::{Shaders, SpritesheetBuilder};

// If you want to just change the fragment shaders, create a default Shaders:
let mut shaders = Shaders::default();

// And then apply your changes:
shaders.shader_330.fragment_shader = fragment_shader_code_330.clone();
shaders.shader_300_es.fragment_shader = fragment_shader_code_330;
shaders.shader_110.fragment_shader = fragment_shader_code_110.clone();
shaders.shader_100_es.fragment_shader = fragment_shader_code_110;

// Then you can use the shaders when creating a Spritesheet:
let spritesheet = SpritesheetBuilder::new()
    .shaders(shaders)
    .build(&mut ctx);
"]
/// ```
#[derive(Clone, Debug)]
pub struct Shaders {
    /// The `#version 330` version of the shader, for OpenGL 3.3 and above.
    pub shader_330: ShaderPair,
    /// The `#version 110` version of the shader, for OpenGL versions before 3.3.
    pub shader_110: ShaderPair,
    /// The `#version 300 es` version of the shader, for OpenGL ES 3.0 and WebGL 2.0.
    pub shader_300_es: ShaderPair,
    /// The `#version 100` version of the shader, for OpenGL ES 2.0 and WebGL 1.0.
    pub shader_100_es: ShaderPair,
}

enum ShaderType {
    Vertex,
    Fragment,
}

/// Contains the code for a vertex shader and a fragment shader.
///
/// See also: [`Shaders`](struct.Shaders.html).
#[derive(Clone, Debug)]
pub struct ShaderPair {
    /// The vertex shader code.
    pub vertex_shader: String,
    /// The fragment shader code.
    pub fragment_shader: String,
}

impl ShaderPair {
    fn get_shader(&self, shader_type: ShaderType) -> &str {
        match shader_type {
            ShaderType::Vertex => &self.vertex_shader,
            ShaderType::Fragment => &self.fragment_shader,
        }
    }
}

impl Default for Shaders {
    fn default() -> Self {
        let legacy = ShaderPair {
            vertex_shader: include_str!("shaders/legacy/texquad.vert").to_string(),
            fragment_shader: include_str!("shaders/legacy/texquad.frag").to_string(),
        };
        let modern = ShaderPair {
            vertex_shader: include_str!("shaders/texquad.vert").to_string(),
            fragment_shader: include_str!("shaders/texquad.frag").to_string(),
        };
        Shaders {
            shader_330: modern.clone(),
            shader_110: legacy.clone(),
            shader_300_es: modern,
            shader_100_es: legacy,
        }
    }
}

impl Shaders {
    pub(crate) fn create_vert_string(&self, api: OpenGlApi, legacy: bool) -> String {
        self.create_string(api, legacy, ShaderType::Vertex)
    }

    pub(crate) fn create_frag_string(&self, api: OpenGlApi, legacy: bool) -> String {
        self.create_string(api, legacy, ShaderType::Fragment)
    }

    fn create_string(&self, api: OpenGlApi, legacy: bool, shader_type: ShaderType) -> String {
        let (base_string, version_string) = match api {
            OpenGlApi::Desktop => {
                if legacy {
                    (self.shader_110.get_shader(shader_type), "#version 110")
                } else {
                    (self.shader_330.get_shader(shader_type), "#version 330")
                }
            }
            OpenGlApi::ES => {
                if legacy {
                    (self.shader_100_es.get_shader(shader_type), "#version 100")
                } else {
                    (
                        self.shader_300_es.get_shader(shader_type),
                        "#version 300 es",
                    )
                }
            }
        };

        if base_string.contains("#version") {
            if cfg!(debug_assertions) && !base_string.contains(version_string) {
                // There is a #version but it isn't what we'd expect.
                if let Some(shader_version) = base_string
                    .lines()
                    .find(|line| line.starts_with("#version"))
                {
                    log::warn!(
                        "Shader has version: '{}', expected '{}'.",
                        shader_version,
                        version_string
                    );
                }
            }
            base_string.to_string()
        } else {
            let mut header = version_string.to_string() + "\n";
            if api != OpenGlApi::Desktop {
                header += "precision mediump float;\n";
            }
            header + base_string
        }
    }
}
