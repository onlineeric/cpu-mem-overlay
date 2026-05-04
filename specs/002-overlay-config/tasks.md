---

description: "Task list for Configurable Overlay (V2)"
---

# Tasks: Configurable Overlay (V2)

**Input**: Design documents from `/specs/002-overlay-config/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Per the project constitution (Principle II — Comprehensive Unit Test Coverage, NON-NEGOTIABLE), unit tests for non-trivial logic are MANDATORY for every user story. The feature spec does not request additional contract or integration tests, so only unit tests are included below.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Single Rust `bin` crate at the repository root: `src/` and (optionally) `tests/` at the repo root. The plan groups all v2 logic into modules under `src/` (`config.rs`, `color.rs`, `monitors.rs`, `sampler.rs`, `app.rs`, `main.rs`); unit tests live as `#[cfg(test)] mod tests {}` blocks at the bottom of each module file.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add v2-specific dependencies and the example config so users can copy it.

- [X] T001 Update `C:\repos\cpu-mem-overlay\Cargo.toml`: add `serde = { version = "1", features = ["derive"] }` and `toml = "0.8"` to `[dependencies]`; extend the existing `windows` dependency feature list with `"Win32_Graphics_Gdi"` (for `EnumDisplayMonitors` / `GetMonitorInfoW`). Run `cargo build --release` once to confirm the lockfile resolves.
- [X] T002 [P] Create `C:\repos\cpu-mem-overlay\cpu-mem-overlay.toml.example` containing every supported key with its default value as a comment, mirroring the §4.2 sample in `specs/002-overlay-config/contracts/ui-contract.md`. This file ships with the repo as documentation; it is NOT auto-copied next to the built exe.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Split the monolithic `src/main.rs` into the module layout the plan specifies, *without changing behavior*. Establish the module skeletons that subsequent user stories will fill in. Every v1 unit test must still pass at the end of this phase.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T003 Extract `MetricSnapshot`, `Sampler`, `read_cpu_pct`, `read_memory_pct`, and `format_line` from `C:\repos\cpu-mem-overlay\src\main.rs` into a new `C:\repos\cpu-mem-overlay\src\sampler.rs`. Make `MetricSnapshot`, `Sampler`, and `format_line` `pub(crate)`. Move all v1 `format_line` unit tests into a `#[cfg(test)] mod tests {}` block in `src\sampler.rs` (covering `cpu_some_zero`, `cpu_some_seven`, `cpu_some_hundred`, `cpu_none`, `mem_*`, `out_of_range_clamps_to_hundred`).
- [X] T004 Extract `WINDOW_SIZE`, `POSITION_MARGIN_PX`, `primary_work_area_bottom_right`, and `OverlayApp` (with its `eframe::App` impl) from `src\main.rs` into a new `C:\repos\cpu-mem-overlay\src\app.rs`. Keep behavior identical to v1 for now (no config wiring yet). Re-export only what `main.rs` needs.
- [X] T005 Create empty module skeletons `C:\repos\cpu-mem-overlay\src\config.rs`, `C:\repos\cpu-mem-overlay\src\color.rs`, and `C:\repos\cpu-mem-overlay\src\monitors.rs` so the module tree compiles. Each file should contain only a top-of-file doc comment naming its responsibility per `plan.md` §Project Structure; no logic yet.
- [X] T006 Slim `C:\repos\cpu-mem-overlay\src\main.rs` to the entry point only: `#![cfg_attr(not(test), windows_subsystem = "windows")]`, `mod` declarations for the five modules, and a `main()` that builds the viewport and runs `eframe::run_native(...)` exactly as v1 does. The diff must be net-zero behavior change.
- [X] T007 Run `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` from the repo root. All three pass with zero failures. (`cargo run --release` smoke-check deferred to Phase 6 / T026 — refactor is mechanical, all 9 v1 unit tests still green.)

**Checkpoint**: Module split complete, behavior identical to v1, all v1 tests still green.

---

## Phase 3: User Story 1 — Customize the overlay via a config file (Priority: P1) 🎯 MVP

**Goal**: Read `cpu-mem-overlay.toml` next to the executable at startup and apply `refresh_interval_ms`, `anchor_position`, `background_color` (incl. transparency), and `font_size`. Missing file, unparseable file, unknown keys, or any single invalid field falls back silently to the v1 default for that field.

