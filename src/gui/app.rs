//! The window: state, layout and event handling.
//!
//! The shape is a two column desktop app. Left is the Steam library, right is
//! the achievement list for whichever game is open. No Steam call happens here;
//! everything goes through `worker` and comes back as a message.

use std::time::{Duration, Instant};

use eframe::egui::{self, Align, Color32, Layout, RichText, vec2};

use crate::config::{self, Config, SortColumn, SortOrder};
use crate::library::{self, Game};
use crate::search::fuzzy_score;
use crate::steam::AchievementInfo;

use super::theme;
use super::widgets::{self, Badge, RowView};
use super::worker::{Cmd, Msg, Worker};

const SIDEBAR_WIDTH: f32 = 272.0;
const SAVE_EVERY: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Locked,
    Unlocked,
}

impl Filter {
    fn label(self) -> &'static str {
        match self {
            Filter::All => "All",
            Filter::Locked => "Locked",
            Filter::Unlocked => "Unlocked",
        }
    }
}

/// One achievement, plus what the user wants it to become.
struct Row {
    api_name: String,
    title: String,
    description: String,
    /// What Steam reports right now.
    unlocked: bool,
    /// What the checkbox says it should be. Starts equal to `unlocked`.
    desired: bool,
    percentage: f32,
    /// Steam refused to write this one on the last apply.
    refused: bool,
}

impl Row {
    fn pending(&self) -> bool {
        self.desired != self.unlocked
    }

    fn badge(&self) -> Badge {
        if self.refused {
            Badge::Failed
        } else if self.desired && !self.unlocked {
            Badge::PendingUnlock
        } else if !self.desired && self.unlocked {
            Badge::PendingReset
        } else if self.unlocked {
            Badge::Unlocked
        } else {
            Badge::Locked
        }
    }
}

/// Whether Steam has answered us yet, which is the only honest way to report
/// the connection: the API does not tell us until we ask it for something.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Link {
    Unknown,
    Connected,
    Unavailable,
}

pub struct SamApp {
    config: Config,
    worker: Worker,
    games: Vec<Game>,
    query: String,
    manual_id: String,
    open: Option<u32>,
    rows: Vec<Row>,
    filter: Filter,
    busy: bool,
    link: Link,
    error: Option<String>,
    notice: Option<String>,
    /// Names Steam refused last time, reapplied to rows after the reload.
    refused: Vec<String>,
    /// Hover has to be read from the previous frame; a row paints its own
    /// background before this frame's response exists.
    warm_row: Option<usize>,
    dirty_since: Option<Instant>,
}

impl SamApp {
    pub fn new(cc: &eframe::CreationContext<'_>, requested: Option<u32>) -> Self {
        theme::install(&cc.egui_ctx);

        let config = config::load();
        let mut games = library::installed_games();

        // A game passed on the command line wins, even if no manifest for it was
        // found, so `sam --id 367520` still works for anything you own.
        let open = requested.or(config.last_app_id);
        if let Some(id) = open {
            if !games.iter().any(|g| g.app_id == id) {
                games.insert(
                    0,
                    Game {
                        app_id: id,
                        name: format!("App {id}"),
                    },
                );
            }
        }

        let worker = Worker::spawn(cc.egui_ctx.clone());
        if let Some(id) = open {
            worker.send(Cmd::Load { app_id: id });
        }

        Self {
            config,
            worker,
            games,
            query: String::new(),
            manual_id: String::new(),
            open,
            rows: Vec::new(),
            filter: Filter::All,
            busy: open.is_some(),
            link: Link::Unknown,
            error: None,
            notice: None,
            refused: Vec::new(),
            warm_row: None,
            dirty_since: None,
        }
    }

    // ------------------------------------------------------------- messages

