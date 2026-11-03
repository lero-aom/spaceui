//! A theme's fonts: the system's Segoe UI family, egui's bundled fonts, or
//! the app's own files. System fonts are read at runtime, never bundled.
use egui::{Context, FontData, FontDefinitions, FontFamily, FontId, Id};
use std::{borrow::Cow, fmt, path::Path, sync::Arc};

#[derive(Clone)]
enum FontBytes {
    Static(&'static [u8]),
    Shared(egui::epaint::text::Blob),
}

impl FontBytes {
    fn as_slice(&self) -> &[u8] {
        match self {
            Self::Static(bytes) => bytes,
            Self::Shared(bytes) => bytes.as_ref().as_ref(),
        }
    }
}

impl PartialEq for FontBytes {
    fn eq(&self, other: &Self) -> bool {
        let (left, right) = (self.as_slice(), other.as_slice());
        std::ptr::eq(left, right) || left == right
    }
}

impl Eq for FontBytes {}

/// The family name [`semibold`] uses.
pub const SEMIBOLD: &str = "semibold";

/// The theme's regular font at `size` points.
#[must_use]
pub fn regular(size: f32) -> FontId {
    FontId::proportional(size)
}

/// The theme's semibold font at `size` points (the regular one where a theme
/// has no semibold).
#[must_use]
pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}

/// The theme's monospace font at `size` points.
#[must_use]
pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

/// A font file, embedded or loaded at runtime. Clones share the font bytes.
///
/// Bytes must contain a valid TTF or OTF font. egui parses them lazily and
/// panics when rendering text if they are invalid.
///
/// ```
/// use spaceui::theme::FontFile;
///
/// # const INTER_REGULAR: &[u8] = &[];
/// // Usually `FontFile::from_static("Inter", include_bytes!("Inter-Regular.ttf"))`:
/// const INTER: FontFile = FontFile::from_static("Inter", INTER_REGULAR);
/// // Or read at runtime, as from `std::fs::read(path)?`:
/// let semibold = FontFile::from_owned("Inter Semibold", Vec::new());
/// assert_eq!((INTER.name(), semibold.name()), ("Inter", "Inter Semibold"));
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct FontFile {
    name: Cow<'static, str>,
    bytes: FontBytes,
}

impl FontFile {
    /// A font compiled into the app, usually with `include_bytes!`. The name
    /// identifies it in diagnostics; different font roles may share a name.
    #[must_use]
    pub const fn from_static(name: &'static str, bytes: &'static [u8]) -> Self {
        Self { name: Cow::Borrowed(name), bytes: FontBytes::Static(bytes) }
    }

    /// A font read at runtime.
    #[must_use]
    pub fn from_owned(name: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self { name: Cow::Owned(name.into()), bytes: FontBytes::Shared(Arc::new(bytes)) }
    }

    /// The name it was given.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    fn data(&self) -> FontData {
        match &self.bytes {
            FontBytes::Static(bytes) => FontData::from_static(bytes),
            FontBytes::Shared(bytes) => FontData::from_blob(Arc::clone(bytes), 0),
        }
    }
}

impl fmt::Debug for FontFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FontFile").field("name", &self.name).field("bytes", &self.bytes.as_slice().len()).finish()
    }
}

/// Where a theme's fonts come from. Missing regular and semibold fonts fall
/// back to the system (when `system` is on), then to egui's bundled fonts.
/// A custom regular font also stands in for a missing custom semibold font.
/// Monospace text falls back to egui's bundled monospace font.
///
/// ```
/// use spaceui::theme::{FontFile, Fonts};
///
/// # const REGULAR: &[u8] = &[];
/// # const SEMIBOLD: &[u8] = &[];
/// let inter = Fonts {
///     regular: Some(FontFile::from_static("Inter", REGULAR)),
///     semibold: Some(FontFile::from_static("Inter Semibold", SEMIBOLD)),
///     ..Fonts::SYSTEM
/// };
/// assert!(inter.system, "Segoe's symbol and emoji fonts still fill in missing glyphs on Windows");
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fonts {
    /// Body text.
    pub regular: Option<FontFile>,
    /// Headings and emphasis; the regular font stands in when there is none.
    pub semibold: Option<FontFile>,
    /// Code and figures.
    pub monospace: Option<FontFile>,
    /// Read Segoe UI, Segoe UI Semibold, Symbol and Emoji from the Windows
    /// font folder where present.
    pub system: bool,
}

