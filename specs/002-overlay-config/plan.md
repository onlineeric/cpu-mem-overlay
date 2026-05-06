# Implementation Plan: Configurable Overlay (V2)

**Branch**: `002-overlay-config` | **Date**: 2026-05-04 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-overlay-config/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Layer a `cpu-mem-overlay.toml` config file (read once at startup, located next to the executable) on top of v1, so users can configure `refresh_interval_ms`, `startup_position` (`bottom_right` or `bottom_left`), `anchor_position` (virtual-screen X/Y), `background_color` (RGBA, transparency-capable), `font_color`, `font_size`, plus a `draggable` toggle that opt-in enables click-drag repositioning. Missing file, unparseable file, unknown keys, and per-field invalid values all silently fall back to v1 defaults. Also fix a v1 regression where mouse hover/movement over the tile drives the metric refresh faster than the configured cadence.

Technical approach (resolved in [research.md](./research.md)): the plan keeps the v1 stack (`eframe`/`egui` + `sysinfo` + `windows`) and adds `serde` + `toml` for parsing. Config loading is split into its own module and per-field validators that consume the raw deserialized struct and produce a fully-defaulted `OverlayConfig`. The hover-driven refresh is fixed by gating `Sampler::sample()` on an elapsed-time check (`Instant`-based) inside `OverlayApp::update`, so input-driven repaints no longer trigger a metric read. Multi-monitor on-screen validation enumerates work areas via Win32 `EnumDisplayMonitors` + `GetMonitorInfoW`. Window transparency is achieved with `ViewportBuilder::with_transparent(true)` plus `App::clear_color` keyed off the configured alpha. Drag-to-reposition uses `ViewportCommand::StartDrag` on press inside the central panel, gated by `OverlayConfig::draggable`. Window size is derived from `font_size` at startup so the two metric lines never clip.

## Technical Context

**Language/Version**: Rust 1.75+ (stable, edition 2021) — already in use for v1.

**Primary Dependencies**:
- `eframe` 0.29 — windowing + immediate-mode GUI, carried from v1. Used additionally for transparent windows (`with_transparent`), drag-to-reposition (`ViewportCommand::StartDrag`), and clear-color override (`App::clear_color`).
- `sysinfo` 0.32 — system metrics, carried from v1.
- `windows` 0.58 — already used in v1 for `SystemParametersInfoW`. Add `Win32_Graphics_Gdi` feature for `EnumDisplayMonitors` / `GetMonitorInfoW` (multi-monitor work-area union).
- **NEW** `serde` (with `derive`) + `toml` — TOML deserialization. Standard, battle-tested, idiomatic Rust config crates.

**Storage**: Single read-only TOML file at `<exe-dir>/cpu-mem-overlay.toml`. Read once at startup (no live-reload, no file-watching, no writes — drag positions are session-only per FR-024).

**Testing**:
- `cargo test` for all pure logic added in v2: config deserialization (every field optional), per-field validation/fallback (`refresh_interval_ms` minimum, `startup_position` accepted values, color hex parsing including alpha=0 and 6-digit/8-digit forms, `font_color`, `font_size` lower bound, anchor coordinate parsing), the multi-monitor on-screen overlap check (with synthetic work-area rects), and the elapsed-time gating logic for hover-driven-refresh suppression.
- Manual smoke test on Windows 11 against the acceptance scenarios in [spec.md](./spec.md) and the GUI-only items in [contracts/ui-contract.md](./contracts/ui-contract.md) (transparency, drag, window auto-fit, anchor placement, hover-suppression observed visually).

**Target Platform**: Windows 11 x64 (carried from v1).

**Project Type**: Desktop application — single `bin` crate, carried from v1.

**Performance Goals**:
- All v1 goals preserved: < 1% steady-state CPU, < 50 MB resident memory, cold-start < 3 s.
- Refresh cadence honors the configured `refresh_interval_ms` (default 1000 ms; minimum 100 ms per spec Assumptions). With the hover-fix in place, the actual sampling rate matches the configured interval ±1 tick over a 1-minute observation (SC-002), regardless of mouse activity.

**Constraints**:
- Silent fallback for every config error (no log file, no stderr, no on-screen indicator — FR-006a).
- Config is read exactly once at startup.
- A fully-transparent background MUST NOT make the window click-through; mouse events still go to the overlay (FR-014a).
- A drag-repositioned overlay MUST NOT persist to the config file (FR-024).
- All v1 functional requirements (display format, always-on-top, borderless, `--` fallback, no in-app close affordance) MUST continue to hold (FR-025).

