# Feature Specification: CPU/Memory Overlay (V1)

**Feature Branch**: `001-cpu-mem-overlay`
**Created**: 2026-05-04
**Status**: Draft
**Input**: User description: "read @specs/v1.md" — build a minimal Windows 11 desktop overlay that displays current CPU and memory usage, refreshing once per second, sourced from `specs/v1.md`.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - At-a-glance CPU and memory monitoring (Priority: P1)

A power user wants to keep an eye on their system's CPU and memory utilization while working in other applications. They launch the overlay app once and immediately see two lines of text — `CPU xx%` and `MEM xx%` — pinned near the taskbar, always on top of every other window. The values refresh themselves once per second so the user can spot spikes or sustained pressure without opening Task Manager.

**Why this priority**: This is the entire purpose of v1. Without continuous, visible CPU and memory percentages, the app delivers no value. Every other capability listed below depends on this baseline working.

**Independent Test**: Launch the application on a Windows 11 machine. Confirm that within a few seconds the overlay appears near the taskbar showing two lines (`CPU xx%` and `MEM xx%`), that both values are integers between 0 and 100, that they update at least once per second, and that the overlay remains visible when other windows are focused or maximized.

**Acceptance Scenarios**:

1. **Given** the overlay app has just been launched on a Windows 11 desktop, **When** the user looks at the screen, **Then** an overlay window appears near the taskbar showing `CPU xx%` on one line and `MEM xx%` on a second line, with both `xx` values being integers in the range 0–100.
2. **Given** the overlay is running, **When** one second elapses, **Then** the displayed CPU and memory values are recomputed and the on-screen text reflects the most recent reading.
3. **Given** the overlay is running and the user opens or maximizes another application, **When** that other application takes focus, **Then** the overlay remains visible on top and continues updating.
4. **Given** the overlay is running, **When** a transient error prevents reading CPU or memory metrics, **Then** the corresponding line shows the fallback text (`CPU --` or `MEM --`) and the app continues running and retries on the next refresh.

---

### Edge Cases

- **Metric read failure**: If the underlying OS query for CPU or memory fails on a given tick, the affected line displays `CPU --` or `MEM --` for that tick and the app must continue running and attempt the next refresh.
- **Very high or very low values**: When CPU usage is reported as 0% or 100% (or memory likewise), the display still renders correctly as `CPU 0%` / `CPU 100%` / `MEM 0%` / `MEM 100%` without truncation, overflow, or extra padding.
- **Long-running session**: The app is expected to remain visible and accurate across many hours of continuous use without drifting, leaking memory, or visibly degrading.
- **Taskbar position varies between systems**: The "near the taskbar" default position may not match every user's taskbar location in v1; the position is fixed by design and adjusting it is explicitly out of scope.
- **App must be closeable**: Although there is no in-app close button in v1, the user must be able to terminate the process by standard OS means (e.g., Task Manager) without ill effects.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The overlay MUST display the current CPU usage as a line of text in the exact format `CPU xx%`, where `xx` is the most recent CPU usage rounded to an integer percentage in the range 0–100.
- **FR-002**: The overlay MUST display the current memory usage as a line of text in the exact format `MEM xx%`, where `xx` is the most recent system memory usage rounded to an integer percentage in the range 0–100.
- **FR-003**: The overlay MUST refresh both displayed values automatically once per second.
- **FR-004**: The overlay window MUST be borderless and have no title bar.
- **FR-005**: The overlay window MUST stay always on top of other windows.
- **FR-006**: The overlay window MUST be small in size — only large enough to comfortably display the two lines of text — and MUST appear at a fixed default position near the Windows 11 taskbar area, with no user-facing controls for repositioning, resizing, or configuring it in v1.
- **FR-007**: The overlay MUST continue running and remain visible until the user manually terminates the process; it MUST NOT exit on its own under normal operation.
- **FR-008**: When the overlay cannot read CPU or memory metrics on a given refresh, it MUST display fallback text `CPU --` and/or `MEM --` for the affected metric, and MUST NOT crash; the next refresh MUST attempt to read the metrics again.
- **FR-009**: The overlay MUST remain lightweight in steady-state operation, so that running it has no perceptible impact on overall system performance.
- **FR-010**: The overlay MUST NOT include any settings UI, right-click menu, tray icon, transparency controls, drag-and-drop support, charts, or styling beyond simple text rendering in v1.

### Key Entities

- **System Metric Reading**: A snapshot of system state at a given instant, comprising a CPU usage percentage (0–100) and a memory usage percentage (0–100). Each reading either contains valid integer values for both metrics, or marks one or both as unavailable so the display can fall back to `--`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After launching the app on Windows 11, the overlay appears on screen with both `CPU xx%` and `MEM xx%` lines visible within 3 seconds.
- **SC-002**: While the overlay is running, the displayed CPU and memory values update at least once per second, with no observed gap longer than 2 seconds between updates during a 10-minute session.
- **SC-003**: The overlay remains visible on top of other windows in 100% of cases when another application is opened, focused, or maximized during a normal desktop session.
- **SC-004**: Across a 1-hour continuous run, the overlay does not crash, freeze, or stop updating, and recovers gracefully (resuming live values within one refresh) from any transient metric-read failure that occurs during the run.
- **SC-005**: Running the overlay alongside the user's normal workload produces no perceptible impact on system responsiveness — users cannot tell from system performance alone whether the overlay is running.
- **SC-006**: A new user can determine current CPU and memory usage by glancing at the overlay in under 2 seconds, without needing any instructions, menus, or interactions.

## Assumptions

- The target environment is Windows 11 on x64; other Windows versions and architectures are out of scope for v1.
- A single primary monitor is assumed; multi-monitor positioning behavior is explicitly out of scope.
- "Near the taskbar" is interpreted as a fixed default position chosen by the implementation (e.g., a corner adjacent to the default Windows 11 taskbar); making this user-configurable is out of scope.
- "Memory usage percentage" refers to overall system memory utilization (used / total physical memory), not per-process memory.
- "CPU usage percentage" refers to overall system CPU utilization across all logical processors, averaged over the most recent sampling interval.
- The user has permission to read standard system performance counters; no elevated privileges are required.
- The app will be terminated by standard OS means (Task Manager, signing out, shutdown); no in-app exit affordance is required in v1.
- The user is comfortable launching the app manually each session; auto-start with Windows is explicitly out of scope.
