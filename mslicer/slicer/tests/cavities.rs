use common::{
    container::{Run, rle},
    progress::Progress,
    slice::{ExposureConfig, Layer, SliceConfig},
    units::Milimeters,
};
use nalgebra::{Vector2, Vector3};
use slicer::post_process::cavities::{CavityKind, detect_cavities};

const WIDTH: usize = 300;
const HEIGHT: usize = 300;
const LAYERS: usize = 120;
const PIXEL_MM: f32 = 0.05;

/// Return a 15 x 15 mm screen with 0.05 mm pixels and 0.05 mm layers.
fn config() -> SliceConfig {
    SliceConfig {
        platform_resolution: Vector2::new(WIDTH as u32, HEIGHT as u32),
        platform_size: Vector3::new(WIDTH as f32 * PIXEL_MM, HEIGHT as f32 * PIXEL_MM, 50.0).map(Milimeters::new),
        slice_height: Milimeters::new(0.05),
        ..Default::default()
    }
}

/// Return `LAYERS` layers where a pixel is lit if `solid` returns True for its column, row and layer.
fn layers_from(solid: impl Fn(usize, usize, usize) -> bool) -> Vec<Layer> {
    (0..LAYERS)
        .map(|layer| {
            let mut runs: Vec<Run> = Vec::new();
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let value = if solid(x, y, layer) { 255 } else { 0 };
                    match runs.last_mut() {
                        Some(run) if run.value == value => run.length += 1,
                        _ => runs.push(Run::new(1, value)),
                    }
                }
            }
            Layer::new(runs, Milimeters::new(0.05 * (layer + 1) as f32), ExposureConfig::default())
        })
        .collect()
}

/// Return True inside a 10 x 10 mm box with 1 mm walls from layer `bottom` to layer 100.
///
/// The box has a roof if `roof` is True, and a hole of 1 mm in the bottom wall if `hole` is True.
fn box_shell(x: usize, y: usize, layer: usize, bottom: usize, roof: bool, hole: bool) -> bool {
    let outer = (50..250).contains(&x) && (50..250).contains(&y) && (bottom..100).contains(&layer);
    let inner = (70..230).contains(&x) && (70..230).contains(&y);
    let floor = layer < bottom + 20;
    let ceiling = roof && layer >= 80;
    let in_hole = hole && floor && x.abs_diff(150) < 10 && y.abs_diff(150) < 10;
    outer && (!inner || floor || ceiling) && !in_hole
}

#[test]
fn closed_box_is_one_resin_trap_with_the_right_volume() {
    let layers = layers_from(|x, y, l| box_shell(x, y, l, 0, true, false));
    let report = detect_cavities(&config(), &layers, &Progress::new());

    assert_eq!(report.cavities.len(), 1);
    let cavity = &report.cavities[0];
    assert_eq!(cavity.kind, CavityKind::ResinTrap);
    let expected = 8.0 * 8.0 * 3.0;
    assert!((cavity.volume_mm3 - expected).abs() < expected * 0.15, "Volume {} is not about {expected}.", cavity.volume_mm3);
    assert!(cavity.first_layer >= 16 && cavity.first_layer <= 24);
    assert!(cavity.last_layer >= 76 && cavity.last_layer <= 84);
}

#[test]
fn box_with_a_drain_hole_on_supports_has_no_cavity() {
    let layers = layers_from(|x, y, l| {
        let support = [(60, 60), (240, 60), (60, 240), (240, 240)]
            .iter()
            .any(|&(cx, cy): &(usize, usize)| x.abs_diff(cx) < 5 && y.abs_diff(cy) < 5);
        if l < 10 { support } else { box_shell(x, y, l, 10, true, true) }
    });
    let report = detect_cavities(&config(), &layers, &Progress::new());
    assert!(report.is_empty(), "Found {:?}.", report.cavities);
}

#[test]
fn box_with_a_drain_hole_on_the_build_plate_is_still_a_resin_trap() {
    let layers = layers_from(|x, y, l| box_shell(x, y, l, 0, true, true));
    let report = detect_cavities(&config(), &layers, &Progress::new());
    assert_eq!(report.cavities.len(), 1);
    assert_eq!(report.cavities[0].kind, CavityKind::ResinTrap);
}

#[test]
fn box_without_a_roof_is_a_suction_cup() {
    let layers = layers_from(|x, y, l| box_shell(x, y, l, 0, false, false));
    let report = detect_cavities(&config(), &layers, &Progress::new());
    assert_eq!(report.cavities.len(), 1);
    assert_eq!(report.cavities[0].kind, CavityKind::SuctionCup);
}

#[test]
fn solid_box_has_no_cavity() {
    let layers = layers_from(|x, y, l| (50..250).contains(&x) && (50..250).contains(&y) && l < 100);
    let report = detect_cavities(&config(), &layers, &Progress::new());
    assert!(report.is_empty());
}

#[test]
fn layer_marks_cover_the_empty_pixels_of_the_trap_only() {
    let layers = layers_from(|x, y, l| box_shell(x, y, l, 0, true, false));
    let config = config();
    let report = detect_cavities(&config, &layers, &Progress::new());

    let marks = report.layer_marks(&layers[50], 50, &config).unwrap();
    let total: u64 = marks.iter().map(|run| run.length).sum();
    assert_eq!(total, (WIDTH * HEIGHT) as u64);

    let pixels = rle::decode_vec(&marks);
    let trap = CavityKind::ResinTrap.mark();
    assert_eq!(pixels[150 * WIDTH + 150], trap, "The middle of the cavity is not marked.");
    assert_eq!(pixels[150 * WIDTH + 60], 0, "The wall is marked.");
    assert_eq!(pixels[10 * WIDTH + 10], 0, "The outside is marked.");
    assert!(report.layer_marks(&layers[110], 110, &config).is_none());
}
