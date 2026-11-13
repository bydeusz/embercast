//! The Embercast look, from the tokens of the design system "DS: Dark Fantasy
//! world" in `design/tokens.json`. The rest of the interface uses only the
//! names in this module and never its own color values.

use std::sync::Arc;

use egui::{
    Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, RichText, Shadow,
    Stroke, TextStyle, Visuals,
    style::{Selection, WidgetVisuals, Widgets},
};

/// The colors of the design system, as sRGB.
pub mod color {
    use egui::Color32;

    /// `bg-base` (ink-900): the deepest ground.
    pub const BG_BASE: Color32 = Color32::from_rgb(0x08, 0x08, 0x0a);
    /// `surface-inset` (ink-800): input fields.
    pub const SURFACE_INSET: Color32 = Color32::from_rgb(0x0c, 0x0c, 0x10);
    /// `surface-card` (ink-700): panels.
    pub const SURFACE_CARD: Color32 = Color32::from_rgb(0x12, 0x12, 0x17);
    /// `surface-card-raised` (ink-600): windows and controls on a panel.
    pub const SURFACE_CARD_RAISED: Color32 = Color32::from_rgb(0x1a, 0x1a, 0x21);
    /// ink-500: a control under the pointer.
    pub const INK_500: Color32 = Color32::from_rgb(0x23, 0x23, 0x2c);
    /// ink-300: the lightest ink, for the build plate grid.
    pub const INK_300: Color32 = Color32::from_rgb(0x3c, 0x3c, 0x48);

    /// `text-primary` (bone-100).
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xf4, 0xee, 0xde);
    /// `text-secondary` (bone-300).
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0xc9, 0xbc, 0xa0);
    /// `text-on-gold`: text on a gold fill.
    pub const TEXT_ON_GOLD: Color32 = Color32::from_rgb(0x1c, 0x12, 0x06);

    /// `accent` (gold-500).
    pub const ACCENT: Color32 = Color32::from_rgb(0xd1, 0x8b, 0x2c);
    /// `accent-hover` (gold-400).
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0xe6, 0xab, 0x4c);
    /// `accent-press` (gold-600).
    pub const ACCENT_PRESS: Color32 = Color32::from_rgb(0xad, 0x6d, 0x1d);

    /// `danger` (crimson-500).
    pub const DANGER: Color32 = Color32::from_rgb(0x9e, 0x2b, 0x25);
    /// crimson-300: danger text on a dark ground.
    pub const DANGER_TEXT: Color32 = Color32::from_rgb(0xd9, 0x65, 0x5c);

    /// `bg-sunken`: the status bar at the bottom of the window.
    pub const BG_SUNKEN: Color32 = Color32::from_rgb(0x05, 0x05, 0x06);
    /// gold-300: the label of the active step, tool or tab.
    pub const GOLD_300: Color32 = Color32::from_rgb(0xf0, 0xc8, 0x78);
    /// bone-400: dim text, such as hints and units.
    pub const BONE_400: Color32 = Color32::from_rgb(0xa8, 0x9c, 0x81);
    /// steel-300: the Z axis.
    pub const STEEL_300: Color32 = Color32::from_rgb(0xa7, 0xa7, 0xb0);
    /// `success` (patina-500): borders of a passed check.
    pub const SUCCESS: Color32 = Color32::from_rgb(0x4e, 0x6e, 0x57);
    /// patina-400: success text and the Y axis.
    pub const SUCCESS_TEXT: Color32 = Color32::from_rgb(0x6f, 0x90, 0x70);

    /// `border-hairline`: thin gold lines at rest.
    pub fn border_hairline() -> Color32 {
        Color32::from_rgba_unmultiplied(0xd1, 0x8b, 0x2c, 41)
    }

    /// `border-strong`: gold lines with emphasis.
    pub fn border_strong() -> Color32 {
        Color32::from_rgba_unmultiplied(0xd1, 0x8b, 0x2c, 87)
    }

    /// `accent-soft`: the fill behind an active step, tool or tab.
    pub fn accent_soft() -> Color32 {
        Color32::from_rgba_unmultiplied(0xd1, 0x8b, 0x2c, 36)
    }

    /// `border-steel`: the border of fields and quiet buttons.
    pub fn border_steel() -> Color32 {
        Color32::from_rgba_unmultiplied(0xa7, 0xa7, 0xb0, 31)
    }

    /// `divider`: quiet lines between sections.
    pub fn divider() -> Color32 {
        Color32::from_rgba_unmultiplied(0xa7, 0xa7, 0xb0, 26)
    }

    /// `success-soft`: the fill behind a passed check.
    pub fn success_soft() -> Color32 {
        Color32::from_rgba_unmultiplied(0x4e, 0x6e, 0x57, 46)
    }

    /// `danger-soft`: the fill behind a failed check.
    pub fn danger_soft() -> Color32 {
        Color32::from_rgba_unmultiplied(0x9e, 0x2b, 0x25, 41)
    }
}

/// The font family for titles: Cinzel, engraved Roman capitals.
pub const DISPLAY: &str = "display";
/// The font family for labels and buttons: Barlow Semi Condensed.
pub const LABEL: &str = "label";
/// The font family for longer prose: EB Garamond.
pub const SERIF: &str = "serif";

/// The corner radii of the design system, in points.
pub const RADIUS_SM: u8 = 2;
pub const RADIUS_MD: u8 = 4;

