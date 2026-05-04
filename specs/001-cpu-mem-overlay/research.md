# Phase 0 Research: CPU/Memory Overlay (V1)

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Date**: 2026-05-04

This document records the technology and design decisions made before any code is written. Every "NEEDS CLARIFICATION" implied by the Technical Context in `plan.md` is resolved below.

---

## R-001: GUI / windowing crate

- **Decision**: Use `eframe` (the framework wrapper around `egui`).
- **Rationale**:
  - Provides borderless, always-on-top, decoration-free windows out of the box via `egui::ViewportBuilder` (`with_decorations(false)`, `with_always_on_top()`, `with_resizable(false)`, `with_inner_size(...)`, `with_position(...)`).
  - Immediate-mode rendering keeps the code small (the entire UI is two `Label`s).
  - Drives the refresh cadence cleanly via `ctx.request_repaint_after(Duration::from_secs(1))`, so the app sleeps between frames instead of busy-spinning — this is what makes the steady-state CPU cost stay near 0%.
  - Mature, maintained, widely used; minimal boilerplate in Rust.
- **Alternatives considered**:
  - **Pure Win32 via `windows` crate**: smallest possible binary (~500 KB) and lowest runtime cost, but several hundred lines of `unsafe` window-procedure code for what is a trivial display. Rejected for v1 — violates the spec's "kept as minimal as possible" intent at the *code* level even though it's minimal at the *binary* level.
  - **`winit` + `softbuffer` + manual font rendering**: middle ground, but text rendering becomes a project of its own. Rejected — the cost of pulling in a font/glyph crate roughly equals the cost of pulling in `egui`, with much worse ergonomics.
  - **`iced`, `slint`, `tauri`**: all heavier than the feature warrants; designed for richer apps. Rejected.
  - **`nwg` (native-windows-gui)**: lightweight Win32 wrapper, but smaller community and less idiomatic for an event loop driven by a 1 s timer. Rejected as a less-supported option vs `eframe`.

---

## R-002: System metrics crate

- **Decision**: Use `sysinfo`.
- **Rationale**:
  - Single dependency that provides both global CPU usage (`System::global_cpu_usage()`) and total/used memory (`System::total_memory()` / `System::used_memory()`) on Windows.
  - Standard refresh model: call `sys.refresh_cpu_usage()` and `sys.refresh_memory()` once per tick. The CPU API explicitly requires at least one prior refresh + a small delay before the first usable reading — handled by doing an initial refresh at startup and computing the displayed value from the second tick onward (the first tick can show `--`).
  - No elevated privileges required on Windows 11.
- **Alternatives considered**:
  - **Direct Windows PDH / NtQuerySystemInformation via the `windows` crate**: lowest overhead and no third-party dependency, but adds nontrivial `unsafe` code and per-counter bookkeeping. Rejected for v1 on the same grounds as R-001 (code-minimality wins over binary-size).
  - **`heim`**: cross-platform metrics library, but its async model is unnecessary here and the project is less actively maintained than `sysinfo`. Rejected.

---

## R-003: Refresh cadence and event loop

- **Decision**: 1 Hz refresh driven by `egui`'s `ctx.request_repaint_after(Duration::from_secs(1))`. Each repaint reads metrics, formats the two strings, and renders. No background thread.
- **Rationale**:
  - Single-threaded, easy to reason about, and matches FR-003 (refresh once per second) precisely.
  - When idle, `eframe` parks the event loop until the next scheduled repaint — keeps steady-state CPU near 0% (Performance Goal in plan.md).
  - Trivially satisfies SC-002 ("no observed gap longer than 2 seconds between updates") because the next repaint is always scheduled at the end of the previous one.
- **Alternatives considered**:
  - **Background sampling thread + channel into UI**: needed for sub-second sampling or expensive metric collection, neither of which apply here. Rejected as unnecessary complexity for v1.
  - **OS timer (`SetTimer` / `WM_TIMER`)**: ties us to direct Win32 — unavailable when using `eframe`. N/A.

---

## R-004: Window position "near the taskbar"

