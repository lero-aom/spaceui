//! The SpaceUI gallery: the shell, both themes, every widget and a chat
//! column with canned replies (no model is called).
//!
//! ```text
//! cargo run --example gallery
//! ```
//!
//! For a screenshot run, set `SPACEUI_SCREENSHOT=out.png`; `SPACEUI_THEME=deep_space`
//! picks the Deep Space theme, `SPACEUI_CHAT=left` puts the chat on the left and
//! `SPACEUI_PAGE=0..=3` picks the page.
use spaceui::chat::{self, ChatLayout, Composer, Tab, TabAction, transcript};
use spaceui::egui::{self, Color32, RichText, ScrollArea, UiBuilder, vec2};
use spaceui::prelude::*;

const LOGO: Svg = Svg::new("gallery/spaceui", include_str!("assets/spaceui.svg"));
const HINT: &str = "Ask the gallery anything… (canned replies)";
const PAGES: [&str; 4] = ["Overview", "Controls", "Palette and icons", "Settings"];
const RAIL: &[RailItem] = &[
    RailItem::page(Icon::Home, PAGES[0]),
    RailItem::page(Icon::Sliders, PAGES[1]),
    RailItem::page(Icon::Palette, PAGES[2]),
    RailItem::Divider,
    RailItem::page(Icon::Settings, PAGES[3]),
];
const EFFORTS: [&str; 5] = ["Low", "Medium", "High", "Extra high", "Max"];

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: window::viewport("SpaceUI Gallery", "spaceui-gallery")
            .with_icon(LOGO.window_icon(256).unwrap_or_default()),
        ..Default::default()
    };
    eframe::run_native("SpaceUI Gallery", options, Box::new(|cc| Ok(Box::new(Gallery::new(cc)))))
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
        Self { id, title: "New chat".into(), entries: Vec::new(), draft: String::new() }
    }

    /// The canned reply: a Markdown echo of the draft.
    fn send(&mut self) {
        let text = std::mem::take(&mut self.draft).trim().to_owned();
        if self.entries.is_empty() {
            self.title = text.chars().take(28).collect();
        }
        let reply = format!("You wrote **{text}**.\n\nThe gallery only echoes; your app plugs its own model in here.");
        self.entries.push(Entry::User(text));
        self.entries.push(Entry::Reply { text: reply, thinking: String::new(), steps: Vec::new(), open: false });
        self.entries.push(Entry::Notice("Canned demo reply: nothing was sent to a model.".into()));
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
    switch: bool,
    search: String,
    screenshot: Option<window::Screenshot>,
}

impl Gallery {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let deep_space = std::env::var("SPACEUI_THEME").is_ok_and(|theme| theme == "deep_space");
        let border = if deep_space { Palette::DEEP_SPACE.border } else { Palette::CARBON_COPPER.border };
        window::round_corners(cc, border);
        let mut welcome = Conversation::new(1);
        welcome.title = "Welcome".into();
        welcome.entries = vec![
            Entry::User("What is SpaceUI?".into()),
            Entry::Reply {
                text: "**SpaceUI** is a design system for *egui* desktop apps:\n\n\
                       - a `Theme` of semantic colours, metrics and fonts\n\
                       - anti-aliased painting and vector icons\n\
                       - widgets, the window shell and this chat kit\n\n\
                       > Canned demo reply: no model was called."
                    .into(),
                thinking: "Demo text.".into(),
                steps: vec![("Read the theme", "Theme::of(ctx)"), ("Painted the shell", "Shell::show")],
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
            switch: true,
            search: String::new(),
            screenshot: window::Screenshot::from_env("SPACEUI_SCREENSHOT"),
        }
    }

