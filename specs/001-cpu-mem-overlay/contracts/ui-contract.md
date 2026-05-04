# UI / Window Contract: CPU/Memory Overlay (V1)

**Feature**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Date**: 2026-05-04

This is the only contract this feature exposes. It is a *visual / window* contract — there are no APIs, command schemas, or network endpoints in v1. Every requirement here is verifiable by manual inspection on a Windows 11 machine.

---

## 1. Window properties

| Property                | Required value                                                              | Source     |
|-------------------------|-----------------------------------------------------------------------------|------------|
| Decorations / title bar | None — fully borderless                                                     | FR-004     |
| Always on top           | Yes — stays above all normal application windows                            | FR-005, SC-003 |
| Resizable               | No                                                                          | FR-006     |
| Size                    | Just large enough to display the two text lines comfortably; no extra padding beyond what readability requires | FR-006 |
| Default position        | Bottom-right of the primary monitor's *work area*, with a small margin (≈ 12 px) so it sits adjacent to the taskbar | R-004, FR-006 |
| User reposition / drag  | None                                                                        | FR-010     |
| Tray icon               | None                                                                        | FR-010     |
| Right-click menu        | None                                                                        | FR-010     |
| Transparency            | Opaque (transparency controls are out of scope; a solid background is fine) | FR-010     |
| Lifetime                | Window remains open until the user terminates the process via OS means      | FR-007     |

---

## 2. Rendered text

The window contains exactly two lines of text, in order, top-to-bottom:

```text
CPU xx%
MEM xx%
```

| Line | Format             | When values are unavailable |
|------|--------------------|-----------------------------|
| 1    | `CPU {pct}%`       | `CPU --`                    |
| 2    | `MEM {pct}%`       | `MEM --`                    |

Where `{pct}` is an integer in `0..=100` (no leading zeros, no decimals, no padding). Examples: `CPU 0%`, `CPU 7%`, `CPU 18%`, `CPU 100%`.

The two lines update independently. The CPU line may show `--` while the MEM line shows a value (e.g., the very first tick after launch — see [data-model.md](../data-model.md)).

No other text, icons, separators, charts, or controls appear in the window in v1.

---

## 3. Refresh behavior

- The displayed values must be recomputed and re-rendered approximately once per second (FR-003, SC-002).
- A transient failure to read either metric on a given tick must not crash the app; the failed line shows `--` and the next tick must attempt the read again (FR-008, SC-004).
- The app must remain visible and continue refreshing until manually terminated (FR-007).

---

## 4. Manual verification steps

Each of the following is a pass/fail check the developer or reviewer performs by eye on a Windows 11 machine. They map 1-to-1 onto the acceptance scenarios in [spec.md](../spec.md).

1. **Cold start**: launch the binary. Within 3 s the overlay appears near the taskbar showing two lines, both starting with `CPU ` and `MEM ` respectively. (SC-001)
2. **Live updates**: watch the overlay for 30 s while opening apps / running a CPU-intensive task. Both values change at least once per second. No gap longer than ~2 s between updates is observed. (FR-003, SC-002)
3. **Always on top**: open Notepad, Edge, and File Explorer; maximize each in turn. The overlay remains visible above each. (FR-005, SC-003)
4. **Borderless / no chrome**: inspect the window. There is no title bar, minimize/maximize/close buttons, or window border. (FR-004)
5. **Fixed position**: the overlay does not move when other windows are dragged near it, and there is no visible drag handle. (FR-006, FR-010)
6. **Format**: at no point during a 1-minute observation does either line display anything other than `CPU xx%`/`CPU --` or `MEM xx%`/`MEM --`, where `xx` is an integer 0–100. (FR-001, FR-002, FR-008)
7. **Long-run stability**: leave the overlay running for 1 hour while using the system normally. It does not crash, freeze, or stop updating. (SC-004)
8. **Lightweight**: in Task Manager during step 7, the overlay's CPU column reads ~0% and its memory remains under ~50 MB. The user perceives no impact on system responsiveness. (SC-005, plan.md performance goals)
9. **Termination**: end the process from Task Manager. The window disappears cleanly with no zombie processes. (FR-007)

---

## 5. Out of scope (explicit non-contract)

The following are *not* part of this contract and must not appear in v1, per the spec's non-goals (§3) and FR-010:

- Settings UI, preferences dialog, or configuration file
- Tray / notification-area icon
- Right-click menu
- User-controlled position, size, font, color, or refresh rate
- Transparency / opacity control
- Drag-and-drop, click-through, or keyboard shortcuts
- Multi-monitor positioning logic
- Charts, graphs, history, or per-process breakdowns
- Auto-start with Windows
