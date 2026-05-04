# Phase 1 Data Model: Configurable Overlay (V2)

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Date**: 2026-05-04

This feature adds one in-memory entity (`OverlayConfig`) and one transient deserialization helper (`RawOverlayConfig`). The v1 entity `MetricSnapshot` carries forward unchanged.

---

## Entity: `OverlayConfig`

A snapshot of resolved user preferences, read once at startup and held immutably for the lifetime of the process. Every field is fully populated — there is no `Option` here, because each field has a documented default that fills in for missing or invalid input upstream.

### Fields

| Field                 | Type      | Domain                                  | Default                          | Source FRs |
|-----------------------|-----------|-----------------------------------------|----------------------------------|------------|
| `refresh_interval`    | `Duration`| `>= 100 ms`                             | `1000 ms`                        | FR-007, FR-009 |
| `anchor`              | `Anchor`  | `Default` or `VirtualScreen { x, y }`   | `Anchor::Default`                | FR-010, FR-011 |
| `background_color`    | `[u8; 4]` | `[r, g, b, a]`, each `0..=255`          | `[27, 27, 27, 255]` (eframe dark `panel_fill`, the colour v1 inherits implicitly) | FR-013, FR-015 |
| `font_size`           | `f32`     | `> 0.0` and `<= 256.0` (sane upper)     | `14.0` (v1 baseline)             | FR-016, FR-018 |
| `draggable`           | `bool`    | `true` / `false`                        | `false`                          | FR-021 |

### Sub-type: `Anchor`

```text
Anchor::Default                       // bottom-right of primary monitor's work area, computed at startup
Anchor::VirtualScreen { x: i32, y: i32 }   // user-supplied virtual-screen coordinates
```

`Anchor::Default` is a marker; the actual pixel position is computed once at window-build time via the v1 `primary_work_area_bottom_right` helper. `VirtualScreen` carries the user-supplied coordinates verbatim; if the resulting window rect doesn't overlap any work area (per R2-003), the window is built with `Anchor::Default` instead (silent fallback per FR-012).

### Validation rules

Resolution flow (per R2-009): `RawOverlayConfig` (all-`Option`) → per-field validator → final value used in `OverlayConfig`. If a per-field validator rejects the raw value (out-of-range, malformed string, NaN, `None`), the field is set to its default in the table above.

| Field                 | Validation                                                                                  |
|-----------------------|---------------------------------------------------------------------------------------------|
| `refresh_interval_ms` | Must parse as `u64` and be `>= 100`. Else default.                                          |
| `anchor_position`     | Must parse as a 2-element `[i32, i32]`. Else default. (On-screen check happens at window build, not validation, because it depends on the live monitor configuration.) |
| `background_color`    | Must parse as `"#RRGGBB"` (alpha defaults to `0xFF`) or `"#RRGGBBAA"`, case-insensitive. Else default. |
| `font_size`           | Must be a finite `f32` in `(0.0, 256.0]`. NaN, infinity, `<= 0.0`, or `> 256.0` → default.  |
| `draggable`           | Must be a TOML boolean. Anything else → default (`false`).                                  |

Unknown keys in the file are accepted and ignored (FR-005). A wholly unparseable file produces `OverlayConfig::default()` (FR-003).

### Lifecycle

- Constructed once in `main()` from the result of `config::load_or_default(<exe-dir>)`.
- Passed by value (or `&`) into `OverlayApp::new(...)` and into the `ViewportBuilder` setup in `main`.
- Immutable for the rest of the process. Drag-repositioning (FR-022) updates the *window* position via the OS; it does not mutate this struct (FR-024).

---

## Helper type: `RawOverlayConfig`

Pure deserialization shape. Every field is `Option<T>` so that a missing key produces `None` rather than a deserialization error.

```rust
#[derive(Debug, Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields = false)]
struct RawOverlayConfig {
    refresh_interval_ms: Option<u64>,
    anchor_position:     Option<[i32; 2]>,
    background_color:    Option<String>,
    font_size:           Option<f32>,
    draggable:           Option<bool>,
}
```

`deny_unknown_fields = false` (the default behavior; called out here to make FR-005 explicit and prevent regression). The struct exists only inside `config.rs` and is consumed by the validator step that produces `OverlayConfig`.

---

## Entity: `MetricSnapshot` *(unchanged from v1)*

Carried over verbatim from v1's `data-model.md`. The only v2-relevant note: `MetricSnapshot` is now produced inside `Sampler::sample` only when the elapsed-time gate fires (R2-006), not on every `App::update` call. The cached snapshot from the previous tick is reused by the renderer until the next gated sample.

---

## Relationships

There are no cross-entity relationships. `OverlayConfig` is read-only configuration; `MetricSnapshot` is per-tick metric state. The two travel together inside `OverlayApp` but neither references the other.

---

## State transitions

`OverlayConfig` has no runtime state transitions — the value is finalized at startup and never mutated.

`MetricSnapshot` transitions identically to v1 (see v1 `data-model.md`); the only change is the tick *cadence* (now configurable via `OverlayConfig::refresh_interval`) and the fact that ticks are gated by elapsed time rather than by every `update()` call.