**Scale/Scope**: Single user, single window, single feature. v1 was ~150 LOC; v2 is estimated < 400 LOC across a small module split.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Evaluated against `.specify/memory/constitution.md` (v1.0.0).

- **Principle I — Rust Best Practices via rust-skills**: PASS. Implementation will consult `rust-skills` for the categories used by this feature: error handling (config loading and per-field validators must use idiomatic `Result`/`Option` flows, not `panic!` or `unwrap()`), API design (the `OverlayConfig` public surface), serde idioms (struct deserialization with all-optional fields), Win32/FFI safety (the `EnumDisplayMonitors` callback path), and testing (table-driven test patterns for validators).
- **Principle II — Comprehensive Unit Test Coverage (NON-NEGOTIABLE)**: PASS. The plan identifies every non-trivial pure-logic seam and pairs each with explicit test cases (see Testing above and Phase 1 design below). UI behavior (transparency, drag, anchor placement) is covered by manual smoke tests recorded in [quickstart.md](./quickstart.md), consistent with v1's R-008 decision and explicitly permitted by Principle II.
- **Principle III — Post-Implementation Self-Review**: PASS. The tasks list will include a final self-review task before "done": re-read the diff against `rust-skills` and the project coding preferences (clean naming, single responsibility, DRY, explicit-over-implicit, purpose-built libraries) and refactor any rough edges before marking the feature complete.
- **Principle IV — Green Tests Before Done (NON-NEGOTIABLE)**: PASS. The tasks list will end with `cargo test` + `cargo clippy --all-targets -- -D warnings` gates that MUST pass with zero failures and zero unjustified `#[ignore]`s before the feature is declared complete.

**Initial gate**: PASS — all four principles addressed by design; no exceptions to record in Complexity Tracking.
**Post-design re-check**: PASS — Phase 1 artifacts (data-model.md, contracts/ui-contract.md, quickstart.md) preserve all four commitments. The module split keeps each file small and single-responsibility, the validator design makes per-field testing straightforward, and the manual-smoke matrix covers every spec acceptance scenario that cannot be unit-tested.

## Project Structure

### Documentation (this feature)

```text
specs/002-overlay-config/
├── plan.md              # This file (/speckit-plan command output)
├── spec.md              # Feature specification (already produced by /speckit-specify)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/
│   └── ui-contract.md   # Phase 1 output — visual/window contract for v2 (delta from v1)
├── checklists/
│   └── requirements.md  # Spec quality checklist (already produced by /speckit-specify)
└── tasks.md             # Phase 2 output (/speckit-tasks command — NOT created here)
```

### Source Code (repository root)

```text
cpu-mem-overlay/                  # Repo root (carried from v1)
├── Cargo.toml                    # Bin crate manifest; v2 adds serde + toml deps and Win32_Graphics_Gdi feature
├── cpu-mem-overlay.toml.example  # NEW — committed example config showing every supported key with its default
├── src/
│   ├── main.rs                   # Entry point: load config, build viewport, run eframe
│   ├── config.rs                 # NEW — OverlayConfig + RawConfig (serde) + load_or_default + per-field validators
│   ├── color.rs                  # NEW — hex-string color parsing ("#RRGGBB" / "#RRGGBBAA")
│   ├── monitors.rs               # NEW — Win32 EnumDisplayMonitors-based work-area union and on-screen overlap check
│   ├── sampler.rs                # MetricSnapshot + Sampler (extracted from v1 main.rs unchanged)
│   └── app.rs                    # OverlayApp (eframe::App impl): interval-gated refresh, drag handling, clear_color
└── specs/                        # Spec Kit feature artifacts (this feature lives at specs/002-overlay-config/)
```

**Structure Decision**: Continue with a single `bin` crate at the repo root, but split `src/main.rs` into focused modules along responsibility lines as v2 nearly triples the LOC count and adds three independent concerns (config parsing, color parsing, multi-monitor checks). Each module is small, single-responsibility, and individually unit-testable, satisfying the constitution's "Small, focused units" standard. Module names match what they own; nothing is split prematurely (e.g., the `Sampler` and `MetricSnapshot` move together because they are one concern).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations — section intentionally empty.
