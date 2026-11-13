//! Small widgets in the look of the design system, for the Embercast layout.

use egui::{
    Align, Align2, Button, Color32, CursorIcon, DragValue, FontFamily, FontId, FontSelection,
    Frame, Layout, Margin, Response, RichText, Sense, Shape, Stroke, StrokeKind, TextWrapMode, Ui,
    WidgetText, pos2, vec2,
};
use egui_phosphor::regular::{CHECK, WARNING};

use crate::interface::{
    components::BeingEditedExt,
    theme::{self, DISPLAY, RADIUS_SM, color},
};

/// The space between the edge of the side panel and its content, in points.
pub const PANEL_MARGIN: i8 = 20;

/// The state of one step in the stepper at the top.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Done,
    Current,
    Later,
}

/// Return `text` as a small gold heading over a section, in tracked capitals.
pub fn eyebrow(text: &str) -> RichText {
    theme::label(text).size(11.0).color(color::ACCENT)
}

/// Return `text` as the label of a field, in tracked capitals.
pub fn field_label(text: &str) -> RichText {
    theme::label(text).size(11.0).color(color::TEXT_SECONDARY)
}

/// Return `text` as the title of a panel, in Cinzel.
pub fn title(text: &str) -> RichText {
    RichText::new(text)
        .family(FontFamily::Name(DISPLAY.into()))
        .size(19.0)
        .color(color::TEXT_PRIMARY)
}

/// Return `text` as a hint in dim text.
pub fn hint(text: impl Into<String>) -> RichText {
    RichText::new(text.into()).size(13.0).color(color::BONE_400)
}

/// Show `add_contents` with the margin of the side panel and a divider line under it.
pub fn section<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    let inner = Frame::new()
        .inner_margin(Margin::symmetric(PANEL_MARGIN, 16))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 8.0;
            add_contents(ui)
        });
    let rect = inner.response.rect;
    ui.painter().hline(
        rect.x_range(),
        rect.bottom(),
        Stroke::new(1.0_f32, color::divider()),
    );
    inner.inner
}

/// Show a full-width gold button for the main action of a step.
pub fn primary_button(ui: &mut Ui, text: &str, enabled: bool) -> Response {
    let button = Button::new(theme::label(text).size(14.0).color(color::TEXT_ON_GOLD))
        .fill(color::ACCENT)
        .stroke(Stroke::new(1.0_f32, color::ACCENT_HOVER))
        .corner_radius(RADIUS_SM)
        .min_size(vec2(ui.available_width(), 48.0));
    ui.add_enabled(enabled, button)
        .on_hover_cursor(CursorIcon::PointingHand)
}

/// Show a button with a gold border and gold text.
pub fn outline_button(ui: &mut Ui, text: &str, width: f32) -> Response {
    let button = Button::new(theme::label(text).size(12.0).color(color::ACCENT))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0_f32, color::border_strong()))
        .corner_radius(RADIUS_SM)
        .min_size(vec2(width, 36.0));
    ui.add(button)
}

/// Show a quiet button with a steel border.
pub fn quiet_button(ui: &mut Ui, text: &str, width: f32) -> Response {
    let button = Button::new(theme::label(text).size(12.0).color(color::TEXT_SECONDARY))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0_f32, color::border_steel()))
        .corner_radius(RADIUS_SM)
        .min_size(vec2(width, 36.0));
    ui.add(button)
}

/// Show a button in crimson for an action that removes work.
pub fn danger_button(ui: &mut Ui, text: &str, width: f32) -> Response {
    let button = Button::new(theme::label(text).size(12.0).color(color::DANGER_TEXT))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0_f32, color::DANGER))
        .corner_radius(RADIUS_SM)
        .min_size(vec2(width, 36.0));
    ui.add(button)
}

