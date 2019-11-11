//! This module contains functionality for reading the current context's OpenGL version.
use crate::gl;
use std::ffi::CStr;

/// Represents the two different variants of OpenGL.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OpenGlApi {
    /// "Normal" OpenGL, which usually means the environment is
    /// the desktop.
    Desktop,
    /// OpenGL ES, which usually means the environment is mobile.
    ES,
    // TODO: Add WebGL, which seems to be "WebGL ?.? (OpenGL ES ?.? ????)" maybe?
}

/// Represents the parsed version of the OpenGL version string.
#[derive(Clone, Debug, PartialEq)]
pub enum OpenGlVersion {
    /// Represents a version of the OpenGL api.
    Available {
        /// Which [`OpenGlApi`](enum.OpenGlApi.html) the version
        /// string would imply.
        api: OpenGlApi,
        /// The major version according to the version string.
        major: u8,
        /// The minor version according to the version string.
        minor: u8,
    },

    /// This is what is returned when the OpenGL version string can't
    /// be read. Please open an issue at
    /// [https://github.com/neonmoe/fae](https://github.com/neonmoe/fae)
    /// and provide the value of `version_string` if you run across
    /// this.
    Unavailable {
        /// The version string that could not be parsed.
        version_string: String,
    },
}

/// Parses the current thread's OpenGL context's version string.
///
/// See the documentation for
/// [`OpenGlVersion`](enum.OpenGlVersion.html).
pub fn get_version() -> OpenGlVersion {
    let version_str = unsafe { CStr::from_ptr(gl::GetString(gl::VERSION) as *const _) };
    let version_str = version_str.to_string_lossy();
    if let Some((es, major, minor)) = parse_version(&version_str) {
        OpenGlVersion::Available {
            api: if es {
                OpenGlApi::ES
            } else {
                OpenGlApi::Desktop
            },
            major,
            minor,
        }
    } else {
        OpenGlVersion::Unavailable {
            version_string: version_str.to_string(),
        }
    }
}

// Sorry for the mess, but OpenGL version strings are unreliable, and
// I'm not sure *how* unreliable. Here's my attempt at a robust way of
// parsing the version. Returns (opengl es?, major version, minor version).
fn parse_version(version_str: &str) -> Option<(bool, u8, u8)> {
    let mut es = false;
    let version_str = if version_str.starts_with("OpenGL ES-") {
        if version_str.len() < 16 {
            // If the string starts with OpenGL ES- but the string is
            // not at least 16 characters long (OpenGL ES-<2 chars>
            // <digit>.<digit>), the version string is somehow really
            // broken.
            return None;
        }

        es = true;
        // Cut off the "OpenGL ES-CM " (or "OpenGL ES-CL ") part.
        &version_str[13..]
    } else {
        &version_str
    };

    let mut split = version_str.split('.'); // Split at .
    let major_str = &split.next()?; // Major version is the first part before the first .
    let major = u8::from_str_radix(major_str, 10).ok()?; // Parse the version

    let rest_of_version = split.next()?; // Find the next part after the first .
    let end_of_version_num = rest_of_version
        .find(|c: char| !c.is_digit(10))
        .unwrap_or(rest_of_version.len()); // Find where the minor version ends
    let minor_str = &rest_of_version[0..end_of_version_num]; // Minor version as str
    let minor = u8::from_str_radix(minor_str, 10).ok()?; // Parse minor version

    Some((es, major, minor))
}
