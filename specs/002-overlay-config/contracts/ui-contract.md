# UI / Window Contract: Configurable Overlay (V2)

**Feature**: [spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md) | **Date**: 2026-05-04

This document is the v2 update to the v1 UI contract at `specs/001-cpu-mem-overlay/contracts/ui-contract.md`. Every v1 contract item still applies as the **default** behavior. This file records the *deltas* introduced by configurability, plus the new contract surface for the config file itself.

---

## 1. Window properties (delta from v1)

| Property                | Default behavior (v1) | v2 configurability                                                       |
|-------------------------|-----------------------|--------------------------------------------------------------------------|
| Decorations / title bar | None — borderless     | Unchanged                                                                |
| Always on top           | Yes                   | Unchanged                                                                |
| Resizable               | No                    | Unchanged                                                                |
| Size                    | Fixed 96×44           | Computed from `font_size` at startup (FR-017); still fixed thereafter    |
| Default position        | Bottom-right of primary monitor's work area | `anchor_position` overrides; off-screen anchors fall back to the v1 default (FR-010, FR-011, FR-012) |
| User reposition / drag  | None                  | When `draggable = true`, the user can click-drag anywhere on the tile to reposition; releasing the mouse leaves it there for the session (FR-022, FR-024) |
| Tray icon               | None                  | Unchanged                                                                |
| Right-click menu        | None                  | Unchanged                                                                |
| Transparency            | Opaque                | Configurable via `background_color` alpha; alpha = 0 → fully transparent rendering, but the window still captures mouse input (FR-013, FR-014, FR-014a) |
| Lifetime                | Window remains open until terminated externally | Unchanged                                              |

---

## 2. Rendered text (unchanged from v1)

The window contains exactly two lines, in order, top-to-bottom:

```text
CPU xx%
MEM xx%
```

with the v1 fallback `CPU --` / `MEM --` rules. `font_size` configures the rendered point/pixel size of both lines; the window auto-fits at startup so neither line clips. No other text, icons, or controls.

---

## 3. Refresh behavior (delta from v1)

- The displayed values update strictly on the configured `refresh_interval_ms` cadence (default 1000 ms; minimum 100 ms). Mouse hover, mouse motion over the tile, click, drag, focus gain, focus loss, or any other input event MUST NOT cause the values to refresh sooner than the next scheduled tick (FR-019, FR-020).
- A transient failure to read either metric on a given tick still falls back to `--` for that line; the next tick re-attempts (carried from v1 FR-008).

---

## 4. Configuration file contract

### 4.1 Location and lifecycle

- File path: `<dir-of-running-exe>/cpu-mem-overlay.toml`
- Read once at process startup. No live-reload, no file-watching, no writes.
- Resolved via the executable's own path, not the current working directory.

### 4.2 Format

TOML, flat — no nested tables. All keys are optional. Unknown keys are silently ignored.

```toml
# All keys are optional; omit a key to use its default.

# Refresh cadence in milliseconds (positive integer >= 100). Default: 1000.
refresh_interval_ms = 500

# Top-left position in virtual-screen pixel coordinates [x, y].
# Origin is the primary monitor's top-left; negatives are valid for monitors above/left of primary.
# If the resulting window would lie entirely outside every monitor's work area,
# the overlay falls back to the bottom-right of the primary monitor's work area.
anchor_position = [200, 100]

# RGBA color as a hex string. "#RRGGBB" implies alpha = FF (opaque).
# "#RRGGBBAA" sets alpha explicitly; "00" alpha = fully transparent.
background_color = "#101820CC"

# Font size for both metric lines. Positive number; window auto-fits to contain both lines. Default: 14.
font_size = 18

# Allow click-drag to reposition the overlay during the current session. Default: false.
# Drag positions are NOT persisted; relaunching restores anchor_position (or its fallback).
draggable = true
```

### 4.3 Per-field validation summary

| Key                  | Accepted shape                                        | Default on missing/invalid |
|----------------------|-------------------------------------------------------|----------------------------|
| `refresh_interval_ms`| TOML integer, `>= 100`                                | `1000`                     |
| `anchor_position`    | TOML array of two integers                             | bottom-right of primary work area (computed at startup) |
| `background_color`   | TOML string, `"#RRGGBB"` or `"#RRGGBBAA"`, case-insensitive | `"#00000000"` (fully transparent — only the metric text renders) |
| `font_size`          | TOML number, finite, `> 0` and `<= 256`               | `14`                       |
| `draggable`          | TOML boolean                                           | `false`                    |

