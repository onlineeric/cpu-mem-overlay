# Phase 0 Research: Configurable Overlay (V2)

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Date**: 2026-05-04

This document resolves the v2-specific technology and design questions implied by the plan's Technical Context. v1 decisions (R-001 … R-008 in `specs/001-cpu-mem-overlay/research.md`) carry forward unchanged unless explicitly revised below.

---

## R2-001: TOML parsing crate

- **Decision**: Use `toml` (latest 0.x) plus `serde` with `derive` for the deserialization layer. Define a flat `RawOverlayConfig` struct where every field is `Option<T>`; deserialize the file into that struct, then run per-field validators that produce a fully-defaulted `OverlayConfig`.
- **Rationale**:
  - `toml` + `serde` is the de facto standard for Rust config parsing — actively maintained, idiomatic, zero learning curve. Pulls in no networking or async machinery.
  - All-`Option<T>` fields make "missing key → default" the natural deserialization path: `serde` simply produces `None`, and the validator step converts each `None` to the field's documented default. This satisfies FR-002, FR-004, and FR-005 with one mechanism.
  - Deserialization errors (entire file unparseable) are caught at the call site and silently swallowed in favor of `OverlayConfig::default()`, satisfying FR-003 and FR-006a.
- **Alternatives considered**:
  - **`serde-toml-value` + manual table walking**: gives finer-grained per-key error reporting, but the spec explicitly forbids any user-visible feedback (FR-006a). The extra precision is unused.
  - **Hand-rolled TOML parser / regex**: ruled out by the project's "purpose-built libraries over manual implementations" preference and the constitution's Code Quality Standard on the same point.
  - **JSON / YAML**: rejected — the spec mandates TOML (Assumptions §). TOML is also the most ergonomic format for end users hand-editing a small flat file.

---

## R2-002: Color serialization format

- **Decision**: Accept `background_color` as a single string in the form `"#RRGGBB"` or `"#RRGGBBAA"`. The 6-digit form implies alpha = `0xFF` (fully opaque). Hex digits are case-insensitive. Any other shape — empty string, wrong length, non-hex digits, missing `#` — is treated as invalid per FR-004 and only this field reverts to the default.
- **Rationale**:
  - Hex strings are the most familiar color notation for users (matches CSS, Photoshop, every screenshot tool). One key, one short value, easy to hand-edit and easy to grep.
  - The optional alpha pair preserves "opaque by default" while still letting users specify any transparency level — including the fully-transparent case required by FR-013/FR-014.
  - Trivial to parse: `u8::from_str_radix(&hex[..2], 16)` per byte. No external crate needed (the `csscolorparser` / `palette` crates would be overkill for a one-call seam).
- **Alternatives considered**:
  - **RGBA component array** (`background_color = [10, 20, 30, 255]`): unambiguous but visually noisier and requires users to memorize the channel order. Rejected as less ergonomic.
  - **Named CSS colors** (`"black"`, `"transparent"`): nice ergonomics but expands the surface area without solving any concrete user request from the spec. Rejected as scope creep.
  - **HSLA / OKLCH**: orthogonal to the user's stated need (matching a wallpaper's RGB tone). Rejected as scope creep.

---

## R2-003: Multi-monitor on-screen check

- **Decision**: Enumerate per-monitor work-area rectangles via Win32 `EnumDisplayMonitors` + `GetMonitorInfoW` (the `Win32_Graphics_Gdi` feature of the existing `windows` crate). Treat the union of all returned `rcWork` rects as the set of valid placement regions. The `anchor_position` is on-screen iff the proposed window rectangle (top-left = `(x, y)`, size = current `WINDOW_SIZE`) overlaps at least one work-area rect by any non-zero area; otherwise fall back to the v1 default anchor (FR-012).
- **Rationale**:
  - `EnumDisplayMonitors` is the standard Win32 way to walk all monitors; `MONITORINFO::rcWork` is exactly the per-monitor rectangle that excludes the taskbar — same semantic v1 already uses for the primary monitor.
  - Working off the union of work areas (rather than the full virtual-screen bounding rect) correctly excludes "gaps" between non-rectangular monitor arrangements and the taskbar strip on each monitor, satisfying the spec's edge-case definition: only fully-off-screen rolls back, partially-off-screen is allowed.
  - Already-in-tree dependency (`windows` 0.58); only the feature flag is new.
