use nalgebra::{Rotation3, Vector3};
use tools::auto_orient::{OrientConfig, best_orientation};

/// Return the vertices and faces of closed boxes, each given by its lowest and highest corner.
fn boxes(list: &[(Vector3<f32>, Vector3<f32>)]) -> (Vec<Vector3<f32>>, Vec<[u32; 3]>) {
    let corner_faces: [[u32; 3]; 12] = [
        [0, 2, 1], [0, 3, 2], [4, 5, 6], [4, 6, 7], [0, 1, 5], [0, 5, 4],
        [1, 2, 6], [1, 6, 5], [2, 3, 7], [2, 7, 6], [3, 0, 4], [3, 4, 7],
    ];
    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    for (low, high) in list {
        let first = vertices.len() as u32;
        for (x, y, z) in [
            (low.x, low.y, low.z), (high.x, low.y, low.z), (high.x, high.y, low.z), (low.x, high.y, low.z),
            (low.x, low.y, high.z), (high.x, low.y, high.z), (high.x, high.y, high.z), (low.x, high.y, high.z),
        ] {
            vertices.push(Vector3::new(x, y, z));
        }
        faces.extend(corner_faces.map(|face| face.map(|i| i + first)));
    }
    (vertices, faces)
}

/// Return the height of the vertices after the rotation with the given Euler angles.
fn height_after(vertices: &[Vector3<f32>], rotation: Vector3<f32>) -> f32 {
    let matrix = Rotation3::from_euler_angles(rotation.x, rotation.y, rotation.z);
    let heights: Vec<f32> = vertices.iter().map(|v| (matrix * v).z).collect();
    heights.iter().cloned().fold(f32::MIN, f32::max) - heights.iter().cloned().fold(f32::MAX, f32::min)
}

#[test]
fn standing_plate_is_laid_flat() {
    let (vertices, faces) = boxes(&[(Vector3::new(0.0, 0.0, 0.0), Vector3::new(30.0, 2.0, 20.0))]);
    let best = best_orientation(&vertices, &faces, 1.0, &OrientConfig::default());
    assert!(best.supported_area < 1.0, "A flat plate still needs {} mm² of supports.", best.supported_area);
    assert!((height_after(&vertices, best.rotation) - 2.0).abs() < 0.05, "The plate does not lie flat.");
}

#[test]
fn mushroom_stands_on_its_cap() {
    let (vertices, faces) = boxes(&[
        (Vector3::new(8.0, 8.0, 0.0), Vector3::new(12.0, 12.0, 10.0)),
        (Vector3::new(0.0, 0.0, 10.0), Vector3::new(20.0, 20.0, 12.0)),
    ]);
    let config = OrientConfig::default();
    let best = best_orientation(&vertices, &faces, 1.0, &config);
    assert!(best.supported_area < 50.0, "The cap is not down: {} mm² needs supports.", best.supported_area);
}

#[test]
fn scale_grows_the_supported_area_with_its_square() {
    let (vertices, faces) = boxes(&[
        (Vector3::new(8.0, 8.0, 0.0), Vector3::new(12.0, 12.0, 10.0)),
        (Vector3::new(0.0, 0.0, 10.0), Vector3::new(20.0, 20.0, 12.0)),
    ]);
    let config = OrientConfig { allow_rotation: false, ..Default::default() };
    let small = best_orientation(&vertices, &faces, 1.0, &config);
    let large = best_orientation(&vertices, &faces, 2.0, &config);
    assert!((large.supported_area - small.supported_area * 4.0).abs() < 1.0);
}