impl Fonts {
    /// Segoe UI from the system on Windows, egui's bundled fonts elsewhere.
    pub const SYSTEM: Fonts = Fonts { regular: None, semibold: None, monospace: None, system: true };

    /// egui's bundled fonts only, on every system.
    pub const BUNDLED: Fonts = Fonts { regular: None, semibold: None, monospace: None, system: false };

    /// egui font definitions for these fonts, reading system fonts from `dir`.
    /// The semibold family always exists, with bundled fonts as a fallback.
    fn definitions(&self, dir: &Path) -> FontDefinitions {
        let mut fonts = FontDefinitions::default();
        let bundled_sans = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
        let bundled_mono = fonts.families.get(&FontFamily::Monospace).cloned().unwrap_or_default();
        let mut add = |key: String, data: FontData| {
            fonts.font_data.insert(key.clone(), Arc::new(data));
            key
        };
        let custom = |role: &str, file: &Option<FontFile>, add: &mut dyn FnMut(String, FontData) -> String| {
            file.as_ref().map(|file| add(format!("spaceui-{role}-{}", file.name), file.data()))
        };
        let regular = custom("regular", &self.regular, &mut add);
        let bold = custom("semibold", &self.semibold, &mut add);
        let mono = custom("monospace", &self.monospace, &mut add);
        let system = |file: &str, add: &mut dyn FnMut(String, FontData) -> String| {
            if !self.system {
                return None;
            }
            let bytes = std::fs::read(dir.join(file)).ok()?;
            Some(add(format!("system-{file}"), FontData::from_owned(bytes)))
        };
        let segoe = system("segoeui.ttf", &mut add);
        let segoe_bold = system("seguisb.ttf", &mut add);
        let fallbacks: Vec<String> =
            ["seguisym.ttf", "seguiemj.ttf"].into_iter().filter_map(|file| system(file, &mut add)).collect();

        let sans: Vec<String> =
            [regular.clone(), segoe.clone()].into_iter().flatten().chain(bundled_sans.clone()).collect();
        let semibold: Vec<String> =
            [bold, regular, segoe_bold, segoe].into_iter().flatten().chain(bundled_sans).collect();
        let monospace: Vec<String> = mono.into_iter().chain(bundled_mono).collect();
        for (family, list) in [
            (FontFamily::Proportional, sans),
            (FontFamily::Name(SEMIBOLD.into()), semibold),
            (FontFamily::Monospace, monospace),
        ] {
            fonts.families.insert(family, list.into_iter().chain(fallbacks.iter().cloned()).collect());
        }
        fonts
    }
}

impl Default for Fonts {
    fn default() -> Self {
        Self::SYSTEM
    }
}