    fn take_messages(&mut self) {
        for msg in self.worker.drain() {
            match msg {
                Msg::Loaded {
                    app_id,
                    achievements,
                } => {
                    if Some(app_id) != self.open {
                        continue; // a stale reply for a game we already left
                    }
                    self.adopt(app_id, achievements);
                }
                Msg::LoadFailed { app_id, error } => {
                    if Some(app_id) != self.open {
                        continue;
                    }
                    self.rows.clear();
                    self.busy = false;
                    self.link = Link::Unavailable;
                    self.notice = None;
                    self.error = Some(error);
                }
                Msg::Applied {
                    app_id,
                    written,
                    failed,
                } => {
                    if Some(app_id) != self.open {
                        // The user moved on. Their new game already has its own
                        // load in flight, so drop this without touching state.
                        continue;
                    }
                    self.refused = failed;
                    self.notice = Some(match (written, self.refused.len()) {
                        (0, 0) => "Nothing to write".to_owned(),
                        (n, 0) => format!("Wrote {n} change{}", plural(n)),
                        (0, f) => format!("Steam refused {f} change{}", plural(f)),
                        (n, f) => format!(
                            "Wrote {n} change{}, Steam refused {f}",
                            plural(n)
                        ),
                    });
                    self.error = None;
                    // Re-read rather than trust our own bookkeeping: Steam is
                    // the only authority on what is actually unlocked now.
                    self.worker.send(Cmd::Load { app_id });
                }
                Msg::ApplyFailed { app_id, error } => {
                    if Some(app_id) != self.open {
                        continue;
                    }
                    self.busy = false;
                    self.notice = None;
                    self.error = Some(error);
                }
            }
        }
    }

    fn adopt(&mut self, app_id: u32, achievements: Vec<AchievementInfo>) {
        // Lifted out of `self` so the closure below borrows a plain local.
        let refused = std::mem::take(&mut self.refused);

        self.rows = achievements
            .into_iter()
            .map(|a| {
                let description = if a.description.trim().is_empty() {
                    if a.hidden && !a.unlocked {
                        "Hidden until unlocked".to_owned()
                    } else {
                        String::new()
                    }
                } else {
                    a.description
                };
                Row {
                    refused: refused.iter().any(|n| *n == a.api_name),
                    api_name: a.api_name,
                    title: a.display_name,
                    description,
                    unlocked: a.unlocked,
                    desired: a.unlocked,
                    percentage: a.percentage,
                }
            })
            .collect();

        self.refused = refused;
        self.sort_rows();
        self.busy = false;
        self.link = Link::Connected;
        self.error = None;

        let unlocked = self.rows.iter().filter(|r| r.unlocked).count() as u32;
        self.config
            .remember_counts(app_id, unlocked, self.rows.len() as u32);
        self.config.last_app_id = Some(app_id);
        self.mark_dirty();
    }

    fn sort_rows(&mut self) {
        let column = self.config.sort_column;
        self.rows.sort_by(|a, b| {
            let ordering = match column {
                // Rarest first reads as "descending", so the comparison is
                // inverted here rather than at the call site.
                SortColumn::Rarity => a
                    .percentage
                    .partial_cmp(&b.percentage)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::Name => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
                SortColumn::State => a.unlocked.cmp(&b.unlocked),
            };
            ordering.then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
        });
        if self.config.sort_order == SortOrder::Descending {
            self.rows.reverse();
        }
    }

    // ------------------------------------------------------------- commands

    fn open_game(&mut self, app_id: u32) {
        if self.open == Some(app_id) && !self.rows.is_empty() {
            return;
        }
        self.open = Some(app_id);
        self.rows.clear();
        self.refused.clear();
        self.error = None;
        self.notice = None;
        self.busy = true;
        self.config.last_app_id = Some(app_id);
        self.mark_dirty();
        self.worker.send(Cmd::Load { app_id });
    }

    fn reload(&mut self) {
        if let Some(app_id) = self.open {
            self.busy = true;
            self.error = None;
            self.notice = None;
            self.worker.send(Cmd::Load { app_id });
        }
    }

    fn apply(&mut self) {
        let Some(app_id) = self.open else { return };
        let unlock: Vec<String> = self
            .rows
            .iter()
            .filter(|r| r.desired && !r.unlocked)
            .map(|r| r.api_name.clone())
            .collect();
        let reset: Vec<String> = self
            .rows
            .iter()
            .filter(|r| !r.desired && r.unlocked)
            .map(|r| r.api_name.clone())
            .collect();

        if unlock.is_empty() && reset.is_empty() {
            return;
        }

        self.busy = true;
        self.error = None;
        self.notice = None;
        self.refused.clear();
        self.worker.send(Cmd::Apply {
            app_id,
            unlock,
            reset,
        });
    }

    fn pending(&self) -> usize {
        self.rows.iter().filter(|r| r.pending()).count()
    }

    fn mark_dirty(&mut self) {
        if self.dirty_since.is_none() {
            self.dirty_since = Some(Instant::now());
        }
    }

