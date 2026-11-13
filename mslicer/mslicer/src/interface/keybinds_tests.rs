use toml::{Table, Value};

use crate::interface::keybinds::{Keybind, Keybinds};

#[test]
fn a_config_from_an_older_version_without_a_keybind_still_loads() {
    let mut table: Table = toml::from_str(&toml::to_string(&Keybinds::default()).unwrap()).unwrap();
    table
        .remove("measure")
        .expect("The default keybinds have a measure shortcut.");
    let old = toml::to_string(&Value::Table(table)).unwrap();

    let loaded: Keybinds = toml::from_str(&old).expect("The old keybinds do not load.");

    assert!(loaded.get(Keybind::Measure) == Keybinds::default().get(Keybind::Measure));
}
