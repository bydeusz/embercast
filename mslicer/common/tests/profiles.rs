use std::{fs, path::Path};

use common::{
    slice::{
        Height, SliceConfig,
        profiles::{halot_one_plus_resolution, halot_one_plus_size, siraya_build_halot_one_plus},
    },
    units::{CentimetersPerSecond, Milimeter, Milimeters, Minute, Second, Seconds},
};
use serde_json::Value;

/// Return the CHITUBOX profile of the user from the `profiles` folder of the repository.
fn chitubox_profile() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../profiles/halot-one-plus_siraya-build-50um.json");
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn number(value: &Value, section: &str, key: &str) -> f32 {
    value[section][key].as_f64().unwrap_or_else(|| panic!("{section}.{key} is missing")) as f32
}

fn assert_close(actual: f32, expected: f32, what: &str) {
    assert!((actual - expected).abs() < 1e-3, "{what}: {actual} is not {expected}");
}

#[test]
fn siraya_profile_has_the_values_of_the_chitubox_profile() {
    let json = chitubox_profile();
    let profile = siraya_build_halot_one_plus();
    let normal = &profile.exposure_config;
    let bottom = &profile.first_exposure_config;

    assert_close(profile.slice_height.get::<Milimeter>(), number(&json, "print", "laaghoogte_mm"), "layer height");
    assert_eq!(profile.first_layers, Height::Layers(number(&json, "print", "bodemlagen") as u32));
    assert_eq!(profile.transition_layers, Height::Layers(0));

    assert_close(normal.exposure_time.get::<Second>(), number(&json, "print", "belichtingstijd_s"), "exposure");
    assert_close(bottom.exposure_time.get::<Second>(), number(&json, "print", "bodem_belichtingstijd_s"), "bottom exposure");
    let delay = number(&json, "print", "licht_uit_vertraging_s");
    assert_close(normal.exposure_delay.get::<Second>(), delay, "light-off delay");
    assert_close(bottom.exposure_delay.get::<Second>(), delay, "bottom light-off delay");

    assert_close(normal.lift_distance.get::<Milimeter>(), number(&json, "print", "liftafstand_mm"), "lift");
    assert_close(bottom.lift_distance.get::<Milimeter>(), number(&json, "print", "bodem_liftafstand_mm"), "bottom lift");
    let per_minute = |speed: CentimetersPerSecond| speed.get::<Milimeter, Minute>();
    assert_close(per_minute(normal.lift_speed), number(&json, "print", "liftsnelheid_mm_per_min"), "lift speed");
    assert_close(per_minute(bottom.lift_speed), number(&json, "print", "bodem_liftsnelheid_mm_per_min"), "bottom lift speed");
    assert_close(per_minute(normal.retract_speed), number(&json, "print", "terugtreksnelheid_mm_per_min"), "retract speed");
    assert_close(per_minute(bottom.retract_speed), number(&json, "print", "bodem_terugtreksnelheid_mm_per_min"), "bottom retract speed");

    assert_eq!(normal.pwm as f32, number(&json, "geavanceerd", "licht_pwm"));
    assert_eq!(bottom.pwm as f32, number(&json, "geavanceerd", "bodem_licht_pwm"));
}

#[test]
fn halot_one_plus_has_the_screen_of_the_chitubox_profile() {
    let json = chitubox_profile();
    let resolution = halot_one_plus_resolution();
    let size = halot_one_plus_size();
    assert_eq!(resolution.x as f32, json["printer"]["resolutie_px"][0].as_f64().unwrap() as f32);
    assert_eq!(resolution.y as f32, json["printer"]["resolutie_px"][1].as_f64().unwrap() as f32);
    for (axis, millimeters) in size.iter().enumerate() {
        let expected = json["printer"]["bouwvolume_mm"][axis].as_f64().unwrap() as f32;
        assert_close(millimeters.get::<Milimeter>(), expected, "build volume");
    }
}

#[test]
fn default_slice_config_is_the_halot_one_plus_with_the_siraya_profile() {
    let config = SliceConfig::default();
    assert_eq!(config.platform_resolution, halot_one_plus_resolution());
    assert_eq!(config.platform_size, halot_one_plus_size());
    assert!(siraya_build_halot_one_plus().matches(&config));
}

#[test]
fn applying_a_profile_keeps_the_printer_and_sets_the_exposure() {
    let mut config = SliceConfig::default();
    config.exposure_config.exposure_time = Seconds::new(9.0);
    config.slice_height = Milimeters::new(0.1);
    let profile = siraya_build_halot_one_plus();
    assert!(!profile.matches(&config));
    profile.apply(&mut config);
    assert!(profile.matches(&config));
    assert_eq!(config.platform_resolution, halot_one_plus_resolution());
}
