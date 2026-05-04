# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan:
specs/001-cpu-mem-overlay/plan.md
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
| Run only tests in `main.rs`'s `tests` module | `cargo test --bin cpu-mem-overlay tests::` |
| Lint | `cargo clippy --all-targets -- -D warnings` |
| Format | `cargo fmt` |

There is no in-app close affordance by design (FR-007). Stop the running overlay with `Ctrl+C` in the terminal or via Task Manager.

## Architecture

This is a **single Rust `bin` crate** at the repo root (`Cargo.toml` + `src/main.rs`, ~150 LOC). There is no library split, no workspace, no submodules. The pre-existing empty `rust-test/` directory is unrelated to this feature.

The whole feature is structured as four small concerns inside `src/main.rs`:

1. **`MetricSnapshot { cpu: Option<u8>, memory: Option<u8> }`** — the only in-memory entity. `Some(0..=100)` for a valid reading, `None` for "unavailable on this tick" (covers the very first CPU tick before `sysinfo` has a usable reading, and any transient read failure). Per-field, so one metric can fall back to `--` while the other still renders normally.
2. **`Sampler`** — wraps a `sysinfo::System`. Calls `refresh_cpu_usage()` and `refresh_memory()` once at startup and again each tick, then derives a `MetricSnapshot` via the pure helpers `read_cpu_pct` / `read_memory_pct` (which clamp to `0..=100` and convert non-finite values to `None` defensively).
3. **`OverlayApp` (`impl eframe::App`)** — the GUI. Each `update()` call samples once, renders two `egui::Label`s, and **schedules the next repaint via `ctx.request_repaint_after(Duration::from_secs(1))`**. This is the entire refresh loop — no background thread, no OS timer. When idle, `eframe` parks the event loop until the next scheduled repaint, which is what keeps steady-state CPU near 0%.
4. **`primary_work_area_bottom_right`** — one-shot Win32 call to `SystemParametersInfoW(SPI_GETWORKAREA, …)` (via the `windows` crate) to compute the bottom-right anchor of the *work area* (not the full screen rect, so the overlay doesn't overlap the taskbar). Falls back to a hardcoded 1920×1040 origin if the call fails.

Window setup happens in `main()` via `egui::ViewportBuilder` with `.with_decorations(false)`, `.with_resizable(false)`, `.with_window_level(egui::WindowLevel::AlwaysOnTop)`, fixed `WINDOW_SIZE`, and the computed position. The binary is built as a Windows GUI subsystem app (`#![cfg_attr(not(test), windows_subsystem = "windows")]`) so launching it doesn't open a console.

Pure logic (`format_line`, percentage clamping, `--` fallback) is unit-tested at the bottom of `src/main.rs`. GUI behavior (borderless, always-on-top, position, refresh cadence) is verified by eye against `specs/001-cpu-mem-overlay/contracts/ui-contract.md` — automated GUI testing is intentionally out of scope for v1 (R-008 in `research.md`).

## Spec Kit workflow

This repo uses Spec Kit. Feature work flows through `specs/<NNN-feature-name>/` artifacts (`spec.md` → `plan.md` → `tasks.md` → implementation). The current and only feature is `specs/001-cpu-mem-overlay/`; its `plan.md`, `research.md`, `data-model.md`, and `contracts/ui-contract.md` are the authoritative source for design decisions.

The project constitution at `.specify/memory/constitution.md` defines four NON-NEGOTIABLE-or-strong gates that apply to every change:

- **I. Rust Best Practices via `rust-skills`** — consult the skill before writing or refactoring Rust.
- **II. Comprehensive Unit Test Coverage (NON-NEGOTIABLE)** — non-trivial logic ships with tests covering each branch and every documented edge case.
- **III. Post-Implementation Self-Review** — re-read the diff against `rust-skills` and project preferences; refactor before declaring the task done.
- **IV. Green Tests Before Done (NON-NEGOTIABLE)** — `cargo test` must pass with zero failures and zero unjustified `#[ignore]`s before a feature is complete.

The `/speckit-plan` Constitution Check section in `plan.md` evaluates each plan against these gates.
