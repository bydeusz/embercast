//! The panel on the right: the active tool, the models on the plate, the print estimate and the next action.

use const_format::concatcp;
use egui::{
    Align, Align2, Button, Color32, ComboBox, Context, CursorIcon, FontId, Frame, Label, Layout,
    Margin, RichText, ScrollArea, Sense, Shape, SidePanel, Stroke, StrokeKind, TopBottomPanel, Ui,
    UiBuilder, vec2,
};
use egui_phosphor::regular::{ARROW_RIGHT, EYE, EYE_SLASH, PLUS, WARNING};

use crate::{
    core::{App, history::ModelAction, project::model::MeshWarnings},
    interface::{
        panels::slice_config::{apply_resin_profile, printer_name},
        shortcuts,
        theme::{self, RADIUS_SM, color},
        workflow::{
            RailTool, Step,
            estimate::estimate,
            go_to, models_on_plate, prepare, slice,
            style::{eyebrow, field_label, hint, primary_button, section, stat_row, title},
            supports,
        },
    },
};
use common::slice::profiles::resin_profiles;

pub fn ui(app: &mut App, ctx: &Context) {
    SidePanel::right("embercast_panel")
        .exact_width(340.0)
        .resizable(false)
        .frame(Frame::new().fill(color::SURFACE_CARD))
        .show(ctx, |ui| {
            TopBottomPanel::bottom("embercast_panel_action")
                .frame(Frame::new().inner_margin(Margin::symmetric(20, 16)))
                .show_inside(ui, |ui| next_action(app, ui));

            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                match app.state.step {
                    Step::Prepare | Step::Supports => {
                        section(ui, |ui| tool(app, ui, ctx));
                        section(ui, |ui| models(app, ui, ctx));
                        section(ui, |ui| print(app, ui));
                    }
                    Step::Slice => {
                        section(ui, |ui| slice::result(app, ui));
                        section(ui, |ui| slice::profile(app, ui));
                    }
                }
            });
        });
}

/// Show the title and the settings of the active tool.
fn tool(app: &mut App, ui: &mut Ui, ctx: &Context) {
    let tool = app.state.rail_tool;
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        ui.label(eyebrow("Tool"));
        ui.label(title(tool.title()));
    });
    ui.add_space(4.0);

    match tool {
        RailTool::Transform => prepare::transform(app, ui),
        RailTool::Orient => prepare::orient(app, ui),
        RailTool::Arrange => prepare::arrange(app, ui, ctx),
        RailTool::Hollow => prepare::hollow(app, ui),
        RailTool::Label => prepare::label(app, ui),
        RailTool::Measure => prepare::measure(app, ui),
        RailTool::Repair => prepare::repair(app, ui),
        RailTool::AutoSupports => supports::auto(app, ui, ctx),
        RailTool::EditSupports => supports::edit(app, ui),
        RailTool::ClearSupports => supports::clear(app, ui),
        RailTool::Layers | RailTool::CheckPrint => {}
    }
}

/// Show the models on the active plate. A click on a row selects that model.
fn models(app: &mut App, ui: &mut Ui, ctx: &Context) {
    ui.horizontal(|ui| {
        ui.label(eyebrow(&format!(
            "On plate {}",
            app.project.active_plate + 1
        )));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let add = theme::label(concatcp!(PLUS, " Add model"))
                .size(11.0)
                .color(color::ACCENT);
            if ui.add(Button::new(add).frame(false)).clicked() {
                (shortcuts::IMPORT_MODEL_SHORTCUT.callback)(app, ctx);
            }
        });
    });

    let indexes = models_on_plate(app);
    if indexes.is_empty() {
        ui.label(hint(
            "No models on this plate yet. Add a model, or drop an STL file on the window.",
        ));
        return;
    }

    ui.spacing_mut().item_spacing.y = 4.0;
    for index in indexes {
        model_row(app, ui, index);
    }
}