/// Installs `fonts` unless they are already the context's fonts.
pub(super) fn install(ctx: &Context, fonts: &Fonts) {
    let installed = Id::unique("spaceui-fonts-installed");
    if ctx.data(|d| d.get_temp::<Arc<Fonts>>(installed)).as_deref() == Some(fonts) {
        return;
    }
    let dir = std::env::var("WINDIR")
        .map_or_else(|_| "/usr/share/fonts".into(), |windows| std::path::PathBuf::from(windows).join("Fonts"));
    ctx.set_fonts(fonts.definitions(&dir));
    ctx.data_mut(|d| d.insert_temp(installed, Arc::new(fonts.clone())));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn family<'a>(fonts: &'a FontDefinitions, family: &FontFamily) -> &'a [String] {
        &fonts.families[family]
    }

    #[test]
    fn every_family_exists_without_system_fonts() {
        let fonts = Fonts::SYSTEM.definitions(Path::new("no such folder"));
        for name in [FontFamily::Proportional, FontFamily::Monospace, FontFamily::Name(SEMIBOLD.into())] {
            let keys = family(&fonts, &name);
            assert_ne!(keys, [] as [String; 0], "{name:?} has fonts");
            assert!(keys.iter().all(|key| fonts.font_data.contains_key(key)));
        }
    }

    #[test]
    fn custom_fonts_come_first_and_regular_stands_in_for_semibold() {
        let custom = Fonts {
            regular: Some(FontFile::from_static("Body", b"not parsed until used")),
            monospace: Some(FontFile::from_owned("Code", b"also not parsed".to_vec())),
            ..Fonts::BUNDLED
        };
        let fonts = custom.definitions(Path::new("no such folder"));
        assert_eq!(family(&fonts, &FontFamily::Proportional)[0], "spaceui-regular-Body");
        assert_eq!(family(&fonts, &FontFamily::Name(SEMIBOLD.into()))[0], "spaceui-regular-Body");
        assert_eq!(family(&fonts, &FontFamily::Monospace)[0], "spaceui-monospace-Code");
    }

    #[test]
    fn fonts_with_the_same_name_keep_their_distinct_role_data() {
        let custom = Fonts {
            regular: Some(FontFile::from_static("Example", b"regular")),
            semibold: Some(FontFile::from_static("Example", b"semibold")),
            monospace: Some(FontFile::from_static("Example", b"monospace")),
            ..Fonts::BUNDLED
        };
        let fonts = custom.definitions(Path::new("no such folder"));
        for (name, expected) in [
            (FontFamily::Proportional, b"regular".as_slice()),
            (FontFamily::Name(SEMIBOLD.into()), b"semibold".as_slice()),
            (FontFamily::Monospace, b"monospace".as_slice()),
        ] {
            let key = &family(&fonts, &name)[0];
            assert_eq!(fonts.font_data[key].bytes(), expected);
        }
    }

    #[test]
    fn a_custom_regular_font_precedes_system_semibold() -> std::io::Result<()> {
        let dir = std::env::temp_dir().join(format!("spaceui-font-fallback-{}", std::process::id()));
        std::fs::create_dir(&dir)?;
        let path = dir.join("seguisb.ttf");
        std::fs::write(&path, b"system semibold")?;
        let fonts = Fonts { regular: Some(FontFile::from_static("Custom", b"custom regular")), ..Fonts::SYSTEM }
            .definitions(&dir);
        std::fs::remove_file(path)?;
        std::fs::remove_dir(dir)?;
        let semibold = family(&fonts, &FontFamily::Name(SEMIBOLD.into()));
        assert_eq!(semibold[0], "spaceui-regular-Custom");
        assert_eq!(semibold[1], "system-seguisb.ttf");
        Ok(())
    }

    #[test]
    fn cloning_and_installing_owned_fonts_share_their_bytes() {
        let font = FontFile::from_owned("Owned", vec![1, 2, 3]);
        let data = font.data();
        assert!(Arc::ptr_eq(&data.font, &font.clone().data().font));
        let fonts = Fonts { regular: Some(font), ..Fonts::BUNDLED };
        let definitions = fonts.definitions(Path::new("no such folder"));
        assert!(Arc::ptr_eq(&data.font, &definitions.font_data["spaceui-regular-Owned"].font));
        assert_eq!(fonts.regular, Some(FontFile::from_static("Owned", &[1, 2, 3])));
    }

    #[test]
    fn fonts_are_reinstalled_only_when_they_change() {
        let ctx = Context::default();
        install(&ctx, &Fonts::BUNDLED);
        let installed = Id::unique("spaceui-fonts-installed");
        let cached = ctx.data(|d| d.get_temp::<Arc<Fonts>>(installed)).unwrap();
        install(&ctx, &Fonts::BUNDLED);
        assert!(Arc::ptr_eq(&cached, &ctx.data(|d| d.get_temp::<Arc<Fonts>>(installed)).unwrap()));
        install(&ctx, &Fonts::SYSTEM);
        assert_eq!(ctx.data(|d| d.get_temp::<Arc<Fonts>>(installed)).as_deref(), Some(&Fonts::SYSTEM));
    }
}
