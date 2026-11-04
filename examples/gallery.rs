//! An astronomy workbench demonstrating SpaceUI's shell, themes, widgets and
//! chat. Sky positions, conditions and assistant replies are illustrative demo data.
//!
//! ```text
//! cargo run --example gallery
//! ```
//!
//! For a screenshot run, set `SPACEUI_SCREENSHOT=out.png`; `SPACEUI_THEME=carbon_copper`
//! picks the alternate theme, `SPACEUI_CHAT=left` puts the chat on the left and
//! `SPACEUI_PAGE=0..=3` picks the page.
use spaceui::chat::{self, ChatLayout, Composer, Tab, TabAction, transcript};
use spaceui::egui::{self, Color32, RichText, ScrollArea, UiBuilder, vec2};
use spaceui::prelude::*;

#[path = "gallery/sky.rs"]
mod sky;

const LOGO: Svg = Svg::new("gallery/spaceui", include_str!("assets/spaceui.svg"));
const HINT: &str = "Ask about targets or your observing plan…";
const PAGES: [&str; 4] = ["Tonight's sky", "Observatory", "Palette and icons", "Preferences"];
const RAIL: &[RailItem] = &[
    RailItem::page(Icon::Orbit, PAGES[0]),
    RailItem::page(Icon::Camera, PAGES[1]),
    RailItem::page(Icon::Palette, PAGES[2]),
    RailItem::Divider,
    RailItem::page(Icon::Settings, PAGES[3]),
];
const EFFORTS: [&str; 5] = ["Low", "Medium", "High", "Extra high", "Max"];
const EXPOSURES: [&str; 5] = ["15 s", "30 s", "60 s", "120 s", "300 s"];
const TARGETS: [(&str, &str, &str, &str); 4] = [
    ("M31", "Andromeda Galaxy", "Galaxy · Andromeda", "60 × 120 s · LRGB"),
    ("M45", "The Pleiades", "Open cluster · Taurus", "40 × 90 s · RGB"),
    ("NGC 869 / 884", "Double Cluster", "Open clusters · Perseus", "30 × 60 s · RGB"),
    ("M33", "Triangulum Galaxy", "Galaxy · Triangulum", "50 × 180 s · LRGB"),
];

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: window::viewport("Observatory · SpaceUI", "spaceui-gallery")
            .with_icon(LOGO.window_icon(256).unwrap_or_default()),
        ..Default::default()
    };
    eframe::run_native("Observatory · SpaceUI", options, Box::new(|cc| Ok(Box::new(Gallery::new(cc)))))
}

/// One transcript entry of the demo chat.
enum Entry {
    User(String),
    Reply { text: String, thinking: String, steps: Vec<(&'static str, &'static str)>, open: bool },
    Notice(String),
}

struct Conversation {
    id: u64,
    title: String,
    entries: Vec<Entry>,
    draft: String,
}

impl Conversation {
    fn new(id: u64) -> Self {
        Self { id, title: "New session".into(), entries: Vec::new(), draft: String::new() }
    }

    /// The canned reply: a Markdown echo of the draft.
    fn send(&mut self) {
        let text = std::mem::take(&mut self.draft).trim().to_owned();
        if self.entries.is_empty() {
            self.title = text.chars().take(28).collect();
        }
        let reply = format!(
            "Your observing note:\n\n> {text}\n\nChoose a target on the sky map, then add it to your plan. \
             The sample sequence includes an exposure count, duration and filter set."
        );
        self.entries.push(Entry::User(text));
        self.entries.push(Entry::Reply { text: reply, thinking: String::new(), steps: Vec::new(), open: false });
        self.entries.push(Entry::Notice("Demo assistant · no telescope connection.".into()));
    }
}

struct Gallery {
    page: usize,
    side: f32,
    /// Where the chat column sits: 0 is left, 1 is right.
    chat_side: usize,
    deep_space: bool,
    installed: Option<bool>,
    chats: Vec<Conversation>,
    active: usize,
    next_id: u64,
    focus: bool,
    tab: usize,
    timeframe: usize,
    effort: usize,
    exposure: usize,
    capture_mode: usize,
    filter: usize,
    switch: bool,
    search: String,
    target: usize,
    queued: [bool; 4],
    screenshot: Option<window::Screenshot>,
}

