//! The tool panels of the Supports step.

use std::mem;

use common::units::{Milimeter, Milimeters};
use egui::{CollapsingHeader, Context, DragValue, Grid, Spinner, Ui, vec2};
use tools::supports::mesh::SupportPreset;

use crate::{
    core::App,
    interface::{
        panels::supports as classic,
        workflow::{
            models_on_plate,
            style::{
                check_row, danger_button, field_label, hint, outline_button, quiet_button,
                segmented,
            },
            target_models,
        },
    },
    task::BuildAccelerationStructures,
};

/// The raft height that the raft switch sets, in millimeters.
const DEFAULT_RAFT_HEIGHT: f32 = 1.0;

/// One of the named support presets, or the custom settings of the project.
#[derive(Clone, Copy, PartialEq)]
enum Strength {
    Named(usize),
    Custom,
}

/// Show the strength, the main settings and the button that generates supports for the target models.
pub fn auto(app: &mut App, ui: &mut Ui, ctx: &Context) {
    let config = &mut app.project.support_config;

    ui.label(field_label("Strength"));
    let mut strength = SupportPreset::NAMED
        .iter()
        .position(|(_, preset)| *preset == config.preset)
        .map_or(Strength::Custom, Strength::Named);
    let options: Vec<(Strength, &str)> = (SupportPreset::NAMED.iter().enumerate())
        .map(|(index, (name, _))| (Strength::Named(index), *name))
        .collect();
    if segmented(ui, &mut strength, &options)
        && let Strength::Named(index) = strength
    {
        config.preset = SupportPreset::NAMED[index].1.clone();
    }
    ui.label(hint(match strength {
        Strength::Named(0) => "Thin tips that are easy to remove. For small and light models.",
        Strength::Named(1) => "Good for most models.",
        Strength::Named(_) => "Thick supports for large and heavy models.",
        Strength::Custom => "Your own settings. Change them under Advanced settings.",
    }));

    ui.add_space(4.0);
    let column = (ui.available_width() - 16.0) / 2.0;
    Grid::new("embercast_support_settings")
        .num_columns(2)
        .spacing([16.0, 6.0])
        .min_col_width(column)
        .show(ui, |ui| {
            ui.label(field_label("Lift height"));
            ui.label(field_label("Overhang angle"));
            ui.end_row();

            config.default_lift.with::<Milimeter, _>(|lift| {
                ui.add(
                    DragValue::new(lift)
                        .suffix(" mm")
                        .speed(0.1)
                        .range(0.0..=50.0),
                );
            });
            ui.add(
                DragValue::new(&mut config.auto_placement.max_angle)
                    .suffix("°")
                    .speed(0.5)
                    .range(0.0..=89.0),
            );
            ui.end_row();

            ui.label(field_label("Tip spacing"));
            ui.label(field_label("Tip diameter"));
            ui.end_row();

            ui.add(
                DragValue::new(&mut config.auto_placement.face_support_spacing)
                    .suffix(" mm")
                    .speed(0.05)
                    .range(0.5..=20.0),
            );
            let mut diameter = config.preset.tip_radius.get::<Milimeter>() * 2.0;
            let response = ui.add(
                DragValue::new(&mut diameter)
                    .suffix(" mm")
                    .speed(0.01)
                    .range(0.05..=5.0),
            );
            if response.changed() {
                config.preset.tip_radius = Milimeters::new(diameter / 2.0);
            }
            ui.end_row();
        });

    let mut raft = config.raft_height.get::<Milimeter>() > 0.0;
    if ui.checkbox(&mut raft, "Raft under the supports").changed() {
        config.raft_height = Milimeters::new(if raft { DEFAULT_RAFT_HEIGHT } else { 0.0 });
    }

    CollapsingHeader::new(field_label("Advanced settings"))
        .id_salt("embercast_support_advanced")
        .default_open(false)
        .show(ui, |ui| classic::ui(app, ui, ctx));

    ui.add_space(4.0);
    let targets = target_models(app);
    let text = match targets.len() {
        1 => "Generate supports".to_owned(),
        count => format!("Generate supports for {count} models"),
    };
    let width = ui.available_width();
    let clicked = ui
        .add_enabled_ui(!targets.is_empty(), |ui| outline_button(ui, &text, width))
        .inner
        .clicked();
    if clicked {
        generate(app, targets);
    }

    if !app.state.pending_auto_supports.is_empty() {
        ui.horizontal(|ui| {
            ui.add(Spinner::new().size(14.0));
            ui.label(hint("Preparing the models for supports…"));
        });
    }
    support_summary(app, ui);
}

