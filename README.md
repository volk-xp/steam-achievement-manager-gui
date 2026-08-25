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
     docs/screenshot.png, then uncomment the line below.

<p align="center">
  <img src="docs/screenshot.png" alt="The library sidebar and achievement list" width="900">
</p>
-->

## Overview

Pick a game from your library on the left and its achievements load on the right.
Tick what you want unlocked, untick what you want reset, then press **Apply
changes**. Nothing is written to Steam until you press that button, and every row
you have changed is marked so you can see exactly what is about to happen before
it happens.

## Features

| | |
|---|---|
| **Your library, listed automatically** | Installed games are discovered from Steam's own library files. Games you own but have not installed can be opened by App ID. |
| **A list you can prune** | Hover any game and press **×** to take it out of the sidebar. Nothing is uninstalled and no achievement is touched; the game is simply no longer in your way. A **removed** chip above the list brings them back. |
| **Rarity at a glance** | Every achievement shows its global unlock percentage and a colour band, so the ones almost nobody has are obvious. |
| **Nothing happens by accident** | Changes are staged locally and applied in one deliberate action. Pending unlocks and pending resets are labelled separately. |
| **Honest about what it knows** | After writing, the app re-reads everything from Steam instead of trusting its own bookkeeping. Cached sidebar counts are labelled "last seen" rather than presented as live. |
| **Bulk actions** | **Unlock all shown** ticks everything the current filter is showing. **Discard** throws away every pending change. |
| **Filter, sort, search** | Filter by All, Locked or Unlocked, sort by rarity, name or state in either direction, and find a game by typing in the search box. |
| **Remembers your setup** | Window size, sort order, the games you removed and the last game you opened all persist between runs. |

## Installation

Download the latest zip from the [releases page](https://github.com/volk-xp/steam-achievement-manager-gui/releases),
extract it anywhere, and run `sam.exe`.

Keep `steam_api64.dll` in the same folder as the executable. It is Steam's
redistributable library and the application will not start without it.

## Requirements

Steam must be installed, running, and signed in. Achievements can only be changed
for games on the signed-in account.

## Usage

Launch with no arguments to pick a game from the window, or jump straight into one
by App ID:

```
sam.exe --id 367520
```

Press **Refresh**, or <kbd>F5</kbd>, to re-read the open game from Steam.

To tidy the sidebar, hover a game and press the **×** on its right edge. The chip
above the list counts what you have removed and reveals those games so you can put
one back.

Settings are stored under `%APPDATA%\sam\`. Delete that folder to reset the
application to its defaults.

## Building from source

You need Rust 1.85 or newer with the MSVC toolchain, and the Microsoft C++ build
tools. Then:

```powershell
git clone https://github.com/volk-xp/steam-achievement-manager-gui
cd steam-achievement-manager-gui
cargo build --release
```

The build produces `target\release\sam.exe` and copies `steam_api64.dll` beside
it. On Windows, `build.bat` does the build and copies both files into a folder of
your choosing in one step.

**[BUILD.md](BUILD.md) is the full walkthrough**, including toolchain setup and a
named fix for every error the build can produce.

## How it works

The interface is [egui](https://github.com/emilk/egui), drawn directly to the
window with no browser engine or web view involved, which is why the whole thing
is a single self-contained executable.

Steam only lets a process initialise against one App ID at a time, so all Steam
work runs on a single dedicated worker thread and the UI communicates with it over
channels. That thread is panic-guarded: if the Steamworks FFI falls over, you get
an error message in the window rather than a vanished application. It also means
per-game counts in the sidebar cannot be live for every game at once, which is why
they are labelled rather than silently stale.

Achievement rarity uses the conventional five bands, at the same thresholds and
colours the terminal version used: **Legendary** at 1% or below, **Epic** at 10% or
below, **Rare** at 25%, **Uncommon** at 50%, and **Common** above that.

## Acknowledgements

This is a fork of [mbwilding/steam-achievement-manager](https://github.com/mbwilding/steam-achievement-manager)
by Matthew Wilding. The Steam integration, the fuzzy search and the achievement
rarity bands are that project's work and are largely unchanged here. What this
fork adds is a windowed interface in place of the original terminal one, plus
automatic Steam library discovery.

## License

[MIT](LICENSE).

Copyright (c) 2025 Matthew Wilding for the original work. Modifications in this
fork are released under the same license, and the original copyright notice is
retained as MIT requires.
