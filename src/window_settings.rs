use std::default::Default;
use std::env;

/// Defines a window.
pub struct WindowSettings {
    /// Title of the window. Default value: Name of the executable file
    pub title: String,
    /// Width of the window in logical pixels. Default value: `640.0`
    pub width: f32,
    /// Height of the window in logical pixels. Default value: `480.0`
    pub height: f32,
    /// Whether or not the application is a dialog. Default value: `true`
    ///
    /// This only affects x11 environments with the `glutin` backend,
    /// where it sets the window type to dialog. In [tiling
    /// environments](https://en.wikipedia.org/wiki/Tiling_window_manager),
    /// like i3 and sway, this can cause the window to pop up as a
    /// floating window, not a tiled one. This is useful for
    /// applications that are supposed to be opened for very short
    /// amounts of time.
    pub is_dialog: bool,
    /// This should always be true for everything except benchmarks.
    pub vsync: bool,
}

impl Default for WindowSettings {
    fn default() -> WindowSettings {
        WindowSettings {
            title: env::current_exe()
                .ok()
                .and_then(|p| p.file_name().map(|s| s.to_os_string()))
                .and_then(|s| s.into_string().ok())
                .unwrap_or_default(),
            width: 640.0,
            height: 480.0,
            is_dialog: false,
            vsync: true,
        }
    }
}