    fn page(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(PAGES[self.page.min(3)]).font(semibold(26.0)));
        widgets::caption(ui, "SpaceUI gallery · everything on this page follows the installed theme.");
        ui.add_space(14.0);
        match self.page {
            0 => self.overview(ui),
            1 => self.controls(ui),
            2 => palette_and_icons(ui),
            _ => self.settings(ui),
        }
    }

    fn overview(&mut self, ui: &mut egui::Ui) {
        ui.columns(4, |columns| {
            widgets::metric(&mut columns[0], "S&P 500 / SPY", "578.80", "+1.19% vs prior close", Tone::Neutral);
            widgets::metric(&mut columns[1], "DAILY CONTEXT", "Bullish", "Price / SMA20 / SMA50", Tone::Positive);
            widgets::metric(&mut columns[2], "RELATIVE STRENGTH", "7", "Positive · 60 min", Tone::Positive);
            widgets::metric(&mut columns[3], "RELATIVE WEAKNESS", "4", "Negative · 60 min", Tone::Negative);
        });
        ui.add_space(12.0);
        widgets::tabs(ui, &mut self.tab, &["Chart", "Overview", "News", "Options"]);
        ui.add_space(8.0);
        widgets::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                widgets::section(ui, "01", "A card");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    widgets::segmented(ui, &mut self.timeframe, &["M5", "H1", "D1"]);
                });
            });
            ui.label(
                "Cards sit on the main plate. Their gradient, border and sheen come from the palette; \
                 their edges are anti-aliased at any scale.",
            );
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                widgets::primary(ui, "Primary action");
                widgets::secondary(ui, "Secondary");
                widgets::chip(ui, "PAPER", Tone::Warning);
                widgets::chip(ui, "LIVE", Tone::Positive);
            });
        });
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            widgets::card(&mut columns[0], |ui| {
                ui.set_width(ui.available_width());
                widgets::section(ui, "01", "Buttons");
                ui.horizontal(|ui| {
                    widgets::primary(ui, "Connect");
                    widgets::secondary(ui, "Cancel");
                    ui.add_enabled_ui(false, |ui| widgets::primary(ui, "Disabled"));
                });
                ui.horizontal(|ui| {
                    for (icon, label) in [
                        (Icon::Search, "Search"),
                        (Icon::Reload, "Reload"),
                        (Icon::Star, "Favorite"),
                        (Icon::More, "More actions"),
                    ] {
                        widgets::icon_button_labeled(ui, icon, 34.0, label);
                    }
                });
                ui.add_space(8.0);
                widgets::section(ui, "02", "Pills, badges and chips");
                ui.horizontal(|ui| {
                    widgets::badge(ui, Icon::Shield, "Read-only", Tone::Positive);
                    widgets::pill(ui, Some(Tone::Warning), "Preview data");
                    widgets::pill(ui, None, "Disconnect");
                });
                ui.horizontal(|ui| {
                    for (text, tone) in [
                        ("ACCENT", Tone::Accent),
                        ("POSITIVE", Tone::Positive),
                        ("NEGATIVE", Tone::Negative),
                        ("INFO", Tone::Info),
                        ("MUTED", Tone::Muted),
                        ("CUSTOM", Tone::Custom(Color32::from_rgb(64, 156, 255))),
                    ] {
                        widgets::chip(ui, text, tone);
                    }
                });
            });
            widgets::card(&mut columns[1], |ui| {
                ui.set_width(ui.available_width());
                widgets::section(ui, "03", "Inputs");
                widgets::text_field(ui, &mut self.search, "Search symbols, companies or strategies…")
                    .accessible_name("Symbol search");
                ui.horizontal(|ui| {
                    widgets::switch_labeled(ui, &mut self.switch, "Enable agent");
                    ui.label(if self.switch { "Agent on" } else { "Agent off" });
                });
                ui.horizontal(|ui| {
                    let label = ui.label("Effort");
                    widgets::notches_labeled(ui, 160.0, EFFORTS.len(), &mut self.effort, "Effort")
                        .labelled_by(label.id);
                    ui.label(EFFORTS[self.effort]);
                });
                ui.add_space(8.0);
                widgets::section(ui, "04", "Tabs");
                widgets::tabs(ui, &mut self.tab, &["Agent", "Manual", "Room"]);
                widgets::segmented(ui, &mut self.timeframe, &["M5", "H1", "D1"]);
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
            let starters = ["Show me the widgets", "What does a theme hold?"];
            let hint = "A canned demo chat: replies are scripted and nothing is sent anywhere.";
            if let Some(i) =
                transcript::empty_state(ui, layout.body, Some(logo), "What should we build?", hint, &starters)
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
                let model = widgets::dropdown(ui, Some(Tone::Positive), "Gallery · Scripted", 150.0)
                    .accessible_name("Model: Gallery · Scripted");
                egui::Popup::menu(&model).show(|ui| ui.label("Apps list their models here."));
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
            transcript::byline(ui, Some(logo), "Gallery · canned reply");
            if transcript::thinking(ui, thinking, false, 2.0, *open) {
                *open = !*open;
            }
            for (i, (title, detail)) in steps.iter().enumerate() {
                let step = transcript::Step {
                    icon: Icon::Wrench,
                    title,
                    detail,
                    time: "09:41",
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
        let shell = Shell::new("SpaceUI").logo(LOGO).rail(RAIL).status_bar(22.0).title_status(|ui| {
            toggle = widgets::pill(ui, Some(Tone::Accent), theme_name).on_hover_text("Switch theme").clicked();
            widgets::badge(ui, Icon::Shield, "Demo data", Tone::Positive);
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
            let text = "SpaceUI gallery · canned data only";
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
