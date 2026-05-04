//! Multi-monitor work-area enumeration and on-screen overlap check.
//!
//! Wraps Win32 `EnumDisplayMonitors` + `GetMonitorInfoW` to compute the
//! union of all visible monitor work areas. Used to decide whether a
//! configured `anchor_position` overlaps any visible work area or should
//! roll back to the default anchor (FR-012). The pure overlap helper
//! (`is_on_any_work_area`) is unit-tested against synthetic rectangles;
//! `enumerate_work_areas` is exercised by the live overlay only.

use windows::Win32::Foundation::{BOOL, LPARAM, RECT, TRUE};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WindowRect {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WorkArea {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

pub(crate) fn is_on_any_work_area(window: WindowRect, work_areas: &[WorkArea]) -> bool {
    if window.right <= window.left || window.bottom <= window.top {
        return false;
    }
    work_areas.iter().any(|wa| {
        if wa.right <= wa.left || wa.bottom <= wa.top {
            return false;
        }
        let ix_left = window.left.max(wa.left);
        let ix_right = window.right.min(wa.right);
        let ix_top = window.top.max(wa.top);
        let ix_bottom = window.bottom.min(wa.bottom);
        ix_left < ix_right && ix_top < ix_bottom
    })
}

pub(crate) fn enumerate_work_areas() -> Vec<WorkArea> {
    let mut out: Vec<WorkArea> = Vec::new();
    let lparam = LPARAM(&mut out as *mut Vec<WorkArea> as isize);
    let _ = unsafe {
        EnumDisplayMonitors(
            HDC(std::ptr::null_mut()),
            None,
            Some(monitor_enum_proc),
            lparam,
        )
    };
    out
}

unsafe extern "system" fn monitor_enum_proc(
    monitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let out = &mut *(lparam.0 as *mut Vec<WorkArea>);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if GetMonitorInfoW(monitor, &mut info as *mut MONITORINFO).as_bool() {
        out.push(WorkArea {
            left: info.rcWork.left,
            top: info.rcWork.top,
            right: info.rcWork.right,
            bottom: info.rcWork.bottom,
        });
    }
    TRUE
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(left: i32, top: i32, right: i32, bottom: i32) -> WindowRect {
        WindowRect {
            left,
            top,
            right,
            bottom,
        }
    }

    fn work(left: i32, top: i32, right: i32, bottom: i32) -> WorkArea {
        WorkArea {
            left,
            top,
            right,
            bottom,
        }
    }

    #[test]
    fn fully_contained_window_is_on_screen() {
        let window = rect(100, 100, 200, 200);
        let work_areas = vec![work(0, 0, 1920, 1080)];
        assert!(is_on_any_work_area(window, &work_areas));
    }

    #[test]
    fn fully_outside_window_is_off_screen() {
        let window = rect(2000, 2000, 2100, 2100);
        let work_areas = vec![work(0, 0, 1920, 1080)];
        assert!(!is_on_any_work_area(window, &work_areas));
    }

    #[test]
    fn straddling_two_adjacent_areas_is_on_screen() {
        let window = rect(1900, 100, 2000, 200);
        let work_areas = vec![work(0, 0, 1920, 1080), work(1920, 0, 3840, 1080)];
        assert!(is_on_any_work_area(window, &work_areas));
    }

    #[test]
    fn in_gap_between_non_adjacent_areas_is_off_screen() {
        let window = rect(1950, 100, 2050, 200);
        let work_areas = vec![work(0, 0, 1920, 1080), work(2100, 0, 4020, 1080)];
        assert!(!is_on_any_work_area(window, &work_areas));
    }

    #[test]
    fn zero_area_work_area_excluded() {
        let window = rect(0, 0, 100, 100);
        let work_areas = vec![work(0, 0, 0, 0), work(50, 50, 50, 50)];
        assert!(!is_on_any_work_area(window, &work_areas));
    }

    #[test]
    fn empty_work_area_list_is_off_screen() {
        let window = rect(0, 0, 100, 100);
        assert!(!is_on_any_work_area(window, &[]));
    }

    #[test]
    fn touching_edge_only_is_off_screen() {
        // Touching but not overlapping (right edge equals work area left): no
        // pixels in common, treated as off-screen.
        let window = rect(1920, 100, 2020, 200);
        let work_areas = vec![work(0, 0, 1920, 1080)];
        assert!(!is_on_any_work_area(window, &work_areas));
    }

    #[test]
    fn negative_coordinates_supported() {
        // Monitor positioned above/left of primary uses negative coords.
        let window = rect(-500, -300, -400, -200);
        let work_areas = vec![work(-1920, -1080, 0, 0)];
        assert!(is_on_any_work_area(window, &work_areas));
    }
}
