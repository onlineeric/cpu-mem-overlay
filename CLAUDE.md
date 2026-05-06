# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan:
specs/002-overlay-config/plan.md
<!-- SPECKIT END -->

## Rust coding

When writing or modifying Rust code in this repository, always consult the `rust-skills` skill for best-practice guidance and apply the relevant rules. This is also Principle I of the project constitution at `.specify/memory/constitution.md`.

## Common commands

All commands run from the repository root (`C:\repos\cpu-mem-overlay`). The platform is Windows 11 x64; PowerShell is the default shell.

| Task | Command |
|------|---------|
| Build (recommended for dev — startup speed needed for SC-001) | `cargo build --release` |
| Run the overlay | `cargo run --release` |
| Run all unit tests | `cargo test` |
| Run a single test | `cargo test cpu_some_hundred` (substring match against test fn name) |
| Run tests in the binary crate | `cargo test --bin cpu-mem-overlay` |
| Lint | `cargo clippy --all-targets -- -D warnings` |
| Format | `cargo fmt` |

There is no in-app close affordance by design (FR-007). Stop the running overlay with `Ctrl+C` in the terminal or via Task Manager.

## Architecture

This is a **single Rust `bin` crate** at the repo root (`Cargo.toml` + `src/`). There is no library crate, no workspace, and no submodules, but the binary is split into small internal modules because v2 added config parsing, monitor enumeration, and GUI behavior beyond the original v1 single-file app.

The current feature is structured across these concerns:

1. **`src/config.rs`** — reads `<exe-dir>/cpu-mem-overlay.toml` exactly once at startup. Missing file, unparseable TOML, unknown keys, and per-field invalid values silently fall back to defaults. Supported keys are `refresh_interval_ms`, `startup_position`, `anchor_position`, `background_color`, `font_color`, `font_size`, and `draggable`.
2. **`OverlayConfig` / `Anchor` / `StartupPosition`** — the resolved config state. `startup_position` accepts `bottom_right` (default, matching v1) or `bottom_left`; both are taskbar-aware primary-work-area anchors. `anchor_position = [x, y]` remains an explicit virtual-screen coordinate override and falls back to `startup_position` if it is entirely outside all monitor work areas.
3. **`src/color.rs`** — parses config colors as `"#RRGGBB"` (opaque) or `"#RRGGBBAA"` (explicit alpha). `background_color` defaults to fully transparent `#00000000`; `font_color` defaults to white `#FFFFFF`.
4. **`src/sampler.rs`** — owns `MetricSnapshot { cpu: Option<u8>, memory: Option<u8> }` and `Sampler`. Percentages clamp to `0..=100`; non-finite or unavailable values become `None`, rendered as `--`.
5. **`src/app.rs`** — owns `OverlayApp` (`impl eframe::App`), window-size calculation, primary work-area corner helpers, optional drag handling, configured background/font colors, and elapsed-time-gated sampling. `update()` samples only when `refresh_interval` has elapsed, then schedules the next repaint with `ctx.request_repaint_after(...)`; mouse hover/movement must not accelerate metric reads.
6. **`src/monitors.rs`** — enumerates monitor work areas with Win32 `EnumDisplayMonitors` / `GetMonitorInfoW` and provides the pure `is_on_any_work_area` overlap check for validating configured anchors.
7. **`src/main.rs`** — loads config, computes window size and resolved startup position, builds `egui::ViewportBuilder`, and runs `eframe`.

Window setup happens in `main()` via `egui::ViewportBuilder` with `.with_decorations(false)`, `.with_resizable(false)`, `.with_inner_size(compute_window_size(config.font_size))`, `.with_position(resolve_position(...))`, `.with_window_level(egui::WindowLevel::AlwaysOnTop)`, and `.with_transparent(true)`. The binary is built as a Windows GUI subsystem app (`#![cfg_attr(not(test), windows_subsystem = "windows")]`) so launching it from Explorer does not open a console.

Pure logic is unit-tested in each module (`config`, `color`, `sampler`, `monitors`, and `app`). GUI behavior (borderless, always-on-top, transparent background, font color, drag, taskbar-aware startup corners, and refresh cadence) is verified manually against `specs/002-overlay-config/contracts/ui-contract.md`; automated GUI testing remains out of scope.

## Spec Kit workflow

This repo uses Spec Kit. Feature work flows through `specs/<NNN-feature-name>/` artifacts (`spec.md` → `plan.md` → `tasks.md` → implementation). The current active feature is `specs/002-overlay-config/`; its `plan.md`, `research.md`, `data-model.md`, `quickstart.md`, and `contracts/ui-contract.md` are the authoritative source for v2 design decisions. `specs/001-cpu-mem-overlay/` remains the v1 baseline reference.

The project constitution at `.specify/memory/constitution.md` defines four NON-NEGOTIABLE-or-strong gates that apply to every change:

- **I. Rust Best Practices via `rust-skills`** — consult the skill before writing or refactoring Rust.
- **II. Comprehensive Unit Test Coverage (NON-NEGOTIABLE)** — non-trivial logic ships with tests covering each branch and every documented edge case.
- **III. Post-Implementation Self-Review** — re-read the diff against `rust-skills` and project preferences; refactor before declaring the task done.
- **IV. Green Tests Before Done (NON-NEGOTIABLE)** — `cargo test` must pass with zero failures and zero unjustified `#[ignore]`s before a feature is complete.

The `/speckit-plan` Constitution Check section in `plan.md` evaluates each plan against these gates.
