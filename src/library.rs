//! Finds the games Steam has installed locally.
//!
//! Steam does not expose a "list my library" call through the Steamworks API a
//! game client can use, so this reads Steam's own files instead:
//!
//!   <steam>/steamapps/libraryfolders.vdf   -> where the library folders are
//!   <library>/steamapps/appmanifest_*.acf  -> one file per installed game
//!
//! Everything here is plain text parsing, no extra crates and no network.

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub app_id: u32,
    pub name: String,
}

/// Things that show up as "installed" but are not games.
const SKIP_IDS: [u32; 6] = [
    228980,  // Steamworks Common Redistributables
    1070560, // Steam Linux Runtime 1.0
    1391110, // Steam Linux Runtime 2.0 (soldier)
    1628350, // Steam Linux Runtime 3.0 (sniper)
    1493710, // Proton Experimental
    2180100, // Proton Hotfix
];

/// Where Steam itself is installed.
///
/// Reading HKCU\Software\Valve\Steam\SteamPath would be more precise, but that
/// needs a registry crate. These candidates cover a normal Windows install, and
/// SAM_STEAM_PATH is the escape hatch for anything unusual.
fn steam_roots() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();

    if let Ok(explicit) = std::env::var("SAM_STEAM_PATH") {
        roots.push(PathBuf::from(explicit));
    }

    for var in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Ok(dir) = std::env::var(var) {
            roots.push(Path::new(&dir).join("Steam"));
        }
    }

    roots.push(PathBuf::from(r"C:\Program Files (x86)\Steam"));
    roots.push(PathBuf::from(r"C:\Program Files\Steam"));
    roots.push(PathBuf::from(r"C:\Steam"));

    // Handy when developing on something other than Windows.
    if let Ok(home) = std::env::var("HOME") {
        roots.push(Path::new(&home).join(".steam/steam"));
        roots.push(Path::new(&home).join(".local/share/Steam"));
        roots.push(Path::new(&home).join("Library/Application Support/Steam"));
    }

    roots.retain(|r| r.join("steamapps").is_dir());
    roots
}

/// Pulls `"key"  "value"` out of a VDF/ACF line. Both formats use the same shape.
fn key_value(line: &str) -> Option<(String, String)> {
    let mut parts = line.split('"').skip(1);
    let key = parts.next()?;
    parts.next()?; // the whitespace between the two quoted strings
    let value = parts.next()?;
    if key.is_empty() {
        return None;
    }
    // VDF escapes backslashes, so D:\\Games becomes D:\Games.
    Some((key.to_ascii_lowercase(), value.replace("\\\\", "\\")))
}

/// Every steamapps folder Steam knows about, including extra drives.
fn library_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = vec![root.join("steamapps")];

    // Newer Steam keeps this in steamapps/, older builds in config/.
    for candidate in [
        root.join("steamapps/libraryfolders.vdf"),
        root.join("config/libraryfolders.vdf"),
    ] {
        let Ok(text) = fs::read_to_string(&candidate) else {
            continue;
        };
        for line in text.lines() {
            if let Some((key, value)) = key_value(line) {
                if key == "path" {
                    let dir = Path::new(&value).join("steamapps");
                    if dir.is_dir() && !dirs.contains(&dir) {
                        dirs.push(dir);
                    }
                }
            }
        }
    }

    dirs
}

fn game_from_manifest(path: &Path) -> Option<Game> {
    let text = fs::read_to_string(path).ok()?;
    let mut app_id: Option<u32> = None;
    let mut name: Option<String> = None;

    for line in text.lines() {
        match key_value(line) {
            Some((key, value)) if key == "appid" => app_id = value.trim().parse().ok(),
            Some((key, value)) if key == "name" => name = Some(value),
            _ => {}
        }
        if app_id.is_some() && name.is_some() {
            break;
        }
    }

    let app_id = app_id?;
    let name = name.unwrap_or_else(|| format!("App {}", app_id));
    Some(Game { app_id, name })
}

/// Installed games, deduplicated and sorted by name.
pub fn installed_games() -> Vec<Game> {
    let mut games: Vec<Game> = Vec::new();

    for root in steam_roots() {
        for dir in library_dirs(&root) {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.filter_map(Result::ok) {
                let file_name = entry.file_name();
                let file_name = file_name.to_string_lossy();
                if !file_name.starts_with("appmanifest_") || !file_name.ends_with(".acf") {
                    continue;
                }
                if let Some(game) = game_from_manifest(&entry.path()) {
                    if SKIP_IDS.contains(&game.app_id) {
                        continue;
                    }
                    if game.name.starts_with("Steamworks")
                        || game.name.starts_with("Steam Linux Runtime")
                        || game.name.starts_with("Proton")
                    {
                        continue;
                    }
                    if !games.iter().any(|g| g.app_id == game.app_id) {
                        games.push(game);
                    }
                }
            }
        }
    }

    games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    games
}