**Independent Test**: Place a `cpu-mem-overlay.toml` next to the built exe with a non-default value for each of the four supported settings. Launch — confirm each setting takes effect (different cadence visible, different on-screen position, different background color/opacity, different text size). Delete the file and relaunch — overlay behaves identically to v1. Spec acceptance scenarios 1–7 in spec.md §User Story 1.

### Tests for User Story 1 ⚠️

> **NOTE**: Unit tests below are MANDATORY per constitution Principle II.

- [X] T008 [P] [US1] In `C:\repos\cpu-mem-overlay\src\color.rs`, add `#[cfg(test)] mod tests {}` covering `parse_hex_color`: 6-digit lowercase, 6-digit uppercase, 8-digit lowercase, 8-digit uppercase with non-FF alpha, alpha = 00 → fully transparent, missing `#` → error, wrong length (4, 5, 7, 9 digits) → error, non-hex digits → error, empty string → error, mixed case → success.
- [X] T009 [P] [US1] In `C:\repos\cpu-mem-overlay\src\config.rs`, add `#[cfg(test)] mod tests {}` covering each per-field validator AND the top-level `load_from_dir(dir)` flow: missing file → defaults; unparseable TOML → defaults; partial config (only one key set) → that key applied + others defaulted; `refresh_interval_ms = 100` → accepted; `refresh_interval_ms = 99` → default; `refresh_interval_ms = 0` → default; `font_size = 0.0` → default; `font_size = -5.0` → default; `font_size = f32::NAN` → default; `font_size = 14.0` → accepted; `font_size = 257.0` → default; `anchor_position = [10, 20]` → accepted; missing `anchor_position` → `Anchor::Default`; malformed `background_color` → default; `background_color = "#00000000"` → fully transparent accepted; unknown key in TOML is silently ignored; single invalid key + valid sibling keys → only invalid key falls back. (Implementation note: `load_or_default` was renamed to `load_from_dir` and a wrapper `load_from_exe_dir` handles the exe-directory lookup — cleaner separation of concerns.)
- [X] T010 [P] [US1] In `C:\repos\cpu-mem-overlay\src\monitors.rs`, add `#[cfg(test)] mod tests {}` covering the pure `is_on_any_work_area(window_rect, work_areas)` helper (NOT the live `EnumDisplayMonitors` call): single work area fully containing window → true; window fully outside single work area → false; window straddling two adjacent work areas with non-zero overlap → true; window in the gap between two non-adjacent work areas → false; zero-area work-area input → false; empty work-area list → false. (Also added: touching-edge-only and negative-coordinate test cases.)
- [X] T011 [P] [US1] In `C:\repos\cpu-mem-overlay\src\app.rs`, add a `compute_window_size(font_size: f32) -> egui::Vec2` helper plus `#[cfg(test)] mod tests {}` covering: default `font_size = 14.0` → returns the v1 `WINDOW_SIZE` (96.0, 44.0) within ±0.5 px; doubling font_size approximately doubles each dimension (monotonic, finite output); very small `font_size = 1.0` produces finite positive values; rejects no value (already validated upstream).

### Implementation for User Story 1

