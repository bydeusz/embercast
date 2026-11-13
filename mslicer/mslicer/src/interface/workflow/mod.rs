//! The Embercast layout: three steps from model to print file, with the
//! tools of the current step on the left, one panel on the right and the
//! work area in the middle.

use egui::{CentralPanel, Context, Frame};
use egui_phosphor::regular::{
    ARROWS_OUT_CARDINAL, BANDAIDS, COMPASS, CROSSHAIR, CYLINDER, HAND_POINTING, RULER, SPARKLE,
    SQUARES_FOUR, STACK, TEXT_T, TRASH,
};

use crate::{
    core::{App, state::Tool},
    interface::{
        elements::{drag_and_drop, welcome},
        panels::viewport,
        popup::PopupApp,
        shortcuts,
        theme::color,
    },
};

mod estimate;
mod footer;
mod header;
mod overlay;
mod panel;
mod prepare;
mod rail;
mod slice;
mod style;
mod supports;
#[cfg(test)]
mod tests;
mod windows;

/// One step of the work from model to print file.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    #[default]
    Prepare,
    Supports,
    Slice,
}

/// One tool on the left rail. Each tool belongs to one step.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum RailTool {
    #[default]
    Transform,
    Orient,
    Arrange,
    Hollow,
    Label,
    Measure,
    Repair,
    AutoSupports,
    EditSupports,
    ClearSupports,
    Layers,
    CheckPrint,
}

impl Step {
    pub const ALL: [Self; 3] = [Self::Prepare, Self::Supports, Self::Slice];

    pub fn name(self) -> &'static str {
        match self {
            Step::Prepare => "Prepare",
            Step::Supports => "Supports",
            Step::Slice => "Slice",
        }
    }

    /// Return the position of the step in the work, from 0 for Prepare.
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|step| *step == self).unwrap()
    }

    /// Return the tools of the step in the order of the rail. The first tool is the default tool.
    pub fn tools(self) -> &'static [RailTool] {
        match self {
            Step::Prepare => &[
                RailTool::Transform,
                RailTool::Orient,
                RailTool::Arrange,
                RailTool::Hollow,
                RailTool::Label,
                RailTool::Measure,
                RailTool::Repair,
            ],
            Step::Supports => &[
                RailTool::AutoSupports,
                RailTool::EditSupports,
                RailTool::ClearSupports,
            ],
            Step::Slice => &[RailTool::Layers, RailTool::CheckPrint],
        }
    }
}

