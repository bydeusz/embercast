use nalgebra::{Vector2, Vector3};

use crate::render::workspace::model::plate::{
    PLATE_THICKNESS, PLATE_TOP, box_mesh, front_label, plate_mesh,
};

/// The build plate of the Creality Halot One Plus, in millimeters.
const HALOT_PLATE: Vector2<f32> = Vector2::new(172.8, 102.4);

/// Return the lowest and the highest corner of `vertices`.
fn bounds(vertices: &[Vector3<f32>]) -> (Vector3<f32>, Vector3<f32>) {
    vertices.iter().fold(
        (Vector3::repeat(f32::MAX), Vector3::repeat(f32::MIN)),
        |(min, max), vertex| (min.inf(vertex), max.sup(vertex)),
    )
}

#[test]
fn every_face_of_a_box_points_outward() {
    let (min, max) = (Vector3::new(-1.0, -2.0, -3.0), Vector3::new(4.0, 5.0, 6.0));
    let (vertices, faces) = box_mesh(min, max);
    let center = (min + max) / 2.0;

    assert_eq!(faces.len(), 12);
    for [a, b, c] in faces {
        let [a, b, c] = [a, b, c].map(|index| vertices[index as usize]);
        let normal = (b - a).cross(&(c - a));
        let face_center = (a + b + c) / 3.0;
        assert!(
            normal.dot(&(face_center - center)) > 0.0,
            "A face points into the box."
        );
    }
}

#[test]
fn the_plate_covers_the_platform_and_its_top_is_just_under_zero() {
    let mesh = plate_mesh(HALOT_PLATE);
    let (min, max) = bounds(mesh.vertices());

    assert!((max.x - min.x - HALOT_PLATE.x).abs() < 1e-3);
    assert!((max.y - min.y - HALOT_PLATE.y).abs() < 1e-3);
    assert!((max.z - PLATE_TOP).abs() < 1e-6);
    assert!((max.z - min.z - PLATE_THICKNESS).abs() < 1e-4);
    assert!(
        max.z < 0.0,
        "The top of the plate hides the grid lines at zero."
    );
}

#[test]
fn the_front_marker_lies_on_the_plate_near_the_front_edge() {
    let label = front_label(HALOT_PLATE).expect("The font has outlines for FRONT.");
    let (min, max) = bounds(label.vertices());

    assert!(
        (min.x + max.x).abs() < 1e-3,
        "The marker is not centered from left to right."
    );
    assert!(
        min.y > -HALOT_PLATE.y / 2.0,
        "The marker hangs over the front edge."
    );
    assert!(
        max.y < 0.0,
        "The marker is not in the front half of the plate."
    );
    assert!(
        (min.z - PLATE_TOP).abs() < 1e-4,
        "The marker does not rest on the plate."
    );
}
