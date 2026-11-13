//! The tool panels of the Prepare step.

use common::units::Milimeter;
use egui::{Button, Context, DragValue, Frame, Margin, RichText, Stroke, Ui, vec2};
use egui_phosphor::regular::{LOCK, LOCK_OPEN};
use nalgebra::Vector3;

use crate::{
    core::{
        App,
        history::{Action, ModelAction, SliceConfigAction},
        project::model::MeshWarnings,
    },
    interface::{
        components::{BeingEditedExt, history_tracked_model, history_tracked_value},
        panels::{slice_config, viewport::measurement_text},
        popup::Popup,
        shortcuts,
        theme::{RADIUS_SM, color},
        tools,
        workflow::{
            RailTool, popup_app,
            style::{
                axis_row, check_row, danger_button, field_label, hint, outline_button, quiet_button,
            },
            target_models,
        },
    },
    task::{AutoOrient, FlipWinding, MeshRepair, SplitBodies},
};

/// What the transform panel does with the selected model after it has shown its fields.
enum ModelChange {
    None,
    Duplicate,
    Delete,
}

/// Return the index of the one selected model, or `None` when no model or more than one model is selected.
fn selected_index(app: &App) -> Option<usize> {
    let id = app.state.selected.single_model()?;
    app.project.models.iter().position(|model| model.id == id)
}

/// Show the hint for a tool that needs one selected model.
fn select_a_model(ui: &mut Ui) {
    ui.label(hint(
        "Click a model on the plate, or pick it in the list below.",
    ));
}

/// Show the position, rotation and scale of the selected model, with buttons that duplicate or delete it.
pub fn transform(app: &mut App, ui: &mut Ui) {
    let Some(index) = selected_index(app) else {
        select_a_model(ui);
        return;
    };

    let platform = app.project.slice_config.platform_size;
    let model = &mut app.project.models[index];
    let id = model.id;
    ui.label(RichText::new(&model.name).color(color::TEXT_PRIMARY));

    ui.label(field_label("Position · mm"));
    let mut position = model.mesh.position();
    let editing = axis_row(ui, position.as_mut_slice(), "", 0.1);
    history_tracked_model(
        (editing, ui, &mut app.history),
        (id, || ModelAction::Position(model.mesh.position())),
    );
    (editing && model.mesh.position() != position).then(|| model.set_position(&platform, position));

    let half = (ui.available_width() - 8.0) / 2.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        if outline_button(ui, "Center", half).clicked() {
            app.history
                .track_model(id, ModelAction::Position(model.mesh.position()));
            let center = model.bounds.center();
            model.set_bounds_center(&platform, Vector3::new(0.0, 0.0, center.z));
        }
        if outline_button(ui, "Drop to plate", half).clicked() {
            app.history
                .track_model(id, ModelAction::Position(model.mesh.position()));
            model.align_to_bed();
            model.update_oob(&platform);
        }
    });

    ui.add_space(4.0);
    ui.label(field_label("Rotation · degrees"));
    let mut rotation = model.mesh.rotation().map(|angle| angle.to_degrees());
    let editing = axis_row(ui, rotation.as_mut_slice(), "°", 0.5);
    history_tracked_model(
        (editing, ui, &mut app.history),
        (id, || ModelAction::Rotation(model.mesh.rotation())),
    );
    if editing {
        let radians = rotation.map(|angle| angle.to_radians());
        (model.mesh.rotation() != radians).then(|| model.set_rotation(&platform, radians));
    }

    ui.add_space(4.0);
    ui.label(field_label("Scale"));
    let factor = model.unit.conversion();
    let mut scale = model.mesh.scale() / factor;
    let mut editing = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        if model.ui.locked_scale {
            let mut percent = scale.x * 100.0;
            ui.spacing_mut().interact_size.x = ui.available_width() - 44.0;
            let field = DragValue::new(&mut percent)
                .suffix(" %")
                .speed(0.5)
                .max_decimals(1)
                .range(0.1..=100_000.0);
            ui.add(field).being_edited(&mut editing);
            if scale.x != 0.0 {
                scale *= percent / 100.0 / scale.x;
            }
        } else {
            ui.allocate_ui(vec2(ui.available_width() - 44.0, 32.0), |ui| {
                editing = axis_row(ui, scale.as_mut_slice(), "×", 0.01);
            });
        }

        let (icon, tip) = if model.ui.locked_scale {
            (
                LOCK,
                "Keep the proportions. Click to scale each axis apart.",
            )
        } else {
            (
                LOCK_OPEN,
                "Scale each axis apart. Click to keep the proportions.",
            )
        };
        let lock = Button::new(RichText::new(icon).size(18.0).color(color::GOLD_300))
            .fill(color::accent_soft())
            .stroke(Stroke::new(1.0_f32, color::border_strong()))
            .corner_radius(RADIUS_SM)
            .min_size(vec2(36.0, 32.0));
        model.ui.locked_scale ^= ui.add(lock).on_hover_text(tip).clicked();
    });
    history_tracked_model(
        (editing, ui, &mut app.history),
        (id, || ModelAction::Scale(model.mesh.scale())),
    );
    scale *= factor;
    (editing && model.mesh.scale() != scale).then(|| model.set_scale(&platform, scale));

    let size = model.bounds.max - model.bounds.min;
    ui.label(hint(format!(
        "Size {:.1} × {:.1} × {:.1} mm",
        size.x, size.y, size.z
    )));

    if quiet_button(ui, "Lay flat · click a face", ui.available_width()).clicked() {
        app.state.rail_tool = RailTool::Orient;
    }

    ui.add_space(4.0);
    let mut change = ModelChange::None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        if quiet_button(ui, "Duplicate", half).clicked() {
            change = ModelChange::Duplicate;
        }
        if danger_button(ui, "Delete", half).clicked() {
            change = ModelChange::Delete;
        }
    });

    match change {
        ModelChange::None => {}
        ModelChange::Duplicate => {
            let copy = app.project.models[index].clone();
            app.history.track(Action::ModelAdded { id: copy.id });
            app.project.models.push(copy);
        }
        ModelChange::Delete => {
            let model = Box::new(app.project.models.remove(index));
            app.history.track(Action::ModelRemoved { index, model });
            app.state.selected.clear();
        }
    }
}

