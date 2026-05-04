#![cfg_attr(not(test), windows_subsystem = "windows")]

mod app;
mod color;
mod config;
mod monitors;
mod sampler;

use eframe::{egui, App};

use crate::app::{
    compute_window_size, primary_work_area_bottom_right, OverlayApp, POSITION_MARGIN_PX,
};
use crate::config::{load_from_exe_dir, Anchor, OverlayConfig};
use crate::monitors::{enumerate_work_areas, is_on_any_work_area, WindowRect};

fn main() -> eframe::Result<()> {
    let config = load_from_exe_dir();

    let window_size = compute_window_size(config.font_size);
    let position = resolve_position(&config, window_size);

    let viewport = egui::ViewportBuilder::default()
        .with_decorations(false)
        .with_resizable(false)
        .with_inner_size(window_size)
        .with_position(position)
        .with_window_level(egui::WindowLevel::AlwaysOnTop)
        .with_transparent(true);

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "cpu-mem-overlay",
        options,
        Box::new(move |_cc| Ok(Box::new(OverlayApp::new(config)) as Box<dyn App>)),
    )
}

fn resolve_position(config: &OverlayConfig, window_size: egui::Vec2) -> egui::Pos2 {
    match config.anchor {
        Anchor::VirtualScreen { x, y } => {
            let candidate = WindowRect {
                left: x,
                top: y,
                right: x + window_size.x.round() as i32,
                bottom: y + window_size.y.round() as i32,
            };
            if is_on_any_work_area(candidate, &enumerate_work_areas()) {
                egui::pos2(x as f32, y as f32)
            } else {
                primary_work_area_bottom_right(window_size, POSITION_MARGIN_PX)
            }
        }
        Anchor::Default => primary_work_area_bottom_right(window_size, POSITION_MARGIN_PX),
    }
}
