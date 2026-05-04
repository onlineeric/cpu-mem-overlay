# Quickstart: Configurable Overlay (V2)

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Date**: 2026-05-04

A short, concrete walk-through for verifying the v2 feature on a Windows 11 machine. This is the manual-smoke counterpart to the unit tests; together they cover the spec's acceptance scenarios and success criteria.

---

## Prerequisites

- Windows 11 x64 with at least one display attached.
- Rust 1.75+ (stable, edition 2021) installed.
- Repo checked out at `C:\repos\cpu-mem-overlay` (or any other path — adjust as needed).

---

## Build and run

From the repo root:

```powershell
cargo build --release
cargo run --release
```

The overlay should appear at the bottom-right of the primary monitor's work area, displaying two lines (`CPU xx%` / `MEM xx%`) and refreshing once per second. This is the v1-equivalent default behavior.

To stop the overlay, press `Ctrl+C` in the terminal or end the process from Task Manager. There is no in-app close affordance (carried from v1 FR-007).

---

## Smoke matrix

Run through the items below in order. Each maps to one or more acceptance scenarios in `spec.md` and one or more verification steps in `contracts/ui-contract.md`.

### Step 1 — Defaults match v1

1. Ensure no `cpu-mem-overlay.toml` exists next to the built executable (typically `target/release/cpu-mem-overlay.exe`).
2. Run the binary.
3. **Expected**: overlay appears in the bottom-right corner of the primary monitor's work area, opaque background, default font size, refresh once per second. Behavior is indistinguishable from v1.

### Step 2 — Custom refresh interval

1. Place this `cpu-mem-overlay.toml` next to the executable:

   ```toml
   refresh_interval_ms = 500
   ```

2. Run the binary.
3. **Expected**: numbers change about twice per second.
4. Stop, change to `refresh_interval_ms = 50`, run again.
5. **Expected**: 50 is below the 100 ms minimum, so this single key rolls back to default. Numbers change about once per second.

### Step 3 — Hover does not refresh faster

1. With the default cadence (no config or `refresh_interval_ms = 1000`), run the overlay.
2. Move the mouse continuously over the overlay tile for 30 seconds while watching the digits.
3. **Expected**: digits change at most once per second; no observable acceleration during hover/movement.

### Step 4 — Custom anchor (primary monitor)

1. Place:

   ```toml
   anchor_position = [400, 50]
   ```

2. Run the binary.
3. **Expected**: top-left of the overlay sits at virtual-screen coordinates (400, 50).

### Step 5 — Custom anchor (secondary monitor, if available)

1. With a multi-monitor setup, find a coordinate that lies on a secondary monitor (e.g., `[2400, 100]` if a 1920-wide primary is to the left of the secondary).
2. Configure:

   ```toml
   anchor_position = [2400, 100]
   ```

3. **Expected**: overlay appears on the secondary monitor at that position.

### Step 6 — Off-screen anchor falls back

1. Configure:

   ```toml
   anchor_position = [999999, 999999]
   ```

2. Run.
3. **Expected**: overlay appears at the v1 default (bottom-right of primary work area). No error, no log message.

### Step 7 — Transparent background

1. Configure:

   ```toml
   background_color = "#00000000"
   ```

2. Run on a desktop with a non-uniform wallpaper.
3. **Expected**: only the metric text is visible; the desktop shows through where the background would have been. Hovering the (now-invisible) tile area still does not accelerate refresh.

### Step 7b — Transparent overlay still captures input (FR-014a)

1. Keep `background_color = "#00000000"` and add `draggable = true`.
2. Run. Click-drag on the (visually invisible) tile rectangle.
3. **Expected**: the click is captured by the overlay and the overlay drags. It does NOT pass through to the desktop (which would otherwise produce a desktop-icon rubber-band selection or no effect at all). This confirms transparency affects rendering only, never input routing.

### Step 8 — Larger font

1. Configure:

   ```toml
   font_size = 28
   ```

2. Run.
3. **Expected**: both lines render at roughly double the default size; the overlay window has grown to contain them without clipping or wrapping.

### Step 9 — Drag enabled

1. Configure:

   ```toml
   draggable = true
   ```

2. Run. Click anywhere on the overlay tile and drag.
3. **Expected**: the overlay follows the cursor and stays at the released position. Restart the overlay; it returns to the configured `anchor_position` (or default), not the dragged position.

### Step 10 — Drag disabled (default)

1. With `draggable = false` (or the key omitted), run.
2. **Expected**: click-dragging the tile does not move the window.

### Step 11 — Single invalid key with others valid

1. Configure:

   ```toml
   refresh_interval_ms = 500
   background_color = "not a color"
   ```

2. Run.
3. **Expected**: refresh runs at 500 ms; background is the v1 default opaque color (single-key rollback, FR-004).

### Step 12 — Unparseable TOML

1. Replace the config with the single line `this is not toml`.
2. Run.
3. **Expected**: overlay launches with all defaults and runs normally. No log, no error.

### Step 13 — Unknown key

1. Add `unrecognized_key = "anything"` to a working config.
2. Run.
3. **Expected**: overlay launches normally; the unknown key is silently ignored.

---

## Unit-test sanity

Before considering the feature done, run:

```powershell
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

All three must pass with zero failures and zero unjustified `#[ignore]`s (constitution Principle IV). The unit tests cover every per-field validator, the color hex parser, the multi-monitor on-screen overlap check (against synthetic rectangles), the elapsed-time refresh gate, and the carried-over `format_line` cases.
