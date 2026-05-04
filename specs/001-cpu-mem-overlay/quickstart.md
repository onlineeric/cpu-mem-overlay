# Quickstart: CPU/Memory Overlay (V1)

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Date**: 2026-05-04

This is how a developer (or a reviewer) gets the overlay running locally after `/speckit-implement` has produced the source. It assumes:

- Windows 11 x64
- Rust toolchain (`rustc` / `cargo`) already installed — confirmed by the user; no setup step needed
- The repository is checked out at `C:\repos\cpu-mem-overlay\` and the working branch is `001-cpu-mem-overlay`

---

## 1. Build

From the repository root:

```powershell
cargo build --release
```

A release build is recommended even during development because it makes startup snappy enough to verify SC-001 ("appears within 3 s").

---

## 2. Run

```powershell
cargo run --release
```

A small borderless window should appear in the bottom-right corner of the primary monitor, adjacent to the taskbar, showing:

```text
CPU 0%
MEM 47%
```

(Values will vary; the CPU line may briefly show `CPU --` on the very first tick — this is expected, see [data-model.md](./data-model.md).)

---

## 3. What to verify by eye

Walk through the manual checks in [contracts/ui-contract.md](./contracts/ui-contract.md) §4. The short version:

1. The window appears within ~3 seconds of launch.
2. Both numbers change at least once per second.
3. The window stays on top when you maximize Notepad, Edge, or File Explorer.
4. There's no title bar, no border, and no drag/right-click affordances.
5. Task Manager shows the overlay using ~0% CPU and < 50 MB memory while idle.

If any of these fail, fix before considering the v1 acceptance criteria met.

---

## 4. Stop the app

There is no in-app close button in v1 (per FR-007 / non-goals). Terminate the process via:

- Task Manager → find the binary by name → **End task**, or
- Press `Ctrl+C` in the terminal where you ran `cargo run`.

---

## 5. Run the unit tests

```powershell
cargo test
```

This runs the small pure-logic test suite covering the percentage formatter and `--` fallback (see R-008 in [research.md](./research.md)). GUI behavior is verified by eye, not in `cargo test`.

---

## 6. Troubleshooting

| Symptom                                                 | Likely cause / fix |
|---------------------------------------------------------|--------------------|
| Window doesn't appear at all                            | Check the terminal output of `cargo run` for a panic. The most common cause is a missing graphics backend on a fresh VM — install the latest GPU drivers. |
| `CPU --` never goes away                                | The `sysinfo::System` is not being refreshed each tick, or the initial refresh + delay was skipped. Verify that `refresh_cpu_usage()` is called once at startup, then again at each tick. |
| Window appears but isn't on top                         | Verify `ViewportBuilder::with_window_level(WindowLevel::AlwaysOnTop)` is set at startup (R-006). |
| Window covers the taskbar / Start menu                  | Position is being computed from the full screen rect instead of the work area. Use the work-area approach in R-004. |
| Visible CPU usage from the overlay itself > 1% at idle  | The repaint loop is busy-spinning instead of using `request_repaint_after`. See R-003. |