/// The Cinzel font file, for titles and for text labels.
pub const CINZEL_FILE: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/Cinzel-Variable.ttf"));
/// The Barlow SemiBold font file, for text labels.
pub const LABEL_FONT_FILE: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/Barlow-SemiBold.ttf"));

macro_rules! font {
    ($name:literal) => {
        FontData::from_static(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/",
            $name
        )))
    };
}

/// Return the fonts of the interface, with the icon font of `egui_phosphor`.
///
/// Barlow is the default for all text and numbers. Cinzel is for titles,
/// Barlow Semi Condensed for labels and buttons, and EB Garamond for longer
/// text. The fonts are in `assets/fonts`, with their SIL Open Font License.
pub fn fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    for (name, data) in [
        ("Barlow", font!("Barlow-Regular.ttf")),
        ("Barlow Medium", font!("Barlow-Medium.ttf")),
        ("Barlow Semi Condensed", font!("BarlowSemiCondensed-Medium.ttf")),
        ("Cinzel", FontData::from_static(CINZEL_FILE)),
        ("EB Garamond", font!("EBGaramond-Variable.ttf")),
    ] {
        fonts.font_data.insert(name.to_owned(), Arc::new(data));
    }

    let defaults = fonts.families[&FontFamily::Proportional].clone();
    let family = |first: &str| [vec![first.to_owned()], defaults.clone()].concat();
    fonts.families.insert(FontFamily::Proportional, family("Barlow"));
    fonts.families.insert(FontFamily::Name(DISPLAY.into()), family("Cinzel"));
    fonts.families.insert(FontFamily::Name(LABEL.into()), family("Barlow Semi Condensed"));
    fonts.families.insert(FontFamily::Name(SERIF.into()), family("EB Garamond"));

    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    for name in [DISPLAY, LABEL, SERIF] {
        let list = fonts.families.get_mut(&FontFamily::Name(name.into())).unwrap();
        list.push("phosphor".to_owned());
    }
    fonts
}

fn widget(fill: Color32, border: Color32, text: Color32, radius: u8) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(1.0_f32, border),
        fg_stroke: Stroke::new(1.0_f32, text),
        corner_radius: CornerRadius::same(radius),
        expansion: 0.0,
    }
}

/// Return the colors and shapes of the interface: ink grounds, gold hairlines, sharp corners.
pub fn visuals() -> Visuals {
    use color::*;

    let deep_shadow = Shadow {
        offset: [0, 6],
        blur: 18,
        spread: 0,
        color: Color32::from_black_alpha(140),
    };

    Visuals {
        dark_mode: true,
        override_text_color: None,
        widgets: Widgets {
            noninteractive: widget(SURFACE_CARD, border_hairline(), TEXT_SECONDARY, RADIUS_SM),
            inactive: widget(SURFACE_CARD_RAISED, border_hairline(), TEXT_PRIMARY, RADIUS_SM),
            hovered: widget(INK_500, ACCENT_HOVER, TEXT_PRIMARY, RADIUS_SM),
            active: widget(ACCENT_PRESS, ACCENT, TEXT_ON_GOLD, RADIUS_SM),
            open: widget(SURFACE_CARD_RAISED, border_strong(), TEXT_PRIMARY, RADIUS_SM),
        },
        selection: Selection {
            bg_fill: ACCENT_PRESS,
            stroke: Stroke::new(1.0_f32, TEXT_PRIMARY),
        },
        hyperlink_color: ACCENT,
        faint_bg_color: SURFACE_CARD_RAISED,
        extreme_bg_color: SURFACE_INSET,
        code_bg_color: SURFACE_INSET,
        warn_fg_color: ACCENT,
        error_fg_color: DANGER_TEXT,
        window_corner_radius: CornerRadius::same(RADIUS_MD),
        window_shadow: deep_shadow,
        window_fill: SURFACE_CARD_RAISED,
        window_stroke: Stroke::new(1.0_f32, border_hairline()),
        menu_corner_radius: CornerRadius::same(RADIUS_MD),
        panel_fill: SURFACE_CARD,
        popup_shadow: deep_shadow,
        collapsing_header_frame: true,
        slider_trailing_fill: true,
        ..Visuals::dark()
    }
}

/// The extra space between the letters of a label, in points.
pub const LABEL_TRACKING: f32 = 1.4;

/// Return `text` as a label of the design system: capitals in Barlow Semi Condensed with wide letter spacing.
pub fn label(text: &str) -> RichText {
    RichText::new(text.to_uppercase())
        .family(FontFamily::Name(LABEL.into()))
        .extra_letter_spacing(LABEL_TRACKING)
}

/// Return `color` as red, green and blue between 0 and 1, for the 3D renderer.
pub fn rgb(color: Color32) -> [f32; 3] {
    [color.r(), color.g(), color.b()].map(|channel| channel as f32 / 255.0)
}

/// Set the fonts once, at the start of the app.
pub fn install(ctx: &Context) {
    ctx.set_fonts(fonts());
    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Heading, FontId::new(17.0, FontFamily::Name(DISPLAY.into()))),
            (TextStyle::Body, FontId::new(13.5, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(13.5, FontFamily::Name(LABEL.into()))),
            (TextStyle::Small, FontId::new(11.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(12.5, FontFamily::Monospace)),
        ]
        .into();
    });
}