- **Alternatives considered**:
  - **`SM_*VIRTUALSCREEN` system metrics**: returns one bounding rect; loses the per-monitor subtraction of taskbars and any inter-monitor gaps. Rejected — would let an off-screen anchor pass the check.
  - **`winit` monitor APIs through `eframe`**: workable, but `eframe` 0.29 only re-exports a subset, and the documented path for *work area* (not full monitor rect) is via the Win32 call anyway. Rejected — extra layer with no benefit.
  - **Defer multi-monitor handling to v3**: rejected — the spec's clarification explicitly mandates virtual-screen coordinates and union-based off-screen detection in v2.

---

## R2-004: Window transparency

- **Decision**: Set `ViewportBuilder::with_transparent(true)` unconditionally. Override `App::clear_color` to return the configured background color in the form `[r, g, b, a]` normalized to `0.0..=1.0`. When `a == 0` the framebuffer is cleared to fully-transparent; eframe forwards transparency to the OS layer-window so the desktop shows through.
- **Rationale**:
  - This is the documented eframe path for transparent windows; `with_transparent(true)` only enables the capability, the actual color comes from `App::clear_color`. Setting it unconditionally simplifies the code (one branch on the alpha would be wasted because the cost is the same).
  - Mouse input is not affected by `with_transparent(true)` — the OS still hit-tests against the window's pixel rectangle. This satisfies FR-014a (transparent window must still capture mouse).
- **Alternatives considered**:
  - **Use `egui::Frame::fill` instead of `clear_color`**: paints the panel background but leaves the window's own clear color opaque, defeating the effect. Rejected.
  - **Toggle `with_transparent(true)` only when configured alpha < 255**: micro-optimization without a measurable benefit; adds a conditional and a different code path for the opaque case to debug. Rejected.

---

## R2-005: Drag-to-reposition

- **Decision**: When `OverlayConfig::draggable` is true, the central panel calls `ui.interact(rect, id, Sense::drag())` on the full panel rect, and on the first frame where the response reports a press, the app emits `ctx.send_viewport_cmd(ViewportCommand::StartDrag)`. The OS then handles the drag natively, including release. Drag positions are session-only and never written to the config (FR-024).
- **Rationale**:
  - `ViewportCommand::StartDrag` is the documented, supported eframe path for moving a borderless window via mouse. The OS handles the modal drag loop, so the app doesn't have to mirror cursor deltas frame-by-frame — keeps the implementation small and avoids drift.
  - Guarding on `OverlayConfig::draggable` keeps v1's "fixed position, no in-app affordances" behavior as the default (FR-021, FR-023).
  - The v1 always-on-top behavior survives the drag because `ViewportCommand::StartDrag` doesn't change window levels.
- **Alternatives considered**:
  - **Manual cursor-delta tracking + `ViewportCommand::OuterPosition`**: works but reinvents OS drag semantics (capture, release, cursor warp). Rejected as unnecessary complexity.
  - **`with_drag_and_drop(true)` from eframe**: that flag is for file drag-and-drop *into* the window, not window movement. N/A.
  - **Persist drag position back to config**: explicitly out of scope per FR-024 and the user's stated preference.

---

## R2-006: Hover/movement-driven refresh fix

- **Decision**: Track `last_sample_at: Instant` in `OverlayApp`. Inside `App::update`, sample only when `now.duration_since(last_sample_at) >= refresh_interval`; otherwise reuse the cached `MetricSnapshot`. Always end `update` with `ctx.request_repaint_after(refresh_interval - elapsed_since_last_sample)` to schedule the next refresh deterministically.
- **Rationale**:
  - The v1 bug was that `App::update` is called whenever egui receives an event (mouse motion, focus change, etc.), and v1 sampled unconditionally inside `update`. Decoupling sampling from rendering — the standard fix — directly satisfies FR-019 and FR-020.
  - Using `Instant` (monotonic clock) is correct for elapsed-interval logic; `SystemTime` would be wrong (subject to clock skew).
  - Recomputing the schedule from the last sample point keeps the actual cadence aligned to the configured interval over time, even if input events trigger many extra `update` calls.
- **Alternatives considered**:
  - **Move sampling to a background thread + channel**: solves the same problem but adds threading complexity and an unnecessary synchronization layer for a single-window utility. Rejected — overengineering.
  - **Disable input events on the window**: would break drag-to-reposition (R2-005) and click-through-prevention (FR-014a). Rejected.
  - **Sample on a fixed wall-clock schedule** (e.g., aligned to seconds): not required by the spec and would add complexity. Rejected.

---

## R2-007: Window auto-fit for `font_size`

