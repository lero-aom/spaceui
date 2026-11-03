// The icon list mirrors `assets/icons/`; `icon_set_matches_the_asset_folder`
// fails when they drift apart. Add an icon by adding its SVG there and a
// variant plus two match arms here.
use super::Svg;
use egui::{Color32, Painter, Rect};

/// The built-in 24 px line icons, drawn in white so painting can tint them.
///
/// ```
/// use spaceui::{Icon, egui};
///
/// # let ctx = egui::Context::default();
/// # ctx.run_ui(Default::default(), |ui| {
/// let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::Vec2::splat(20.0));
/// Icon::Search.paint(ui.painter(), rect, egui::Color32::WHITE);
/// # }).drop_without_applying_deltas();
/// assert_eq!(Icon::Search.name(), "search");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Icon {
    /// `assets/icons/alert.svg`
    Alert,
    /// `assets/icons/back.svg`
    Back,
    /// `assets/icons/bell.svg`
    Bell,
    /// `assets/icons/book.svg`
    Book,
    /// `assets/icons/bookmark.svg`
    Bookmark,
    /// `assets/icons/briefcase.svg`
    Briefcase,
    /// `assets/icons/camera.svg`
    Camera,
    /// `assets/icons/chart.svg`
    Chart,
    /// `assets/icons/chat.svg`
    Chat,
    /// `assets/icons/check.svg`
    Check,
    /// `assets/icons/chevron_down.svg`
    ChevronDown,
    /// `assets/icons/chevron_right.svg`
    ChevronRight,
    /// `assets/icons/close.svg`
    Close,
    /// `assets/icons/code.svg`
    Code,
    /// `assets/icons/copy.svg`
    Copy,
    /// `assets/icons/cursor.svg`
    Cursor,
    /// `assets/icons/cut.svg`
    Cut,
    /// `assets/icons/download.svg`
    Download,
    /// `assets/icons/external.svg`
    External,
    /// `assets/icons/eye.svg`
    Eye,
    /// `assets/icons/eye_off.svg`
    EyeOff,
    /// `assets/icons/file.svg`
    File,
    /// `assets/icons/folder.svg`
    Folder,
    /// `assets/icons/folder_plus.svg`
    FolderPlus,
    /// `assets/icons/forward.svg`
    Forward,
    /// `assets/icons/globe.svg`
    Globe,
    /// `assets/icons/history.svg`
    History,
    /// `assets/icons/home.svg`
    Home,
    /// `assets/icons/hourglass.svg`
    Hourglass,
    /// `assets/icons/image.svg`
    Image,
    /// `assets/icons/key.svg`
    Key,
    /// `assets/icons/keyboard.svg`
    Keyboard,
    /// `assets/icons/link.svg`
    Link,
    /// `assets/icons/lock.svg`
    Lock,
    /// `assets/icons/more.svg`
    More,
    /// `assets/icons/new_chat.svg`
    NewChat,
    /// `assets/icons/orbit.svg`
    Orbit,
    /// `assets/icons/page.svg`
    Page,
    /// `assets/icons/palette.svg`
    Palette,
    /// `assets/icons/paperclip.svg`
    Paperclip,
    /// `assets/icons/paste.svg`
    Paste,
    /// `assets/icons/pencil.svg`
    Pencil,
    /// `assets/icons/plus.svg`
    Plus,
    /// `assets/icons/reload.svg`
    Reload,
    /// `assets/icons/scroll.svg`
    Scroll,
    /// `assets/icons/search.svg`
    Search,
    /// `assets/icons/select_all.svg`
    SelectAll,
    /// `assets/icons/send.svg`
    Send,
    /// `assets/icons/settings.svg`
    Settings,
    /// `assets/icons/shield.svg`
    Shield,
    /// `assets/icons/sliders.svg`
    Sliders,
    /// `assets/icons/sparkle.svg`
    Sparkle,
    /// `assets/icons/star.svg`
    Star,
    /// `assets/icons/star_filled.svg`
    StarFilled,
    /// `assets/icons/stop.svg`
    Stop,
    /// `assets/icons/stop_load.svg`
    StopLoad,
    /// `assets/icons/swap.svg`
    Swap,
    /// `assets/icons/tabs.svg`
    Tabs,
    /// `assets/icons/terminal.svg`
    Terminal,
    /// `assets/icons/trash.svg`
    Trash,
    /// `assets/icons/unlock.svg`
    Unlock,
    /// `assets/icons/upload.svg`
    Upload,
    /// `assets/icons/win_close.svg`
    WinClose,
    /// `assets/icons/win_max.svg`
    WinMax,
    /// `assets/icons/win_min.svg`
    WinMin,
    /// `assets/icons/win_restore.svg`
    WinRestore,
    /// `assets/icons/wrench.svg`
    Wrench,
    /// `assets/icons/x_logo.svg`
    XLogo,
    /// `assets/icons/zoom_in.svg`
    ZoomIn,
    /// `assets/icons/zoom_out.svg`
    ZoomOut,
}

