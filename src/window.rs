//! Native window setup for the [`Shell`](crate::Shell): an undecorated
//! viewport, rounded corners on Windows 11, and auto-exiting screenshot runs
//! for visual checks.
//!
//! ```no_run
//! use spaceui::{Theme, window};
//!
//! let options = eframe::NativeOptions { viewport: window::viewport("My App", "my-app"), ..Default::default() };
//! # let _ = options;
//! // In the app creator:
//! // window::round_corners(cc, Theme::of(&cc.egui_ctx).palette.border);
//! ```
use egui::{Color32, ColorImage, Context, ViewportBuilder, ViewportCommand};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// An undecorated viewport for [`Shell`](crate::Shell), which draws its own
/// title bar and resize edges. Chain more builder calls as needed, such as
/// `.with_icon(...)` from [`Svg::window_icon`](crate::Svg::window_icon).
#[must_use]
pub fn viewport(title: &str, app_id: &str) -> ViewportBuilder {
    ViewportBuilder::default()
        .with_title(title)
        .with_app_id(app_id)
        .with_inner_size([1480.0, 920.0])
        .with_min_inner_size([960.0, 600.0])
        .with_decorations(false)
}

/// Asks Windows 11 to round and anti-alias the corners of a borderless
/// window and to draw its one-pixel border in `border` (the palette's
/// `border` suits). Earlier Windows versions and other systems keep their
/// default corners. Pass eframe's `CreationContext`.
pub fn round_corners(window: &impl raw_window_handle::HasWindowHandle, border: Color32) {
    #[cfg(windows)]
    if let Ok(handle) = window.window_handle()
        && let raw_window_handle::RawWindowHandle::Win32(win32) = handle.as_raw()
    {
        const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
        const DWMWA_BORDER_COLOR: u32 = 34;
        const DWMWCP_ROUND: u32 = 2;
        // COLORREF is 0x00BBGGRR.
        let colorref = u32::from(border.r()) | (u32::from(border.g()) << 8) | (u32::from(border.b()) << 16);
        for (attribute, value) in [(DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND), (DWMWA_BORDER_COLOR, colorref)] {
            dwm_set_window_attribute(win32.hwnd.get(), attribute, value);
        }
    }
    #[cfg(not(windows))]
    let _ = (window, border);
}

/// `DwmSetWindowAttribute` with a 32-bit value. Unknown attributes (on
/// Windows 10) return an error, which leaves the window as it was.
#[cfg(windows)]
#[allow(unsafe_code)]
fn dwm_set_window_attribute(hwnd: isize, attribute: u32, value: u32) {
    #[link(name = "dwmapi")]
    unsafe extern "system" {
        fn DwmSetWindowAttribute(hwnd: isize, attribute: u32, value: *const core::ffi::c_void, size: u32) -> i32;
    }
    // SAFETY: `hwnd` comes from a live window handle, and `value` is a valid
    // 4-byte buffer for the duration of the call.
    unsafe {
        DwmSetWindowAttribute(hwnd, attribute, (&raw const value).cast(), 4);
    }
}

/// A screenshot run: renders a few frames, saves the window as a PNG and
/// closes the app. For native visual checks of pages and themes.
///
/// ```no_run
/// # use spaceui::{egui, window::Screenshot};
/// # struct App { screenshot: Option<Screenshot> }
/// # impl App {
/// fn ui(&mut self, ui: &mut egui::Ui) {
///     // ... draw the app ...
///     if let Some(shot) = &mut self.screenshot
///         && let Some(Err(error)) = shot.update(ui.ctx())
///     {
///         eprintln!("screenshot failed: {error}");
///     }
/// }
/// # }
/// # let _ = App { screenshot: Screenshot::from_env("MYAPP_SCREENSHOT") };
/// ```
#[derive(Debug)]
pub struct Screenshot {
    path: PathBuf,
    frames: u64,
    outcome: Arc<Mutex<Option<Result<(), String>>>>,
}

impl Screenshot {
    /// A screenshot run when the environment variable `var` names an output path.
    #[must_use]
    pub fn from_env(var: &str) -> Option<Self> {
        std::env::var_os(var).map(|path| Self::new(path.into()))
    }

    /// A screenshot run saving to `path`.
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self { path, frames: 0, outcome: Arc::default() }
    }

    /// Call once per frame, after drawing. Returns the outcome once the PNG is
    /// written (or failed); the app is closing by then.
    pub fn update(&mut self, ctx: &Context) -> Option<Result<(), String>> {
        // A few frames let fonts, textures and layout settle.
        const SETTLE_FRAMES: u64 = 8;
        self.frames += 1;
        if self.frames == SETTLE_FRAMES {
            let (path, outcome, closer) = (self.path.clone(), self.outcome.clone(), ctx.clone());
            ctx.request_screenshot(move |image| {
                let result = save_png(&image, &path);
                if let Ok(mut slot) = outcome.lock() {
                    *slot = Some(result);
                }
                closer.send_viewport_cmd(ViewportCommand::Close);
            });
        }
        ctx.request_repaint();
        self.outcome.lock().ok().and_then(|mut slot| slot.take())
    }
}

fn save_png(image: &ColorImage, path: &Path) -> Result<(), String> {
    let [width, height] = image.size;
    let data: Vec<u8> = image.pixels.iter().flat_map(Color32::to_array).collect();
    let size = resvg::tiny_skia::IntSize::from_wh(width as u32, height as u32).ok_or("empty screenshot")?;
    let pixmap = resvg::tiny_skia::Pixmap::from_vec(data, size).ok_or("bad screenshot buffer")?;
    pixmap.save_png(path).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screenshots_are_written_as_png() {
        let path = std::env::temp_dir().join(format!("spaceui-shot-{}.png", std::process::id()));
        let image = ColorImage::new([3, 2], vec![Color32::from_rgb(234, 124, 52); 6]);
        save_png(&image, &path).expect("written");
        let bytes = std::fs::read(&path).expect("readable");
        let _ = std::fs::remove_file(&path);
        assert!(bytes.starts_with(b"\x89PNG"));
    }
}