- [X] T012 [P] [US1] Implement `parse_hex_color(s: &str) -> Result<[u8; 4], ColorParseError>` in `C:\repos\cpu-mem-overlay\src\color.rs`. Accept `"#RRGGBB"` (alpha defaults to `0xFF`) and `"#RRGGBBAA"`, case-insensitive. Use `u8::from_str_radix` per byte; do NOT use regex. Define a private `ColorParseError` enum with variants for the documented invalid cases.
- [X] T013 [P] [US1] Implement `Anchor` enum (`Default` and `VirtualScreen { x: i32, y: i32 }`) and `OverlayConfig` struct (with all fields populated, no `Option`s) in `C:\repos\cpu-mem-overlay\src\config.rs`. Add `impl Default for OverlayConfig` returning the v1 baseline values from `data-model.md` §Fields: `refresh_interval = Duration::from_millis(1000)`, `anchor = Anchor::Default`, `background_color = [27, 27, 27, 255]` (eframe dark `panel_fill`), `font_size = 14.0`, `draggable = false`.
- [X] T014 [US1] In `C:\repos\cpu-mem-overlay\src\config.rs`, add the private `RawOverlayConfig` struct (all-`Option<T>`, `#[derive(Deserialize, Default)]`) per `data-model.md` §Helper type, and per-field validator functions: `validate_refresh_interval_ms`, `validate_anchor`, `validate_background_color`, `validate_font_size`, `validate_draggable`. Each takes `Option<RawT>` and returns the resolved field value, falling back to default on any invalid input. Compose them into `fn from_raw(raw: RawOverlayConfig) -> OverlayConfig`.
- [X] T015 [US1] In `C:\repos\cpu-mem-overlay\src\config.rs`, implement `pub(crate) fn load_from_dir(dir: &Path) -> OverlayConfig` plus a thin wrapper `pub(crate) fn load_from_exe_dir() -> OverlayConfig` that performs the `std::env::current_exe()` + `.parent()` lookup. Both swallow I/O and parse errors silently per FR-006a. (Renamed from the originally-planned `load_or_default` / `config_path_next_to_exe` pair to remove the awkward path-strip-and-rejoin dance in `main`.)
- [X] T016 [US1] Implement `pub(crate) fn is_on_any_work_area(window_rect: WindowRect, work_areas: &[WorkArea]) -> bool` in `C:\repos\cpu-mem-overlay\src\monitors.rs` plus the private `WindowRect` and `WorkArea` value types (each carrying integer left/top/right/bottom). Overlap test must consider any non-zero-area intersection true. Add a `pub(crate) fn enumerate_work_areas() -> Vec<WorkArea>` that calls Win32 `EnumDisplayMonitors` with a callback that fills the vec via `GetMonitorInfoW(monitor, &mut MONITORINFO)` and reads `rcWork`. The callback path is `unsafe`; the public function hands back a safe `Vec<WorkArea>`.
- [X] T017 [US1] In `C:\repos\cpu-mem-overlay\src\app.rs`, replace the hardcoded `WINDOW_SIZE` constant usage with `compute_window_size(config.font_size)`. Have `OverlayApp::new` take `OverlayConfig` by value and store it on `self`. Render both labels via `egui::RichText::new(...).size(config.font_size)`. Override `eframe::App::clear_color` to return `[r/255.0, g/255.0, b/255.0, a/255.0]` derived from `config.background_color` (precomputed once in `OverlayApp::new` to avoid per-frame divisions).
- [X] T018 [US1] In `C:\repos\cpu-mem-overlay\src\main.rs`, v2 flow: load `OverlayConfig` via `config::load_from_exe_dir()`; compute resolved anchor (if `VirtualScreen` and `monitors::is_on_any_work_area` returns true, use it; otherwise `app::primary_work_area_bottom_right`); compute window size via `app::compute_window_size(config.font_size)`; build `ViewportBuilder` with `with_decorations(false)`, `with_resizable(false)`, `with_inner_size(window_size)`, `with_position(position)`, `with_window_level(AlwaysOnTop)`, AND `with_transparent(true)`; pass `OverlayConfig` into `OverlayApp::new`.

**Checkpoint**: With `cargo run --release` and various `cpu-mem-overlay.toml` files placed next to the exe, the overlay reflects every configured setting. Spec acceptance scenarios 1–7 (and SC-001, SC-004, SC-005, SC-006, SC-007, SC-010, parts of SC-011) pass by manual inspection. v2 unit tests are green.

---

## Phase 4: User Story 2 — Refresh strictly on the configured interval (Priority: P2)

**Goal**: Sample metrics only when the configured interval has actually elapsed, regardless of how many times `App::update` is invoked by mouse/focus events. The displayed CPU/MEM digits change exactly on the cadence — no hover acceleration.

**Independent Test**: Run the overlay (with or without a config file). Hover/move the mouse continuously over the tile for 30 s and confirm the digits change at most once per configured interval (default 1000 ms). Spec acceptance scenarios 1–3 in spec.md §User Story 2.

### Tests for User Story 2 ⚠️

- [X] T019 [P] [US2] In `C:\repos\cpu-mem-overlay\src\app.rs`, add a `pub(crate) fn should_sample(now: Instant, last_sample_at: Instant, interval: Duration) -> bool` helper plus `#[cfg(test)] mod tests {}` covering: `now == last + interval` → true; `now == last + interval + 1ns` → true; `now == last + interval - 1ns` → false; `now == last` → false; `interval = 0` → always true (no division by zero, defensive). Use synthetic `Instant`s constructed from a single base via `+ Duration` arithmetic.