impl Icon {
    /// Every built-in icon, in alphabetical order.
    pub const ALL: &'static [Icon] = &[
        Icon::Alert,
        Icon::Back,
        Icon::Bell,
        Icon::Book,
        Icon::Bookmark,
        Icon::Briefcase,
        Icon::Camera,
        Icon::Chart,
        Icon::Chat,
        Icon::Check,
        Icon::ChevronDown,
        Icon::ChevronRight,
        Icon::Close,
        Icon::Code,
        Icon::Copy,
        Icon::Cursor,
        Icon::Cut,
        Icon::Download,
        Icon::External,
        Icon::Eye,
        Icon::EyeOff,
        Icon::File,
        Icon::Folder,
        Icon::FolderPlus,
        Icon::Forward,
        Icon::Globe,
        Icon::History,
        Icon::Home,
        Icon::Hourglass,
        Icon::Image,
        Icon::Key,
        Icon::Keyboard,
        Icon::Link,
        Icon::Lock,
        Icon::More,
        Icon::NewChat,
        Icon::Orbit,
        Icon::Page,
        Icon::Palette,
        Icon::Paperclip,
        Icon::Paste,
        Icon::Pencil,
        Icon::Plus,
        Icon::Reload,
        Icon::Scroll,
        Icon::Search,
        Icon::SelectAll,
        Icon::Send,
        Icon::Settings,
        Icon::Shield,
        Icon::Sliders,
        Icon::Sparkle,
        Icon::Star,
        Icon::StarFilled,
        Icon::Stop,
        Icon::StopLoad,
        Icon::Swap,
        Icon::Tabs,
        Icon::Terminal,
        Icon::Trash,
        Icon::Unlock,
        Icon::Upload,
        Icon::WinClose,
        Icon::WinMax,
        Icon::WinMin,
        Icon::WinRestore,
        Icon::Wrench,
        Icon::XLogo,
        Icon::ZoomIn,
        Icon::ZoomOut,
    ];

    /// The icon's file name in `assets/icons`, without `.svg`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Icon::Alert => "alert",
            Icon::Back => "back",
            Icon::Bell => "bell",
            Icon::Book => "book",
            Icon::Bookmark => "bookmark",
            Icon::Briefcase => "briefcase",
            Icon::Camera => "camera",
            Icon::Chart => "chart",
            Icon::Chat => "chat",
            Icon::Check => "check",
            Icon::ChevronDown => "chevron_down",
            Icon::ChevronRight => "chevron_right",
            Icon::Close => "close",
            Icon::Code => "code",
            Icon::Copy => "copy",
            Icon::Cursor => "cursor",
            Icon::Cut => "cut",
            Icon::Download => "download",
            Icon::External => "external",
            Icon::Eye => "eye",
            Icon::EyeOff => "eye_off",
            Icon::File => "file",
            Icon::Folder => "folder",
            Icon::FolderPlus => "folder_plus",
            Icon::Forward => "forward",
            Icon::Globe => "globe",
            Icon::History => "history",
            Icon::Home => "home",
            Icon::Hourglass => "hourglass",
            Icon::Image => "image",
            Icon::Key => "key",
            Icon::Keyboard => "keyboard",
            Icon::Link => "link",
            Icon::Lock => "lock",
            Icon::More => "more",
            Icon::NewChat => "new_chat",
            Icon::Orbit => "orbit",
            Icon::Page => "page",
            Icon::Palette => "palette",
            Icon::Paperclip => "paperclip",
            Icon::Paste => "paste",
            Icon::Pencil => "pencil",
            Icon::Plus => "plus",
            Icon::Reload => "reload",
            Icon::Scroll => "scroll",
            Icon::Search => "search",
            Icon::SelectAll => "select_all",
            Icon::Send => "send",
            Icon::Settings => "settings",
            Icon::Shield => "shield",
            Icon::Sliders => "sliders",
            Icon::Sparkle => "sparkle",
            Icon::Star => "star",
            Icon::StarFilled => "star_filled",
            Icon::Stop => "stop",
            Icon::StopLoad => "stop_load",
            Icon::Swap => "swap",
            Icon::Tabs => "tabs",
            Icon::Terminal => "terminal",
            Icon::Trash => "trash",
            Icon::Unlock => "unlock",
            Icon::Upload => "upload",
            Icon::WinClose => "win_close",
            Icon::WinMax => "win_max",
            Icon::WinMin => "win_min",
            Icon::WinRestore => "win_restore",
            Icon::Wrench => "wrench",
            Icon::XLogo => "x_logo",
            Icon::ZoomIn => "zoom_in",
            Icon::ZoomOut => "zoom_out",
        }
    }

    /// The icon's SVG, for painting at any size or as a window icon.
    #[must_use]
    pub const fn svg(self) -> Svg {
        match self {
            Icon::Alert => Svg::new("spaceui/alert", include_str!("../../assets/icons/alert.svg")),
            Icon::Back => Svg::new("spaceui/back", include_str!("../../assets/icons/back.svg")),
            Icon::Bell => Svg::new("spaceui/bell", include_str!("../../assets/icons/bell.svg")),
            Icon::Book => Svg::new("spaceui/book", include_str!("../../assets/icons/book.svg")),
            Icon::Bookmark => Svg::new("spaceui/bookmark", include_str!("../../assets/icons/bookmark.svg")),
            Icon::Briefcase => Svg::new("spaceui/briefcase", include_str!("../../assets/icons/briefcase.svg")),
            Icon::Camera => Svg::new("spaceui/camera", include_str!("../../assets/icons/camera.svg")),
            Icon::Chart => Svg::new("spaceui/chart", include_str!("../../assets/icons/chart.svg")),
            Icon::Chat => Svg::new("spaceui/chat", include_str!("../../assets/icons/chat.svg")),
            Icon::Check => Svg::new("spaceui/check", include_str!("../../assets/icons/check.svg")),
            Icon::ChevronDown => Svg::new("spaceui/chevron_down", include_str!("../../assets/icons/chevron_down.svg")),
            Icon::ChevronRight => {
                Svg::new("spaceui/chevron_right", include_str!("../../assets/icons/chevron_right.svg"))
            }
            Icon::Close => Svg::new("spaceui/close", include_str!("../../assets/icons/close.svg")),
            Icon::Code => Svg::new("spaceui/code", include_str!("../../assets/icons/code.svg")),
            Icon::Copy => Svg::new("spaceui/copy", include_str!("../../assets/icons/copy.svg")),
            Icon::Cursor => Svg::new("spaceui/cursor", include_str!("../../assets/icons/cursor.svg")),
            Icon::Cut => Svg::new("spaceui/cut", include_str!("../../assets/icons/cut.svg")),
            Icon::Download => Svg::new("spaceui/download", include_str!("../../assets/icons/download.svg")),
            Icon::External => Svg::new("spaceui/external", include_str!("../../assets/icons/external.svg")),
            Icon::Eye => Svg::new("spaceui/eye", include_str!("../../assets/icons/eye.svg")),
            Icon::EyeOff => Svg::new("spaceui/eye_off", include_str!("../../assets/icons/eye_off.svg")),
            Icon::File => Svg::new("spaceui/file", include_str!("../../assets/icons/file.svg")),
            Icon::Folder => Svg::new("spaceui/folder", include_str!("../../assets/icons/folder.svg")),
            Icon::FolderPlus => Svg::new("spaceui/folder_plus", include_str!("../../assets/icons/folder_plus.svg")),
            Icon::Forward => Svg::new("spaceui/forward", include_str!("../../assets/icons/forward.svg")),
            Icon::Globe => Svg::new("spaceui/globe", include_str!("../../assets/icons/globe.svg")),
            Icon::History => Svg::new("spaceui/history", include_str!("../../assets/icons/history.svg")),
            Icon::Home => Svg::new("spaceui/home", include_str!("../../assets/icons/home.svg")),
            Icon::Hourglass => Svg::new("spaceui/hourglass", include_str!("../../assets/icons/hourglass.svg")),
            Icon::Image => Svg::new("spaceui/image", include_str!("../../assets/icons/image.svg")),
            Icon::Key => Svg::new("spaceui/key", include_str!("../../assets/icons/key.svg")),
            Icon::Keyboard => Svg::new("spaceui/keyboard", include_str!("../../assets/icons/keyboard.svg")),
            Icon::Link => Svg::new("spaceui/link", include_str!("../../assets/icons/link.svg")),
            Icon::Lock => Svg::new("spaceui/lock", include_str!("../../assets/icons/lock.svg")),
            Icon::More => Svg::new("spaceui/more", include_str!("../../assets/icons/more.svg")),
            Icon::NewChat => Svg::new("spaceui/new_chat", include_str!("../../assets/icons/new_chat.svg")),
            Icon::Orbit => Svg::new("spaceui/orbit", include_str!("../../assets/icons/orbit.svg")),
            Icon::Page => Svg::new("spaceui/page", include_str!("../../assets/icons/page.svg")),
            Icon::Palette => Svg::new("spaceui/palette", include_str!("../../assets/icons/palette.svg")),
            Icon::Paperclip => Svg::new("spaceui/paperclip", include_str!("../../assets/icons/paperclip.svg")),
            Icon::Paste => Svg::new("spaceui/paste", include_str!("../../assets/icons/paste.svg")),
            Icon::Pencil => Svg::new("spaceui/pencil", include_str!("../../assets/icons/pencil.svg")),
            Icon::Plus => Svg::new("spaceui/plus", include_str!("../../assets/icons/plus.svg")),
            Icon::Reload => Svg::new("spaceui/reload", include_str!("../../assets/icons/reload.svg")),
            Icon::Scroll => Svg::new("spaceui/scroll", include_str!("../../assets/icons/scroll.svg")),
            Icon::Search => Svg::new("spaceui/search", include_str!("../../assets/icons/search.svg")),
            Icon::SelectAll => Svg::new("spaceui/select_all", include_str!("../../assets/icons/select_all.svg")),
            Icon::Send => Svg::new("spaceui/send", include_str!("../../assets/icons/send.svg")),
            Icon::Settings => Svg::new("spaceui/settings", include_str!("../../assets/icons/settings.svg")),
            Icon::Shield => Svg::new("spaceui/shield", include_str!("../../assets/icons/shield.svg")),
            Icon::Sliders => Svg::new("spaceui/sliders", include_str!("../../assets/icons/sliders.svg")),
            Icon::Sparkle => Svg::new("spaceui/sparkle", include_str!("../../assets/icons/sparkle.svg")),
            Icon::Star => Svg::new("spaceui/star", include_str!("../../assets/icons/star.svg")),
            Icon::StarFilled => Svg::new("spaceui/star_filled", include_str!("../../assets/icons/star_filled.svg")),
            Icon::Stop => Svg::new("spaceui/stop", include_str!("../../assets/icons/stop.svg")),
            Icon::StopLoad => Svg::new("spaceui/stop_load", include_str!("../../assets/icons/stop_load.svg")),
            Icon::Swap => Svg::new("spaceui/swap", include_str!("../../assets/icons/swap.svg")),
            Icon::Tabs => Svg::new("spaceui/tabs", include_str!("../../assets/icons/tabs.svg")),
            Icon::Terminal => Svg::new("spaceui/terminal", include_str!("../../assets/icons/terminal.svg")),
            Icon::Trash => Svg::new("spaceui/trash", include_str!("../../assets/icons/trash.svg")),
            Icon::Unlock => Svg::new("spaceui/unlock", include_str!("../../assets/icons/unlock.svg")),
            Icon::Upload => Svg::new("spaceui/upload", include_str!("../../assets/icons/upload.svg")),
            Icon::WinClose => Svg::new("spaceui/win_close", include_str!("../../assets/icons/win_close.svg")),
            Icon::WinMax => Svg::new("spaceui/win_max", include_str!("../../assets/icons/win_max.svg")),
            Icon::WinMin => Svg::new("spaceui/win_min", include_str!("../../assets/icons/win_min.svg")),
            Icon::WinRestore => Svg::new("spaceui/win_restore", include_str!("../../assets/icons/win_restore.svg")),
            Icon::Wrench => Svg::new("spaceui/wrench", include_str!("../../assets/icons/wrench.svg")),
            Icon::XLogo => Svg::new("spaceui/x_logo", include_str!("../../assets/icons/x_logo.svg")),
            Icon::ZoomIn => Svg::new("spaceui/zoom_in", include_str!("../../assets/icons/zoom_in.svg")),
            Icon::ZoomOut => Svg::new("spaceui/zoom_out", include_str!("../../assets/icons/zoom_out.svg")),
        }
    }

    /// Paints the icon tinted with `color`, filling `rect`.
    pub fn paint(self, painter: &Painter, rect: Rect, color: Color32) {
        self.svg().paint(painter, rect, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_set_matches_the_asset_folder() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/icons");
        let mut files: Vec<String> = std::fs::read_dir(dir)
            .expect("assets/icons")
            .filter_map(|e| e.ok()?.file_name().to_str()?.strip_suffix(".svg").map(str::to_owned))
            .collect();
        files.sort();
        let listed: Vec<&str> = Icon::ALL.iter().map(|i| i.name()).collect();
        assert_eq!(listed, files);
    }

    #[test]
    fn every_icon_renders_something() {
        for icon in Icon::ALL {
            let image = icon.svg().rasterize(24).unwrap_or_else(|| panic!("{icon:?} does not parse"));
            assert!(image.pixels.iter().any(|p| p.a() > 0), "{icon:?} is blank");
        }
    }
}