/// Generate supports for the models at `indexes`.
///
/// A model that has no acceleration structures yet gets a task that builds
/// them. The model then waits in `pending_auto_supports` until
/// `generate_pending` sees the structures.
fn generate(app: &mut App, indexes: Vec<usize>) {
    for index in indexes {
        let model = &app.project.models[index];
        if model.has_acceleration_structures() {
            classic::generate_support(app, index);
        } else if !app.state.pending_auto_supports.contains(&model.id) {
            app.state.pending_auto_supports.push(model.id);
            app.tasks.add(BuildAccelerationStructures::new(model));
        }
    }
}

/// Generate supports for the waiting models that now have their acceleration structures.
///
/// The function removes a model from the list when it gets its supports,
/// and also when the model no longer exists.
pub fn generate_pending(app: &mut App) {
    let pending = mem::take(&mut app.state.pending_auto_supports);
    for id in pending {
        match app.project.models.iter().position(|model| model.id == id) {
            Some(index) if app.project.models[index].has_acceleration_structures() => {
                classic::generate_support(app, index);
            }
            Some(_) => app.state.pending_auto_supports.push(id),
            None => {}
        }
    }
}

/// Show how many supports the models on the plate have.
fn support_summary(app: &App, ui: &mut Ui) {
    let indexes = models_on_plate(app);
    let supports: usize = (indexes.iter())
        .map(|&index| app.project.models[index].supports.support_set().len())
        .sum();
    let models = (indexes.iter())
        .filter(|&&index| !app.project.models[index].supports.support_set().is_empty())
        .count();

    if supports == 0 {
        ui.label(hint("The models on this plate have no supports yet."));
    } else {
        let result = format!(
            "{supports} on {models} model{}",
            if models == 1 { "" } else { "s" }
        );
        check_row(ui, true, "Supports", &result);
    }
}

/// Show the manual support tool: the hint for the clicks, and the actions for the selected supports.
pub fn edit(app: &mut App, ui: &mut Ui) {
    ui.label(hint(
        "Click under a model in the 3D view to add a support. Click a support to select it. Shift-click selects more.",
    ));

    let unprepared: Vec<usize> = (models_on_plate(app).into_iter())
        .filter(|&index| !app.project.models[index].has_acceleration_structures())
        .collect();
    if !unprepared.is_empty() {
        ui.label(hint("Some models are not ready for supports yet."));
        let width = ui.available_width();
        if outline_button(ui, "Prepare models", width).clicked() {
            for index in unprepared {
                app.tasks
                    .add(BuildAccelerationStructures::new(&app.project.models[index]));
            }
        }
    }

    let selected = &mut app.state.selected_supports;
    let count = selected.count();
    if count == 0 {
        return;
    }

    ui.add_space(4.0);
    ui.label(field_label(&format!(
        "{count} support{} selected",
        if count == 1 { "" } else { "s" }
    )));
    let half = (ui.available_width() - 8.0) / 2.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
        if danger_button(ui, "Delete", half).clicked() {
            for support in selected.iter() {
                if let Some(model) = app
                    .project
                    .models
                    .iter_mut()
                    .find(|model| model.id == support.model)
                {
                    model.supports.remove(support.support);
                    model.supports.invalidate_cache();
                }
            }
            selected.clear();
        }
        if quiet_button(ui, "Deselect", half).clicked() {
            selected.clear();
        }
    });
}

/// Show the button that removes all supports from the models on the plate.
pub fn clear(app: &mut App, ui: &mut Ui) {
    ui.label(hint(
        "Removes every support of the models on this plate. You cannot undo this. The models stay where they are.",
    ));

    let indexes = models_on_plate(app);
    let any =
        (indexes.iter()).any(|&index| !app.project.models[index].supports.support_set().is_empty());
    let width = ui.available_width();
    let clicked = ui
        .add_enabled_ui(any, |ui| danger_button(ui, "Remove all supports", width))
        .inner
        .clicked();
    if clicked {
        for index in indexes {
            app.project.models[index].supports.clear();
        }
        app.state.selected_supports.clear();
    }
}
