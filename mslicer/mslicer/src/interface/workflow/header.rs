//! The bar at the top: the name of the app, the open project, the three steps and the file actions.

use std::cmp::Ordering;

use const_format::concatcp;
use egui::{
    Align, Context, FontFamily, Frame, Layout, Margin, Rect, RichText, Sense, Stroke,
    TopBottomPanel, Ui, UiBuilder, vec2,
};
use egui_phosphor::regular::{
    ARROW_CLOCKWISE, ARROW_COUNTER_CLOCKWISE, FLOPPY_DISK, FOLDER_OPEN, GEAR_SIX, LIST,
};

use crate::{
    core::App,
    interface::{
        elements::top_bar,
        shortcuts,
        theme::{self, DISPLAY, color},
        workflow::{
            Step, go_to,
            style::{StepState, hint, icon_button, outline_button, step_button},
        },
    },
};

const STEP_WIDTH: f32 = 160.0;
const CONNECTOR_WIDTH: f32 = 28.0;
const STEP_GAP: f32 = 10.0;

pub fn ui(app: &mut App, ctx: &Context) {
    TopBottomPanel::top("embercast_header")
        .exact_height(60.0)
        .frame(
            Frame::new()
                .fill(color::SURFACE_INSET)
                .inner_margin(Margin::symmetric(20, 0)),
        )
        .show(ctx, |ui| {
            let full = ui.max_rect();

            let left = UiBuilder::new()
                .max_rect(full)
                .layout(Layout::left_to_right(Align::Center));
            ui.scope_builder(left, |ui| {
                ui.label(
                    RichText::new("EMBERCAST")
                        .family(FontFamily::Name(DISPLAY.into()))
                        .size(20.0)
                        .extra_letter_spacing(2.8)
                        .color(color::ACCENT),
                );
                ui.add_space(12.0);
                ui.label(hint(project_name(app)));
            });

            let right = UiBuilder::new()
                .max_rect(full)
                .layout(Layout::right_to_left(Align::Center));
            ui.scope_builder(right, |ui| actions(app, ui, ctx));

            let width = STEP_WIDTH * 3.0 + (CONNECTOR_WIDTH + 2.0 * STEP_GAP) * 2.0;
            let middle = Rect::from_center_size(full.center(), vec2(width, full.height()));
            let middle = UiBuilder::new()
                .max_rect(middle)
                .layout(Layout::left_to_right(Align::Center));
            ui.scope_builder(middle, |ui| stepper(app, ui));
        });
}

/// Return the file name of the project, or a placeholder for a project that has no file yet.
fn project_name(app: &App) -> String {
    match app.project.path.as_ref().and_then(|path| path.file_name()) {
        Some(name) => name.to_string_lossy().into_owned(),
        None => "New project".to_owned(),
    }
}

/// Show the three steps with a line between them. A click on a step goes to that step.
fn stepper(app: &mut App, ui: &mut Ui) {
    ui.spacing_mut().item_spacing.x = STEP_GAP;
    let current = app.state.step.index();

    for (index, step) in Step::ALL.into_iter().enumerate() {
        if index > 0 {
            let (rect, _) = ui.allocate_exact_size(vec2(CONNECTOR_WIDTH, 1.0), Sense::hover());
            let line = if index <= current {
                color::border_strong()
            } else {
                color::border_steel()
            };
            ui.painter()
                .hline(rect.x_range(), rect.center().y, Stroke::new(1.0_f32, line));
        }

        let state = match index.cmp(&current) {
            Ordering::Less => StepState::Done,
            Ordering::Equal => StepState::Current,
            Ordering::Greater => StepState::Later,
        };
        if step_button(ui, index + 1, step.name(), state, STEP_WIDTH).clicked() {
            go_to(app, step);
        }
    }
}

/// Show the file actions from right to left: the menu, the settings, save, open, the tasks, redo and undo.
fn actions(app: &mut App, ui: &mut Ui, ctx: &Context) {
    ui.spacing_mut().item_spacing.x = 6.0;

    ui.menu_button(theme::label(concatcp!(LIST, " Menu")), |ui| {
        top_bar::menus(app, ui, ctx);
    });

    if icon_button(ui, GEAR_SIX, "Settings of the app", true).clicked() {
        app.state.show_app_settings ^= true;
    }

    if outline_button(ui, concatcp!(FLOPPY_DISK, " Save"), 0.0)
        .on_hover_text("Save the project.")
        .clicked()
    {
        (shortcuts::SAVE_PROJECT_SHORTCUT.callback)(app, ctx);
    }

    if outline_button(ui, concatcp!(FOLDER_OPEN, " Open"), 0.0)
        .on_hover_text("Open a project.")
        .clicked()
    {
        (shortcuts::LOAD_PROJECT_SHORTCUT.callback)(app, ctx);
    }

    ui.add_space(6.0);
    top_bar::tasks_button(app, ctx, ui);

    let can_redo = app.history.can_redo();
    if icon_button(ui, ARROW_CLOCKWISE, "Redo", can_redo).clicked() {
        (shortcuts::REDO_SHORTCUT.callback)(app, ctx);
    }
    let can_undo = app.history.can_undo();
    if icon_button(ui, ARROW_COUNTER_CLOCKWISE, "Undo", can_undo).clicked() {
        (shortcuts::UNDO_SHORTCUT.callback)(app, ctx);
    }
}
