//! Segoe UI (regular and semibold) with Segoe's symbol and emoji fonts as
//! fallbacks, read from the system at runtime and never redistributed.
//! Elsewhere egui's bundled fonts stand in under the same family names.
use egui::{Context, FontData, FontDefinitions, FontFamily, FontId, Id};
use std::{path::Path, sync::Arc};

/// The family name [`semibold`] uses.
pub const SEMIBOLD: &str = "semibold";

/// Segoe UI (or egui's sans) at `size` points.
#[must_use]
pub fn regular(size: f32) -> FontId {
    FontId::proportional(size)
}

/// Segoe UI Semibold (or the closest available) at `size` points.
#[must_use]
pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}

/// The monospace font at `size` points.
#[must_use]
pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

/// Installs the fonts once per context; later calls do nothing.
pub(super) fn install(ctx: &Context) {
    let done = Id::unique("spaceui-fonts-installed");
    if ctx.data(|d| d.get_temp::<bool>(done)).unwrap_or(false) {
        return;
    }
    let dir = std::env::var("WINDIR")
        .map_or_else(|_| "/usr/share/fonts".into(), |w| std::path::PathBuf::from(w).join("Fonts"));
    ctx.set_fonts(definitions(&dir));
    ctx.data_mut(|d| d.insert_temp(done, true));
}

/// Font definitions with Segoe from `dir` where present. The semibold family
/// always exists, so [`semibold`] never panics.
fn definitions(dir: &Path) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let defaults = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mut load = |key: &str, file: &str| match std::fs::read(dir.join(file)) {
        Ok(bytes) => {
            fonts.font_data.insert(key.into(), Arc::new(FontData::from_owned(bytes)));
            true
        }
        Err(_) => false,
    };
    let regular = load("segoe", "segoeui.ttf");
    let semibold = load("segoe-sb", "seguisb.ttf");
    let symbols = load("segoe-sym", "seguisym.ttf");
    let emoji = load("segoe-emoji", "seguiemj.ttf");

    let mut semibold_family = Vec::new();
    if semibold {
        semibold_family.push("segoe-sb".to_owned());
    }
    if regular {
        fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "segoe".into());
        semibold_family.push("segoe".into());
    }
    semibold_family.extend(defaults);
    for (present, key) in [(symbols, "segoe-sym"), (emoji, "segoe-emoji")] {
        if present {
            for family in [FontFamily::Proportional, FontFamily::Monospace] {
                fonts.families.entry(family).or_default().push(key.into());
            }
            semibold_family.push(key.into());
        }
    }
    fonts.families.insert(FontFamily::Name(SEMIBOLD.into()), semibold_family);
    fonts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_semibold_family_exists_without_system_fonts() {
        let fonts = definitions(Path::new("no such folder"));
        let family = &fonts.families[&FontFamily::Name(SEMIBOLD.into())];
        assert_ne!(family.as_slice(), [] as [String; 0], "the family has fonts");
        assert!(family.iter().all(|key| fonts.font_data.contains_key(key)));
    }
}
