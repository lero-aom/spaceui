//! Native window setup for SpaceUI's shell: an undecorated
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
#[cfg(feature = "native")]
use egui::Color32;
use egui::ViewportBuilder;

#[cfg(feature = "screenshot")]
pub use screenshot::{Screenshot, ScreenshotError};

/// An undecorated viewport for SpaceUI's shell, which draws its own
/// title bar and resize edges. Chain more builder calls as needed, such as
/// `.with_icon(...)` from `Svg::window_icon` (the `svg` feature).
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
#[cfg(feature = "native")]
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
#[cfg(all(windows, feature = "native"))]
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

#[cfg(feature = "screenshot")]
mod screenshot {
    use egui::{Color32, ColorImage, Context, ViewportCommand};
    use std::{
        error::Error,
        fmt,
        path::{Path, PathBuf},
        sync::{Arc, Mutex, PoisonError},
    };

    /// A screenshot could not be encoded or written.
    #[derive(Debug)]
    #[non_exhaustive]
    pub enum ScreenshotError {
        /// The image has invalid dimensions or a mismatched pixel buffer.
        InvalidImage,
        /// Encoding or writing the PNG failed.
        Write {
            /// Output path that failed.
            path: PathBuf,
            /// Underlying encoder or I/O error.
            source: Box<dyn Error + Send + Sync>,
        },
    }

