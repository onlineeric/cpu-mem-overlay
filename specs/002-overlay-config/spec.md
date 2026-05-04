# Feature Specification: Configurable Overlay (V2)

**Feature Branch**: `002-overlay-config`
**Created**: 2026-05-04
**Status**: Draft
**Input**: User description: "(1) Read `cpu-mem-overlay.toml` next to the exe; default everything if missing. (2) Allow configuring refresh interval (ms), anchor position (any on-screen point, fall back if off-screen), background color (incl. transparent), font size. (3) The current app updates the displayed numbers when the mouse hovers/moves over the tile — that is undesired; refresh strictly on the configured interval. (4) Allow drag-to-reposition behind a boolean toggle."

## Clarifications

### Session 2026-05-04

- Q: When `background_color` is fully transparent, should the overlay still capture mouse input (hover/drag/click) or become click-through? → A: The overlay always captures mouse input regardless of `background_color`; transparency only affects rendering, never input. Click-through behavior is not offered in v2.
- Q: How is `anchor_position` interpreted across multiple monitors? → A: Virtual-screen coordinates — a single (X, Y) pair across all monitors, origin at the primary monitor's top-left, with negative values valid for monitors above/left of primary. Off-screen rollback is evaluated against the union of all monitor work areas.
- Q: When a config value is invalid (or the file fails to parse), should the user receive any feedback? → A: Silent fallback only. No log file, no on-screen indicator, no stderr message. Invalid keys revert to defaults and the overlay launches normally; users debug typos by observing whether their setting visibly took effect.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Customize the overlay via a config file (Priority: P1)

A user wants to adapt the overlay to their own desktop setup — different refresh cadence, a specific corner of a non-primary monitor, a softer (or fully transparent) background to blend with their wallpaper, and larger text for a high-DPI display. They drop a `cpu-mem-overlay.toml` file next to the executable, edit a few keys, and relaunch. The overlay starts up using their settings; any keys they omit (or get wrong) fall back to the v1 defaults instead of failing.

**Why this priority**: Configurability is the headline change in v2. Without it, none of the user's three concrete needs (cadence, position, appearance, size) can be addressed. Every other v2 capability is meaningful only because there is a config file to switch it on.

**Independent Test**: Place a `cpu-mem-overlay.toml` next to the built executable with a non-default value for each supported setting. Launch the overlay. Confirm each setting takes effect (different refresh cadence visible, different on-screen position, different background color/opacity, different text size). Then delete the file, launch again, and confirm the overlay behaves identically to v1.

**Acceptance Scenarios**:

1. **Given** no `cpu-mem-overlay.toml` exists next to the executable, **When** the user launches the overlay, **Then** the overlay starts with all v1 defaults (1000 ms refresh, bottom-right of the primary monitor's work area, opaque v1 background, v1 text size) and runs without error.
2. **Given** a valid `cpu-mem-overlay.toml` is present with a complete set of supported keys, **When** the user launches the overlay, **Then** every configured value is applied and the overlay reflects all four settings simultaneously.
3. **Given** a `cpu-mem-overlay.toml` is present but contains an invalid value for one setting (e.g., a negative refresh interval), **When** the user launches the overlay, **Then** only that setting reverts to its default and the remaining settings still take effect; the overlay starts without crashing.
4. **Given** a `cpu-mem-overlay.toml` is present whose entire contents cannot be parsed as TOML, **When** the user launches the overlay, **Then** the overlay starts using all defaults and runs without error.
5. **Given** the configured `anchor_position` would place the overlay entirely off every visible monitor's work area, **When** the user launches the overlay, **Then** the overlay appears at the default anchor position instead.
6. **Given** the configured `background_color` has zero alpha (fully transparent), **When** the user launches the overlay, **Then** only the text is rendered on top of the desktop content beneath the overlay, with no visible background fill.
7. **Given** `font_size` is configured to a clearly larger value than the default, **When** the user launches the overlay, **Then** both metric lines render at the larger size and the window automatically expands to contain them without truncation.

---

### User Story 2 - Refresh strictly on the configured interval (Priority: P2)

A user notices that hovering or moving the mouse over the overlay tile causes the CPU and memory numbers to update faster than once per second. They expect the overlay to be quiet and predictable: the values should change exactly on the configured cadence and nothing else should drive a re-read.

**Why this priority**: This is a correctness fix. Refreshing on mouse activity contradicts the v1 promise of a fixed once-per-second cadence (v1 SC-002) and will continue to be wrong for any custom cadence in v2. The fix is independent of the config file — it applies even when the user runs with all defaults.

**Independent Test**: Launch the overlay (with or without a config file). Watch the CPU/MEM numbers for at least 30 seconds while alternating between (a) leaving the mouse stationary far from the overlay, (b) hovering the mouse over the overlay tile, and (c) repeatedly moving the mouse on top of the overlay. Confirm that during all three conditions the displayed values change at the configured cadence (default 1000 ms) and not faster.

**Acceptance Scenarios**:

1. **Given** the overlay is running with the default 1000 ms refresh interval, **When** the user moves the mouse over the overlay tile continuously, **Then** the displayed CPU and MEM values change at most once per second.
2. **Given** the overlay is running with a configured refresh interval of N ms, **When** any user interaction occurs over the overlay window (hover, click, focus change), **Then** the displayed values still change at most once every N ms.
3. **Given** the overlay is running, **When** the user moves the mouse away from the overlay after hovering it, **Then** the displayed values continue to change at the configured cadence with no observable jump or extra refresh.

---

### User Story 3 - Drag-to-reposition when enabled (Priority: P3)

A user occasionally wants to nudge the overlay out of the way of an open window without restarting or hand-editing the config file. They set `draggable = true` in the config, restart, click and drag the overlay to a new spot on-screen for the rest of the session, and accept that the next launch returns it to the configured anchor.

**Why this priority**: Drag-to-reposition is a convenience layered on top of the static anchor. It only makes sense once configurability (Story 1) exists, and it is opt-in by design so that v1's "fixed position, no in-app affordances" behavior remains the default.

**Independent Test**: With `draggable = true` in the config, launch the overlay, click and drag it to a different on-screen position, release the mouse, and confirm the overlay stays at the new position for the rest of the session. Restart the overlay and confirm it returns to the configured anchor (i.e., the drag did not persist). Then set `draggable = false` (or remove the key), restart, and confirm dragging the overlay does not move it.

**Acceptance Scenarios**:

1. **Given** `draggable = true` is set in the config, **When** the user click-drags the overlay window, **Then** the overlay follows the cursor and remains at the released position for the rest of the session.
2. **Given** `draggable = false` (or unset) in the config, **When** the user attempts to click-drag the overlay window, **Then** the overlay does not move.
3. **Given** the user repositioned the overlay by dragging during a session, **When** the user terminates and relaunches the overlay, **Then** the overlay reappears at the configured `anchor_position` (or its default), not at the dragged position.
4. **Given** `draggable = true` and the user drags the overlay, **When** the drag is in progress, **Then** dragging does not increase the metric refresh cadence beyond the configured interval (consistent with Story 2).

---

### Edge Cases

- **Config file missing or empty**: All settings default; overlay launches normally.
- **Config file unreadable or contains invalid TOML**: All settings default; overlay launches normally.
- **Unknown keys in the config**: Silently ignored so future versions can add keys without breaking older binaries.
- **Out-of-range numeric value**: That single key reverts to its default; siblings still apply.
- **`refresh_interval_ms` below the supported minimum**: That single key reverts to its default to protect system performance.
- **`anchor_position` partially off-screen**: Still considered valid as long as some portion of the window overlaps a visible work area; only entirely-off-screen configurations roll back to default.
- **Multi-monitor setups**: `anchor_position` is in absolute virtual-screen pixel coordinates; the off-screen check considers the union of all monitor work areas.
- **`background_color` with alpha = 0**: Window background is fully transparent; the metric text remains opaque and readable. The overlay still captures mouse input over its pixel rectangle (hover, click, drag) — alpha = 0 affects only rendering, not input routing.
- **`font_size` value that grows the text beyond the v1 window size**: The window auto-resizes so both lines remain fully visible.
- **`draggable = true` combined with a transparent background**: The user can still grab and drag the window over the area occupied by the (transparent) tile.
- **Long-running session with non-default settings**: All v1 long-run guarantees (no leaks, no drift, recovery from transient metric-read failures) still hold.

## Requirements *(mandatory)*

### Functional Requirements

#### Config file loading

- **FR-001**: On startup the overlay MUST attempt to read a file named `cpu-mem-overlay.toml` located in the same directory as the running executable.
- **FR-002**: If the config file does not exist, the overlay MUST start using all default values and MUST NOT report this as an error.
- **FR-003**: If the config file exists but its contents cannot be parsed, the overlay MUST start using all default values and MUST NOT crash.
- **FR-004**: The overlay MUST evaluate each supported setting independently; if a single setting is missing or holds an invalid/out-of-range value, only that setting reverts to its default and all other valid settings MUST still take effect.
- **FR-005**: Unknown keys in the config file MUST be ignored without affecting startup.
- **FR-006**: The overlay reads configuration only at startup; changes to the file while the overlay is running MUST NOT be required to take effect until the next launch.
- **FR-006a**: Fallbacks triggered by a missing file, an unparseable file, or any invalid setting MUST be silent — no log file, no on-screen indicator, no stderr or other user-visible message. The overlay simply launches with the affected setting(s) at their defaults.

#### Configurable refresh interval

- **FR-007**: The overlay MUST support a `refresh_interval_ms` setting expressed as a positive integer number of milliseconds, defaulting to `1000` (matching v1).
- **FR-008**: A configured `refresh_interval_ms` MUST control both the metric sampling cadence and the on-screen update cadence so the two stay aligned.
- **FR-009**: Values for `refresh_interval_ms` outside the supported range (see Assumptions for the minimum) MUST be treated as invalid per FR-004.

#### Configurable anchor position

- **FR-010**: The overlay MUST support an `anchor_position` setting describing the top-left on-screen coordinate (X and Y in pixels) at which the overlay window appears on launch. Coordinates are interpreted in **virtual-screen space**: a single (X, Y) pair across all monitors, origin at the primary monitor's top-left, with negative values valid for monitors positioned above or to the left of the primary monitor.
- **FR-011**: The default `anchor_position` MUST place the overlay at the bottom-right of the primary monitor's work area (matching v1).
- **FR-012**: If the configured `anchor_position` would place the entire overlay window outside the union of all visible monitor work areas, the overlay MUST fall back to the default anchor position.

#### Configurable background color (incl. transparency)

- **FR-013**: The overlay MUST support a `background_color` setting that accepts a color value with an alpha channel, allowing any color from fully opaque to fully transparent.
- **FR-014**: When `background_color` is set to a fully transparent value, the overlay window MUST render only the metric text over whatever desktop content lies beneath; no opaque background MUST be drawn.
- **FR-014a**: The overlay window MUST continue to capture mouse input (hover, click, drag) regardless of the configured `background_color` and its alpha. A fully transparent background MUST NOT make the window click-through; mouse events occurring over the window's pixel rectangle MUST be delivered to the overlay, not to whatever lies beneath it.
- **FR-015**: The default `background_color` MUST match v1's opaque background so that omitting the key reproduces the v1 appearance.

#### Configurable font size

- **FR-016**: The overlay MUST support a `font_size` setting (positive number) controlling the rendered size of both the CPU and the MEM lines.
- **FR-017**: When `font_size` differs from the default, the overlay window MUST automatically resize so both lines render fully without clipping or wrapping.
- **FR-018**: The default `font_size` MUST reproduce v1's text size when the key is omitted.

#### Refresh cadence not driven by user input

- **FR-019**: The displayed CPU and memory values MUST update only on the configured refresh interval; mouse hover, mouse movement over the overlay, click, drag, focus gain, or focus loss MUST NOT cause the displayed values to refresh sooner than the next scheduled tick.
- **FR-020**: User interaction with the overlay MUST NOT alter the metric sampling cadence in any direction (no acceleration, no deceleration).

#### Optional drag-to-reposition

- **FR-021**: The overlay MUST support a `draggable` boolean setting whose default is `false`.
- **FR-022**: When `draggable = true`, the user MUST be able to reposition the overlay during a session by click-dragging anywhere on its surface.
- **FR-023**: When `draggable = false` (or unset), the overlay window MUST remain fixed at the position determined at launch (configured anchor or its fallback).
- **FR-024**: A position reached by dragging during a session MUST NOT be persisted to the config file; relaunching the overlay MUST place it at the configured `anchor_position` (or its default fallback).

#### Continuity with v1

- **FR-025**: All v1 functional requirements (display format `CPU xx%` / `MEM xx%`, always-on-top, borderless/no title bar, fallback to `--` on transient read failure, no in-app close affordance) MUST continue to hold under v2 with all default settings, and MUST also hold when settings are configured.

### Key Entities

- **Overlay Configuration**: A snapshot of user preferences read once at startup, comprising `refresh_interval_ms`, `anchor_position` (X, Y), `background_color` (with alpha), `font_size`, and `draggable`. Each field is independently optional; missing or invalid fields fall back to documented defaults.
- **System Metric Reading** *(carried over from v1)*: A snapshot of system state at a given instant, with CPU and memory percentages or per-field unavailability markers.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: With no `cpu-mem-overlay.toml` next to the executable, the overlay's behavior at launch is indistinguishable from v1 (same default position, same 1-second cadence, same background, same text size).
- **SC-002**: When `refresh_interval_ms` is set to a value N (within the supported range), the displayed values change exactly once every N ms (±1 tick) over a 1-minute observation, regardless of mouse activity over the overlay.
- **SC-003**: With the mouse continuously moving over the overlay tile for at least 30 seconds, the displayed CPU and MEM values change no more often than once per configured interval (i.e., the v1 hover-driven extra refresh is gone).
- **SC-004**: When `anchor_position` is set to any valid coordinate that overlaps a visible monitor work area, the overlay appears at that coordinate within 3 seconds of launch.
- **SC-005**: When `anchor_position` is set to a coordinate that places the entire window outside every monitor's work area, the overlay appears at the default anchor position instead, without errors.
- **SC-006**: When `background_color` is set to a fully transparent value, a screenshot of the overlay region shows the desktop content beneath through the window background, with the metric text still legible.
- **SC-007**: When `font_size` is set to roughly double the default, both metric lines render at the larger size and the overlay window expands to contain them without clipping; reverting to the default reproduces the v1 footprint.
- **SC-008**: With `draggable = true`, the user can drag the overlay to any visible screen position with the mouse, and the overlay stays at the released position for the remainder of that session.
- **SC-009**: With `draggable = false` (or unset), no mouse drag action moves the overlay from its launch position.
- **SC-010**: When a single config value is invalid (e.g., negative refresh interval, malformed color) while the rest are valid, the overlay launches successfully, ignores only the invalid value, and applies all other configured values.
- **SC-011**: The overlay reproduces all v1 success criteria (visible within 3s, on-top in 100% of cases, no crash over a 1-hour session, no perceptible system-performance impact, glanceable in under 2s) with default settings, and also when each setting is set to a representative non-default value.

## Assumptions

- The target environment remains Windows 11 on x64; any v2 cross-platform behavior is out of scope.
- The config file format is TOML, located in the same directory as the running `.exe` (resolved from the executable's own path, not the current working directory).
- Configuration is read once at startup. Live-reload, file-watching, or in-app reload commands are out of scope.
- The supported minimum for `refresh_interval_ms` is **100 ms**; values below that are treated as invalid (per FR-004) so the overlay cannot be configured into a state that perceptibly impacts system performance. There is no documented maximum beyond positive-integer range.
- `anchor_position` uses absolute virtual-screen pixel coordinates with the origin at the top-left of the primary monitor; the off-screen check considers the union of all monitor work areas.
- `background_color` accepts an RGBA representation (color components plus an alpha component). The exact serialization (hex string vs. component array) is an implementation detail chosen during planning; whichever is chosen, both fully-opaque and fully-transparent values must be expressible.
- `font_size` is expressed in the implementation's natural unit for label text (e.g., logical points/pixels). The exact unit is an implementation detail; the spec only requires that doubling the value produces visibly larger text and an auto-fit window. Implementations MAY enforce a defensive upper bound (e.g., 256) so a typo cannot create a window larger than the screen; values exceeding that bound are treated as invalid per FR-004.
- Drag is in-session only; the overlay never writes to the config file.
- All v1 assumptions (single primary monitor expected for the default position, overall-system metric semantics, no elevated privileges, manual termination via OS) continue to hold.