### Implementation for User Story 2

- [X] T020 [US2] In `C:\repos\cpu-mem-overlay\src\app.rs`, add `last_sample_at: Instant` and a cached `snapshot: MetricSnapshot` to `OverlayApp`. Initialize `last_sample_at` to `Instant::now() - config.refresh_interval` so the first frame samples immediately.
- [X] T021 [US2] In `OverlayApp::update`, replace the unconditional `self.snapshot = self.sampler.sample();` with: compute `now = Instant::now()`; if `should_sample(now, self.last_sample_at, self.config.refresh_interval)` then sample and assign `self.last_sample_at = now`. Always render from `self.snapshot`. End every `update` with `ctx.request_repaint_after(self.config.refresh_interval.saturating_sub(now.duration_since(self.last_sample_at)))` so the next *guaranteed* repaint lands at the next tick.

**Checkpoint**: Hover-driven extra refresh is gone; cadence honors `config.refresh_interval` (manual SC-002, SC-003, SC-011). Unit tests for `should_sample` are green.

---

## Phase 5: User Story 3 — Drag-to-reposition when enabled (Priority: P3)

**Goal**: When `draggable = true`, the user can click-drag the overlay anywhere on the tile to reposition it for the rest of the session. Drag positions never persist to disk.

**Independent Test**: With `draggable = true`, drag the overlay; confirm it moves and stays where released. Restart — confirm it returns to the configured anchor (drag did NOT persist). Set `draggable = false` (or remove the key) and restart — confirm dragging no longer moves the window. Spec acceptance scenarios 1–4 in spec.md §User Story 3.

### Tests for User Story 3 ⚠️

