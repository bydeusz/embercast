//! The estimate of layers, print time and resin before the plate is sliced.

use common::units::{Centimeter, Milimeter};

use crate::core::App;

/// The print estimate of the active build plate, before slicing.
pub struct Estimate {
    pub layers: u32,
    pub print_time: String,
    pub resin_ml: f32,
}

/// Return the number of layers that a print needs when its top is `top` millimeters above the plate.
///
/// The layers are `layer_height` millimeters high. The function ignores a
/// rest under a thousandth of a layer, because the height of a mesh often
/// has rounding noise.
pub fn layer_count(top: f32, layer_height: f32) -> u32 {
    if layer_height <= 0.0 || top <= 0.0 {
        return 0;
    }
    (top / layer_height - 0.001).ceil().max(0.0) as u32
}

/// Return the estimate for the visible models on the active plate.
///
/// The layer count comes from the highest point of the models. The print
/// time comes from the exposure settings of the project. The resin is the
/// volume of the models without the supports.
pub fn estimate(app: &App) -> Estimate {
    let plate = app.project.active_plate;
    let models = || {
        app.project
            .models
            .iter()
            .filter(|model| model.is_shown(plate))
    };

    let top = models().map(|model| model.bounds.max.z).fold(0.0, f32::max);
    let config = &app.project.slice_config;
    let layers = layer_count(top, config.slice_height.get::<Milimeter>());
    let resin_ml = models()
        .filter_map(|model| model.volume())
        .map(|volume| volume.get::<Centimeter>())
        .sum();

    Estimate {
        layers,
        print_time: short_duration(config.print_time(layers).raw()),
        resin_ml,
    }
}

/// Return a print time of `seconds` in hours and whole minutes, such as "1 h 05 min".
///
/// The function rounds to the nearest minute. A time under half a minute
/// gives "< 1 min", except a time of zero, which gives "0 min".
pub fn short_duration(seconds: f32) -> String {
    let minutes = (seconds.max(0.0) / 60.0).round() as u64;
    match (minutes / 60, minutes % 60) {
        (0, 0) if seconds > 0.0 => "< 1 min".to_owned(),
        (0, minutes) => format!("{minutes} min"),
        (hours, minutes) => format!("{hours} h {minutes:02} min"),
    }
}
