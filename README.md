# cpu-mem-overlay

A tiny always-on-top Windows overlay that shows current CPU and memory usage. Borderless, click-through-friendly when transparent, and quiet when idle (event-loop parks between ticks).

Written in Rust with [`eframe`/`egui`](https://github.com/emilk/egui) and [`sysinfo`](https://github.com/GuillaumeGomez/sysinfo).

## Features

- Two-line readout: `CPU: NN%` and `MEM: NN%`, refreshing on a fixed cadence (default 1000 ms).
- Always-on-top, undecorated, non-resizable window.
- Anchored by default to the bottom-right of the primary monitor's work area (taskbar-aware), with bottom-left available via config.
- Fully configurable via a `cpu-mem-overlay.toml` file dropped next to the executable. All keys optional; missing or invalid values silently fall back to defaults.

## Build & run

Requires Rust (stable) on Windows 11 x64.

```powershell
cargo build --release
cargo run --release
```

The overlay has no in-app close affordance by design. Stop it with `Ctrl+C` in the terminal or via Task Manager.

## Configuration

Place `cpu-mem-overlay.toml` in the same directory as the built `cpu-mem-overlay.exe`:

```toml
refresh_interval_ms = 1000          # min 100
startup_position    = "bottom_right" # "bottom_right" or "bottom_left"
anchor_position     = [100, 100]     # optional exact virtual-screen pixels; off-screen rolls back to startup_position
background_color    = "#00000000"   # RGBA hex; alpha 00 = fully transparent
font_color          = "#FFFFFF"     # RGBA hex; black "#000000", white "#FFFFFF"
font_size           = 12.0          # max 256
draggable           = false         # click-drag to reposition for the session
```

Behavior:

- Missing file → all defaults.
- Unparseable TOML → all defaults.
- Per-key invalid value → that key reverts to default; siblings still apply.
- Unknown keys are ignored (forward-compatible).
- Errors are silent — no log file, no on-screen indicator. Debug typos by observing whether the setting visibly took effect.

## Tests

```powershell
cargo test                              # all
cargo test cpu_some_hundred             # single test by substring
cargo clippy --all-targets -- -D warnings
cargo fmt
```

GUI behavior (borderless, always-on-top, anchor) is verified by eye against `specs/001-cpu-mem-overlay/contracts/ui-contract.md`.

## Project layout

Single-binary crate. The whole feature lives in `src/`:

- `src/main.rs` — window setup, `OverlayApp` (`impl eframe::App`), refresh loop, primary work-area lookup.
- `src/config.rs` — TOML parsing, validation, and defaults.
- `specs/` — Spec Kit artifacts (`spec.md`, `plan.md`, `tasks.md`) for each feature.
- `.specify/memory/constitution.md` — project gates (Rust best practices, unit tests, self-review, green tests).

## License

[MIT](LICENSE)
