# Building sam.exe

Follow this once and you get a single windowed application: `sam.exe`, plus one
DLL that has to stay beside it. No terminal window, no Python, no Node.

Everything below assumes Windows. Times are for a first build on a mid-range
machine; later builds take seconds.

---

## Step 1 — Install the Rust toolchain (once, ~5 minutes)

Download and run the installer from <https://rustup.rs>.

When it asks, choose **1) Proceed with standard installation**. That gives you
the `stable-x86_64-pc-windows-msvc` toolchain, which is the one you want.

This project uses Rust edition 2024, so you need **Rust 1.85 or newer**. If you
installed Rust some time ago, update it:

```
rustup update stable
```

Check what you have:

```
rustc --version
```

You want `1.85.0` or higher.

## Step 2 — Install the Microsoft C++ build tools (once, ~10 minutes)

Rust on Windows uses Microsoft's linker. rustup usually offers to install it for
you; if it did, skip this step.

Otherwise get **Visual Studio Build Tools** from
<https://visualstudio.microsoft.com/visual-cpp-build-tools/>, run the installer
and tick:

- **Desktop development with C++**
- Under Individual components, confirm **Windows 11 SDK** (or Windows 10 SDK) is
  selected

You do not need full Visual Studio. The Build Tools are enough.

### Shortcut: skip steps 3 to 5

Once Rust and the C++ tools are installed, `build.bat` in this folder does the
rest — it builds, retries if the Steam DLL was not ready in time, and copies both
files into a folder you can run the app from. Double-click it, or:

```
build.bat
```

It defaults to `C:\Users\MSI\Videos\Volk SAM`. To send it somewhere else, pass
the folder (quote it if the path has a space):

```
build.bat "D:\Games\SAM"
```

The manual steps below are the same thing spelled out, and are worth reading once
so you know what the script is doing.

## Step 3 — Open a terminal in the project folder

Press <kbd>Win</kbd>+<kbd>R</kbd>, type `cmd`, press Enter, then:

```
cd C:\Users\MSI\Music\SAM\steam-achievement-manager-main
```

Or open the folder in Explorer, click the address bar, type `cmd` and press
Enter.

## Step 4 — Build

```
cargo build --release
```

The first run downloads about 250 crates and compiles all of them, which takes
roughly **5 to 12 minutes**. You will see a long list of `Compiling ...` lines.
Warnings are normal. What matters is the last line:

```
Finished `release` profile [optimized] target(s) in 7m 41s
```

You may also see `Updating crates.io index` and a note that `Cargo.lock` was
rewritten. That is expected — the checked-in lock file still describes the old
terminal build, and cargo replaces it on the first build of the new one. Commit
the new `Cargo.lock` if you use git.

### If this is the very first build and it stops with a warning about
### `steam_api64.dll`

Run `cargo build --release` a second time. The Steamworks SDK is unpacked by a
dependency during the same build, and on a cold start the copy step can run
before the file exists. The second build always finds it.

## Step 5 — Collect the output

Look in:

```
C:\Users\MSI\Music\SAM\steam-achievement-manager-main\target\release\
```

You need exactly two files:

| File | What it is |
|---|---|
| `sam.exe` | The application, around 12–18 MB |
| `steam_api64.dll` | Steam's runtime library, around 250 KB |

**These two must stay in the same folder.** Copy them anywhere you like — the
Desktop, a `C:\Tools\SAM` folder, a USB stick — as long as they travel together.
`sam.exe` will not start without the DLL beside it.

