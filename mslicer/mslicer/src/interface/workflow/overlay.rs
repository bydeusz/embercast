//! The controls over the 3D view: the plate summary, the camera views, the build plates and a hint.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use common::units::Milimeter;
use egui::{Align2, Area, Button, Context, Id, Order, Rect, Ui, pos2, vec2};
use nalgebra::{Vector2, Vector3};

use crate::{
    core::{App, state::Tool},
    interface::{
        theme::{self, color},
        workflow::{
            Step, models_on_plate,
            style::{choice_button, hint},
        },
    },
};

/// The space between the edge of the 3D view and the controls, in points.
const MARGIN: f32 = 16.0;

/// One fixed direction of the camera.
#[derive(Clone, Copy)]
enum View {
    Iso,
    Top,
    Front,
    Side,
}

impl View {
    const ALL: [Self; 4] = [Self::Iso, Self::Top, Self::Front, Self::Side];

    fn name(self) -> &'static str {
        match self {
            View::Iso => "Iso",
            View::Top => "Top",
            View::Front => "Front",
            View::Side => "Side",
        }
    }

    /// Return the turn and the tilt of the camera for this view, in radians.
    fn angle(self) -> Vector2<f32> {
        match self {
            View::Iso => Vector2::new(-FRAC_PI_2 + 0.55, FRAC_PI_4 * 0.8),
            View::Top => Vector2::new(-FRAC_PI_2, FRAC_PI_2 - 0.01),
            View::Front => Vector2::new(-FRAC_PI_2, 0.02),
            View::Side => Vector2::new(0.0, 0.02),
        }
    }
}

pub fn ui(app: &mut App, ctx: &Context, rect: Rect) {
    Area::new(Id::new("embercast_plate_summary"))
        .order(Order::Middle)
        .fixed_pos(rect.min + vec2(MARGIN + 4.0, MARGIN))
        .show(ctx, |ui| {
            let text = theme::label(&plate_summary(app))
                .size(12.0)
                .color(color::TEXT_SECONDARY);
            ui.label(text);
        });

    let basis = app.config.render.basis_size;
    let right = rect.max.x - MARGIN - if basis > 0.0 { basis + 8.0 } else { 0.0 };
    Area::new(Id::new("embercast_camera_views"))
        .order(Order::Middle)
        .pivot(Align2::RIGHT_TOP)
        .fixed_pos(pos2(right, rect.min.y + MARGIN))
        .show(ctx, |ui| camera_views(app, ui, rect));

    Area::new(Id::new("embercast_build_plates"))
        .order(Order::Middle)
        .pivot(Align2::LEFT_BOTTOM)
        .fixed_pos(pos2(rect.min.x + MARGIN, rect.max.y - MARGIN))
        .show(ctx, |ui| build_plates(app, ui));

    let text = match app.state.step {
        Step::Supports if app.state.tool == Tool::Support => {
            "Click under a model to add a support · click a support to select it"
        }
        _ => "Drag to turn · right-drag to pan · scroll to zoom",
    };
    Area::new(Id::new("embercast_view_hint"))
        .order(Order::Middle)
        .interactable(false)
        .pivot(Align2::RIGHT_BOTTOM)
        .fixed_pos(pos2(rect.max.x - MARGIN, rect.max.y - MARGIN - 6.0))
        .show(ctx, |ui| {
            ui.label(hint(text));
        });
}

/// Return the plate number, the number of models and the size of the selected model.
fn plate_summary(app: &App) -> String {
    let count = models_on_plate(app).len();
    let mut text = format!(
        "Plate {} · {count} model{}",
        app.project.active_plate + 1,
        if count == 1 { "" } else { "s" }
    );

    if let Some(id) = app.state.selected.single_model()
        && let Some(model) = app.project.models.iter().find(|model| model.id == id)
    {
        let size = model.bounds.max - model.bounds.min;
        text += &format!(" · {:.1} × {:.1} × {:.1} mm", size.x, size.y, size.z);
    }
    text
}

/// Show one button per camera view, and a button that fits the whole plate in view.
fn camera_views(app: &mut App, ui: &mut Ui, rect: Rect) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for view in View::ALL {
            if choice_button(ui, view.name(), false, 0.0).clicked() {
                app.camera.angle = view.angle();
            }
        }

        if choice_button(ui, "Fit", false, 0.0)
            .on_hover_text("Show the whole build plate.")
            .clicked()
        {
            let platform = app
                .project
                .slice_config
                .platform_size
                .map(|x| x.get::<Milimeter>());
            let radius = (platform.x.powi(2) + platform.y.powi(2)).sqrt() / 2.0 * 1.05;
            app.camera.target = Vector3::zeros();
            app.camera
                .visible_sphere(rect.width() / rect.height(), radius);
        }
    });
}

/// Show one tab per build plate and a button that adds a plate.
fn build_plates(app: &mut App, ui: &mut Ui) {
    let current = app.project.active_plate;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        for plate in 0..app.project.plate_count() {
            let models = (app.project.models.iter())
                .filter(|model| model.plate == plate)
                .count();
            let response =
                choice_button(ui, &format!("Plate {}", plate + 1), plate == current, 0.0)
                    .on_hover_text(format!(
                        "{models} model{}",
                        if models == 1 { "" } else { "s" }
                    ));
            if response.clicked() {
                app.project.active_plate = plate;
            }
        }

        let add = theme::label("+ New plate").size(11.5).color(color::ACCENT);
        if ui.add(Button::new(add).frame(false)).clicked() {
            app.project.add_plate();
        }
    });

    if app.project.active_plate != current {
        app.state.selected.clear();
        app.state.selected_supports.clear();
    }
}
