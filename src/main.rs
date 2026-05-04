#![cfg_attr(not(test), windows_subsystem = "windows")]

use std::time::Duration;

use eframe::{egui, App, Frame};
use sysinfo::System;

#[derive(Debug, Default, Clone, Copy)]
struct MetricSnapshot {
    cpu: Option<u8>,
    memory: Option<u8>,
}

fn format_line(label: &str, value: Option<u8>) -> String {
    match value {
        Some(n) => format!("{label} {pct}%", pct = n.min(100)),
        None => format!("{label} --"),
    }
}

struct Sampler {
    sys: System,
}

impl Sampler {
    fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        Self { sys }
    }

    fn sample(&mut self) -> MetricSnapshot {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        MetricSnapshot {
            cpu: read_cpu_pct(&self.sys),
            memory: read_memory_pct(&self.sys),
        }
    }
}

fn read_cpu_pct(sys: &System) -> Option<u8> {
    let raw = sys.global_cpu_usage();
    if !raw.is_finite() {
        return None;
    }
    Some(raw.round().clamp(0.0, 100.0) as u8)
}

fn read_memory_pct(sys: &System) -> Option<u8> {
    let total = sys.total_memory();
    if total == 0 {
        return None;
    }
    let used = sys.used_memory();
    let pct = (used as f64 * 100.0) / total as f64;
    if !pct.is_finite() {
        return None;
    }
    Some(pct.round().clamp(0.0, 100.0) as u8)
}

const WINDOW_SIZE: egui::Vec2 = egui::vec2(96.0, 44.0);
const POSITION_MARGIN_PX: f32 = 12.0;

fn primary_work_area_bottom_right(window_size: egui::Vec2, margin_px: f32) -> egui::Pos2 {
    use core::ffi::c_void;
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPI_GETWORKAREA, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    };

    let mut rect = RECT::default();
    let result = unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut rect as *mut RECT as *mut c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    if result.is_err() {
        return egui::pos2(
            1920.0 - window_size.x - margin_px,
            1040.0 - window_size.y - margin_px,
        );
    }

    let x = rect.right as f32 - window_size.x - margin_px;
    let y = rect.bottom as f32 - window_size.y - margin_px;
    egui::pos2(x.max(0.0), y.max(0.0))
}

struct OverlayApp {
    sampler: Sampler,
    snapshot: MetricSnapshot,
}

impl OverlayApp {
    fn new() -> Self {
        Self {
            sampler: Sampler::new(),
            snapshot: MetricSnapshot::default(),
        }
    }
}

impl App for OverlayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.snapshot = self.sampler.sample();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label(format_line("CPU", self.snapshot.cpu));
                ui.label(format_line("MEM", self.snapshot.memory));
            });
        });
        ctx.request_repaint_after(Duration::from_secs(1));
    }
}

fn main() -> eframe::Result<()> {
    let position = primary_work_area_bottom_right(WINDOW_SIZE, POSITION_MARGIN_PX);
    let viewport = egui::ViewportBuilder::default()
        .with_decorations(false)
        .with_resizable(false)
        .with_inner_size(WINDOW_SIZE)
        .with_position(position)
        .with_window_level(egui::WindowLevel::AlwaysOnTop);

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "cpu-mem-overlay",
        options,
        Box::new(|_cc| Ok(Box::new(OverlayApp::new()) as Box<dyn App>)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_some_zero() {
        assert_eq!(format_line("CPU", Some(0)), "CPU 0%");
    }

    #[test]
    fn cpu_some_seven() {
        assert_eq!(format_line("CPU", Some(7)), "CPU 7%");
    }

    #[test]
    fn cpu_some_hundred() {
        assert_eq!(format_line("CPU", Some(100)), "CPU 100%");
    }

    #[test]
    fn cpu_none() {
        assert_eq!(format_line("CPU", None), "CPU --");
    }

    #[test]
    fn mem_some_zero() {
        assert_eq!(format_line("MEM", Some(0)), "MEM 0%");
    }

    #[test]
    fn mem_some_seven() {
        assert_eq!(format_line("MEM", Some(7)), "MEM 7%");
    }

    #[test]
    fn mem_some_hundred() {
        assert_eq!(format_line("MEM", Some(100)), "MEM 100%");
    }

    #[test]
    fn mem_none() {
        assert_eq!(format_line("MEM", None), "MEM --");
    }

    #[test]
    fn out_of_range_clamps_to_hundred() {
        assert_eq!(format_line("CPU", Some(123)), "CPU 100%");
    }
}