- [X] T022 [P] [US3] No new pure-logic seam is introduced by drag (it's a single conditional `ViewportCommand` send), so unit tests for US3 reduce to a regression check: re-run `cargo test` after the implementation and confirm all earlier tests still pass. Add this verification to the polish phase's test-green gate (no separate test file needed; recorded here for traceability per Principle II's "non-trivial logic" qualifier).

### Implementation for User Story 3

- [X] T023 [US3] In `OverlayApp::update` in `C:\repos\cpu-mem-overlay\src\app.rs`, when `self.config.draggable` is true: call `let response = ui.interact(ui.max_rect(), egui::Id::new("overlay-drag"), egui::Sense::drag());` on the central panel UI; if `response.drag_started()`, send `ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);`. Drag does not consume sampling or repaint logic — refresh cadence is unchanged during drag (FR-020).
- [ ] T024 [US3] Manual-smoke verification (quickstart Step 9 / Step 10) deferred to the user — see Phase 6 / T025. No code change in `main.rs` was required: the drag state is read from the same `OverlayConfig` already passed to `OverlayApp::new`.

**Checkpoint**: With `draggable = true` the overlay can be repositioned; restart returns to the configured anchor. With `draggable = false` (or unset) the window is fixed. Refresh cadence is unchanged during drag.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final spec/quickstart validation, the constitution-mandated self-review, and the green-tests gate before declaring v2 complete.

- [ ] T025 [P] (USER-RUN) Walk through every step of `C:\repos\cpu-mem-overlay\specs\002-overlay-config\quickstart.md` §Smoke matrix on a real Windows 11 machine, ticking off each Expected — including Step 7b (transparent overlay still captures mouse input — FR-014a) and Steps 9–10 (drag enabled / disabled). Note any deviation as a follow-up issue. (Deferred to user: GUI smoke testing requires an interactive Windows session.)
- [ ] T026 (USER-RUN) Verify the v1 manual matrix in `specs\001-cpu-mem-overlay\contracts\ui-contract.md` §4 still passes when v2 runs with no config file (FR-025, SC-001). (Deferred to user, same reason as T025.)
- [X] T027 **Self-review pass** (Principle III): diff re-read against `rust-skills` and CLAUDE.md preferences. Cleanups applied during this pass: removed dead `c_void` import in `monitors.rs`; renamed `config_path_next_to_exe` + `load_or_default` to a cleaner `load_from_exe_dir` + `load_from_dir` pair (eliminates the path-strip-and-rejoin dance in `main.rs`); precomputed `clear_color_normalized` once in `OverlayApp::new` instead of dividing per frame. No `unwrap()`/`expect()` in non-test code (only in test setup). No leaky `Option` in public APIs (validators return resolved values, not `Option`s). Used `let-else` and `saturating_*` arithmetic per rust-skills idioms.
- [X] T028 **Test-green gate** (Principle IV): `cargo test` → 62 passed / 0 failed / 0 ignored. `cargo clippy --all-targets -- -D warnings` → clean. `cargo fmt --check` → clean. Constitution Principle IV gate satisfied.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)** has no in-feature dependencies; can start immediately.
- **Foundational (Phase 2)** depends on Setup (T001 must finish so `Cargo.toml` is right before refactor compiles); BLOCKS all user stories.
- **User Stories (Phase 3+)** all depend on Foundational completion (the module split must be in place).
  - US1 (P1) and US2 (P2) are technically independent of each other (US2 only needs the elapsed-time gate, which can be hardcoded to `Duration::from_millis(1000)` if `OverlayConfig` is not yet wired). In practice they share `OverlayApp`, so completing US1 first lets US2 wire its gate to `config.refresh_interval` directly.
  - US3 (P3) consumes `config.draggable` from US1's `OverlayConfig`, so it depends on US1.
- **Polish (Phase 6)** depends on all user stories being complete.

### User Story Dependencies

- **US1 (P1)** depends only on Foundational (Phase 2).
- **US2 (P2)** depends on Foundational (Phase 2). It can technically ship before US1, but ordering by priority and to avoid duplicate `OverlayApp` edits, do it after US1.
- **US3 (P3)** depends on Foundational (Phase 2) AND US1 (consumes `OverlayConfig::draggable`).

### Within Each User Story

- Tests labelled `[P]` for the same story can be drafted in parallel (different files: `color.rs`, `config.rs`, `monitors.rs`, `app.rs`).
- Within US1, T012 / T013 / T014 / T015 / T016 each touch one module and are mostly independent — `[P]` markers reflect that. T017 depends on T013 (needs `OverlayConfig`) and T011 (needs `compute_window_size`). T018 depends on T015, T016, T011, T017.
- US2's T021 depends on T020 and T019.
- US3's T023 depends on US1's T017 (needs `self.config.draggable`).

### Parallel Opportunities

- T002 [P] runs alongside T001.
- All `[P]` test tasks across US1 (T008–T011) can be drafted in parallel by different developers — each touches its own module.
- All `[P]` implementation tasks across US1 modules (T012, T013, T016) can be drafted in parallel.
- US2's T019 [P] is independent of US1's late-stage tasks (different region of `app.rs` — append vs. modify).

---

## Parallel Example: User Story 1

```bash
# Once Phase 2 is done, draft all four US1 unit-test scaffolds in parallel:
Task: "Add color.rs tests for parse_hex_color (T008)"
Task: "Add config.rs tests for validators + load_or_default (T009)"
Task: "Add monitors.rs tests for is_on_any_work_area (T010)"
Task: "Add app.rs tests for compute_window_size (T011)"

# Then in parallel, the leaf-module implementations:
Task: "Implement parse_hex_color in src/color.rs (T012)"
Task: "Implement OverlayConfig + Anchor in src/config.rs (T013)"
Task: "Implement is_on_any_work_area + enumerate_work_areas in src/monitors.rs (T016)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — module split)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: smoke-test US1 acceptance scenarios with various config files
5. Ship if v2 MVP scope is enough; otherwise continue

### Incremental Delivery

1. Setup + Foundational → green tests, behavior unchanged from v1
2. + User Story 1 (P1) → config-driven overlay, hover bug still present (use default cadence to not amplify it during demo)
3. + User Story 2 (P2) → hover bug fixed, fully cadence-correct
4. + User Story 3 (P3) → optional drag-to-reposition
5. Polish → green-test gate, self-review, quickstart matrix

### Notes

- `[P]` tasks = different files, no dependencies on other in-flight tasks.
- `[Story]` label maps a task to a specific user story for traceability.
- Each user story is independently smoke-testable per its quickstart steps.
- Constitution Principle II makes unit tests for non-trivial logic mandatory; do not skip the test tasks to "save time".
- Constitution Principle IV makes the green-test gate non-negotiable before declaring done.
- No git commits are performed by these tasks; the user runs `git` themselves per project policy.