    impl fmt::Display for ScreenshotError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::InvalidImage => f.write_str("invalid screenshot dimensions or pixel buffer"),
                Self::Write { path, source } => write!(f, "{}: {source}", path.display()),
            }
        }
    }

    impl Error for ScreenshotError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                Self::Write { source, .. } => Some(source.as_ref()),
                Self::InvalidImage => None,
            }
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
    #[must_use = "call update or try_update each frame to drive the screenshot run"]
    pub struct Screenshot {
        path: PathBuf,
        state: CaptureState,
        outcome: Arc<Mutex<Option<Result<(), ScreenshotError>>>>,
    }

    #[derive(Debug)]
    enum CaptureState {
        Settling(u8),
        Requested,
        Finished,
    }

    impl Screenshot {
        /// A screenshot run when the environment variable `var` names an output path.
        #[must_use]
        pub fn from_env(var: &str) -> Option<Self> {
            std::env::var_os(var).map(|path| Self::new(path.into()))
        }

        /// A screenshot run saving to `path`.
        pub fn new(path: PathBuf) -> Self {
            Self { path, state: CaptureState::Settling(0), outcome: Arc::default() }
        }

        /// Call once per frame, after drawing. Returns the outcome once the PNG is
        /// written (or failed); the app is closing by then.
        pub fn update(&mut self, ctx: &Context) -> Option<Result<(), String>> {
            self.try_update(ctx).map(|result| result.map_err(|error| error.to_string()))
        }

        /// Like [`Self::update`], preserving the error type and its source.
        /// After eight settling frames, requests one capture and waits for the
        /// backend callback without continuously repainting. Completion closes
        /// the captured viewport and returns its outcome exactly once.
        ///
        /// # Errors
        /// The returned outcome reports invalid image buffers or encoding/I/O
        /// failures. `None` means the capture has not completed (or was consumed).
        pub fn try_update(&mut self, ctx: &Context) -> Option<Result<(), ScreenshotError>> {
            // A few frames let fonts, textures and layout settle.
            const SETTLE_FRAMES: u8 = 8;
            if matches!(self.state, CaptureState::Finished) {
                return None;
            }
            let result = self.outcome.lock().unwrap_or_else(PoisonError::into_inner).take();
            if result.is_some() {
                self.state = CaptureState::Finished;
                return result;
            }
            let CaptureState::Settling(frames) = &mut self.state else {
                return None;
            };
            *frames += 1;
            if *frames == SETTLE_FRAMES {
                self.state = CaptureState::Requested;
                let (path, outcome, closer) = (self.path.clone(), self.outcome.clone(), ctx.clone());
                let viewport = ctx.viewport_id();
                ctx.request_screenshot(move |image| {
                    let result = save_png(&image, &path);
                    *outcome.lock().unwrap_or_else(PoisonError::into_inner) = Some(result);
                    // The callback may run after another viewport became current.
                    // Sending the command also wakes the captured viewport.
                    closer.send_viewport_cmd_to(viewport, ViewportCommand::Close);
                });
            } else {
                ctx.request_repaint();
            }
            None
        }
    }

    fn save_png(image: &ColorImage, path: &Path) -> Result<(), ScreenshotError> {
        let [width, height] = image.size;
        if width.checked_mul(height) != Some(image.pixels.len()) {
            return Err(ScreenshotError::InvalidImage);
        }
        let width = u32::try_from(width).map_err(|_| ScreenshotError::InvalidImage)?;
        let height = u32::try_from(height).map_err(|_| ScreenshotError::InvalidImage)?;
        let size = resvg::tiny_skia::IntSize::from_wh(width, height).ok_or(ScreenshotError::InvalidImage)?;
        let data: Vec<u8> = image.pixels.iter().flat_map(Color32::to_array).collect();
        let pixmap = resvg::tiny_skia::Pixmap::from_vec(data, size).ok_or(ScreenshotError::InvalidImage)?;
        pixmap
            .save_png(path)
            .map_err(|source| ScreenshotError::Write { path: path.to_owned(), source: Box::new(source) })
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use egui::{RawInput, ViewportId};
        use std::time::Duration;

        #[test]
        fn invalid_screenshot_buffers_are_rejected_before_writing() {
            let path = std::env::temp_dir().join("spaceui-invalid-shot.png");
            for size in [[0, 1], [1, 0], [usize::MAX, 2], [3, 2]] {
                let mut image = ColorImage::new([1, 1], vec![Color32::WHITE]);
                image.size = size;
                assert!(matches!(save_png(&image, &path), Err(ScreenshotError::InvalidImage)));
            }
        }

        #[test]
        fn capture_is_one_shot_and_closes_its_original_viewport_without_busy_repainting() {
            let path = std::env::temp_dir().join(format!("spaceui-shot-lifecycle-{}.png", std::process::id()));
            let mut shot = Screenshot::new(path.clone());
            let ctx = Context::default();
            let viewport = ViewportId::from_hash_of("screenshot-child");
            let input = || {
                let mut input = RawInput { viewport_id: viewport, ..RawInput::default() };
                input
                    .viewports
                    .insert(viewport, egui::ViewportInfo { parent: Some(ViewportId::ROOT), ..Default::default() });
                input
            };
            let mut callbacks = Vec::new();
            for _ in 0..12 {
                let output = ctx.run_ui(input(), |ui| assert!(shot.try_update(ui.ctx()).is_none()));
                callbacks.extend(output.viewport_output[&viewport].commands.iter().filter_map(|command| {
                    if let ViewportCommand::Screenshot(callback) = command { Some(callback.clone()) } else { None }
                }));
                output.drop_without_applying_deltas();
            }
            assert_eq!(callbacks.len(), 1, "only one request while waiting for the backend");
            let output = ctx.run_ui(input(), |ui| assert!(shot.try_update(ui.ctx()).is_none()));
            assert_eq!(output.viewport_output[&viewport].repaint_delay, Duration::MAX);
            output.drop_without_applying_deltas();

            // The backend completes the capture after the root viewport ran.
            ctx.run_ui(RawInput::default(), |_| {}).drop_without_applying_deltas();
            callbacks
                .pop()
                .expect("capture requested")
                .complete(Arc::new(ColorImage::new([1, 1], vec![Color32::WHITE])));
            let output = ctx.run_ui(input(), |ui| {
                assert!(shot.try_update(ui.ctx()).expect("completed outcome").is_ok());
                assert!(shot.try_update(ui.ctx()).is_none(), "outcome is consumed once");
            });
            assert!(output.viewport_output[&viewport].commands.contains(&ViewportCommand::Close));
            assert!(!output.viewport_output[&ViewportId::ROOT].commands.contains(&ViewportCommand::Close));
            output.drop_without_applying_deltas();
            // Let egui's own repaint bookkeeping settle after the close command.
            for _ in 0..4 {
                ctx.run_ui(input(), |ui| assert!(shot.try_update(ui.ctx()).is_none())).drop_without_applying_deltas();
            }
            let output = ctx.run_ui(input(), |ui| assert!(shot.try_update(ui.ctx()).is_none()));
            assert_eq!(output.viewport_output[&viewport].repaint_delay, Duration::MAX);
            output.drop_without_applying_deltas();
            std::fs::remove_file(path).expect("remove screenshot");
        }

        #[test]
        fn screenshots_are_written_as_png() {
            let path = std::env::temp_dir().join(format!("spaceui-shot-{}.png", std::process::id()));
            let image = ColorImage::new([3, 2], vec![Color32::from_rgb(234, 124, 52); 6]);
            save_png(&image, &path).expect("written");
            let bytes = std::fs::read(&path).expect("readable");
            let _ = std::fs::remove_file(&path);
            assert!(bytes.starts_with(b"\x89PNG"));
        }

        #[test]
        fn screenshot_errors_preserve_the_path_and_source() {
            let path = std::env::temp_dir().join(format!("spaceui-missing-{}/out.png", std::process::id()));
            let image = ColorImage::new([1, 1], vec![Color32::WHITE]);
            let error = save_png(&image, &path).expect_err("missing parent");
            assert!(error.source().is_some());
            assert!(error.to_string().contains("out.png"));
        }
    }
}
