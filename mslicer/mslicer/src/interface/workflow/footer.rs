//! The status bar at the bottom: the printer and its build volume, and the credit to mslicer.

use common::units::Milimeter;
use egui::{Align, Button, Context, Frame, Layout, Margin, RichText, TopBottomPanel};

use crate::{
    core::App,
    interface::{panels::slice_config::printer_name, theme::color},
};

pub fn ui(app: &mut App, ctx: &Context) {
    TopBottomPanel::bottom("embercast_footer")
        .exact_height(28.0)
        .frame(
            Frame::new()
                .fill(color::BG_SUNKEN)
                .inner_margin(Margin::symmetric(20, 0)),
        )
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(
                    RichText::new(printer_summary(app))
                        .size(12.0)
                        .color(color::BONE_400),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let credit = RichText::new("Based on mslicer · GPL-3.0")
                        .size(12.0)
                        .color(color::BONE_400);
                    if ui.add(Button::new(credit).frame(false)).clicked() {
                        app.config.ui.about = true;
                    }
                });
            });
        });
}

/// Return the printer, its screen resolution, its pixel size and its build volume in one line.
fn printer_summary(app: &App) -> String {
    let config = &app.project.slice_config;
    let resolution = config.platform_resolution;
    let size = config.platform_size.map(|length| length.get::<Milimeter>());
    let pixel = size.x / resolution.x.max(1) as f32 * 1000.0;
    format!(
        "{} · {} × {} px · {pixel:.0} µm pixels · build volume {:.1} × {:.1} × {:.0} mm",
        printer_name(&app.config, &app.state.selected_printer),
        resolution.x,
        resolution.y,
        size.x,
        size.y,
        size.z,
    )
}
