//! The rail on the left with the tools of the current step.

use egui::{Context, Frame, Margin, Sense, SidePanel, Stroke, vec2};

use crate::{
    core::App,
    interface::{
        theme::color,
        workflow::{RailTool, slice, style::rail_button},
    },
};

pub fn ui(app: &mut App, ctx: &Context) {
    SidePanel::left("embercast_rail")
        .exact_width(92.0)
        .resizable(false)
        .frame(
            Frame::new()
                .fill(color::SURFACE_INSET)
                .inner_margin(Margin::same(8)),
        )
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            for &tool in app.state.step.tools() {
                if tool.is_separated() {
                    let size = vec2(ui.available_width(), 17.0);
                    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                    ui.painter().hline(
                        rect.shrink2(vec2(6.0, 0.0)).x_range(),
                        rect.center().y,
                        Stroke::new(1.0_f32, color::divider()),
                    );
                }

                let selected = app.state.rail_tool == tool;
                let response =
                    rail_button(ui, tool.icon(), tool.name(), selected, tool.is_danger())
                        .on_hover_text(tool.description());
                if response.clicked() {
                    select(app, tool);
                }
            }
        });
}

/// Make `tool` the active tool. The check tool also starts the print check.
fn select(app: &mut App, tool: RailTool) {
    app.state.rail_tool = tool;
    if tool == RailTool::CheckPrint {
        slice::start_check(app);
    }
}