impl Gallery {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let deep_space = !std::env::var("SPACEUI_THEME").is_ok_and(|theme| theme == "carbon_copper");
        let border = if deep_space { Palette::DEEP_SPACE.border } else { Palette::CARBON_COPPER.border };
        window::round_corners(cc, border);
        let mut welcome = Conversation::new(1);
        welcome.title = "Evening plan".into();
        welcome.entries = vec![
            Entry::User("Help me plan an evening of deep-sky imaging.".into()),
            Entry::Reply {
                text: "**Start with Andromeda.** Its wide field makes a lovely opening target for this sample session.\n\n\
                       ### Your observing plan\n\
                       - **M31** · 60 × 120 s, LRGB\n\
                       - **Double Cluster** · 30 × 60 s, RGB\n\
                       - **M45** · 40 × 90 s, RGB\n\n\
                       Keep a short focus check between targets. Leave a little time for a final dark-frame sequence.\n\n\
                       > Select a marker on the sky map to explore a target and edit your plan."
                    .into(),
                thinking: "Compared the sample targets and grouped their exposure sequences into an evening plan.".into(),
                steps: vec![("Reviewed target list", "4 deep-sky objects"), ("Prepared a session", "3 imaging sequences")],
                open: false,
            },
        ];
        Self {
            page: std::env::var("SPACEUI_PAGE").ok().and_then(|page| page.parse().ok()).unwrap_or(0),
            side: 380.0,
            chat_side: usize::from(!std::env::var("SPACEUI_CHAT").is_ok_and(|side| side == "left")),
            deep_space,
            installed: None,
            chats: vec![welcome, Conversation::new(2)],
            active: 0,
            next_id: 3,
            focus: false,
            tab: 0,
            timeframe: 0,
            effort: 3,
            exposure: 3,
            capture_mode: 0,
            filter: 0,
            switch: true,
            search: String::new(),
            target: 0,
            queued: [true, true, true, false],
            screenshot: window::Screenshot::from_env("SPACEUI_SCREENSHOT"),
        }
    }

    fn page(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(PAGES[self.page.min(3)]).font(semibold(26.0)));
        widgets::caption(ui, "A quiet place to explore the sky and plan your next observation.");
        ui.add_space(12.0);
        match self.page {
            0 => self.overview(ui),
            1 => self.controls(ui),
            2 => palette_and_icons(ui),
            _ => self.settings(ui),
        }
    }

    fn overview(&mut self, ui: &mut egui::Ui) {
        ui.columns(4, |columns| {
            widgets::metric(&mut columns[0], "SKY QUALITY", "21.4", "mag/arcsec² · dark skies", Tone::Neutral);
            widgets::metric(&mut columns[1], "SEEING", "1.2″", "Steady · sample conditions", Tone::Positive);
            widgets::metric(&mut columns[2], "MOON", "18%", "Illumination · demo session", Tone::Accent);
            let count = self.queued.iter().filter(|queued| **queued).count();
            widgets::metric(
                &mut columns[3],
                "OBSERVING PLAN",
                &format!("{count} targets"),
                "Ready for the night",
                Tone::Info,
            );
        });
        ui.add_space(12.0);
        widgets::tabs(ui, &mut self.tab, &["Sky map", "Target list", "Session notes"]);
        ui.add_space(8.0);
        if self.tab == 0 {
            widgets::card(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    widgets::section(ui, "01", "Explore the northern sky");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        widgets::segmented(ui, &mut self.timeframe, &["Wide", "Field", "Detail"]);
                    });
                });
                ui.add_space(6.0);
                sky::show(ui, &mut self.target, self.timeframe);
            });
            ui.add_space(10.0);
            self.target_card(ui);
        } else if self.tab == 1 {
            widgets::card(ui, |ui| {
                ui.set_width(ui.available_width());
                widgets::section(ui, "01", "Deep-sky targets");
                for (index, (catalogue, name, kind, exposure)) in TARGETS.iter().enumerate() {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.target == index, *catalogue).clicked() {
                            self.target = index;
                        }
                        ui.label(*name);
                        widgets::caption(ui, *kind);
                    });
                    widgets::caption(ui, *exposure);
                    ui.add_space(8.0);
                }
            });
            ui.add_space(10.0);
            self.target_card(ui);
        } else {
            widgets::card(ui, |ui| {
                ui.set_width(ui.available_width());
                widgets::section(ui, "01", "An evening under the stars");
                chat::markdown::render(
                    ui,
                    "### Before the first exposure\n- Check polar alignment and focus.\n- Start with **M31**, then revisit focus before moving to a star cluster.\n- Save calibration frames with the session.\n\nThese are sample observing notes; the sky map and conditions are illustrative.",
                );
            });
        }
    }

    fn target_card(&mut self, ui: &mut egui::Ui) {
        let (catalogue, name, kind, exposure) = TARGETS[self.target];
        widgets::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                widgets::section(ui, "02", name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    widgets::chip(ui, catalogue, Tone::Accent);
                });
            });
            widgets::caption(ui, format!("{kind}  ·  Sample sequence: {exposure}"));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let queued = &mut self.queued[self.target];
                if widgets::primary(ui, if *queued { "Remove from plan" } else { "Add to plan" }).clicked() {
                    *queued = !*queued;
                }
                if widgets::secondary(ui, "Ask about this target").clicked() {
                    self.chats[self.active].draft = format!("Help me plan an observation of {name} ({catalogue}).");
                    self.focus = true;
                }
                widgets::chip(ui, if *queued { "IN YOUR PLAN" } else { "AVAILABLE" }, Tone::Positive);
            });
        });
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            widgets::card(&mut columns[0], |ui| {
                ui.set_width(ui.available_width());
                widgets::section(ui, "01", "Mount and camera");
                ui.horizontal(|ui| {
                    if widgets::primary(ui, if self.switch { "Pause tracking" } else { "Start tracking" }).clicked() {
                        self.switch = !self.switch;
                    }
                    if widgets::secondary(ui, "Park mount").clicked() {
                        self.switch = false;
                    }
                    ui.add_enabled_ui(false, |ui| widgets::primary(ui, "Abort"));
                });
                ui.horizontal(|ui| {
                    for (icon, label) in [
                        (Icon::Search, "Find target"),
                        (Icon::Reload, "Refresh catalogue"),
                        (Icon::Star, "Favorite target"),
                        (Icon::Camera, "Camera settings"),
                    ] {
                        widgets::icon_button_labeled(ui, icon, 34.0, label);
                    }
                });
                ui.add_space(8.0);
                widgets::section(ui, "02", "Session conditions");
                ui.horizontal(|ui| {
                    widgets::badge(ui, Icon::Star, "Clear sky", Tone::Positive);
                    widgets::pill(ui, Some(Tone::Warning), "Dew watch");
                    widgets::pill(ui, None, "Sample night");
                });
                ui.horizontal(|ui| {
                    for (text, tone) in [
                        ("LRGB", Tone::Accent),
                        ("GUIDING", Tone::Positive),
                        ("CLOUD", Tone::Negative),
                        ("COOLING", Tone::Info),
                        ("IDLE", Tone::Muted),
                    ] {
                        widgets::chip(ui, text, tone);
                    }
                });
                ui.add_space(10.0);
                widgets::caption(ui, "Equipment controls use local demo state; no hardware is connected.");
            });
            widgets::card(&mut columns[1], |ui| {
                ui.set_width(ui.available_width());
                widgets::section(ui, "03", "Capture sequence");
                widgets::text_field(ui, &mut self.search, "Search nebulae, galaxies or clusters…")
                    .accessible_name("Target search");
                ui.horizontal(|ui| {
                    widgets::switch_labeled(ui, &mut self.switch, "Sidereal tracking");
                    ui.label(if self.switch { "Tracking on" } else { "Tracking paused" });
                });
                ui.horizontal(|ui| {
                    let label = ui.label("Exposure");
                    widgets::notches_labeled(ui, 160.0, EXPOSURES.len(), &mut self.exposure, "Exposure duration")
                        .labelled_by(label.id);
                    ui.label(EXPOSURES[self.exposure]);
                });
                ui.add_space(8.0);
                widgets::section(ui, "04", "Imaging workflow");
                widgets::tabs(ui, &mut self.capture_mode, &["Capture", "Calibration", "Review"]);
                widgets::segmented(ui, &mut self.filter, &["Luminance", "Red", "Green", "Blue"]);
                ui.add_space(8.0);
                widgets::caption(ui, "Choose a filter and exposure for your sample sequence.");
            });
        });
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        widgets::card(ui, |ui| {
            ui.set_width(ui.available_width());
            widgets::section(ui, "01", "Theme");
            let mut choice = usize::from(self.deep_space);
            widgets::segmented(ui, &mut choice, &["Carbon / Copper", "Deep Space"]);
            self.deep_space = choice == 1;
            widgets::caption(ui, "Installing a theme restyles every widget on the next frame.");
            ui.add_space(10.0);
            widgets::section(ui, "02", "Chat column");
            widgets::segmented(ui, &mut self.chat_side, &["Left", "Right"]);
            widgets::caption(
                ui,
                "The shell takes columns on either side, or both; the chat kit draws into any rectangle.",
            );
        });
    }

    fn chat_column(&mut self, ui: &mut egui::Ui, column: egui::Rect) {
        let logo = LOGO;
        let draft = &self.chats[self.active].draft;
        let layout = ChatLayout::new(column, Composer::measure(ui, column.width() - 24.0, draft, HINT));

        let tabs: Vec<Tab<'_>> = self
            .chats
            .iter()
            .map(|c| Tab { id: c.id, title: &c.title, busy: false, closable: self.chats.len() > 1 })
            .collect();
        match chat::tab_strip(ui, layout.strip, &tabs, self.active) {
            Some(TabAction::Select(i)) => self.active = i,
            Some(TabAction::Close(i)) => {
                self.chats.remove(i);
                self.active = self.active.min(self.chats.len() - 1);
            }
            Some(TabAction::New) => {
                self.chats.push(Conversation::new(self.next_id));
                self.next_id += 1;
                self.active = self.chats.len() - 1;
                self.focus = true;
            }
            None => {}
        }

        let chat = &mut self.chats[self.active];
        if chat.entries.is_empty() {
            let starters = ["Plan a deep-sky session", "Help me choose a target"];
            let hint = "Explore targets, plan exposures and keep observing notes. Demo replies stay on this device.";
            if let Some(i) =
                transcript::empty_state(ui, layout.body, Some(logo), "Where shall we look?", hint, &starters)
            {
                starters[i].clone_into(&mut chat.draft);
                chat.send();
            }
        } else {
            let mut body = ui.new_child(UiBuilder::new().max_rect(layout.body));
            ScrollArea::vertical()
                .id_salt(("gallery-chat", chat.id))
                .stick_to_bottom(true)
                .auto_shrink([false, false])
                .show(&mut body, |ui| {
                    ui.set_width(ui.available_width() - 10.0);
                    for entry in &mut chat.entries {
                        show_entry(ui, entry, logo);
                    }
                });
        }

        let mut effort = self.effort;
        let response = Composer::new(chat.id, &mut chat.draft).hint(HINT).focus(std::mem::take(&mut self.focus)).show(
            ui,
            layout.composer,
            |ui| {
                let model = widgets::dropdown(ui, Some(Tone::Positive), "Sky guide · Demo", 150.0)
                    .accessible_name("Assistant: Sky guide demo");
                egui::Popup::menu(&model).show(|ui| ui.label("Local demonstration · scripted observing notes"));
                let level = widgets::dropdown(ui, None, EFFORTS[effort], 100.0)
                    .accessible_name(format!("Reasoning effort: {}", EFFORTS[effort]));
                egui::Popup::menu(&level).show(|ui| {
                    for (i, name) in EFFORTS.iter().enumerate() {
                        if ui.selectable_label(i == effort, *name).clicked() {
                            effort = i;
                        }
                    }
                });
            },
        );
        self.effort = effort;
        if response.send {
            chat.send();
        }
    }
}

