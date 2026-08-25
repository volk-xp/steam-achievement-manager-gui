<h1 align="center">Steam Achievement Manager</h1>

<p align="center">
  Unlock and reset Steam achievements from a proper desktop window.<br>
  One 5&nbsp;MB executable. No terminal, no installer, no runtime to install.
</p>

<p align="center">
  <img alt="Platform: Windows x64" src="https://img.shields.io/badge/platform-Windows%20x64-0078D6">
  <img alt="Built with Rust" src="https://img.shields.io/badge/built%20with-Rust%202024-CE422B">
  <img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-yellow">
  <!-- This one queries the GitHub API, which returns nothing for a private repo,
       so it renders broken until the repo is public and has a release. Uncomment
       it when you flip the repo to public.
  <img alt="Latest release" src="https://img.shields.io/github/v/release/volk-xp/steam-achievement-manager-gui">
  -->
</p>

<!-- Add a screenshot here. It is the single biggest improvement you can make to
     this page. Run the app, press Win+Shift+S to capture the window, save it as
     docs/screenshot.png, then uncomment the block below.

<p align="center">
  <img src="docs/screenshot.png" alt="The library sidebar and achievement list" width="900">
</p>
-->

## Contents

[What it is](#what-it-is) · [Features](#features) · [Requirements](#requirements) ·
[Installation](#installation) · [Using it](#using-it) · [Command line](#command-line) ·
[Where your settings live](#where-your-settings-live) · [How it works](#how-it-works) ·
[Troubleshooting](#troubleshooting) · [Building from source](#building-from-source) ·
[Project layout](#project-layout) · [Acknowledgements](#acknowledgements) · [License](#license)

## What it is

A desktop application for unlocking and resetting the Steam achievements of games
on your own Steam account.

Pick a game from your library on the left and its achievements load on the right.
Tick what you want unlocked, untick what you want reset, then press **Apply
changes**. Nothing is written to Steam until you press that button, and every row
you have changed is labelled so you can see exactly what is about to happen before
it happens.

It talks to Steam through the official Steamworks API as your signed-in account,
which sets the boundaries of what it can do. It works on games you actually own,
it cannot touch anyone else's account, and it is not a way to get games or
achievements you do not have access to. Achievements you unlock are real
achievements: they appear on your public profile, they count toward your
completion percentage, and resetting one is not undone from inside this app —
Steam simply has the new state.

A few games compute achievements on their own servers rather than trusting the
client. Those will refuse to change, and the row will say **Steam refused** rather
than pretending it worked.

## Features

| | |
|---|---|
| **Your library, listed automatically** | Installed games are discovered from Steam's own library files, across every drive Steam knows about. Games you own but have not installed can be opened by App ID. |
| **A list you can prune** | Hover any game and press **×** to take it out of the sidebar. Nothing is uninstalled and no achievement is touched; the game is simply no longer in your way. A **removed** chip above the list brings them back. |
| **Rarity at a glance** | Every achievement shows its global unlock percentage and a colour band, so the ones almost nobody has are obvious. |
| **Nothing happens by accident** | Changes are staged locally and applied in one deliberate action. Pending unlocks and pending resets are labelled separately, and a running count sits next to the Apply button. |
| **Honest about what it knows** | After writing, the app re-reads everything from Steam instead of trusting its own bookkeeping. Cached sidebar counts are labelled "last seen" rather than presented as live. |
| **Bulk actions** | **Unlock all shown** ticks everything the current filter is showing. **Discard** throws away every pending change. |
| **Filter, sort, search** | Filter by All, Locked or Unlocked, sort by rarity, name or state in either direction, and find a game by fuzzy-matching its name or App ID in the search box. |
| **Survives a crash in Steam's FFI** | The Steamworks bindings run on a guarded worker thread. If they fall over you get a red message in the window, not a vanished application. |
| **Remembers your setup** | Window size, sort order, the games you removed and the last game you opened all persist between runs. |

## Requirements

Windows x64, and Steam installed, running, and signed in before you start the app.

Achievements can only be changed for games on the signed-in account, so switching
Steam accounts changes what this can do. There is nothing else to install: the
executable carries its own interface, and the only file that has to travel with it
is `steam_api64.dll`.

The source does contain Linux and macOS paths, and `build.rs` copies `.so` and
`.dylib` runtimes as well, but only the Windows build is compiled and tested.

## Installation

Download the latest zip from the [releases page](https://github.com/volk-xp/steam-achievement-manager-gui/releases),
extract it anywhere, and run `sam.exe`.

The zip contains exactly two files and both have to stay in the same folder:

| File | What it is |
|---|---|
| `sam.exe` | The application, around 5 MB. |
| `steam_api64.dll` | Steam's redistributable runtime library, around 310 KB. The app will not start without it. |

There is no installer and nothing is written outside the folder you extracted to,
apart from a small settings file described [below](#where-your-settings-live). To
uninstall, delete the folder and, if you want to leave nothing behind, the
`%APPDATA%\sam` folder as well.

Prefer to compile it yourself? See [Building from source](#building-from-source).

## Using it

**Start Steam first.** The status line at the bottom of the sidebar tells you where
you stand: **Steam connected** once a game has loaded successfully, **Steam not
responding** if a call failed, and **Waiting for Steam** before the first answer
arrives. The API does not report a connection until something is actually asked of
it, so "Waiting for Steam" on a fresh launch with no game open is normal.

**Opening a game.** Click it in the sidebar. The sidebar lists games Steam has
*installed*. To reach a game you own but have not installed, type its App ID into
the box at the bottom of the sidebar and press <kbd>Enter</kbd> or **Open**. An App
ID is the number in a game's store URL — `store.steampowered.com/app/367520/`
is App ID `367520`.

**Reading the list.** Each row shows the achievement's title, its description, the
percentage of players who have it, and a status badge: **Unlocked**, **Locked**,
**Pending unlock**, **Pending reset** or **Steam refused**. Hover a row for the
exact percentage, the rarity band and the achievement's internal API name.
Achievements Steam keeps secret until they are earned show as
"Hidden until unlocked" instead of a description.

**Rarity bands** are the conventional five, at the same thresholds and colours the
terminal version used:

| Band | Share of players | Colour |
|---|---|---|
| Legendary | 1% or below | orange |
| Epic | 10% or below | purple |
| Rare | 25% or below | blue |
| Uncommon | 50% or below | green |
| Common | above 50% | white |

**Making changes.** A ticked box means "this should be unlocked". Click a row to
toggle it. Rows that differ from Steam's current state are highlighted and labelled
**Pending unlock** or **Pending reset**, and the count of them appears both in the
toolbar and on the button. Press **Apply *n* changes** to write them. Afterwards the
app re-reads the game from Steam, so what you are looking at is Steam's state and
not a guess. If Steam accepted some writes and refused others, the message says so
and the refused rows are marked individually.

**Bulk actions.** **Unlock all shown** ticks everything the current filter is
displaying, so filtering to Locked first is a quick way to complete a game.
**Discard** returns every row to Steam's current state and throws the pending
changes away.

**Tidying the sidebar.** Hover a game and press the **×** on its right edge to take
it out of the list. Nothing is uninstalled and no achievement is touched. Once
anything is removed, a chip above the list counts them and reveals those games
faded, each with a **+** to put it back. Opening a game by App ID also restores it,
since naming a game is an explicit request to see it. A game that is currently open
stays open when you remove it, so a mis-click cannot throw away pending changes.

**The numbers in the sidebar.** Only the game you have open shows live counts. The
rest show what was true the last time you opened them, labelled "last seen". This
is a limit of the Steam API rather than laziness: Steam initialises against one App
ID per process, so there is no way to read several games' achievements at once.

**Keyboard.** <kbd>F5</kbd> reloads the open game, the same as pressing **Refresh**.

## Command line

```
sam.exe                 pick a game from the window
sam.exe --id 367520     open that App ID straight away
```

`--id` also accepts `-i`, and works for any game on your account whether or not it
is installed.

One quirk of a windowless application: `sam.exe --help` prints nothing, because a
release build has no console attached to print to. That is deliberate — it is what
stops a black console window appearing behind the app. Run `cargo run -- --help`
from a source checkout if you want to read the help text.

## Where your settings live

Settings are written to a single TOML file under `%APPDATA%\sam\`. To find it:

```
dir /s /b "%APPDATA%\sam\*.toml"
```

It holds your sort column and direction, the window size, the last game you had
open, the list of App IDs you removed from the sidebar, and the cached
unlocked/total counts shown as "last seen". Delete the file to reset the
application to its defaults; nothing in it affects Steam or your achievements.

Writes are batched, at most one every couple of seconds, so dragging the window
edge around does not hammer the disk.

**If your Steam is somewhere unusual**, set `SAM_STEAM_PATH` to its folder and that
location is searched first:

```powershell
$env:SAM_STEAM_PATH = 'D:\Steam'
.\sam.exe
```

Otherwise the usual locations under `Program Files` and `Program Files (x86)` are
tried, plus `C:\Steam`.

## How it works

The interface is [egui](https://github.com/emilk/egui), drawn directly to the
window with no browser engine or web view involved. That is why the whole thing is
one self-contained executable of a few megabytes rather than a folder of a hundred
and fifty.

**Finding your games.** Steam offers no "list my library" call that a client can
use, so the sidebar is built by reading Steam's own files: `libraryfolders.vdf` for
the set of library folders, including ones on other drives, and one
`appmanifest_*.acf` per installed game for its App ID and name. Both are plain text
and parsed by hand, with no extra dependency and no network access. Redistributables,
the Steam Linux Runtimes and Proton are filtered out, since they appear as installed
games but are not.

Because that list is rebuilt from disk on every launch, games you remove from the
sidebar have to be remembered in the settings file, or they would come straight
back the next time you start the app.

**Talking to Steam.** `Client::init_app` initialises the Steam API against a single
App ID for the whole process, and its calls block for as long as Steam feels like
taking. So every Steam call is queued onto one dedicated worker thread and run in
order, and the UI sends commands and drains replies once per frame. This is also
the reason sidebar counts cannot be live for every game at once.

That thread is panic-guarded. The Steamworks bindings are FFI and can fail in ways
this code cannot prevent, so a panic is caught and turned into a red message in the
window rather than a disappearing application. The Steamworks SDK also writes
directly to standard output, which a windowless build has nowhere to put, so those
streams are suppressed while a call is in flight.

**Global unlock percentages** arrive asynchronously. Steam is asked for them, then
callbacks are pumped for up to a second and a half until real numbers land —
without that step every achievement reads back as 0.0%.

**Writing.** Unlocking and resetting are separate Steam calls, so the two lists go
over one at a time, each result is recorded individually, and the stats are stored
once at the end. The app then throws away everything it thought it knew and
re-reads the game, because Steam is the only authority on what is actually
unlocked.

Two behaviours are optional at compile time, as `display-names` and
`global-percent`, both on by default. They exist because the underlying
`steamworks` crate has changed the shape of those calls between releases, so
`--no-default-features` is an escape hatch that trades pretty titles and real
percentages for a build that compiles anyway. BUILD.md covers when you would want
it.

## Troubleshooting

This section is about running the app. If a **build** fails, BUILD.md has a section
named for each error it can produce.

| What you see | What it means |
|---|---|
| **Steam not responding**, or "Steam crashed while reading achievements" | Steam is not running, or is running but not signed in. Start it, sign in, then press **Refresh**. |
| **App *n* not in your library** | That App ID is not on the signed-in account, or the number is wrong. Check it against the game's store URL, and check which account Steam is signed in to. |
| The app does not start at all, or reports a missing DLL | `steam_api64.dll` has to sit in the same folder as `sam.exe`. Extract both files out of the zip together. |
| A game you have installed is not in the sidebar | Steam is installed somewhere unusual, so its library files were not found. Set `SAM_STEAM_PATH` as described [above](#where-your-settings-live). You can always reach the game by App ID in the meantime. |
| Every achievement reads 0.0% | The global percentages did not arrive before the deadline, usually a slow first call. Press **Refresh**. |
| Achievement titles look like `DEFEAT_MORPHA` | Steam did not return display names, so the list fell back to internal API names. Press **Refresh**; if it persists for one game, that game simply has no localised strings for them. |
| A row says **Steam refused** | Steam accepted the request and declined that specific achievement. Some games validate achievements on their own servers, and some are driven by stats rather than set directly. There is no way around this from the client. |
| An achievement changed here but the Steam client still shows the old state | The Steam client caches achievement state for its own UI. Restart Steam. The account state is already correct. |
| The window opens completely black | A graphics driver problem with the rendering backend rather than an application error. BUILD.md's "The window opens completely black" covers it. |

## Building from source

You need Rust 1.85 or newer with the MSVC toolchain, and the Microsoft C++ build
tools. Then:

```powershell
git clone https://github.com/volk-xp/steam-achievement-manager-gui
cd steam-achievement-manager-gui
cargo build --release
```

The build produces `target\release\sam.exe` and copies `steam_api64.dll` beside it.
The first build takes several minutes, mostly Steamworks and egui. On Windows,
`build.bat` does the build and copies both files into a folder of your choosing in
one step, and `release.ps1` builds, zips and publishes a GitHub release.

**[BUILD.md](BUILD.md) is the full walkthrough**, including toolchain setup and a
named fix for every error the build can produce.

`eframe` is pinned to 0.31 on purpose. egui renamed `Rounding` to `CornerRadius`
and moved `Margin` to integer fields in that release, so the UI is written against
that exact API. Everything version-sensitive is deliberately confined to
`src/gui/theme.rs`, so if you do widen the pin, the compiler will point at one file.

## Project layout

| Path | What lives there |
|---|---|
| `src/main.rs` | Window setup and the `windows_subsystem` attribute that suppresses the console. |
| `src/args.rs` | The command line, via clap. |
| `src/library.rs` | Steam library discovery: the VDF and ACF parsing. |
| `src/steam.rs` | Every Steamworks call, and nothing else. |
| `src/config.rs` | The settings file, via confy. |
| `src/search.rs` | Fuzzy matching for the search box. |
| `src/gui/app.rs` | State, layout and event handling. |
| `src/gui/worker.rs` | The one thread allowed to talk to Steam. |
| `src/gui/widgets.rs` | Hand-painted rows, chips, badges and buttons. |
| `src/gui/theme.rs` | Colours, fonts, and every egui call that changes between versions. |
| `build.rs` | Copies Steam's runtime library next to the executable. |

