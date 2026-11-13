//! The Slice step: the layer preview in the middle, the result, the checks and the export on the right.

use common::{
    slice::{
        format::{Format, RasterFormat},
        profiles::resin_profiles,
    },
    units::{Centimeter, Milimeter, Minute},
};
use const_format::concatcp;
use egui::{Align, Button, Frame, Grid, Layout, Margin, ProgressBar, RichText, Spinner, Ui, vec2};
use egui_phosphor::regular::DOWNLOAD_SIMPLE;
use slicer::post_process::cavities::{Cavity, CavityKind};

use crate::{
    core::{
        App,
        slice::{
            annotations::Annotation,
            result::{GenericSliceResult, RasterSliceResult},
        },
    },
    interface::{
        panels::sliced,
        theme::{self, color},
        workflow::{
            estimate::short_duration,
            models_on_plate,
            style::{
                check_row, eyebrow, field_label, hint, outline_button, primary_button,
                quiet_button, stat_row, title,
            },
        },
    },
    task::FileDialog,
};

/// Show the layer preview, or a button that slices the plate when no slice exists.
pub fn central(app: &mut App, ui: &mut Ui) {
    if app.slice_operation.is_some() {
        Frame::new()
            .inner_margin(Margin::same(16))
            .show(ui, |ui| sliced::preview(app, ui));
        return;
    }

    let plate = app.project.active_plate + 1;
    let has_models = !models_on_plate(app).is_empty();
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() * 0.35);
        ui.label(title("Not sliced yet"));
        ui.add_space(4.0);
        ui.label(hint(
            "Slice the plate to see every layer and to make the print file.",
        ));
        ui.add_space(16.0);
        ui.allocate_ui(vec2(320.0, 48.0), |ui| {
            if primary_button(ui, &format!("Slice plate {plate}"), has_models).clicked() {
                app.slice();
            }
        });
    });
}

/// Show the print time, the resin, the layers and the checks of the last slice.
pub fn result(app: &mut App, ui: &mut Ui) {
    let plate = app.project.active_plate + 1;
    let Some(operation) = &app.slice_operation else {
        heading(ui, &format!("Plate {plate}"), "Not sliced yet");
        ui.label(hint("Use the gold button below to slice the plate."));
        return;
    };

    let mut result = operation.result();
    let Some(result) = result.as_mut() else {
        heading(ui, &format!("Plate {plate}"), "Slicing");
        ui.add(ProgressBar::new(operation.progress.progress()).show_percentage());
        return;
    };

    let GenericSliceResult::Raster(raster) = &mut result.inner else {
        heading(ui, &format!("Plate {plate} · sliced"), "Sliced");
        ui.label(hint(
            "This slice has no layer images, so there is nothing to check.",
        ));
        return;
    };

    heading(ui, &format!("Plate {plate} · sliced"), "Ready to print");
    let layers = raster.layers.len();
    let height = layers as f32 * result.config.slice_height.get::<Milimeter>();
    stat_row(
        ui,
        &[
            ("Print time", short_duration(raster.print_time.raw())),
            (
                "Resin",
                format!("{:.1} ml", raster.volume.get::<Centimeter>()),
            ),
        ],
    );
    stat_row(
        ui,
        &[
            ("Layers", layers.to_string()),
            ("Height", format!("{height:.1} mm")),
        ],
    );

    ui.add_space(4.0);
    ui.label(field_label("Checks"));
    if raster.detected_islands {
        checks(raster, &mut app.state.preview_layer, ui);
    } else {
        ui.label(hint(
            "Look for islands, resin traps and suction cups before you print.",
        ));
        let width = ui.available_width();
        if outline_button(ui, "Check print", width).clicked() {
            sliced::start_print_check(&mut app.tasks, &result.config, raster);
        }
    }
}

/// Show a gold eyebrow over a title.
fn heading(ui: &mut Ui, eyebrow_text: &str, title_text: &str) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        ui.label(eyebrow(eyebrow_text));
        ui.label(title(title_text));
    });
    ui.add_space(4.0);
}

/// Show one line per check. A found cavity gets a link that moves the preview to its first layer.
fn checks(raster: &RasterSliceResult, preview_layer: &mut usize, ui: &mut Ui) {
    let island_layers = {
        let annotations = raster.annotations.lock();
        (0..raster.layers.len())
            .filter(|&layer| annotations.worst(layer) == Some(Annotation::Island))
            .count()
    };
    let Some(cavities) = raster.annotations.cavities() else {
        ui.horizontal(|ui| {
            ui.add(Spinner::new().size(14.0));
            ui.label(hint("Checking the layers…"));
        });
        return;
    };

    let islands = match island_layers {
        0 => "none found".to_owned(),
        1 => "on 1 layer".to_owned(),
        count => format!("on {count} layers"),
    };
    check_row(ui, island_layers == 0, "Islands", &islands);

    for (kind, name) in [
        (CavityKind::ResinTrap, "Resin traps"),
        (CavityKind::SuctionCup, "Suction cups"),
    ] {
        let found: Vec<&Cavity> = cavities
            .iter()
            .filter(|cavity| cavity.kind == kind)
            .collect();
        let text = match found.len() {
            0 => "none found".to_owned(),
            count => format!("{count} found"),
        };
        check_row(ui, found.is_empty(), name, &text);
        for cavity in found {
            let text = format!(
                "{:.1} mm³ · layers {} to {}",
                cavity.volume_mm3,
                cavity.first_layer + 1,
                cavity.last_layer + 1
            );
            if ui.link(RichText::new(text).size(13.0)).clicked() {
                *preview_layer = cavity.first_layer + 1;
            }
        }
    }
}