/// Show automatic orientation for the target models, and the lay flat click.
pub fn orient(app: &mut App, ui: &mut Ui) {
    ui.label(hint(
        "Auto orient turns the model so that the least area needs supports, and puts it on the plate. With equal supports, the lower orientation wins.",
    ));
    let targets = target_models(app);
    let text = match targets.len() {
        1 => "Auto orient".to_owned(),
        count => format!("Auto orient {count} models"),
    };
    let width = ui.available_width();
    if ui
        .add_enabled_ui(!targets.is_empty(), |ui| outline_button(ui, &text, width))
        .inner
        .clicked()
    {
        for index in targets {
            app.tasks.add(AutoOrient::new(&app.project.models[index]));
        }
    }

    ui.add_space(6.0);
    ui.label(field_label("Lay flat"));
    ui.label(hint(
        "Click a face of a model in the 3D view. The model turns so that this face rests on the plate.",
    ));
}

/// Show the quick layout with its spacing, and a button for the window with more layout options.
pub fn arrange(app: &mut App, ui: &mut Ui, ctx: &Context) {
    ui.label(hint(
        "Puts all models of this plate in a neat layout, with space between them.",
    ));

    ui.horizontal(|ui| {
        ui.label(field_label("Space between models"));
        app.config
            .ui
            .quick_layout_spacing
            .with::<Milimeter, _>(|spacing| {
                ui.add(
                    DragValue::new(spacing)
                        .suffix(" mm")
                        .speed(0.1)
                        .range(0.0..=50.0),
                );
            });
    });

    let width = ui.available_width();
    if outline_button(ui, "Arrange plate", width).clicked() {
        (shortcuts::LAYOUT_SHORTCUT.callback)(app, ctx);
    }
    if quiet_button(ui, "More layout options", width).clicked() {
        let popup = Popup::new_seeded(tools::auto_layout::NAME, tools::auto_layout::interface);
        app.popup.open(popup.close_button(true));
    }
}

/// Show the hollow settings of the project. The slicer makes the models hollow when it slices.
pub fn hollow(app: &mut App, ui: &mut Ui) {
    let post_processing = &mut app.project.post_processing;
    let old = post_processing.hollow.clone();

    ui.checkbox(
        &mut post_processing.hollow.enabled,
        "Hollow the models when slicing",
    );
    if post_processing.hollow.enabled != old.enabled {
        app.history.track(SliceConfigAction::Hollow(old).into());
    }

    if !post_processing.hollow.enabled {
        ui.label(hint(
            "A hollow model uses less resin and pulls less on the film. Drain holes let the resin out.",
        ));
        return;
    }

    ui.add_space(4.0);
    let editing = slice_config::hollow(&mut post_processing.hollow, ui);
    history_tracked_value(
        (editing, ui, &mut app.history),
        ("embercast_hollow", || {
            SliceConfigAction::Hollow(post_processing.hollow.clone()).into()
        }),
    );
}

/// Show the text label tool. The tool adds the text as a new model.
pub fn label(app: &mut App, ui: &mut Ui) {
    tools::text_label::interface(&mut popup_app(app), ui);
}

/// Show the measured distance and a button that starts a new measurement.
pub fn measure(app: &mut App, ui: &mut Ui) {
    ui.label(hint("Click two points on a model in the 3D view."));
    Frame::new()
        .fill(color::SURFACE_CARD_RAISED)
        .stroke(Stroke::new(1.0_f32, color::border_hairline()))
        .corner_radius(RADIUS_SM)
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new(measurement_text(app)).color(color::TEXT_PRIMARY));
        });

    let width = ui.available_width();
    if !app.state.measure.points().is_empty() && quiet_button(ui, "Clear", width).clicked() {
        app.state.measure.clear();
    }
}

/// Show the problems of the mesh of the selected model, and the repairs.
pub fn repair(app: &mut App, ui: &mut Ui) {
    let Some(index) = selected_index(app) else {
        select_a_model(ui);
        return;
    };

    let model = &mut app.project.models[index];
    ui.label(RichText::new(&model.name).color(color::TEXT_PRIMARY));

    if model.warnings.contains(MeshWarnings::Defective) {
        check_row(ui, false, "Mesh", "holes or errors");
    } else {
        check_row(ui, true, "Mesh", "no problems found");
    }
    if model.warnings.contains(MeshWarnings::OutOfBounds) {
        check_row(ui, false, "Position", "outside the build volume");
    } else {
        check_row(ui, true, "Position", "inside the build volume");
    }

    ui.add_space(4.0);
    let width = ui.available_width();
    if outline_button(ui, "Repair mesh", width)
        .on_hover_text("Closes holes and removes broken faces.")
        .clicked()
    {
        app.tasks.add(MeshRepair::new(model));
    }
    if quiet_button(ui, "Flip faces inside out", width)
        .on_hover_text("Use this when the model looks inside out.")
        .clicked()
    {
        app.tasks.add(FlipWinding::new(model));
    }
    if quiet_button(ui, "Split into parts", width)
        .on_hover_text("Makes one model for each loose part of the mesh.")
        .clicked()
    {
        app.tasks.add(SplitBodies::new(model));
    }
}
