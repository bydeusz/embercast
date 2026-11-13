use common::{
    container::{Run, rle},
    progress::Progress,
    slice::{ExposureConfig, Layer, SliceConfig},
    units::Milimeters,
};
use nalgebra::{Vector2, Vector3};
use slicer::post_process::hollow::Hollow;

const WIDTH: usize = 400;
const HEIGHT: usize = 400;
const LAYERS: usize = 100;
const PIXEL_MM: f32 = 0.05;

/// Return a 20 x 20 mm screen with 0.05 mm pixels and 0.05 mm layers.
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

/// Return True for the pixels of a 10 x 10 mm block that is 5 mm high, in the middle of the screen.
fn block(x: usize, y: usize, _layer: usize) -> bool {
    (100..300).contains(&x) && (100..300).contains(&y)
}

fn lit(layers: &[Layer], x: usize, y: usize, layer: usize) -> bool {
    rle::decode_vec(&*layers[layer].data)[y * WIDTH + x] != 0
}

fn hollow(wall: f32) -> Hollow {
    Hollow {
        enabled: true,
        wall_thickness: wall,
        drain_holes: 0,
        ..Default::default()
    }
}

#[test]
fn middle_of_a_thick_block_becomes_empty() {
    let mut layers = layers_from(block);
    let report = hollow(1.0).apply(&config(), &mut layers, &Progress::new());

    assert!(report.removed_pixels > 0);
    assert!(!lit(&layers, 200, 200, 50), "The middle of the block is still lit.");
}

#[test]
fn walls_keep_at_least_the_wall_thickness() {
    let mut layers = layers_from(block);
    hollow(1.0).apply(&config(), &mut layers, &Progress::new());

    let side_wall = (100..300).take_while(|&x| lit(&layers, x, 200, 50)).count();
    let wall_pixels = (1.0 / PIXEL_MM) as usize;
    assert!(side_wall >= wall_pixels, "Side wall is {side_wall} pixels, less than {wall_pixels}.");
    assert!(side_wall <= wall_pixels + 12, "Side wall is {side_wall} pixels, much thicker than {wall_pixels}.");

    assert!(lit(&layers, 200, 200, 5), "The bottom wall is open.");
    assert!(lit(&layers, 200, 200, LAYERS - 5), "The top wall is open.");
}

#[test]
fn thin_parts_stay_solid() {
    let bar = |x: usize, y: usize, _layer: usize| (190..220).contains(&x) && (50..350).contains(&y);
    let mut layers = layers_from(bar);
    let report = hollow(1.0).apply(&config(), &mut layers, &Progress::new());
    assert_eq!(report.removed_pixels, 0);
}

#[test]
fn every_layer_still_covers_the_whole_screen() {
    let mut layers = layers_from(block);
    let mut settings = hollow(1.0);
    settings.drain_holes = 2;
    settings.pillar_spacing = 2.0;
    settings.pillar_width = 0.5;
    settings.apply(&config(), &mut layers, &Progress::new());

    for layer in &layers {
        let total: u64 = layer.data.iter().map(|run| run.length).sum();
        assert_eq!(total, (WIDTH * HEIGHT) as u64);
    }
}

#[test]
fn pillars_stay_inside_the_cavity() {
    let mut layers = layers_from(block);
    let mut settings = hollow(1.0);
    settings.pillar_spacing = 2.0;
    settings.pillar_width = 0.5;
    settings.apply(&config(), &mut layers, &Progress::new());

    let pixels = rle::decode_vec(&*layers[50].data);
    let middle = (150..250).flat_map(|y| (150..250).map(move |x| (x, y)));
    let lit_in_middle = middle.filter(|&(x, y)| pixels[y * WIDTH + x] != 0).count();
    assert!(lit_in_middle > 0, "There are no pillars.");
    assert!(lit_in_middle < 100 * 100 / 2, "The cavity is mostly lit.");
}

#[test]
fn drain_holes_open_the_bottom_wall_near_the_build_plate() {
    let mut layers = layers_from(block);
    let mut settings = hollow(1.0);
    settings.drain_holes = 2;
    settings.drain_hole_diameter = 1.0;
    let report = settings.apply(&config(), &mut layers, &Progress::new());

    assert_eq!(report.drain_holes.len(), 2);
    for hole in &report.drain_holes {
        assert_eq!(hole.bottom_layer, 0, "The hole does not reach the bottom of the block.");
        let (x, y) = (hole.center.x as usize, hole.center.y as usize);
        for layer in 0..=hole.top_layer {
            assert!(!lit(&layers, x, y, layer), "Layer {layer} closes the hole at ({x}, {y}).");
        }
    }
    let (a, b) = (report.drain_holes[0].center, report.drain_holes[1].center);
    assert!((a - b).norm() > 20.0, "The two holes are at the same place.");
}

#[test]
fn disabled_hollow_changes_nothing() {
    let mut layers = layers_from(block);
    let before: Vec<_> = layers.iter().map(|layer| layer.data.clone()).collect();
    Hollow::default().post_slice(&config(), &mut layers, Progress::new());
    for (layer, old) in layers.iter().zip(before) {
        assert_eq!(*layer.data, *old);
    }
}

#[test]
fn drain_hole_stops_at_the_bottom_of_a_block_on_supports() {
    let lifted_block = |x: usize, y: usize, layer: usize| {
        let corner_support = [(110, 110), (290, 110), (110, 290), (290, 290)]
            .iter()
            .any(|&(cx, cy): &(usize, usize)| x.abs_diff(cx) < 8 && y.abs_diff(cy) < 8);
        if layer < 20 { corner_support } else { block(x, y, layer) }
    };
    let mut layers = layers_from(lifted_block);
    let mut settings = hollow(1.0);
    settings.drain_holes = 1;
    settings.drain_hole_diameter = 1.0;
    let report = settings.apply(&config(), &mut layers, &Progress::new());

    let hole = report.drain_holes[0];
    assert_eq!(hole.bottom_layer, 20, "The hole goes below the bottom of the block.");
    for (cx, cy) in [(110, 110), (290, 110), (110, 290), (290, 290)] {
        assert!(lit(&layers, cx, cy, 10), "The support at ({cx}, {cy}) is cut.");
    }
}