- **Decision**: Compute `WINDOW_SIZE` from `font_size` at startup using a simple linear model derived from v1's known-good 14 pt → 96×44 px proportion. The exact formula (with a small fudge for line spacing and right-edge padding) lives in `app.rs`. The window is created at this size via `ViewportBuilder::with_inner_size(...)` and stays fixed thereafter (no live resize — the config is read once at startup, FR-006).
- **Rationale**:
  - The spec only requires that the window grow to fit the text without clipping (FR-017, SC-007). A startup-computed inner size is the simplest mechanism that meets that bar — no on-every-frame measurement, no resize loop.
  - Keeping the window non-resizable preserves v1's UI contract item "Resizable: No".
- **Alternatives considered**:
  - **Measure text via `egui::Painter::layout_no_wrap` and resize on first frame**: more accurate but adds a one-frame flicker and more code. Rejected — the linear model is good enough for the two short labels and the spec asks only that text be fully visible.
  - **Let egui auto-shrink/auto-grow per frame**: violates the borderless-fixed-size visual contract and would cause the window to twitch on each frame. Rejected.

---

## R2-008: Config file location resolution

- **Decision**: Resolve the config path as `<dir-of-current-exe>/cpu-mem-overlay.toml`, where the executable's directory comes from `std::env::current_exe()` followed by `.parent()`. If `current_exe()` fails (extremely rare on Windows 11 — typically only with broken process state), the loader returns the default config silently per FR-006a.
- **Rationale**:
  - Spec Assumptions explicitly call out "resolved from the executable's own path, not the current working directory". `current_exe()` is the standard library's purpose-built API for that.
  - Treating a `current_exe()` failure as "no config found" is consistent with the spec's silent-fallback rule and avoids any user-visible error.
- **Alternatives considered**:
  - **`std::env::current_dir()` (CWD)**: spec explicitly rejects this — would behave differently when launched via Start menu vs. a shell. Rejected.
  - **Platform config dir (`%APPDATA%/cpu-mem-overlay/`)**: arguably cleaner for distributed apps but contradicts the spec's stated location and inflates the install footprint for no user benefit. Rejected.

---

## R2-009: Per-field validation policy

- **Decision**: Each setting has its own validator function with this contract: `fn validate_<field>(raw: Option<RawT>) -> ValidatedT`, where `ValidatedT` is the field's resolved value (always defaulted, never `None`). Invalid input — including out-of-range numbers, malformed color strings, NaN/infinite floats — produces the field's documented default. Validators are pure (no I/O) and individually unit-tested.
- **Rationale**:
  - Pulls the spec's per-field independence rule (FR-004) to the type level: there is no "config-wide error" path the validators can take, so the only possible outcome is a `OverlayConfig` populated entirely with valid values.
  - Pure functions keyed on `Option<RawT>` are trivial to unit-test — one happy-path test plus one test per documented invalid input per validator.
- **Alternatives considered**:
  - **Single monolithic validator that returns `Result<OverlayConfig, …>`**: violates per-field independence — one bad field would otherwise infect the whole result. Rejected.
  - **Use `serde` deserializer error hooks** (e.g., `#[serde(default, deserialize_with = …)]`): possible but bakes the validation into the deserializer and makes it harder to test the validator in isolation. Rejected — preferred a clean two-stage pipeline (deserialize → validate).

---

## R2-010: Testing strategy delta from v1

- **Decision**:
  - Unit tests cover every pure-logic seam added in v2: each per-field validator (every documented invalid case), the color hex parser (6-digit, 8-digit, lower/upper case, malformed), the multi-monitor on-screen overlap check (synthetic rectangles passed in directly — not enumerated from the OS), the `Sampler` interval gating (call into a small `should_sample(now, last, interval)` helper).
  - Manual smoke matrix in `quickstart.md` covers GUI behavior: hover-no-extra-refresh, transparent rendering, drag enabled/disabled, anchor placement on/off-screen, font-size auto-fit.
  - Carry-forward: v1's `format_line` tests remain.
- **Rationale**: Continues v1's R-008 strategy; the new pure logic is the highest-risk surface and the cheapest to cover. GUI testing on Windows is still out of scope.
- **Alternatives considered**:
  - **WinAppDriver / `enigo`-based UI tests**: rejected on the same grounds as v1 — disproportionate complexity for the size of this utility.
  - **Property-based tests via `proptest`**: nice for the color parser but adds a dev-dependency for marginal gain on a fixed-format string. Rejected — table-driven tests are sufficient.
