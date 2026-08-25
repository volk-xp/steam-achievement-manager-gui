use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SortColumn {
    Rarity,
    Name,
    State,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortColumn {
    pub fn label(self) -> &'static str {
        match self {
            SortColumn::Rarity => "Rarity",
            SortColumn::Name => "Name",
            SortColumn::State => "State",
        }
    }
}

/// The last unlocked/total we saw for a game.
///
/// The Steam API initialises against one App ID at a time, so the sidebar
/// cannot read live counts for every installed game at once. We remember what
/// we saw the last time each game was opened and label those numbers as cached.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CachedCounts {
    pub app_id: u32,
    pub unlocked: u32,
    pub total: u32,
}

/// IMPORTANT: every scalar field must be declared before `counts`.
/// confy writes TOML, and TOML cannot emit a plain value after an array of
/// tables. Moving `counts` above the scalars makes saving fail silently.
/// `hidden` is a plain array of integers rather than an array of tables, so it
/// is safe here, but keep it above `counts` for the same reason.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub sort_column: SortColumn,
    pub sort_order: SortOrder,
    pub last_app_id: Option<u32>,
    pub window_width: f32,
    pub window_height: f32,
    /// App IDs the user removed from the library list. They stay installed and
    /// can still be opened by App ID; this only hides them from the sidebar.
    ///
    /// `serde(default)` matters: without it, a sam.toml written by an earlier
    /// build has no `hidden` key and the whole config fails to parse, silently
    /// resetting every one of the user's settings.
    #[serde(default)]
    pub hidden: Vec<u32>,
    pub counts: Vec<CachedCounts>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // Rarity ascending puts the rarest achievements at the top, which is
            // the order the terminal version opened in.
            sort_column: SortColumn::Rarity,
            sort_order: SortOrder::Ascending,
            last_app_id: None,
            window_width: 1180.0,
            window_height: 780.0,
            hidden: Vec::new(),
            counts: Vec::new(),
        }
    }
}

impl Config {
    pub fn counts_for(&self, app_id: u32) -> Option<&CachedCounts> {
        self.counts.iter().find(|c| c.app_id == app_id)
    }

    pub fn remember_counts(&mut self, app_id: u32, unlocked: u32, total: u32) {
        match self.counts.iter_mut().find(|c| c.app_id == app_id) {
            Some(existing) => {
                existing.unlocked = unlocked;
                existing.total = total;
            }
            None => self.counts.push(CachedCounts {
                app_id,
                unlocked,
                total,
            }),
        }
    }

    pub fn is_hidden(&self, app_id: u32) -> bool {
        self.hidden.contains(&app_id)
    }

    /// True if this changed anything, so a caller can skip an unnecessary save.
    pub fn hide(&mut self, app_id: u32) -> bool {
        if self.is_hidden(app_id) {
            return false;
        }
        self.hidden.push(app_id);
        true
    }

    /// True if it was hidden and no longer is. `retain` rather than a single
    /// removal, so a config hand-edited to list the same ID twice still clears.
    pub fn unhide(&mut self, app_id: u32) -> bool {
        let before = self.hidden.len();
        self.hidden.retain(|id| *id != app_id);
        self.hidden.len() != before
    }
}

pub fn load() -> Config {
    confy::load("sam", None).unwrap_or_default()
}

pub fn store(config: &Config) {
    let _ = confy::store("sam", None, config);
}