fn show_entry(ui: &mut egui::Ui, entry: &mut Entry, logo: Svg) {
    match entry {
        Entry::User(text) => transcript::user_bubble(ui, text, "now"),
        Entry::Reply { text, thinking, steps, open } => {
            transcript::byline(ui, Some(logo), "Sky guide · observing assistant");
            if transcript::thinking(ui, thinking, false, 2.0, *open) {
                *open = !*open;
            }
            for (i, (title, detail)) in steps.iter().enumerate() {
                let step = transcript::Step {
                    icon: Icon::Star,
                    title,
                    detail,
                    time: "21:30",
                    status: transcript::StepStatus::Done,
                };
                step.show(ui, i > 0, i + 1 < steps.len());
            }
            transcript::reply(ui, text, false);
        }
        Entry::Notice(text) => transcript::notice(ui, text, false),
    }
}

fn palette_and_icons(ui: &mut egui::Ui) {
    let p = Theme::of(ui.ctx()).palette;
    let roles = [
        ("panel", p.panel),
        ("panel_hi", p.panel_hi),
        ("field", p.field),
        ("well", p.well),
        ("border", p.border),
        ("text", p.text),
        ("text_2", p.text_2),
        ("muted", p.muted),
        ("accent", p.accent),
        ("accent_hi", p.accent_hi),
        ("accent_lo", p.accent_lo),
        ("bubble", p.bubble),
        ("positive", p.positive),
        ("negative", p.negative),
        ("warning", p.warning),
        ("info", p.info),
    ];
    widgets::card(ui, |ui| {
        ui.set_width(ui.available_width());
        widgets::section(ui, "01", "Palette roles");
        rows(ui, &roles, 84.0, |ui, (name, color)| {
            ui.vertical(|ui| {
                let (swatch, _) = ui.allocate_exact_size(vec2(84.0, 40.0), egui::Sense::hover());
                paint::gradient(ui.painter(), swatch, *color, *color, 8.0);
                let edge = egui::Stroke::new(1.0, Color32::from_white_alpha(20));
                ui.painter().rect_stroke(swatch, 8.0, edge, egui::StrokeKind::Inside);
                ui.label(RichText::new(*name).size(11.5));
                let hex = format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b());
                ui.label(RichText::new(hex).size(10.5).color(p.muted));
            });
        });
    });
    ui.add_space(12.0);
    widgets::card(ui, |ui| {
        ui.set_width(ui.available_width());
        widgets::section(ui, "02", &format!("Icons · {}", Icon::ALL.len()));
        rows(ui, Icon::ALL, 40.0, |ui, icon| {
            let (cell, response) = ui.allocate_exact_size(vec2(40.0, 40.0), egui::Sense::hover());
            response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Image, ui.is_enabled(), icon.name()));
            if response.hovered() {
                ui.painter().rect_filled(cell, 8.0, p.field_hover);
            }
            icon.paint(ui.painter(), egui::Rect::from_center_size(cell.center(), egui::Vec2::splat(22.0)), p.text_2);
            response.on_hover_text(format!("Icon::{icon:?}"));
        });
    });
}

