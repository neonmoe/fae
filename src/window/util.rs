use std::default::Default;
use std::env;

#[allow(dead_code)]
pub(crate) fn get_env_dpi() -> f32 {
    let get_var = |name: &str| {
        env::var(name)
            .ok()
            .and_then(|var| var.parse::<f32>().ok())
            .filter(|f| *f > 0.0)
    };
    if let Some(dpi_factor) = get_var("QT_AUTO_SCREEN_SCALE_FACTOR") {
        return dpi_factor;
    }
    if let Some(dpi_factor) = get_var("QT_SCALE_FACTOR") {
        return dpi_factor;
    }
    if let Some(dpi_factor) = get_var("GDK_SCALE") {
        return dpi_factor;
    }
    if let Some(dpi_factor) = get_var("ELM_SCALE") {
        return dpi_factor;
    }
    1.0
}

/// Defines a window.
pub struct WindowSettings {
    /// Title of the window. Default value: Name of the executable
    /// file.
    pub title: String,
    /// Width of the window in logical pixels. Default value: `640.0`.
    pub width: f32,
    /// Height of the window in logical pixels. Default value:
    /// `480.0`.
    pub height: f32,
    /// Whether or not the application is a dialog. Default value:
    /// `true`.
    ///
    /// This only affects x11 environments with the `glutin` backend,
    /// where it sets the window type to dialog. In [tiling
    /// environments](https://en.wikipedia.org/wiki/Tiling_window_manager),
    /// like i3 and sway, this can cause the window to pop up as a
    /// floating window, not a tiled one. This is useful for
    /// applications that are supposed to be opened for very short
    /// amounts of time.
    // TODO: Replace with UnixOptions or something similar?
    // See docs for glutin::os::unix::WindowBuilderExt for more options.
    // Or perhaps remove this and provide a way to affect glutin Window and Context creation?
    // A handle to the glutin window would probably be good as well.
    pub is_dialog: bool,
    /// This should always be true for everything except
    /// benchmarks. Default value: `true`.
    pub vsync: bool,
    /// Sets the multisampling level. Default value: `4`.
    pub multisample: u16,
}

impl Default for WindowSettings {
    fn default() -> WindowSettings {
        WindowSettings {
            title: env::current_exe()
                .ok()
                .and_then(|p| p.file_name().map(std::ffi::OsStr::to_os_string))
                .and_then(|s| s.into_string().ok())
                .unwrap_or_default(),
            width: 640.0,
            height: 480.0,
            is_dialog: false,
            vsync: true,
            multisample: 4,
        }
    }
}