- **Decision**: Default position = bottom-right corner of the primary monitor's *work area*, with a small margin (e.g., 12 px) so the overlay sits just above the taskbar (or just to the left of a vertical taskbar). Computed once at startup and passed via `ViewportBuilder::with_position(...)`.
- **Rationale**:
  - On a default Windows 11 install the taskbar is at the bottom; bottom-right is the most "out-of-the-way but visible" anchor and matches the common interpretation of "near the taskbar" used by similar utilities.
  - Computing from the *work area* (not the full screen rect) automatically avoids overlapping the taskbar without needing to know the taskbar's exact geometry.
  - Per FR-006, the position is fixed in v1 — no user controls — so a single hardcoded computation is acceptable.
- **Implementation note**: `eframe` exposes monitor info through the `egui_winit` re-exports; if reliably reading the work area through the `eframe` API turns out to be awkward, a one-shot call to the Win32 `SystemParametersInfoW(SPI_GETWORKAREA, ...)` via the `windows` crate is an acceptable fallback. This is an implementation detail to confirm during `/speckit-implement`; either path satisfies the spec.
- **Alternatives considered**:
  - **Top-right or top-left corner**: reasonable but doesn't match the "near the taskbar" wording for the default Windows 11 layout. Rejected for v1.
  - **Overlap onto the taskbar via `ViewportBuilder::with_window_level(WindowLevel::AlwaysOnTop)` plus full-screen rect**: visually noisier, can cover Start menu artifacts. Rejected.
  - **User-configurable position**: explicitly out of scope per FR-006 and the v1 non-goals. Rejected.

---

## R-005: Display formatting and fallback

- **Decision**:
  - Render two lines: `format!("CPU {pct}%")` and `format!("MEM {pct}%")` where `pct` is the rounded integer 0–100.
  - On a failed read for either metric, render the corresponding line as the literal `CPU --` or `MEM --`.
  - Round to nearest integer with `.round() as u8`, clamped to `[0, 100]` defensively.
- **Rationale**: Matches FR-001/FR-002/FR-008 verbatim; clamping protects the display against any out-of-range value the metric source might briefly emit.
- **Alternatives considered**:
  - **Show one decimal place** (`CPU 18.4%`): rejected — spec says integers.
  - **Pad to a fixed width** (`CPU 005%`): rejected — spec example shows `CPU 18%` with no padding.

---

## R-006: Always-on-top behavior across focus changes

- **Decision**: Set the window level to *Always on Top* via `ViewportBuilder::with_window_level(egui::WindowLevel::AlwaysOnTop)` at startup. No re-assertion on focus loss is required because Windows 11 honors this level for the lifetime of the window.
- **Rationale**: Satisfies FR-005 and SC-003 with a single setup call.
- **Alternatives considered**:
  - **Re-poke `SetWindowPos(... HWND_TOPMOST ...)` periodically**: occasionally seen in older overlay apps, but unnecessary with modern `winit`/`eframe` on Windows 11. Rejected as cargo-culted.

---

## R-007: Project layout

- **Decision**: Single `bin` Cargo crate at the repository root. `Cargo.toml` next to `src/main.rs`. No workspace, no library split.
- **Rationale**: The feature is one small binary; any further structure is premature. The empty pre-existing `rust-test/` directory is unrelated and is left alone.
- **Alternatives considered**:
  - **Workspace with separate `metrics` and `ui` crates**: rejected — overengineering for ~150 LOC.
  - **Crate inside `rust-test/`**: rejected — confusing name for the production binary.

---

## R-008: Testing strategy

- **Decision**:
  - Unit-test the small pure-logic surface only: the formatter that turns an `Option<u8>` percentage into the displayed line (covers `Some(0)`, `Some(100)`, `None` → `--`, and the clamping path).
  - Verify GUI behavior (borderless, always-on-top, position, refresh) manually against the acceptance scenarios in `spec.md`.
- **Rationale**: Automated GUI testing on Windows is high-effort and out of scope for a v1 utility this size. Keeping the pure-logic seam tested catches the only error-prone code; the rest is configuration that's best verified by eye.
- **Alternatives considered**:
  - **End-to-end UI automation (e.g., `enigo`, WinAppDriver)**: rejected — disproportionate to v1 scope.
  - **No tests at all**: rejected — the formatter/fallback is the easiest place for a regression to land silently and trivial to cover.