/// Show the resin profile of the project with its main exposure settings.
pub fn profile(app: &mut App, ui: &mut Ui) {
    let config = &app.project.slice_config;
    let name = (resin_profiles().iter())
        .find(|profile| profile.matches(config))
        .map_or("Custom profile".to_owned(), |profile| {
            profile.name.to_owned()
        });

    ui.horizontal(|ui| {
        ui.label(eyebrow(&name));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let edit = theme::label("Edit profile").size(11.0).color(color::ACCENT);
            if ui.add(Button::new(edit).frame(false)).clicked() {
                app.state.show_print_settings = true;
            }
        });
    });

    let config = &app.project.slice_config;
    let normal = &config.exposure_config;
    let bottom = &config.first_exposure_config;
    let rows = [
        (
            "Layer height",
            format!("{:.2} mm", config.slice_height.get::<Milimeter>()),
        ),
        (
            "Exposure",
            format!("{:.1} s per layer", normal.exposure_time.raw()),
        ),
        (
            "Bottom layers",
            format!(
                "{} × {:.0} s",
                config.first_layers.layers(config.slice_height),
                bottom.exposure_time.raw()
            ),
        ),
        ("Light off", format!("{:.0} s", normal.exposure_delay.raw())),
        (
            "Lift",
            format!(
                "{:.0} mm at {:.0} mm/min",
                normal.lift_distance.get::<Milimeter>(),
                normal.lift_speed.convert::<Milimeter, Minute>().raw()
            ),
        ),
    ];

    Grid::new("embercast_profile")
        .num_columns(2)
        .spacing([16.0, 4.0])
        .show(ui, |ui| {
            for (label, value) in rows {
                ui.label(RichText::new(label).size(13.0).color(color::TEXT_SECONDARY));
                ui.label(RichText::new(value).size(13.0).color(color::TEXT_PRIMARY));
                ui.end_row();
            }
        });
}

/// Show the gold export button, the other formats and a button that slices again.
pub fn export(app: &mut App, ui: &mut Ui) {
    let plate = app.project.active_plate + 1;
    let Some(operation) = &app.slice_operation else {
        let has_models = !models_on_plate(app).is_empty();
        if primary_button(ui, &format!("Slice plate {plate}"), has_models).clicked() {
            app.slice();
        }
        return;
    };

    let result = operation.result();
    let ready = (result.as_ref())
        .is_some_and(|result| result.inner.as_raster().is_some() && !result.variable_layer_height);

    let mut task: Option<FileDialog> = None;
    if primary_button(ui, concatcp!(DOWNLOAD_SIMPLE, " Export .cxdlp"), ready).clicked()
        && let Some(result) = result.as_ref()
    {
        task = Some(sliced::save_file(
            result.config.clone(),
            operation.preview(),
            Format::Raster(RasterFormat::Cxdlp),
            result.slice_data(),
        ));
    }
    ui.add_space(4.0);
    ui.vertical_centered(|ui| {
        ui.label(hint("Open the file in UVtools before the first print."));
    });
    ui.add_space(4.0);

    let half = (ui.available_width() - 8.0) / 2.0;
    let mut slice_again = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        ui.add_enabled_ui(ready, |ui| {
            ui.allocate_ui(vec2(half, 36.0), |ui| {
                ui.menu_button(field_label("Other formats"), |ui| {
                    ui.set_width(180.0);
                    for format in Format::RASTER {
                        let mut response = ui.button(format.name());
                        if let Some(tip) = format.tip() {
                            response = response.on_hover_text(tip);
                        }
                        if response.clicked()
                            && let Some(result) = result.as_ref()
                        {
                            task = Some(sliced::save_file(
                                result.config.clone(),
                                operation.preview(),
                                format,
                                result.slice_data(),
                            ));
                        }
                    }
                });
            });
        });
        slice_again = quiet_button(ui, "Slice again", half).clicked();
    });
    drop(result);

    if let Some(task) = task {
        app.tasks.add(task);
    }
    if slice_again {
        app.slice();
    }
}

/// Start the print check of the last slice, once.
pub fn start_check(app: &mut App) {
    let Some(operation) = &app.slice_operation else {
        return;
    };
    let mut result = operation.result();
    if let Some(result) = result.as_mut()
        && let GenericSliceResult::Raster(raster) = &mut result.inner
        && !raster.detected_islands
    {
        sliced::start_print_check(&mut app.tasks, &result.config, raster);
    }
}
