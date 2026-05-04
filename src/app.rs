//! GUI app entry: window setup helpers and the `eframe::App` impl.
//!
//! Owns the rendering loop, the v1 default-anchor computation, the
//! interval-gated sampling logic (so input events do NOT drive extra metric
//! reads — FR-019/FR-020), and the optional drag-to-reposition handling
//! (FR-021..FR-024). Pure helpers (`compute_window_size`, `should_sample`)
//! are unit-tested at the bottom of this file.

use std::time::{Duration, Instant};

use eframe::{egui, App, Frame};

use crate::config::OverlayConfig;
use crate::sampler::{format_line, MetricSnapshot, Sampler};

pub(crate) const POSITION_MARGIN_PX: f32 = 12.0;
const WORK_AREA_BOTTOM_LEFT_X_OFFSET_PX: f32 = 150.0;
const WORK_AREA_BOTTOM_LEFT_Y_OFFSET_PX: f32 = 55.0;

const V1_FONT_SIZE: f32 = 14.0;
const V1_WINDOW_WIDTH: f32 = 96.0;
const V1_WINDOW_HEIGHT: f32 = 44.0;

pub(crate) fn compute_window_size(font_size: f32) -> egui::Vec2 {
    let scale = font_size / V1_FONT_SIZE;
    egui::vec2(V1_WINDOW_WIDTH * scale, V1_WINDOW_HEIGHT * scale)
}

pub(crate) fn should_sample(now: Instant, last_sample_at: Instant, interval: Duration) -> bool {
    if interval.is_zero() {
        return true;
    }
    now.saturating_duration_since(last_sample_at) >= interval
}

pub(crate) fn primary_work_area_bottom_left(
    window_size: egui::Vec2,
    margin_px: f32,
) -> egui::Pos2 {
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
        let x = WORK_AREA_BOTTOM_LEFT_X_OFFSET_PX + margin_px;
        let y = 1040.0 - window_size.y - margin_px + WORK_AREA_BOTTOM_LEFT_Y_OFFSET_PX;
        return egui::pos2(x.max(0.0), y.max(0.0));
    }

    let x = rect.left as f32 + WORK_AREA_BOTTOM_LEFT_X_OFFSET_PX + margin_px;
    let y = rect.bottom as f32 - window_size.y - margin_px + WORK_AREA_BOTTOM_LEFT_Y_OFFSET_PX;
    egui::pos2(x.max(0.0), y.max(0.0))
}

pub(crate) struct OverlayApp {
    config: OverlayConfig,
    sampler: Sampler,
    snapshot: MetricSnapshot,
    last_sample_at: Instant,
    clear_color_normalized: [f32; 4],
}

impl OverlayApp {
    pub(crate) fn new(config: OverlayConfig) -> Self {
        // Ensure the very first frame triggers a sample.
        let last_sample_at = Instant::now()
            .checked_sub(config.refresh_interval)
            .unwrap_or_else(Instant::now);
        let [r, g, b, a] = config.background_color;
        Self {
            config,
            sampler: Sampler::new(),
            snapshot: MetricSnapshot::default(),
            last_sample_at,
            clear_color_normalized: [
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
            ],
        }
    }
}

impl App for OverlayApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        self.clear_color_normalized
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let now = Instant::now();
        if should_sample(now, self.last_sample_at, self.config.refresh_interval) {
            self.snapshot = self.sampler.sample();
            self.last_sample_at = now;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.config.draggable {
                let response = ui.interact(
                    ui.max_rect(),
                    egui::Id::new("overlay-drag"),
                    egui::Sense::drag(),
                );
                if response.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
            }
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(format_line("CPU", self.snapshot.cpu))
                        .size(self.config.font_size),
                );
                ui.label(
                    egui::RichText::new(format_line("MEM", self.snapshot.memory))
                        .size(self.config.font_size),
                );
            });
        });

        let elapsed = now.saturating_duration_since(self.last_sample_at);
        let next = self.config.refresh_interval.saturating_sub(elapsed);
        ctx.request_repaint_after(next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_window_size_default_matches_v1() {
        let size = compute_window_size(V1_FONT_SIZE);
        assert!((size.x - V1_WINDOW_WIDTH).abs() < 0.5);
        assert!((size.y - V1_WINDOW_HEIGHT).abs() < 0.5);
    }

    #[test]
    fn compute_window_size_doubles_when_font_doubles() {
        let single = compute_window_size(V1_FONT_SIZE);
        let double = compute_window_size(V1_FONT_SIZE * 2.0);
        assert!((double.x - single.x * 2.0).abs() < 0.5);
        assert!((double.y - single.y * 2.0).abs() < 0.5);
    }

    #[test]
    fn compute_window_size_small_font_finite_positive() {
        let size = compute_window_size(1.0);
        assert!(size.x.is_finite() && size.x > 0.0);
        assert!(size.y.is_finite() && size.y > 0.0);
    }

    #[test]
    fn compute_window_size_monotonic() {
        let a = compute_window_size(10.0);
        let b = compute_window_size(20.0);
        assert!(b.x > a.x);
        assert!(b.y > a.y);
    }

    #[test]
    fn should_sample_at_exact_interval_returns_true() {
        let base = Instant::now();
        let interval = Duration::from_millis(1000);
        let now = base + interval;
        assert!(should_sample(now, base, interval));
    }

    #[test]
    fn should_sample_just_after_interval_returns_true() {
        let base = Instant::now();
        let interval = Duration::from_millis(1000);
        let now = base + interval + Duration::from_nanos(1);
        assert!(should_sample(now, base, interval));
    }

    #[test]
    fn should_sample_just_before_interval_returns_false() {
        let base = Instant::now();
        let interval = Duration::from_millis(1000);
        let now = base + interval - Duration::from_nanos(1);
        assert!(!should_sample(now, base, interval));
    }

    #[test]
    fn should_sample_now_equals_last_returns_false() {
        let base = Instant::now();
        let interval = Duration::from_millis(1000);
        assert!(!should_sample(base, base, interval));
    }

    #[test]
    fn should_sample_zero_interval_always_true() {
        let base = Instant::now();
        assert!(should_sample(base, base, Duration::ZERO));
    }
}
