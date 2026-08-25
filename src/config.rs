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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub sort_column: SortColumn,
    pub sort_order: SortOrder,
    pub last_app_id: Option<u32>,
    pub window_width: f32,
    pub window_height: f32,
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
}

pub fn load() -> Config {
    confy::load("sam", None).unwrap_or_default()
}

pub fn store(config: &Config) {
    let _ = confy::store("sam", None, config);
}
