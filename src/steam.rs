use anyhow::{Context, Result, bail};
use gag::Gag;
use steamworks::{GameId, SteamError};

#[derive(Clone, Debug)]
pub struct AchievementInfo {
    /// The internal Steam key, e.g. "DefeatMorpha". This is what gets set/cleared.
    pub api_name: String,
    /// The player-facing title, e.g. "Nailed it". Falls back to `api_name`.
    pub display_name: String,
    /// The player-facing description. Empty when Steam withholds or omits it.
    pub description: String,
    /// Steam hides the title and description of these until they are earned.
    pub hidden: bool,
    pub unlocked: bool,
    pub percentage: f32,
}

#[derive(Clone, Debug)]
pub struct AchievementData {
    pub achievements: Vec<AchievementInfo>,
}

pub struct ProcessResult {
    pub name: String,
    pub success: bool,
}

/// Accepts whatever shape `get_achievement_display_attribute` returns.
///
/// That call has been spelled `&str`, `Option<&str>` and `Result<&str, _>` across
/// steamworks releases. Implementing one trait for all of them means the call
/// site compiles regardless of which one this version uses, so the only thing
/// that can go wrong is the method name itself.
#[cfg(feature = "display-names")]
mod attr {
    pub trait OptStr {
        fn opt(self) -> Option<String>;
    }

    impl OptStr for &str {
        fn opt(self) -> Option<String> {
            Some(self.to_string())
        }
    }

    impl OptStr for String {
        fn opt(self) -> Option<String> {
            Some(self)
        }
    }

    impl OptStr for Option<&str> {
        fn opt(self) -> Option<String> {
            self.map(|s| s.to_string())
        }
    }

    impl OptStr for Option<String> {
        fn opt(self) -> Option<String> {
            self
        }
    }

    impl<E> OptStr for Result<&str, E> {
        fn opt(self) -> Option<String> {
            self.ok().map(|s| s.to_string())
        }
    }

    impl<E> OptStr for Result<String, E> {
        fn opt(self) -> Option<String> {
            self.ok()
        }
    }
}

pub fn get_achievements(id: u32) -> Result<AchievementData> {
    let _stdout_gag = Gag::stdout().ok();
    let _stderr_gag = Gag::stderr().ok();

    let client = steamworks::Client::init_app(id)
        .with_context(|| format!("App {} not in your library", id))?;

    let user_stats = client.user_stats();

    match user_stats.get_num_achievements() {
        Ok(_) => {}
        Err(_) => bail!("Failed to get achievement names for app {}", id),
    };

    // NOTE: Required to get the global percentages
    let game_id = GameId::from_raw(id as u64);
    user_stats.request_global_achievement_percentages(move |result: Result<GameId, SteamError>| {
        result.unwrap_or(game_id);
    });

    let achievement_names = match user_stats.get_achievement_names() {
        Some(x) => x,
        None => bail!("Failed to get achievement names for app {}", id),
    };

    // A game with no achievements is a valid answer, not an error.
    if achievement_names.is_empty() {
        return Ok(AchievementData {
            achievements: Vec::new(),
        });
    }

    // The percentage request above is asynchronous. Without pumping callbacks
    // here every achievement reads back as 0.0%, which is the behaviour the
    // terminal version shipped with. Stop as soon as a real number lands.
    #[cfg(feature = "global-percent")]
    {
        let probe = achievement_names[0].clone();
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(1500);
        while std::time::Instant::now() < deadline {
            client.run_callbacks();
            let arrived = user_stats
                .achievement(&probe)
                .get_achievement_achieved_percent()
                .map(|p| p > 0.0)
                .unwrap_or(false);
            if arrived {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
    }

    let achievements = achievement_names
        .into_iter()
        .map(|name| {
            let achievement = user_stats.achievement(&name);

            let unlocked = achievement.get().unwrap_or(false);

            let percentage = achievement
                .get_achievement_achieved_percent()
                .unwrap_or_default();

            // Kept inline on purpose: naming the helper's type in a function
            // signature would add a second thing that can drift between
            // steamworks versions.
            #[cfg(feature = "display-names")]
            let (display_name, description, hidden) = {
                use attr::OptStr;
                let display_name = achievement
                    .get_achievement_display_attribute("name")
                    .opt()
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| name.clone());
                let description = achievement
                    .get_achievement_display_attribute("desc")
                    .opt()
                    .unwrap_or_default();
                let hidden = achievement
                    .get_achievement_display_attribute("hidden")
                    .opt()
                    .map(|s| s.trim() == "1")
                    .unwrap_or(false);
                (display_name, description, hidden)
            };

            #[cfg(not(feature = "display-names"))]
            let (display_name, description, hidden) = (name.clone(), String::new(), false);

            AchievementInfo {
                api_name: name,
                display_name,
                description,
                hidden,
                unlocked,
                percentage,
            }
        })
        .collect();

    Ok(AchievementData { achievements })
}

pub fn process_achievements(
    id: u32,
    achievement_names: Vec<String>,
    clear: bool,
) -> Result<Vec<ProcessResult>, String> {
    let _stdout_gag = Gag::stdout().ok();
    let _stderr_gag = Gag::stderr().ok();

    let client = match steamworks::Client::init_app(id) {
        Ok(x) => x,
        Err(_) => {
            return Err(format!("App {} not in your library", id));
        }
    };

    let user_stats = client.user_stats();

    let results: Vec<ProcessResult> = achievement_names
        .iter()
        .map(|name| {
            let achievement = user_stats.achievement(name);

            let success = if clear {
                achievement.clear()
            } else {
                achievement.set()
            }
            .is_ok();

            ProcessResult {
                name: name.clone(),
                success,
            }
        })
        .collect();

    let all_success = results.iter().all(|r| r.success);
    let stored = user_stats.store_stats().is_ok();

    if all_success && stored {
        Ok(results)
    } else if !stored {
        Err("Failed to store stats to Steam".to_string())
    } else {
        Ok(results)
    }
}
