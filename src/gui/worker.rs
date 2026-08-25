//! The one thread that is allowed to talk to Steam.
//!
//! Two hard rules drive this design:
//!
//! 1. `Client::init_app` initialises the Steam API against a single App ID for
//!    the process. Running two of them at once, or from two threads, is asking
//!    for trouble. So every Steam call is queued onto one worker thread and run
//!    in order.
//! 2. Those calls block for as long as Steam feels like taking. On the UI thread
//!    that means a frozen window, so nothing here runs on the UI thread.
//!
//! The UI sends `Cmd`s and drains `Msg`s once per frame. `ctx.request_repaint()`
//! wakes the UI when a reply lands, so the window does not sit idle on a result.

use std::sync::mpsc::{Receiver, Sender, channel};

use eframe::egui;

use crate::steam::{self, AchievementInfo};

pub enum Cmd {
    /// Read every achievement for this game.
    Load { app_id: u32 },
    /// Write the pending changes back to Steam, then re-read.
    Apply {
        app_id: u32,
        unlock: Vec<String>,
        reset: Vec<String>,
    },
    /// Let the thread finish when the window closes.
    Quit,
}

pub enum Msg {
    Loaded {
        app_id: u32,
        achievements: Vec<AchievementInfo>,
    },
    LoadFailed {
        app_id: u32,
        error: String,
    },
    /// `failed` holds the API names Steam refused, so the rows can say so.
    Applied {
        app_id: u32,
        written: usize,
        failed: Vec<String>,
    },
    ApplyFailed {
        app_id: u32,
        error: String,
    },
}

pub struct Worker {
    tx: Sender<Cmd>,
    rx: Receiver<Msg>,
}

impl Worker {
    pub fn spawn(ctx: egui::Context) -> Self {
        let (cmd_tx, cmd_rx) = channel::<Cmd>();
        let (msg_tx, msg_rx) = channel::<Msg>();

        std::thread::Builder::new()
            .name("steam".to_owned())
            .spawn(move || {
                while let Ok(cmd) = cmd_rx.recv() {
                    let reply = match cmd {
                        Cmd::Quit => break,
                        Cmd::Load { app_id } => run_load(app_id),
                        Cmd::Apply {
                            app_id,
                            unlock,
                            reset,
                        } => run_apply(app_id, unlock, reset),
                    };

                    if msg_tx.send(reply).is_err() {
                        break;
                    }
                    ctx.request_repaint();
                }
            })
            .expect("failed to start the Steam thread");

        Self {
            tx: cmd_tx,
            rx: msg_rx,
        }
    }

    pub fn send(&self, cmd: Cmd) {
        // A dead worker is not worth crashing over: the UI already shows the
        // last error and the user can retry, which is more useful than a panic.
        let _ = self.tx.send(cmd);
    }

    /// Everything that has arrived since the last frame.
    pub fn drain(&self) -> Vec<Msg> {
        self.rx.try_iter().collect()
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.tx.send(Cmd::Quit);
    }
}

/// The Steamworks bindings are FFI, and FFI can panic in ways we cannot fix.
/// Catching it here turns "the whole app disappears" into "one red message".
fn guard<T>(what: &str, job: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(job)) {
        Ok(result) => result,
        Err(_) => Err(format!(
            "Steam crashed while {what}. Make sure Steam is running and you are signed in, then try again."
        )),
    }
}

fn run_load(app_id: u32) -> Msg {
    let outcome = guard("reading achievements", || {
        steam::get_achievements(app_id).map_err(|e| e.to_string())
    });

    match outcome {
        Ok(data) => Msg::Loaded {
            app_id,
            achievements: data.achievements,
        },
        Err(error) => Msg::LoadFailed { app_id, error },
    }
}

fn run_apply(app_id: u32, unlock: Vec<String>, reset: Vec<String>) -> Msg {
    let requested = unlock.len() + reset.len();
    let mut failed: Vec<String> = Vec::new();

    let outcome = guard("writing achievements", || {
        // Unlocking and resetting are separate calls in the Steam API, so the
        // two lists go over one at a time.
        for (names, clear) in [(unlock, false), (reset, true)] {
            if names.is_empty() {
                continue;
            }
            let results = steam::process_achievements(app_id, names, clear)?;
            failed.extend(
                results
                    .into_iter()
                    .filter(|r| !r.success)
                    .map(|r| r.name),
            );
        }
        Ok(())
    });

    match outcome {
        Ok(()) => Msg::Applied {
            app_id,
            written: requested - failed.len(),
            failed,
        },
        Err(error) => Msg::ApplyFailed { app_id, error },
    }
}