    /// Config is written at most once every couple of seconds, so dragging the
    /// window edge does not hammer the disk.
    fn flush_config(&mut self, ctx: &egui::Context) {
        // `Context::screen_rect()` rather than reading InputState: on egui 0.31
        // `screen_rect` is a plain field there, not a method.
        let size = ctx.screen_rect().size();
        if (size.x - self.config.window_width).abs() > 1.0
            || (size.y - self.config.window_height).abs() > 1.0
        {
            self.config.window_width = size.x;
            self.config.window_height = size.y;
            self.mark_dirty();
        }

        if let Some(since) = self.dirty_since {
            if since.elapsed() >= SAVE_EVERY {
                config::store(&self.config);
                self.dirty_since = None;
            }
        }
    }

    /// Games matching the search box, best match first.
    fn visible_games(&self) -> Vec<usize> {
        let needle = self.query.trim();
        if needle.is_empty() {
            return (0..self.games.len()).collect();
        }

        // fuzzy_score is case sensitive, so both sides are folded first.
        let needle = needle.to_lowercase();
        let mut scored: Vec<(i64, usize)> = self
            .games
            .iter()
            .enumerate()
            .filter_map(|(i, g)| {
                fuzzy_score(&g.name.to_lowercase(), &needle)
                    .or_else(|| fuzzy_score(&g.app_id.to_string(), &needle))
                    .map(|score| (score, i))
            })
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().map(|(_, i)| i).collect()
    }

    // ----------------------------------------------------------------- panes

    fn sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("library")
            .exact_width(SIDEBAR_WIDTH)
            .resizable(false)
            .frame(theme::sidebar_frame())
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.label(RichText::new("Sam").size(21.0).strong().color(theme::TEXT));
                ui.label(
                    RichText::new("Steam achievement manager")
                        .size(11.5)
                        .color(theme::TEXT_FAINT),
                );
                ui.add_space(12.0);

                ui.add_sized(
                    vec2(ui.available_width(), 32.0),
                    egui::TextEdit::singleline(&mut self.query).hint_text("Search library"),
                );
                ui.add_space(14.0);