impl RailTool {
    /// Return the short name under the icon on the rail.
    pub fn name(self) -> &'static str {
        match self {
            RailTool::Transform => "Transform",
            RailTool::Orient => "Orient",
            RailTool::Arrange => "Arrange",
            RailTool::Hollow => "Hollow",
            RailTool::Label => "Label",
            RailTool::Measure => "Measure",
            RailTool::Repair => "Repair",
            RailTool::AutoSupports => "Auto",
            RailTool::EditSupports => "Edit",
            RailTool::ClearSupports => "Clear",
            RailTool::Layers => "Layers",
            RailTool::CheckPrint => "Check",
        }
    }

    /// Return the title of the tool in the side panel.
    pub fn title(self) -> &'static str {
        match self {
            RailTool::Transform => "Transform",
            RailTool::Orient => "Orientation",
            RailTool::Arrange => "Arrange",
            RailTool::Hollow => "Hollow",
            RailTool::Label => "Text label",
            RailTool::Measure => "Measure",
            RailTool::Repair => "Repair",
            RailTool::AutoSupports => "Auto supports",
            RailTool::EditSupports => "Edit supports",
            RailTool::ClearSupports => "Clear supports",
            RailTool::Layers => "Layers",
            RailTool::CheckPrint => "Check print",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            RailTool::Transform => ARROWS_OUT_CARDINAL,
            RailTool::Orient => COMPASS,
            RailTool::Arrange => SQUARES_FOUR,
            RailTool::Hollow => CYLINDER,
            RailTool::Label => TEXT_T,
            RailTool::Measure => RULER,
            RailTool::Repair => BANDAIDS,
            RailTool::AutoSupports => SPARKLE,
            RailTool::EditSupports => HAND_POINTING,
            RailTool::ClearSupports => TRASH,
            RailTool::Layers => STACK,
            RailTool::CheckPrint => CROSSHAIR,
        }
    }

    /// Return the tooltip of the tool on the rail.
    pub fn description(self) -> &'static str {
        match self {
            RailTool::Transform => "Move, turn and scale the selected model.",
            RailTool::Orient => {
                "Turn a model so that it needs fewer supports, or lay it flat on a face."
            }
            RailTool::Arrange => "Put all models of the plate in a neat layout.",
            RailTool::Hollow => "Make thick parts hollow to save resin, with drain holes.",
            RailTool::Label => "Make a 3D text model.",
            RailTool::Measure => "Measure the distance between two points on a model.",
            RailTool::Repair => "Find and repair problems in the mesh of a model.",
            RailTool::AutoSupports => "Add supports to the models automatically.",
            RailTool::EditSupports => "Add or remove single supports by hand.",
            RailTool::ClearSupports => "Remove all supports from the plate.",
            RailTool::Layers => "Look through the layers of the print.",
            RailTool::CheckPrint => "Look for islands, resin traps and suction cups.",
        }
    }

    /// Return true for a tool that removes work, which the rail shows in crimson.
    pub fn is_danger(self) -> bool {
        self == RailTool::ClearSupports
    }

    /// Return true for a tool that the rail puts under a divider line.
    pub fn is_separated(self) -> bool {
        matches!(self, RailTool::Repair | RailTool::ClearSupports)
    }

    /// Return the action of a click in the 3D view while this tool is active.
    pub fn viewport_tool(self) -> Tool {
        match self {
            RailTool::Orient => Tool::Orient,
            RailTool::Measure => Tool::Measure,
            RailTool::EditSupports => Tool::Support,
            _ => Tool::Select,
        }
    }
}

/// Show the Embercast layout, and handle the shortcuts, dropped files and the welcome window.
pub fn ui(app: &mut App, ctx: &Context) {
    shortcuts::handle(app, ctx);
    drag_and_drop::update(app, ctx);
    welcome::ui(app, ctx);

    app.state.tool = app.state.rail_tool.viewport_tool();
    supports::generate_pending(app);

    header::ui(app, ctx);
    footer::ui(app, ctx);
    rail::ui(app, ctx);
    panel::ui(app, ctx);
    windows::ui(app, ctx);

    CentralPanel::default()
        .frame(Frame::NONE.fill(color::BG_BASE))
        .show(ctx, |ui| match app.state.step {
            Step::Prepare | Step::Supports => {
                let rect = ui.max_rect();
                viewport::ui(app, ui, ctx);
                overlay::ui(app, ctx, rect);
            }
            Step::Slice => slice::central(app, ui),
        });
}

/// Go to `step` and select the first tool of that step.
pub fn go_to(app: &mut App, step: Step) {
    if app.state.step != step {
        app.state.step = step;
        app.state.rail_tool = step.tools()[0];
    }
}

/// Return the indexes of the models on the active build plate.
pub fn models_on_plate(app: &App) -> Vec<usize> {
    let plate = app.project.active_plate;
    (app.project.models.iter().enumerate())
        .filter(|(_, model)| model.plate == plate)
        .map(|(index, _)| index)
        .collect()
}

/// Return the indexes of the selected models on the active plate, or of all its models when no model is selected.
pub fn target_models(app: &App) -> Vec<usize> {
    let on_plate = models_on_plate(app);
    let selected: Vec<usize> = (on_plate.iter().copied())
        .filter(|&index| {
            app.state
                .selected
                .contains_model(app.project.models[index].id)
        })
        .collect();
    if selected.is_empty() {
        on_plate
    } else {
        selected
    }
}

/// Return the parts of `app` that the windows of the tools use.
pub fn popup_app(app: &mut App) -> PopupApp<'_> {
    PopupApp {
        config_dir: &mut app.config_dir,
        panels: &mut app.panels,
        tasks: &mut app.tasks,
        remote_print: &mut app.remote_print,
        slice_operation: &mut app.slice_operation,
        camera: &mut app.camera,
        state: &mut app.state,
        config: &mut app.config,
        project: &mut app.project,
        history: &mut app.history,
    }
}