/// Show a button that is one choice of several, such as a build plate or a camera view.
pub fn choice_button(ui: &mut Ui, text: &str, selected: bool, width: f32) -> Response {
    let (text_color, fill, stroke) = if selected {
        (
            color::GOLD_300,
            color::accent_soft(),
            color::border_strong(),
        )
    } else {
        (
            color::TEXT_SECONDARY,
            color::SURFACE_INSET,
            color::border_steel(),
        )
    };
    let button = Button::new(theme::label(text).size(11.5).color(text_color))
        .fill(fill)
        .stroke(Stroke::new(1.0_f32, stroke))
        .corner_radius(RADIUS_SM)
        .min_size(vec2(width, 32.0));
    ui.add(button)
}

/// Show a button with only an icon, and the name of its action as a tooltip.
pub fn icon_button(ui: &mut Ui, icon: &str, name: &str, enabled: bool) -> Response {
    let button = Button::new(RichText::new(icon).size(20.0).color(color::TEXT_SECONDARY))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE)
        .min_size(vec2(36.0, 36.0));
    ui.add_enabled(enabled, button).on_hover_text(name)
}

/// Show choices side by side over the full width, and return true when the user picks another one.
pub fn segmented<T: PartialEq + Copy>(ui: &mut Ui, current: &mut T, options: &[(T, &str)]) -> bool {
    let gap = 4.0;
    let count = options.len() as f32;
    let width = (ui.available_width() - gap * (count - 1.0)) / count;
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (value, name) in options {
            if choice_button(ui, name, *current == *value, width).clicked() && *current != *value {
                *current = *value;
                changed = true;
            }
        }
    });
    changed
}

/// Show one tool of the rail: an icon over a short name.
pub fn rail_button(ui: &mut Ui, icon: &str, name: &str, selected: bool, danger: bool) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 64.0), Sense::click());
    let (fill, stroke, text) = if selected {
        (
            color::accent_soft(),
            color::border_strong(),
            color::GOLD_300,
        )
    } else if response.hovered() {
        (
            color::INK_500,
            color::border_hairline(),
            color::TEXT_PRIMARY,
        )
    } else if danger {
        (
            Color32::TRANSPARENT,
            Color32::TRANSPARENT,
            color::DANGER_TEXT,
        )
    } else {
        (
            Color32::TRANSPARENT,
            Color32::TRANSPARENT,
            color::TEXT_SECONDARY,
        )
    };

    let painter = ui.painter();
    painter.rect(
        rect,
        RADIUS_SM,
        fill,
        Stroke::new(1.0_f32, stroke),
        StrokeKind::Inside,
    );
    painter.text(
        pos2(rect.center().x, rect.top() + 24.0),
        Align2::CENTER_CENTER,
        icon,
        FontId::proportional(22.0),
        text,
    );

    let galley = WidgetText::from(theme::label(name).size(10.5).color(text)).into_galley(
        ui,
        Some(TextWrapMode::Extend),
        f32::INFINITY,
        FontSelection::Default,
    );
    let position = pos2(rect.center().x - galley.size().x / 2.0, rect.top() + 42.0);
    ui.painter().galley(position, galley, text);

    response.on_hover_cursor(CursorIcon::PointingHand)
}