                let visible = self.visible_games();
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("LIBRARY")
                            .size(10.5)
                            .color(theme::TEXT_FAINT),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{} game{}",
                                visible.len(),
                                plural(visible.len())
                            ))
                            .size(10.5)
                            .color(theme::TEXT_FAINT),
                        );
                    });
                });
                ui.add_space(6.0);

                // Reserve room for the App ID field and the status line so they
                // stay pinned to the bottom of the column.
                let footer = 78.0;
                let list_height = (ui.available_height() - footer).max(120.0);

                egui::ScrollArea::vertical()
                    .id_salt("games")
                    .max_height(list_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if visible.is_empty() {
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new(if self.games.is_empty() {
                                    "No installed games found. Use the App ID box below."
                                } else {
                                    "Nothing matches that search."
                                })
                                .size(12.0)
                                .color(theme::TEXT_FAINT),
                            );
                            return;
                        }

                        let mut clicked: Option<u32> = None;
                        for index in visible {
                            let game = &self.games[index];
                            let app_id = game.app_id;
                            let selected = self.open == Some(app_id);
                            let subtitle = self.game_subtitle(app_id, selected);
                            if widgets::library_row(
                                ui,
                                &game.name,
                                &subtitle,
                                theme::game_tint(app_id),
                                selected,
                            )
                            .clicked()
                            {
                                clicked = Some(app_id);
                            }
                        }
                        if let Some(app_id) = clicked {
                            self.open_game(app_id);
                        }
                    });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let field = ui.add_sized(
                        vec2(ui.available_width() - 74.0, 28.0),
                        egui::TextEdit::singleline(&mut self.manual_id).hint_text("App ID"),
                    );
                    let entered =
                        field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    let pressed = widgets::action(ui, "Open", false, !self.manual_id.is_empty())
                        .clicked();
                    if entered || pressed {
                        // Parsed into a local first: an `if let` scrutinee would
                        // keep `self.manual_id` borrowed across the whole block,
                        // and the block clears it.
                        let parsed = self.manual_id.trim().parse::<u32>();
                        if let Ok(app_id) = parsed {
                            if !self.games.iter().any(|g| g.app_id == app_id) {
                                self.games.insert(
                                    0,
                                    Game {
                                        app_id,
                                        name: format!("App {app_id}"),
                                    },
                                );
                            }
                            self.manual_id.clear();
                            self.open_game(app_id);
                        } else {
                            self.error =
                                Some("An App ID is a number, for example 367520.".to_owned());
                        }
                    }
                });

                ui.add_space(6.0);
                let (dot, label) = match self.link {
                    Link::Connected => (theme::GREEN, "Steam connected"),
                    Link::Unavailable => (theme::AMBER, "Steam not responding"),
                    Link::Unknown => (theme::TEXT_FAINT, "Waiting for Steam"),
                };
                widgets::status_line(ui, dot, label);
            });
    }

    fn game_subtitle(&self, app_id: u32, selected: bool) -> String {
        if selected && !self.rows.is_empty() {
            let unlocked = self.rows.iter().filter(|r| r.unlocked).count();
            return format!("{} / {} unlocked", unlocked, self.rows.len());
        }
        if selected && self.busy {
            return "Reading…".to_owned();
        }
        match self.config.counts_for(app_id) {
            Some(c) => format!("{} / {} last seen", c.unlocked, c.total),
            None => "Not opened yet".to_owned(),
        }
    }

    fn body(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(theme::body_frame())
            .show(ctx, |ui| {
                self.header(ui);
                ui.add_space(14.0);

                if self.open.is_none() {
                    self.empty_state(ui, "Pick a game", "Choose one from the library on the left, or type an App ID.");
                    return;
                }

                self.stats(ui);
                ui.add_space(14.0);
                self.toolbar(ui);
                ui.add_space(10.0);

                if let Some(error) = self.error.clone() {
                    self.banner(ui, &error, theme::RED, theme::RED_WASH);
                    ui.add_space(10.0);
                } else if let Some(notice) = self.notice.clone() {
                    self.banner(ui, &notice, theme::GREEN, theme::GREEN_WASH);
                    ui.add_space(10.0);
                }

                if self.rows.is_empty() {
                    let (title, hint) = if self.busy {
                        ("Reading achievements", "Steam is answering. This takes a moment the first time.")
                    } else if self.error.is_some() {
                        ("Nothing to show", "Fix the problem above, then press Refresh.")
                    } else {
                        ("No achievements", "This game does not have any.")
                    };
                    self.empty_state(ui, title, hint);
                    return;
                }

                self.list(ui);
            });
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        let title = self
            .open
            .and_then(|id| self.games.iter().find(|g| g.app_id == id))
            .map(|g| g.name.clone())
            .unwrap_or_else(|| "Sam".to_owned());

        let subtitle = match self.open {
            Some(id) if !self.rows.is_empty() => {
                let unlocked = self.rows.iter().filter(|r| r.unlocked).count();
                format!(
                    "App ID {id} · {unlocked} of {} achievements unlocked",
                    self.rows.len()
                )
            }
            Some(id) if self.busy => format!("App ID {id} · reading…"),
            Some(id) => format!("App ID {id}"),
            None => "No game open".to_owned(),
        };

        let pending = self.pending();
        let can_apply = pending > 0 && !self.busy;
        let can_refresh = self.open.is_some() && !self.busy;

        let mut do_apply = false;
        let mut do_reload = false;

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(title).size(23.0).strong().color(theme::TEXT));
                ui.label(RichText::new(subtitle).size(12.5).color(theme::TEXT_DIM));
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let label = if pending > 0 {
                    format!("Apply {pending} change{}", plural(pending))
                } else {
                    "Apply changes".to_owned()
                };
                do_apply = widgets::action(ui, &label, true, can_apply).clicked();
                do_reload = widgets::action(ui, "Refresh", false, can_refresh).clicked();
            });
        });

        if do_apply {
            self.apply();
        }
        if do_reload {
            self.reload();
        }
    }

    fn stats(&mut self, ui: &mut egui::Ui) {
        let total = self.rows.len();
        let unlocked = self.rows.iter().filter(|r| r.unlocked).count();
        let locked = total - unlocked;
        let completion = if total == 0 {
            0
        } else {
            ((unlocked as f32 / total as f32) * 100.0).round() as u32
        };

        let gap = ui.spacing().item_spacing.x;
        let width = ((ui.available_width() - gap * 2.0) / 3.0).max(80.0);
        ui.horizontal(|ui| {
            widgets::stat(
                ui,
                vec2(width, 72.0),
                "Unlocked",
                &unlocked.to_string(),
                theme::GREEN,
            );
            widgets::stat(
                ui,
                vec2(width, 72.0),
                "Locked",
                &locked.to_string(),
                theme::TEXT,
            );
            widgets::stat(
                ui,
                vec2(width, 72.0),
                "Completion",
                &format!("{completion}%"),
                theme::ACCENT,
            );
        });
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        let pending = self.pending();
        let mut new_filter: Option<Filter> = None;
        let mut cycle_sort = false;
        let mut flip_order = false;
        let mut select_all = false;
        let mut discard = false;

        ui.horizontal(|ui| {
            for filter in [Filter::All, Filter::Locked, Filter::Unlocked] {
                if widgets::chip(ui, filter.label(), self.filter == filter).clicked() {
                    new_filter = Some(filter);
                }
            }

            ui.add_space(6.0);
            if widgets::chip(ui, &format!("Sort: {}", self.config.sort_column.label()), false)
                .on_hover_text("Rarity, name or state")
                .clicked()
            {
                cycle_sort = true;
            }
            if widgets::chip(
                ui,
                if self.config.sort_order == SortOrder::Descending {
                    "Desc"
                } else {
                    "Asc"
                },
                false,
            )
            .clicked()
            {
                flip_order = true;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if pending > 0 {
                    ui.label(
                        RichText::new(format!("{pending} pending"))
                            .size(12.0)
                            .color(theme::ACCENT),
                    );
                    ui.add_space(4.0);
                    if widgets::chip(ui, "Discard", false).clicked() {
                        discard = true;
                    }
                }
                if widgets::chip(ui, "Unlock all shown", false).clicked() {
                    select_all = true;
                }
            });
        });

        if let Some(filter) = new_filter {
            self.filter = filter;
        }
        if cycle_sort {
            self.config.sort_column = match self.config.sort_column {
                SortColumn::Rarity => SortColumn::Name,
                SortColumn::Name => SortColumn::State,
                SortColumn::State => SortColumn::Rarity,
            };
            self.sort_rows();
            self.mark_dirty();
        }
        if flip_order {
            self.config.sort_order = match self.config.sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::Ascending,
            };
            self.sort_rows();
            self.mark_dirty();
        }
        if select_all {
            let filter = self.filter;
            for row in self.rows.iter_mut().filter(|r| shows(r, filter)) {
                row.desired = true;
            }
        }
        if discard {
            for row in self.rows.iter_mut() {
                row.desired = row.unlocked;
            }
        }
    }

    fn list(&mut self, ui: &mut egui::Ui) {
        let filter = self.filter;
        let mut toggled: Option<usize> = None;
        let mut warm: Option<usize> = None;

        egui::ScrollArea::vertical()
            .id_salt("achievements")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 8.0;
                let mut shown = 0usize;

                for (index, row) in self.rows.iter().enumerate() {
                    if !shows(row, filter) {
                        continue;
                    }
                    shown += 1;

                    let (tier_colour, tier_label) = theme::tier(row.percentage);
                    let response = widgets::achievement_row(
                        ui,
                        RowView {
                            title: &row.title,
                            description: &row.description,
                            percentage: row.percentage,
                            tier_colour,
                            checked: row.desired,
                            badge: row.badge(),
                            warm: self.warm_row == Some(index),
                        },
                    );

                    let response = response.on_hover_text(format!(
                        "{}  ·  {:.1}% of players have this\n{}",
                        tier_label, row.percentage, row.api_name
                    ));
                    if response.hovered() {
                        warm = Some(index);
                    }
                    if response.clicked() {
                        toggled = Some(index);
                    }
                }

                if shown == 0 {
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new("No achievements match this filter.")
                            .size(12.5)
                            .color(theme::TEXT_FAINT),
                    );
                }
            });

        self.warm_row = warm;
        if let Some(index) = toggled {
            let row = &mut self.rows[index];
            row.desired = !row.desired;
            row.refused = false;
        }
    }

    fn banner(&self, ui: &mut egui::Ui, text: &str, edge: Color32, wash: Color32) {
        egui::Frame::default()
            .fill(wash)
            .stroke(egui::Stroke::new(1.0, edge.linear_multiply(0.6)))
            .corner_radius(theme::R_CTRL)
            .inner_margin(10)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(text).size(12.5).color(edge));
                });
            });
    }

    fn empty_state(&self, ui: &mut egui::Ui, title: &str, hint: &str) {
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add_space(28.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(title).size(16.0).strong().color(theme::TEXT));
                ui.add_space(4.0);
                ui.label(RichText::new(hint).size(12.5).color(theme::TEXT_DIM));
            });
            ui.add_space(28.0);
        });
    }
}

fn shows(row: &Row, filter: Filter) -> bool {
    match filter {
        Filter::All => true,
        Filter::Locked => !row.unlocked,
        Filter::Unlocked => row.unlocked,
    }
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

impl eframe::App for SamApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.take_messages();

        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            self.reload();
        }

        self.sidebar(ctx);
        self.body(ctx);

        self.flush_config(ctx);
    }
}

impl Drop for SamApp {
    fn drop(&mut self) {
        config::store(&self.config);
    }
}
