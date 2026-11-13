//! The windows for the print settings and the settings of the app.

use egui::{Context, ScrollArea, Window};

use crate::{
    core::App,
    interface::panels::{slice_config, workspace},
};

pub fn ui(app: &mut App, ctx: &Context) {
    let mut open = app.state.show_print_settings;
    Window::new("Print settings")
        .open(&mut open)
        .default_width(440.0)
        .default_height(560.0)
        .show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| slice_config::ui(app, ui, ctx));
        });
    app.state.show_print_settings &= open;

    let mut open = app.state.show_app_settings;
    Window::new("Settings")
        .open(&mut open)
        .default_width(440.0)
        .default_height(560.0)
        .show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| workspace::ui(app, ui, ctx));
        });
    app.state.show_app_settings &= open;
}