/// Show one step of the stepper: its number in a diamond, then its name.
pub fn step_button(
    ui: &mut Ui,
    number: usize,
    name: &str,
    state: StepState,
    width: f32,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(width, 38.0), Sense::click());
    let hovered = response.hovered() && state != StepState::Current;
    let (fill, border, text, diamond_fill, diamond_border) = match state {
        StepState::Current => (
            color::accent_soft(),
            color::border_strong(),
            color::GOLD_300,
            color::ACCENT,
            color::ACCENT,
        ),
        StepState::Done => (
            Color32::TRANSPARENT,
            color::SUCCESS,
            color::TEXT_SECONDARY,
            Color32::TRANSPARENT,
            color::SUCCESS_TEXT,
        ),
        StepState::Later => (
            Color32::TRANSPARENT,
            color::border_steel(),
            color::TEXT_SECONDARY,
            Color32::TRANSPARENT,
            color::BONE_400,
        ),
    };
    let border = if hovered { color::ACCENT_HOVER } else { border };

    let painter = ui.painter();
    painter.rect(
        rect,
        RADIUS_SM,
        fill,
        Stroke::new(1.0_f32, border),
        StrokeKind::Inside,
    );

    let center = pos2(rect.left() + 24.0, rect.center().y);
    let radius = 11.0;
    let diamond = vec![
        center + vec2(0.0, -radius),
        center + vec2(radius, 0.0),
        center + vec2(0.0, radius),
        center + vec2(-radius, 0.0),
    ];
    painter.add(Shape::convex_polygon(
        diamond,
        diamond_fill,
        Stroke::new(1.0_f32, diamond_border),
    ));

    let (mark, mark_color) = match state {
        StepState::Done => (CHECK.to_owned(), color::SUCCESS_TEXT),
        StepState::Current => (number.to_string(), color::TEXT_ON_GOLD),
        StepState::Later => (number.to_string(), color::TEXT_SECONDARY),
    };
    painter.text(
        center,
        Align2::CENTER_CENTER,
        mark,
        FontId::proportional(12.0),
        mark_color,
    );

    let galley = WidgetText::from(theme::label(name).size(13.0).color(text)).into_galley(
        ui,
        Some(TextWrapMode::Extend),
        f32::INFINITY,
        FontSelection::Default,
    );
    let position = pos2(rect.left() + 46.0, rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(position, galley, text);

    response.on_hover_cursor(CursorIcon::PointingHand)
}

/// Show a small tile with a label over a value, such as the print time.
pub fn stat_tile(ui: &mut Ui, label: &str, value: &str, width: f32) {
    Frame::new()
        .fill(color::SURFACE_CARD_RAISED)
        .stroke(Stroke::new(1.0_f32, color::border_hairline()))
        .corner_radius(RADIUS_SM)
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(width - 22.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.label(theme::label(label).size(10.5).color(color::TEXT_SECONDARY));
                ui.label(RichText::new(value).size(17.0).color(color::TEXT_PRIMARY));
            });
        });
}

/// Show tiles in one row over the full width.
pub fn stat_row(ui: &mut Ui, tiles: &[(&str, String)]) {
    let gap = 8.0;
    let count = tiles.len() as f32;
    let width = (ui.available_width() - gap * (count - 1.0)) / count;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (label, value) in tiles {
            stat_tile(ui, label, value, width);
        }
    });
}

/// Show one result of a check: a mark, the name of the check and the result in words.
pub fn check_row(ui: &mut Ui, passed: bool, name: &str, result: &str) {
    let (fill, border, mark, mark_color) = if passed {
        (
            color::success_soft(),
            color::SUCCESS,
            CHECK,
            color::SUCCESS_TEXT,
        )
    } else {
        (
            color::danger_soft(),
            color::DANGER,
            WARNING,
            color::DANGER_TEXT,
        )
    };
    Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0_f32, border))
        .corner_radius(RADIUS_SM)
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(mark).size(16.0).color(mark_color));
                ui.label(RichText::new(name).color(color::TEXT_PRIMARY));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(result)
                            .size(13.0)
                            .color(color::TEXT_SECONDARY),
                    );
                });
            });
        });
}

/// Return the color of the letter of an axis: crimson for X, patina for Y and steel for Z.
pub fn axis_color(axis: usize) -> Color32 {
    [color::DANGER_TEXT, color::SUCCESS_TEXT, color::STEEL_300][axis]
}

/// Show three number fields for X, Y and Z in one row, and return true while the user edits one.
pub fn axis_row(ui: &mut Ui, values: &mut [f32], suffix: &str, speed: f64) -> bool {
    let gap = 8.0;
    let width = (ui.available_width() - gap * 2.0) / 3.0;
    let mut editing = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (axis, value) in values.iter_mut().enumerate().take(3) {
            ui.allocate_ui(vec2(width, 32.0), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.label(
                        RichText::new(["X", "Y", "Z"][axis])
                            .strong()
                            .color(axis_color(axis)),
                    );
                    ui.spacing_mut().interact_size.x = width - 20.0;
                    let field = DragValue::new(value)
                        .speed(speed)
                        .max_decimals(2)
                        .suffix(suffix);
                    ui.add(field).being_edited(&mut editing);
                });
            });
        }
    });
    editing
}