Everything else in `target\` is build scratch and can be ignored.

## Step 6 — Run it

1. Start Steam and sign in. Leave it running.
2. Double-click `sam.exe`.

The window opens with your installed games listed down the left. Click one and
its achievements load. Tick the ones you want unlocked, untick ones you want
reset, then press **Apply changes**.

---

## Using the app

**Opening a game.** Click it in the sidebar. The sidebar lists games Steam has
*installed*, read from Steam's own `appmanifest` files. To reach a game you own
but have not installed, type its App ID in the box at the bottom of the sidebar
and press Enter. You can also launch straight into one:

```
sam.exe --id 367520
```

One quirk of a windowless build: `sam.exe --help` prints nothing, because the
release build has no console attached to print to. `--id` still works. Run
`cargo run -- --help` if you want to read the help text.

**The numbers in the sidebar.** Only the game you have open shows live counts.
The others show what was true the last time you opened them, labelled
"last seen". This is a hard limit of the Steam API, not laziness: Steam
initialises against one App ID per process, so there is no way to read six
games' achievements at once.

**Rarity.** The coloured square beside each achievement is how rare it is, using
the same bands the terminal version used: orange under 1% of players, purple
under 10%, blue under 25%, green under 50%, white above that. The percentage is
printed to the left of the status badge. Hover a row for the exact figure.

**Applying changes.** A ticked box means "this should be unlocked". Rows that
differ from Steam's current state turn blue and read "Pending unlock" or
"Pending reset". Nothing is written until you press **Apply changes**. After
writing, the app re-reads everything from Steam rather than trusting its own
bookkeeping, so what you see afterwards is the truth.

**Keyboard.** <kbd>F5</kbd> reloads the open game.

**Settings.** Sort order, window size, the last game you opened and the cached
counts are saved under `%APPDATA%\sam\`. To find the exact file:

```
dir /s /b "%APPDATA%\sam\*.toml"
```

Delete it to reset everything back to defaults.

---

## Rebuilding while you change things

`Cargo.toml` already sets `strip = true`, `lto = true` and `codegen-units = 1`
for release builds, so `cargo build --release` is as small and fast as this
project gets. There is nothing to add afterwards.

For quick iteration use the debug profile. It compiles in about 20 seconds
instead of minutes, but the exe is larger, the UI is slower, and a console window
appears behind it (which is deliberate — `windows_subsystem = "windows"` is
applied to release builds only, so you can still see `println!` output while
developing):

```
cargo run
```

---

## If the build fails

Work through these in order. The three "unverified call" sections at the end
cover the only places where this code depends on an API I could not check
against the live crate registry, and each has a one-line fix.

### `error: linker 'link.exe' not found`

Step 2 was skipped or incomplete. Install the Visual Studio Build Tools with the
**Desktop development with C++** workload, then close and reopen your terminal.

### `error: package 'sam' cannot be built because it requires rustc 1.85`

Your Rust is too old. Run `rustup update stable`.

### `error: failed to run custom build command for 'sam'` mentioning
### `steam_api files not found`

Run `cargo build --release` again. If it still fails, copy the DLL by hand:

```
dir /s /b target\release\build\steamworks-sys-*\out\steam_api64.dll
copy <the path it printed> target\release\
```

### `failed to fetch 'https://github.com/rust-lang/crates.io-index'`

No internet, or a proxy or VPN is blocking it. The first build must download
dependencies; later builds work offline.

### `error[E0599]: no method named 'get_achievement_display_attribute'`

This is the achievement *title* and *description* lookup. If your version of the
`steamworks` crate does not expose it, switch the feature off:

```
cargo build --release --no-default-features
```

The app then lists raw Steam keys such as `DefeatMorpha` instead of "Nailed it".
Everything else works identically. This is the guaranteed-to-compile fallback —
if you hit any error you cannot place, try this command first.

### `error[E0599]: no method named 'run_callbacks'`

This is the pump that waits for global unlock percentages to arrive. Turn off
just that feature:

```
cargo build --release --no-default-features --features display-names
```

Rarity colours will read 0.0% for every achievement, and the rarity sort becomes
meaningless. Nothing else is affected.

### `error[E0599]: no method named 'corner_radius'` (or `inner_margin`,
### or a complaint about `Arc<FontData>`)

Your `eframe` resolved to something other than 0.31. `Cargo.toml` pins
`eframe = "0.31"` for exactly this reason; check it has not been widened. Then:

```
cargo update -p eframe --precise 0.31.1
cargo build --release
```

If you deliberately want a newer egui, every call that changed lives in
`src/gui/theme.rs`, and the alternative spelling for each is written in a
comment directly above it. On egui 0.30 and older: `corner_radius` was
`rounding`, integer margins were floats, `FontData` was not wrapped in an `Arc`,
and `ScrollArea::id_salt` was `id_source`.

### The window opens but says "Steam not responding"

Steam is not running, or you are not signed in, or the App ID is not on your
account. Start Steam, sign in, then press **Refresh**.

### The window opens completely black, or does not appear at all

egui draws through OpenGL. Update your graphics driver, and if you are on a
laptop with two GPUs, right-click `sam.exe` and run it on the integrated one.

If it still fails, switch renderer. Add the `wgpu` backend to `Cargo.toml`:

```toml
eframe = { version = "0.31", features = ["wgpu"] }
```

then in `src/main.rs`, inside `NativeOptions`, add:

```rust
renderer: eframe::Renderer::Wgpu,
```

and rebuild. That path uses Direct3D 12 on Windows instead of OpenGL.

---

## What changed from the terminal version

The terminal interface has been removed, not hidden. `src/tui/` is gone, and
`ratatui` and `crossterm` are no longer dependencies. Everything the old
interface did is still here:

| Old | New |
|---|---|
| Typed the App ID at launch | Click a game in the sidebar, or type an App ID |
| Fuzzy search over achievements | Fuzzy search over the game library |
| Sort by rarity / name / state | The **Sort** chip, same three columns |
| Space to toggle, Enter to apply | Click a row, then **Apply changes** |
| Rarity colours in the list | The same colours, as swatches and percentages |

Kept from the original, untouched: `src/search.rs` (the fuzzy matcher), the
Steam read and write calls in `src/steam.rs`, and the rarity thresholds.

The old files are in `.backup/` if you want to compare.