/// Lays out `items` in rows of fixed-width cells that fit the available width.
/// (egui's wrapping layouts don't wrap nested child layouts.)
fn rows<T>(ui: &mut egui::Ui, items: &[T], cell_width: f32, mut add: impl FnMut(&mut egui::Ui, &T)) {
    let gap = ui.spacing().item_spacing.x;
    let per_row = ((ui.available_width() + gap) / (cell_width + gap)).floor().max(1.0) as usize;
    for row in items.chunks(per_row) {
        ui.horizontal(|ui| {
            for item in row {
                add(ui, item);
            }
        });
    }
}

impl eframe::App for Gallery {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.installed != Some(self.deep_space) {
            let theme = if self.deep_space { Theme::deep_space() } else { Theme::carbon_copper() };
            theme.install(&ctx);
            self.installed = Some(self.deep_space);
        }
        let theme_name = Theme::of(&ctx).name;
        let mut toggle = false;
        let shell = Shell::new("Observatory").logo(LOGO).rail(RAIL).status_bar(22.0).title_status(|ui| {
            toggle = widgets::pill(ui, Some(Tone::Accent), theme_name).on_hover_text("Switch theme").clicked();
            widgets::badge(ui, Icon::Star, "Demo session", Tone::Positive);
        });
        let range = 330.0..=600.0;
        let shell = if self.chat_side == 0 {
            shell.left_column(&mut self.side, range)
        } else {
            shell.right_column(&mut self.side, range)
        };
        let regions = shell.show(ui, &mut self.page);
        if toggle {
            self.deep_space = !self.deep_space;
        }

