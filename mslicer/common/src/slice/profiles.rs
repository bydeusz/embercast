//! Built-in printer and resin profiles.

use nalgebra::{Vector2, Vector3};

use crate::{
    slice::{ExposureConfig, Height, SliceConfig},
    units::{CentimetersPerSecond, Milimeters, MilimetersPerMinute, Seconds},
};

/// The exposure and movement settings for one resin on one printer.
#[derive(Clone, Debug, PartialEq)]
pub struct ResinProfile {
    pub name: &'static str,
    pub printer: &'static str,
    pub slice_height: Milimeters,
    pub first_layers: Height,
    pub transition_layers: Height,
    pub exposure_config: ExposureConfig,
    pub first_exposure_config: ExposureConfig,
}

impl ResinProfile {
    /// Copy the settings of the profile into `config`. The printer settings of `config` stay the same.
    pub fn apply(&self, config: &mut SliceConfig) {
        config.slice_height = self.slice_height;
        config.first_layers = self.first_layers;
        config.transition_layers = self.transition_layers;
        config.exposure_config = self.exposure_config.clone();
        config.first_exposure_config = self.first_exposure_config.clone();
    }

    /// Return True if `config` has all the settings of the profile.
    pub fn matches(&self, config: &SliceConfig) -> bool {
        config.slice_height == self.slice_height
            && config.first_layers == self.first_layers
            && config.transition_layers == self.transition_layers
            && config.exposure_config == self.exposure_config
            && config.first_exposure_config == self.first_exposure_config
    }
}

fn millimeters_per_minute(speed: f32) -> CentimetersPerSecond {
    MilimetersPerMinute::new(speed).convert()
}

/// Return the Siraya Tech Build profile at 50 µm for the Creality Halot One Plus.
///
/// The values come from the CHITUBOX profile of the user, in
/// `profiles/halot-one-plus_siraya-build-50um.json`. The light-off delay of
/// that profile is the `exposure_delay`.
pub fn siraya_build_halot_one_plus() -> ResinProfile {
    ResinProfile {
        name: "Siraya Tech Build 50 µm",
        printer: "Creality HALOT-ONE PLUS",
        slice_height: Milimeters::new(0.05),
        first_layers: Height::Layers(6),
        transition_layers: Height::Layers(0),
        exposure_config: ExposureConfig {
            exposure_time: Seconds::new(3.2),
            exposure_delay: Seconds::new(4.0),
            pwm: 255,
            lift_distance: Milimeters::new(6.0),
            lift_speed: millimeters_per_minute(60.0),
            retract_speed: millimeters_per_minute(180.0),
        },
        first_exposure_config: ExposureConfig {
            exposure_time: Seconds::new(35.0),
            exposure_delay: Seconds::new(4.0),
            pwm: 255,
            lift_distance: Milimeters::new(6.0),
            lift_speed: millimeters_per_minute(60.0),
            retract_speed: millimeters_per_minute(150.0),
        },
    }
}

/// Return all built-in resin profiles.
pub fn resin_profiles() -> Vec<ResinProfile> {
    vec![siraya_build_halot_one_plus()]
}

/// Return the screen resolution of the Creality Halot One Plus, in pixels.
pub const fn halot_one_plus_resolution() -> Vector2<u32> {
    Vector2::new(4320, 2560)
}

/// Return the build volume of the Creality Halot One Plus.
pub const fn halot_one_plus_size() -> Vector3<Milimeters> {
    Vector3::new(
        Milimeters::new(172.8),
        Milimeters::new(102.4),
        Milimeters::new(160.0),
    )
}