### 4.4 Error handling contract

- Missing file → all defaults; overlay launches normally (FR-002).
- File present but unparseable as TOML → all defaults; overlay launches normally (FR-003).
- File present but a single key is invalid → only that key reverts to default; other valid keys take effect (FR-004).
- Unknown keys → ignored (FR-005).
- All fallback paths are silent: no log file, no on-screen indicator, no stderr message (FR-006a).

---

## 5. Manual verification steps (delta — see v1 contract for the carried-forward checks)

These steps cover behavior new to or changed in v2. v1's nine manual checks (cold start, live updates, always-on-top, borderless, fixed position, format, long-run stability, lightweight, termination) all still apply with the overlay running on its v2 defaults.

10. **Defaults equal v1**: with no `cpu-mem-overlay.toml` next to the exe, run the v1 manual matrix end-to-end. Every v1 check still passes. (SC-001)
11. **Custom refresh interval**: set `refresh_interval_ms = 500`. Watch for 30 s — values change about twice per second. Set `= 200` — values change about five times per second. Set `= 50` — overlay falls back to 1000 ms (50 < 100 minimum, single-field rollback). (SC-002, SC-010)
12. **Hover does not accelerate refresh**: with default cadence, move the mouse continuously over the overlay tile for 30 s. Values still change at most once per second. (SC-003)
13. **Custom anchor on primary monitor**: set `anchor_position = [400, 50]`. Overlay appears with its top-left at (400, 50) on the primary monitor within 3 s of launch. (SC-004)
14. **Custom anchor on secondary monitor**: with two monitors arranged side-by-side, set `anchor_position` to a coordinate inside the secondary monitor's virtual-screen region (e.g., `[2000, 200]`). Overlay appears on the secondary monitor. (SC-004)
15. **Off-screen anchor falls back**: set `anchor_position = [99999, 99999]`. Overlay appears at the v1 default position. (SC-005)
16. **Transparent background**: set `background_color = "#00000000"`. Overlay shows only the two metric lines on top of the desktop with no visible background fill. The text remains legible. Mouse hover over the (now-invisible) tile area still produces no extra refresh. (SC-006)
16a. **Transparent overlay captures input**: with `background_color = "#00000000"` and `draggable = true`, click-drag the (visually invisible) tile rectangle. The overlay receives the click and drags; the click does NOT pass through to the desktop. (FR-014a)
17. **Tinted opaque background**: set `background_color = "#101820"`. Overlay shows a solid dark background. (Sanity check that the 6-digit form parses as opaque.)
18. **Larger font**: set `font_size = 28`. Both lines render at the larger size and the window grows to contain them without clipping. Reverting to default reproduces the v1 footprint. (SC-007)
19. **Drag enabled**: set `draggable = true`. Click-drag the overlay to a new on-screen position. It follows the cursor; on release it stays there for the rest of the session. (SC-008)
20. **Drag disabled**: with `draggable = false` (or unset), click-drag does not move the window. (SC-009)
21. **Drag does not persist**: drag the overlay during a session, then quit and relaunch. The overlay reappears at the configured `anchor_position` (or its v1 default), not at the dragged position. (FR-024)
22. **Single invalid key, others valid**: in the config, set a valid `refresh_interval_ms = 500` and an invalid `background_color = "not a color"`. The overlay launches with the custom interval applied and the v1 default background. (SC-010)
23. **Unparseable TOML**: replace the config with a single line `this is not toml`. The overlay launches with all defaults and runs normally. (FR-003)
24. **Unknown key ignored**: add `unrecognized_key = "anything"` to the config. The overlay launches normally with all other configured keys applied. (FR-005)

---

## 6. Out of scope (explicit non-contract — additions to v1)

The following are *not* added in v2 and must not appear:

- Live config reload / file-watching.
- Writing the dragged position back to the config file.
- Click-through windows (alpha = 0 affects rendering only; mouse input is always captured — FR-014a).
- A settings UI, tray icon, right-click menu, or any in-app affordance for changing settings.
- Per-monitor DPI awareness adjustments beyond what eframe provides by default.
- Multi-instance support, hotkeys, or persistence of any kind beyond reading the TOML at startup.

All other v1 non-contract items continue to apply.