/// Show one model as a row: a diamond, the name, a warning when the mesh has a problem, and a button that hides the model.
fn model_row(app: &mut App, ui: &mut Ui, index: usize) {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::click());
    let model = &mut app.project.models[index];
    let id = model.id;
    let selected = app.state.selected.contains_model(id);

    let (fill, stroke) = if selected {
        (color::accent_soft(), color::border_strong())
    } else if response.hovered() {
        (color::SURFACE_CARD_RAISED, color::border_hairline())
    } else {
        (Color32::TRANSPARENT, Color32::TRANSPARENT)
    };
    let painter = ui.painter();
    painter.rect(
        rect,
        RADIUS_SM,
        fill,
        Stroke::new(1.0_f32, stroke),
        StrokeKind::Inside,
    );

    let center = rect.left_center() + vec2(14.0, 0.0);
    let diamond = vec![
        center + vec2(0.0, -5.0),
        center + vec2(5.0, 0.0),
        center + vec2(0.0, 5.0),
        center + vec2(-5.0, 0.0),
    ];
    let diamond_color = if selected {
        color::ACCENT_HOVER
    } else {
        color::BONE_400
    };
    painter.add(Shape::convex_polygon(diamond, diamond_color, Stroke::NONE));

    let name_color = if model.hidden {
        color::BONE_400
    } else {
        color::TEXT_PRIMARY
    };
    painter.text(
        rect.left_center() + vec2(28.0, 0.0),
        Align2::LEFT_CENTER,
        &model.name,
        FontId::proportional(13.5),
        name_color,
    );

    let content = UiBuilder::new()
        .max_rect(rect.shrink2(vec2(6.0, 0.0)))
        .layout(Layout::right_to_left(Align::Center));
    ui.scope_builder(content, |ui| {
        let icon = if model.hidden { EYE_SLASH } else { EYE };
        let tip = if model.hidden { "Show" } else { "Hide" };
        let eye =
            Button::new(RichText::new(icon).size(16.0).color(color::TEXT_SECONDARY)).frame(false);
        if ui.add(eye).on_hover_text(tip).clicked() {
            app.history
                .track_model(id, ModelAction::Hidden(model.hidden));
            model.hidden ^= true;
        }

        if !model.warnings.is_empty() {
            let mut warning = ui.label(RichText::new(WARNING).size(16.0).color(color::DANGER_TEXT));
            if model.warnings.contains(MeshWarnings::Defective) {
                warning =
                    warning.on_hover_text("The mesh has holes or errors. Use the Repair tool.");
            }
            if model.warnings.contains(MeshWarnings::OutOfBounds) {
                warning.on_hover_text("The model is partly outside the build volume.");
            }
        }
    });

    if response.on_hover_cursor(CursorIcon::PointingHand).clicked() {
        let shift = ui.input(|input| input.modifiers.shift);
        app.state.selected.model_clicked(id, shift);
    }
}

/// Show the printer, the resin profile and the estimate of layers, print time and resin.
fn print(app: &mut App, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label(eyebrow("Print"));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let settings = theme::label("Settings").size(11.0).color(color::ACCENT);
            if ui.add(Button::new(settings).frame(false)).clicked() {
                app.state.show_print_settings = true;
            }
        });
    });

    let width = (ui.available_width() - 8.0) / 2.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.allocate_ui(vec2(width, 52.0), |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                ui.label(field_label("Printer"));
                let name = printer_name(&app.config, &app.state.selected_printer);
                let label = Label::new(RichText::new(name).color(color::TEXT_PRIMARY)).truncate();
                ui.add_sized(vec2(width, 28.0), label);
            });
        });
        ui.allocate_ui(vec2(width, 52.0), |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                ui.label(field_label("Resin profile"));
                resin_profile(app, ui, width);
            });
        });
    });

    let estimate = estimate(app);
    stat_row(
        ui,
        &[
            ("Layers", estimate.layers.to_string()),
            ("Print time", estimate.print_time),
            ("Resin", format!("{:.1} ml", estimate.resin_ml)),
        ],
    );
    ui.label(hint(
        "An estimate before slicing. The resin is without the supports.",
    ));
}

/// Show a list of the resin profiles. A pick copies the settings of the profile into the project.
fn resin_profile(app: &mut App, ui: &mut Ui, width: f32) {
    let profiles = resin_profiles();
    let config = &mut app.project.slice_config;
    let current = profiles.iter().find(|profile| profile.matches(config));
    ComboBox::from_id_salt("embercast_resin_profile")
        .width(width)
        .selected_text(current.map_or("Custom", |profile| profile.name))
        .show_ui(ui, |ui| {
            for profile in &profiles {
                let selected = current == Some(profile);
                let response = ui.selectable_label(selected, profile.name);
                if response.on_hover_text(profile.printer).clicked() {
                    apply_resin_profile(profile, config, &mut app.history);
                }
            }
        });
}

/// Show the gold button at the bottom of the panel, which leads to the next step.
fn next_action(app: &mut App, ui: &mut Ui) {
    match app.state.step {
        Step::Prepare => {
            if primary_button(ui, concatcp!("Next · Supports  ", ARROW_RIGHT), true).clicked() {
                go_to(app, Step::Supports);
            }
        }
        Step::Supports => {
            let text = format!(
                "Slice plate {}  {ARROW_RIGHT}",
                app.project.active_plate + 1
            );
            let enabled = !models_on_plate(app).is_empty() && !app.is_slicing();
            if primary_button(ui, &text, enabled).clicked() {
                app.slice();
                go_to(app, Step::Slice);
            }
        }
        Step::Slice => slice::export(app, ui),
    }
}
