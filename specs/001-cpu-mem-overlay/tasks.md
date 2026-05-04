---
description: "Task list for CPU/Memory Overlay (V1)"
---

# Tasks: CPU/Memory Overlay (V1)

**Input**: Design documents from `/specs/001-cpu-mem-overlay/`
**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/ui-contract.md](./contracts/ui-contract.md), [quickstart.md](./quickstart.md)

**Tests**: Limited unit tests are included per R-008 in `research.md` (covers the percentage formatter and `--` fallback only — the single error-prone seam in the codebase). GUI behavior is verified manually against the UI contract; no GUI test automation in v1.

**Organization**: There is exactly one user story (US1, P1) in this feature. Tasks are still grouped by phase per the template, but the User Stories section contains a single phase.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (US1 only in v1)
- Include exact file paths in descriptions

## Path Conventions

- Single Rust `bin` crate at the repository root (`C:\repos\cpu-mem-overlay\`).
- `Cargo.toml` at the root.
- `src/main.rs` is the only source file in v1 — the codebase is small enough that submodules would be overengineering. Tests live in a `#[cfg(test)] mod tests` block at the bottom of `src/main.rs`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialize the Rust crate at the repository root.

- [ ] T001 Create `Cargo.toml` at the repository root (`C:\repos\cpu-mem-overlay\Cargo.toml`) declaring a `[package]` named `cpu-mem-overlay` with `edition = "2021"`, a single `[[bin]]` named `cpu-mem-overlay` with `path = "src/main.rs"`, and a `[profile.release]` with `lto = true`, `codegen-units = 1`, `strip = true` to keep the release binary small.
- [ ] T002 Add runtime dependencies to `C:\repos\cpu-mem-overlay\Cargo.toml`: `eframe` (latest 0.x, default features) and `sysinfo` (latest 0.x, default features). Run `cargo build` once locally to populate `Cargo.lock` and confirm the dependency graph resolves.
- [ ] T003 [P] Create `C:\repos\cpu-mem-overlay\src\main.rs` with an empty `fn main() {}` placeholder so `cargo check` succeeds before any feature code lands. (No need to update `.gitignore` — the existing root `.gitignore` already covers `target/` and Rust build artifacts.)

**Checkpoint**: `cargo build` succeeds against an empty `main` and produces `target/debug/cpu-mem-overlay.exe`.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Define the in-memory data shape and the pure formatter that the UI layer depends on. Both live in `src/main.rs`.

**⚠️ CRITICAL**: User-story implementation (Phase 3) depends on these being in place.

- [ ] T004 In `C:\repos\cpu-mem-overlay\src\main.rs`, define the `MetricSnapshot` struct with two `Option<u8>` fields (`cpu`, `memory`) per [data-model.md](./data-model.md). No methods yet — just the struct and a `Default` impl that returns `MetricSnapshot { cpu: None, memory: None }` so the very first frame renders cleanly before any sample is taken.
- [ ] T005 In `C:\repos\cpu-mem-overlay\src\main.rs`, add a free function `fn format_line(label: &str, value: Option<u8>) -> String` that returns `format!("{label} {n}%")` for `Some(n)` (with `n` clamped to `0..=100` defensively) and `format!("{label} --")` for `None`. This is the single seam covered by the unit tests in T012.

**Checkpoint**: `cargo build` and `cargo test` (with no tests yet) both succeed; the data model and formatter are callable from the rest of `main.rs`.

---

## Phase 3: User Story 1 - At-a-glance CPU and memory monitoring (Priority: P1) 🎯 MVP

**Goal**: A borderless, always-on-top overlay window appears near the taskbar showing `CPU xx%` and `MEM xx%`, refreshing once per second, with `--` fallback on read failure.

**Independent Test**: Run `cargo run --release` on a Windows 11 machine. Within 3 s a small borderless window appears in the bottom-right corner. Both lines update at least once per second. The window stays on top when other apps are maximized. Walk through all nine manual checks in [contracts/ui-contract.md](./contracts/ui-contract.md) §4 and confirm each passes.

### Tests for User Story 1 (limited per R-008)

> Pure-logic tests only. Write these tests **before** wiring them to a working implementation, run `cargo test` to confirm they fail against the current `format_line`, then proceed.

- [ ] T006 [P] [US1] In `C:\repos\cpu-mem-overlay\src\main.rs`, add a `#[cfg(test)] mod tests { ... }` block with unit tests for `format_line`: covers `Some(0)` → `"CPU 0%"`, `Some(7)` → `"CPU 7%"`, `Some(100)` → `"CPU 100%"`, `None` → `"CPU --"`, the same four cases for `MEM`, and a clamping case where a hypothetical out-of-range source value (e.g., `Some(123)` constructed in the test) renders as `"CPU 100%"`. Confirm `cargo test` lists each as a separate test.

### Implementation for User Story 1

- [ ] T007 [P] [US1] In `C:\repos\cpu-mem-overlay\src\main.rs`, add a `Sampler` struct that owns a `sysinfo::System` and exposes `fn new() -> Self` (calling `refresh_cpu_usage()` and `refresh_memory()` once at construction so the next read produces usable values) and `fn sample(&mut self) -> MetricSnapshot` which calls `refresh_cpu_usage()` + `refresh_memory()`, reads `global_cpu_usage()` and computes `used_memory * 100 / total_memory`, rounds each to `u8`, and returns the snapshot. Wrap each metric read so that any panic-free failure path (e.g., `total_memory() == 0`) yields `None` for that field rather than crashing — see FR-008.
- [ ] T008 [US1] In `C:\repos\cpu-mem-overlay\src\main.rs`, add `fn primary_work_area_bottom_right(window_size: egui::Vec2, margin_px: f32) -> egui::Pos2` that computes the overlay's top-left position so the window sits flush against the bottom-right corner of the primary monitor's work area (taskbar-aware), with `margin_px` (default 12.0) of clearance on each side. Per R-004, prefer the path through `eframe`/`egui_winit` monitor info; if that proves awkward, fall back to a single Win32 `SystemParametersInfoW(SPI_GETWORKAREA, ...)` call via the `windows` crate (add it as a minimal dependency in `Cargo.toml` if needed). Either implementation must compile on Windows 11 x64 only — no other targets required.
- [ ] T009 [US1] In `C:\repos\cpu-mem-overlay\src\main.rs`, define an `OverlayApp` struct holding `Sampler` and the latest `MetricSnapshot`, implementing `eframe::App::update` so each frame: (a) calls `sampler.sample()` to refresh the snapshot, (b) renders a `CentralPanel` containing two `egui::Label`s built from `format_line("CPU", snapshot.cpu)` and `format_line("MEM", snapshot.memory)` stacked vertically, (c) calls `ctx.request_repaint_after(Duration::from_secs(1))` before returning so the next frame fires in 1 s. Choose a small fixed window size (e.g., `egui::vec2(96.0, 44.0)` — adjust as needed so both lines fit at the default font without truncation; verify by eye during T013).
- [ ] T010 [US1] In `C:\repos\cpu-mem-overlay\src\main.rs`, replace the placeholder `main` with `fn main() -> eframe::Result<()>` that builds `eframe::NativeOptions` with a `ViewportBuilder` configured per [contracts/ui-contract.md](./contracts/ui-contract.md) §1: `with_decorations(false)`, `with_resizable(false)`, `with_window_level(egui::WindowLevel::AlwaysOnTop)`, `with_inner_size(...)` matching T009's chosen size, and `with_position(...)` from `primary_work_area_bottom_right(...)`. Call `eframe::run_native("cpu-mem-overlay", options, Box::new(|_cc| Box::new(OverlayApp::new())))` and propagate the result. Add `#![windows_subsystem = "windows"]` at the top of the file so launching from Explorer does not flash a console window.
- [ ] T011 [US1] In `C:\repos\cpu-mem-overlay\src\main.rs`, harden the read path in `Sampler::sample`: wrap the metric reads so any computation that would otherwise produce a value outside `0..=100` (e.g., divide-by-zero on `total_memory == 0`, or a `global_cpu_usage()` returning `f32::NAN`) yields `None` for that field. Verify by eye during T013 that on the very first frame the CPU line briefly shows `CPU --` and then transitions to a number — exactly as described in `data-model.md`.

**Checkpoint**: `cargo run --release` from the repo root opens the overlay and all nine manual checks in [contracts/ui-contract.md](./contracts/ui-contract.md) §4 pass.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Final verification against the spec's acceptance criteria and a release-mode smoke test. No new features.

- [ ] T012 [P] Run `cargo fmt` and `cargo clippy --all-targets -- -D warnings` from `C:\repos\cpu-mem-overlay\`; fix any warnings reported.
- [ ] T013 Run the full manual verification walk-through in [quickstart.md](./quickstart.md) §3 and the nine-step checklist in [contracts/ui-contract.md](./contracts/ui-contract.md) §4 against a `cargo run --release` build. Record any failures and address them before considering v1 complete.
- [ ] T014 Run a 1-hour soak test of the release binary on Windows 11 (per SC-004 / SC-005): leave the overlay running while using the system normally, then confirm in Task Manager that the binary's CPU column reads ≈ 0%, RSS stays under ~50 MB, and the overlay never stopped updating. If any of these fail, file a follow-up before declaring v1 done.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately. T001 must precede T002 (deps go into the manifest T001 created); T003 can run in parallel with T002.
- **Foundational (Phase 2)**: Depends on Phase 1 completion. T004 and T005 both edit `src/main.rs` so they must be sequenced (T004 → T005), not run in parallel.
- **User Story 1 (Phase 3)**: Depends on Phase 2 completion. T006 (tests) is written before the implementation tasks T007–T011, but all of T006–T011 land in the same `src/main.rs` file so they cannot run in true parallel; the `[P]` marker on T006 and T007 indicates only that they have no logical dependency on each other and can be authored independently.
- **Polish (Phase 4)**: Depends on US1 completion. T012 can run in parallel with T013/T014 once US1 is done, since `cargo fmt`/`clippy` doesn't require the binary to be running.

### User Story Dependencies

- **User Story 1 (P1, MVP)**: Only story in v1. No cross-story dependencies.

### Within User Story 1

- T006 (tests) is written first, fails against the placeholder, then T007–T011 make it pass.
- T007 (`Sampler`) and T008 (position helper) are independent of each other → can be authored in either order; both must precede T009.
- T009 (`OverlayApp::update`) depends on T004, T005, T007.
- T010 (`fn main`) depends on T008 and T009.
- T011 (read-path hardening) depends on T007 and is verified in tandem with T013.

### Parallel Opportunities

Because the entire feature lives in a single `src/main.rs`, true file-level parallelism is limited. The `[P]` markers above identify tasks that have no logical dependency on each other and could be split across developers using a single shared file (with care). In practice for a solo developer, work the tasks in numeric order.

---

## Parallel Example: User Story 1

```text
# Tasks that can be authored independently (single-file caveat applies):
Task: "T006 — write unit tests for format_line in src/main.rs (#[cfg(test)] mod tests block)"
Task: "T007 — implement Sampler struct in src/main.rs"
```

Both touch `src/main.rs`, so coordinate the merge — they are logically independent but textually adjacent.

---

## Implementation Strategy

### MVP First (User Story 1 — the only story)

1. Complete Phase 1: Setup (T001–T003).
2. Complete Phase 2: Foundational (T004–T005).
3. Complete Phase 3: User Story 1 (T006–T011) in numeric order.
4. **STOP and VALIDATE**: walk through [contracts/ui-contract.md](./contracts/ui-contract.md) §4 and [quickstart.md](./quickstart.md) §3 by eye on Windows 11.
5. Complete Phase 4: Polish (T012–T014).

This is intentionally a single-deliverable plan. There are no later increments planned for v1.

### Incremental Delivery

Not applicable in v1 — a single user story means the MVP *is* the v1 deliverable.

### Parallel Team Strategy

Not applicable — the codebase is small enough (~150 LOC, single file) that a single developer is the right team size.

---

## Notes

- `[P]` tasks have no logical dependency on each other; they can still touch the same file, so coordinate textual merges.
- `[Story]` label maps a task to its user story for traceability — only US1 exists in v1.
- Tests in T006 cover the only error-prone pure-logic seam (the formatter). The rest of the codebase is verified by eye against [contracts/ui-contract.md](./contracts/ui-contract.md) §4.
- Commit after each task or logical group.
- Stop at the Phase 3 checkpoint to validate US1 against the spec before doing any polish.
- Avoid: vague tasks, adding test infrastructure beyond R-008, premature workspace/library splits, extra Cargo features that aren't needed for v1.
