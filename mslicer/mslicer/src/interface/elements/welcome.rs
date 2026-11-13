use egui::{Color32, Context, FontFamily, FontId, Frame, Id, Modal, RichText};

use crate::{
    VERSION,
    core::App,
    interface::{
        components::button_row,
        theme::{self, color},
    },
};

const BACKGROUND_TINT: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 100);

const DESCRIPTION: &str = "Embercast is a free, open-source resin slicer for the Creality Halot One Plus. It is a fork of mslicer, the high-performance MSLA slicer by Connor Slade, and keeps its GPL-3.0 license.";
const CREDITS: &str = "The CXDLP file format follows UVtools by Tiago Conceição. The fonts Cinzel, Barlow and EB Garamond are under the SIL Open Font License.";

pub const EMBERCAST_LINK: &str = "https://github.com/bydeusz/embercast";
pub const MSLICER_LINK: &str = "https://github.com/connorslade/mslicer";
const UVTOOLS_LINK: &str = "https://github.com/sn4k3/UVtools";

/// Show the welcome and About window, with the credits to mslicer.
pub fn ui(app: &mut App, ctx: &Context) {
    if !app.config.ui.about {
        return;
    }

    let modal = Modal::new(Id::new("about"))
        .backdrop_color(BACKGROUND_TINT)
        .frame(Frame::window(&ctx.style()))
        .show(ctx, |ui| {
            ui.set_width(420.0);

            ui.vertical_centered(|ui| {
                let wordmark = FontId::new(30.0, FontFamily::Name(theme::DISPLAY.into()));
                ui.label(RichText::new("EMBERCAST").font(wordmark).color(color::ACCENT));
                ui.label(RichText::new(format!("VERSION {VERSION}")).color(color::TEXT_SECONDARY));
            });
            ui.separator();

            let prose = FontId::new(15.0, FontFamily::Name(theme::SERIF.into()));
            ui.label(RichText::new(DESCRIPTION).font(prose.clone()));
            ui.add_space(5.0);
            ui.label(RichText::new(CREDITS).font(prose));
            ui.add_space(5.0);

            ui.horizontal_wrapped(|ui| {
                ui.hyperlink_to("Embercast source code", EMBERCAST_LINK)
                    .on_hover_text(EMBERCAST_LINK);
                ui.hyperlink_to("mslicer", MSLICER_LINK)
                    .on_hover_text(MSLICER_LINK);
                ui.hyperlink_to("UVtools", UVTOOLS_LINK)
                    .on_hover_text(UVTOOLS_LINK);
            });

            ui.add_space(5.0);
            button_row(ui, [("Continue", &mut || app.config.ui.about = false)]);
        });

    if ctx.input(|i| !i.keys_down.is_empty()) || modal.should_close() {
        app.config.ui.about = false;
    }
}
