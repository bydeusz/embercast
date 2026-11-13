use egui::{Button, ComboBox, DragValue, TextEdit, Ui, Widget, vec2};
use slicer::geometry::mesh::Mesh;
use tools::text_mesh::text_mesh;

use crate::{
    interface::{
        components::grid,
        popup::PopupApp,
        theme::{CINZEL_FILE, LABEL_FONT_FILE},
    },
    task::GenerateMesh,
};

pub const NAME: &str = "Text Label";
pub const DESCRIPTION: &str = "Makes a 3D text model. Move it onto or partly into a model to print the text on that model.";

/// The settings of the text label window.
pub struct TextLabel {
    pub text: String,
    pub font: LabelFont,
    pub size: f32,
    pub depth: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LabelFont {
    Barlow,
    Cinzel,
}

impl LabelFont {
    const ALL: [Self; 2] = [Self::Barlow, Self::Cinzel];

    fn name(self) -> &'static str {
        match self {
            LabelFont::Barlow => "Barlow SemiBold",
            LabelFont::Cinzel => "Cinzel",
        }
    }

    fn data(self) -> &'static [u8] {
        match self {
            LabelFont::Barlow => LABEL_FONT_FILE,
            LabelFont::Cinzel => CINZEL_FILE,
        }
    }
}

impl Default for TextLabel {
    fn default() -> Self {
        Self {
            text: "EMBERCAST".to_owned(),
            font: LabelFont::Barlow,
            size: 6.0,
            depth: 1.0,
        }
    }
}

pub fn interface(app: &mut PopupApp, ui: &mut Ui) -> bool {
    ui.label(DESCRIPTION);
    ui.add_space(8.0);

    let tool = &mut app.state.tools.text_label;
    grid("text_label").show(ui, |ui| {
        ui.label("Text");
        TextEdit::singleline(&mut tool.text).desired_width(180.0).ui(ui);
        ui.end_row();

        ui.label("Font");
        ComboBox::from_id_salt("text_label_font")
            .selected_text(tool.font.name())
            .show_ui(ui, |ui| {
                for font in LabelFont::ALL {
                    ui.selectable_value(&mut tool.font, font, font.name());
                }
            });
        ui.end_row();

        ui.label("Size");
        DragValue::new(&mut tool.size)
            .speed(0.1)
            .range(1.0..=100.0)
            .suffix(" mm")
            .ui(ui);
        ui.end_row();

        ui.label("Depth");
        DragValue::new(&mut tool.depth)
            .speed(0.05)
            .range(0.2..=20.0)
            .suffix(" mm")
            .ui(ui);
        ui.end_row();
    });
    ui.add_space(8.0);

    let has_text = !tool.text.trim().is_empty();
    ui.vertical_centered(|ui| {
        let button = Button::new("Generate").min_size(vec2(ui.available_width(), 0.0));
        if ui.add_enabled(has_text, button).clicked() {
            let (text, font, size, depth) = (tool.text.clone(), tool.font, tool.size, tool.depth);
            let name = format!("Text {text}");
            app.tasks.add(GenerateMesh::new(name.into(), move |progress| {
                progress.set_total(1);
                let (vertices, faces) = text_mesh(font.data(), &text, size, depth).unwrap_or_default();
                progress.set_finished();
                Mesh::new_boxed_centered(vertices.into(), faces.into())
            }));
        }
    });

    false
}
