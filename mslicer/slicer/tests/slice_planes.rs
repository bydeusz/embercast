use std::{fs, path::Path};

use common::{
    progress::Progress,
    serde::SliceDeserializer,
    slice::SliceConfig,
    units::{Milimeter, Milimeters},
};
use nalgebra::{Vector2, Vector3};
use slicer::{
    geometry::{bounds::Bounds3D, mesh::Mesh},
    slicer::{Slicer, SlicerModel},
};

const PIXELS_PER_MM: f32 = 10.0;

/// Return a closed cube mesh with sides of `size` millimeters.
///
/// The mesh constructor centers the cube on X and Y and puts its bottom at Z = 0.
fn cube_mesh(size: f32) -> Mesh {
    let h = size / 2.0;
    let vertices: Vec<Vector3<f32>> = [
        (-h, -h, -h),
        (h, -h, -h),
        (h, h, -h),
        (-h, h, -h),
        (-h, -h, h),
        (h, -h, h),
        (h, h, h),
        (-h, h, h),
    ]
    .into_iter()
    .map(|(x, y, z)| Vector3::new(x, y, z))
    .collect();
    let faces: Vec<[u32; 3]> = vec![
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [1, 2, 6],
        [1, 6, 5],
        [2, 3, 7],
        [2, 7, 6],
        [3, 0, 4],
        [3, 4, 7],
    ];
    Mesh::new_boxed_centered(vertices.into(), faces.into())
}

/// Return the teapot mesh from the app assets, with its bottom at Z = 0.
fn teapot_mesh() -> Mesh {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../mslicer/assets/teapot.stl");
    let data = fs::read(path).unwrap();
    let des = SliceDeserializer::new(&data);
    let mesh = mesh_format::load_mesh(des, mesh_format::Format::Stl, &Progress::new()).unwrap();
    Mesh::new_boxed_centered(mesh.verts, mesh.faces)
}

/// Return a slicer for one mesh that stands on the build plate in the middle of a 40 x 40 mm screen.
///
/// The app and the CLI move each model down by half a layer. Thus the slicer
/// cuts each layer at the middle of its height, and not at its bottom.
fn slicer_on_plate(mut mesh: Mesh) -> Slicer {
    let config = SliceConfig {
        platform_resolution: Vector2::new(400, 400),
        platform_size: Vector3::new(40.0, 40.0, 40.0).map(Milimeters::new),
        slice_height: Milimeters::new(0.05),
        ..Default::default()
    };
    let half_layer = config.slice_height.get::<Milimeter>() / 2.0;
    mesh.set_position(Vector3::new(200.0, 200.0, -half_layer));
    mesh.set_scale(Vector3::new(PIXELS_PER_MM, PIXELS_PER_MM, 1.0));
    let bounds = Bounds3D::from_mesh(&mesh);
    Slicer::new(
        config,
        vec![SlicerModel {
            mesh,
            bounds,
            exposure: 255,
        }],
    )
}

#[test]
fn cube_on_the_plate_has_one_layer_per_layer_height() {
    let layers = slicer_on_plate(cube_mesh(2.0)).slice_raster();
    assert_eq!(layers.len(), 40);
}

#[test]
fn first_layer_of_a_model_on_the_plate_has_pixels() {
    let layers = slicer_on_plate(cube_mesh(2.0)).slice_raster();
    assert!(layers[0].area > 0, "Layer 0 is empty.");
}

#[test]
fn every_layer_of_a_cube_has_the_same_area() {
    let layers = slicer_on_plate(cube_mesh(2.0)).slice_raster();
    let expected = (2.0 * PIXELS_PER_MM) as u64 * (2.0 * PIXELS_PER_MM) as u64;
    for (index, layer) in layers.iter().enumerate() {
        assert_eq!(layer.area, expected, "Layer {index} has the wrong area.");
    }
}

#[test]
fn first_layer_of_the_teapot_has_pixels() {
    let layers = slicer_on_plate(teapot_mesh()).slice_raster();
    assert!(layers[0].area > 0, "Layer 0 is empty.");
}