        ui.scope_builder(UiBuilder::new().max_rect(regions.main), |ui| {
            ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                ui.set_width(ui.available_width());
                self.page(ui);
            });
        });
        if let Some(column) = regions.left.or(regions.right) {
            self.chat_column(ui, column);
        }
        if let Some(status) = regions.status {
            let muted = Theme::of(&ctx).palette.muted;
            let font = egui::FontId::proportional(11.0);
            let text = "OBSERVATORY  /  SpaceUI example  ·  Illustrative sky and conditions  ·  No equipment connected";
            ui.painter().text(status.left_center(), egui::Align2::LEFT_CENTER, text, font, muted);
        }
        if let Some(shot) = &mut self.screenshot
            && let Some(Err(error)) = shot.update(&ctx)
        {
            eprintln!("screenshot: {error}");
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Theme::default().palette.bg_bottom.to_normalized_gamma_f32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_gallery_page_renders_with_portable_bundled_fonts() {
        let ctx = egui::Context::default();
        let mut gallery = Gallery::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
        gallery.screenshot = None;
        let mut frame = eframe::Frame::_new_kittest();
        for deep_space in [false, true] {
            let base = if deep_space { Theme::deep_space() } else { Theme::carbon_copper() };
            Theme { fonts: Fonts::BUNDLED, ..base }.install(&ctx);
            gallery.deep_space = deep_space;
            gallery.installed = Some(deep_space);
            for page in 0..PAGES.len() {
                gallery.page = page;
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1480.0, 920.0))),
                    ..Default::default()
                };
                ctx.run_ui(input, |ui| eframe::App::ui(&mut gallery, ui, &mut frame)).drop_without_applying_deltas();
            }
        }
    }
}
