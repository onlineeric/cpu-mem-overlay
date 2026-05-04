# Phase 1 Data Model: CPU/Memory Overlay (V1)

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Date**: 2026-05-04

This feature has no persistence and only one in-memory entity.

---

## Entity: `MetricSnapshot`

A single sample of system state at one refresh tick. Lives in memory for the lifetime of one redraw and is then discarded.

### Fields

| Field    | Type         | Domain                              | Notes |
|----------|--------------|-------------------------------------|-------|
| `cpu`    | `Option<u8>` | `Some(0..=100)` or `None`           | Overall system CPU utilization, rounded to nearest integer percent. `None` represents "unavailable on this tick" (covers the very first tick before `sysinfo` has produced a usable reading, and any transient read failure). |
| `memory` | `Option<u8>` | `Some(0..=100)` or `None`           | Overall system memory utilization (`used / total * 100`), rounded to nearest integer percent. `None` represents "unavailable on this tick". |

### Validation rules

- Each field must be either `None` or a `u8` in the range `0..=100`. Any value computed outside this range from the source library is clamped to `100` before being stored (defensive — see R-005 in `research.md`).
- The two fields are independent: one can be `Some` while the other is `None`, and the display falls back per-line accordingly.

### Lifecycle / state transitions

```text
                 read OK         read OK
   None  ─────────────────►  Some(p)  ─────────────────►  Some(p')
    ▲                           │                            │
    │                           │ read fails                 │ read fails
    └───────────────────────────┴────────────────────────────┘
                              None
```

- Each tick produces a fresh `MetricSnapshot`; there is no persisted state between ticks beyond the `sysinfo::System` instance that owns the OS handles.
- The first tick after launch is expected to yield `cpu = None` because `sysinfo` requires at least one prior `refresh_cpu_usage()` plus a small delay before producing a usable CPU value (R-002). Memory is available on the first tick.
- Subsequent ticks should produce `Some(_)` for both fields under normal operation. `None` on a later tick indicates a transient read failure (FR-008) and the next tick is expected to recover.

### Display projection

The view layer maps each field to one line of text:

| Field value       | Rendered line     |
|-------------------|-------------------|
| `cpu = Some(p)`   | `CPU {p}%`        |
| `cpu = None`      | `CPU --`          |
| `memory = Some(p)`| `MEM {p}%`        |
| `memory = None`   | `MEM --`          |

There are no other entities. There are no relationships, no persistence, no cross-tick aggregation.
