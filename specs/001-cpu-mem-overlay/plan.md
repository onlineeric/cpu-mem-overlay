# Implementation Plan: CPU/Memory Overlay (V1)

**Branch**: `001-cpu-mem-overlay` | **Date**: 2026-05-04 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-cpu-mem-overlay/spec.md`

**Note**: This template is filled in by the `/speckit-plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Build a tiny native Windows 11 desktop overlay in Rust that renders two lines — `CPU xx%` and `MEM xx%` — in a borderless, always-on-top window pinned near the taskbar. The display refreshes once per second from system performance counters. On metric read failure the line falls back to `--` and the app keeps running.

Technical approach (resolved in Phase 0): a single binary built with `eframe`/`egui` for the borderless always-on-top window, and `sysinfo` for sampling overall CPU and memory usage. A 1 Hz refresh loop is driven by `egui`'s `request_repaint_after` so the app idles between ticks. See [research.md](./research.md) for the decisions and alternatives.

## Technical Context

**Language/Version**: Rust 1.75+ (stable, edition 2021), already installed on the developer machine — no toolchain setup required.
**Primary Dependencies**:
- `eframe` (latest 0.x) — windowing + immediate-mode GUI (`egui`); supports borderless, always-on-top, transparent windows on Windows.
- `sysinfo` (latest 0.x) — cross-platform system metrics; provides `global_cpu_usage()` and total/used memory in bytes.

**Storage**: N/A — no persistence in v1.
**Testing**: `cargo test` for the small pure-logic surface (formatting, fallback handling); manual visual smoke test on Windows 11 against the acceptance scenarios in [spec.md](./spec.md). UI behavior (always-on-top, borderless, position) is verified manually because automated GUI testing is out of scope for v1.
**Target Platform**: Windows 11 x64. No support for Windows 10 or other OS in v1.
**Project Type**: Desktop application — single binary (`bin` crate).
**Performance Goals**:
- < 1% steady-state CPU on a modern desktop while idling between refreshes.
- < 50 MB resident memory.
- 1.0 s refresh cadence with no observed gap > 2 s (per SC-002).
- Cold start to first frame visible in < 3 s (per SC-001).
**Constraints**:
- Borderless, no title bar, always-on-top, fixed default position near the taskbar (bottom-right by default).
- Must not crash on transient metric read failure (must show `--` and recover next tick).
- No settings UI, tray icon, drag, or right-click menu in v1.
**Scale/Scope**: Single user, single window, single feature. Estimated < 200 LOC of Rust including the GUI.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The repository's `.specify/memory/constitution.md` is the unmodified Spec Kit placeholder — it contains no project-specific principles, gates, or constraints. As a result there are no project-specific gates to evaluate against this plan.

**Initial gate**: PASS (vacuously — no gates defined).
**Post-design re-check**: PASS (vacuously — no gates defined).

If the project later adopts a real constitution, this section should be revisited.

## Project Structure

### Documentation (this feature)

```text
specs/001-cpu-mem-overlay/
├── plan.md              # This file (/speckit-plan command output)
├── spec.md              # Feature specification (already produced by /speckit-specify)
├── research.md          # Phase 0 output (/speckit-plan command)
├── data-model.md        # Phase 1 output (/speckit-plan command)
├── quickstart.md        # Phase 1 output (/speckit-plan command)
├── contracts/
│   └── ui-contract.md   # Phase 1 output — visual/window contract for v1
├── checklists/
│   └── requirements.md  # Spec quality checklist (already produced by /speckit-specify)
└── tasks.md             # Phase 2 output (/speckit-tasks command — NOT created here)
```

### Source Code (repository root)

```text
cpu-mem-overlay/                  # Repo root
├── Cargo.toml                    # Single bin crate manifest (created by /speckit-implement)
├── src/
│   └── main.rs                   # App entry: window setup, refresh loop, rendering
├── specs/
│   └── 001-cpu-mem-overlay/      # This feature's planning artifacts
└── rust-test/                    # Pre-existing empty scratch dir; not used by this feature
```

**Structure Decision**: Single Rust `bin` crate at the repository root. The whole feature is one small binary, so a flat layout (`Cargo.toml` + `src/main.rs`) is the right scope — no library split, no submodules, no workspace. The pre-existing empty `rust-test/` directory at the repo root was a toolchain check by the user; it is not part of this feature and can be left alone or removed at the user's discretion. The crate manifest and `src/main.rs` will be created by `/speckit-implement` (this command produces only the planning artifacts above).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations — section intentionally empty.
