//! GUI app entry: window setup helpers and the `eframe::App` impl.
//!
//! Owns the rendering loop, the startup-corner anchor computation, the
//! interval-gated sampling logic (so input events do NOT drive extra metric
//! reads — FR-019/FR-020), and the optional drag-to-reposition handling
//! (FR-021..FR-024). Pure helpers (`compute_window_size`, `should_sample`)
//! are unit-tested at the bottom of this file.

use std::time::{Duration, Instant};

use eframe::{egui, App, Frame};

use crate::config::{OverlayConfig, StartupPosition};
use crate::sampler::{format_line, MetricSnapshot, Sampler};

pub(crate) const POSITION_MARGIN_PX: f32 = 12.0;

const V1_FONT_SIZE: f32 = 14.0;
const V1_WINDOW_WIDTH: f32 = 96.0;
const V1_WINDOW_HEIGHT: f32 = 44.0;

#[derive(Debug, Clone, Copy)]
struct WorkAreaBounds {
    left: f32,
    right: f32,
    bottom: f32,
}

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

fn primary_work_area_position(
    startup_position: StartupPosition,
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
        return position_in_work_area(
            WorkAreaBounds {
                left: 0.0,
                right: 1920.0,
                bottom: 1040.0,
            },
            startup_position,
            window_size,
            margin_px,
        );
    }

    position_in_work_area(
        WorkAreaBounds {
            left: rect.left as f32,
            right: rect.right as f32,
            bottom: rect.bottom as f32,
        },
        startup_position,
        window_size,
        margin_px,
    )
}

pub(crate) fn primary_work_area_bottom_right(
    window_size: egui::Vec2,
    margin_px: f32,
) -> egui::Pos2 {
    primary_work_area_position(StartupPosition::BottomRight, window_size, margin_px)
}

pub(crate) fn primary_work_area_bottom_left(window_size: egui::Vec2, margin_px: f32) -> egui::Pos2 {
    primary_work_area_position(StartupPosition::BottomLeft, window_size, margin_px)
}

fn position_in_work_area(
    work_area: WorkAreaBounds,
    startup_position: StartupPosition,
    window_size: egui::Vec2,
    margin_px: f32,
) -> egui::Pos2 {
    let x = match startup_position {
        StartupPosition::BottomLeft => work_area.left + margin_px,
        StartupPosition::BottomRight => work_area.right - window_size.x - margin_px,
    };
    let y = work_area.bottom - window_size.y - margin_px;
    egui::pos2(x.max(0.0), y.max(0.0))
}

pub(crate) struct OverlayApp {
    config: OverlayConfig,
    sampler: Sampler,
    snapshot: MetricSnapshot,
    last_sample_at: Instant,
    background: egui::Color32,
    font_color: egui::Color32,
}

impl OverlayApp {
    pub(crate) fn new(config: OverlayConfig) -> Self {
        // Ensure the very first frame triggers a sample.
        let last_sample_at = Instant::now()
            .checked_sub(config.refresh_interval)
            .unwrap_or_else(Instant::now);
        let [r, g, b, a] = config.background_color;
        let [font_r, font_g, font_b, font_a] = config.font_color;
        Self {
            config,
            sampler: Sampler::new(),
            snapshot: MetricSnapshot::default(),
            last_sample_at,
            background: egui::Color32::from_rgba_unmultiplied(r, g, b, a),
            font_color: egui::Color32::from_rgba_unmultiplied(font_r, font_g, font_b, font_a),
        }
    }
}

impl App for OverlayApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Always clear to fully transparent; the panel's Frame::fill paints
        // the actual configured background color (with correct alpha
        // blending). This keeps the OS-level window transparency available
        // when the configured alpha is 0.
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let now = Instant::now();
        if should_sample(now, self.last_sample_at, self.config.refresh_interval) {
            self.snapshot = self.sampler.sample();
            self.last_sample_at = now;
        }

        let panel_frame = egui::Frame::central_panel(&ctx.style()).fill(self.background);
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
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
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(format_line("CPU", self.snapshot.cpu))
                                .color(self.font_color)
                                .size(self.config.font_size),
                        )
                        .selectable(false),
                    );
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(format_line("MEM", self.snapshot.memory))
                                .color(self.font_color)
                                .size(self.config.font_size),
                        )
                        .selectable(false),
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

    #[test]
    fn bottom_right_position_uses_work_area_bottom_edge() {
        let pos = position_in_work_area(
            WorkAreaBounds {
                left: 0.0,
                right: 1920.0,
                bottom: 1040.0,
            },
            StartupPosition::BottomRight,
            egui::vec2(96.0, 44.0),
            12.0,
        );

        assert_eq!(pos, egui::pos2(1812.0, 984.0));
    }

    #[test]
    fn bottom_left_position_uses_same_work_area_bottom_edge() {
        let pos = position_in_work_area(
            WorkAreaBounds {
                left: 0.0,
                right: 1920.0,
                bottom: 1040.0,
            },
            StartupPosition::BottomLeft,
            egui::vec2(96.0, 44.0),
            12.0,
        );

        assert_eq!(pos, egui::pos2(12.0, 984.0));
    }
}
