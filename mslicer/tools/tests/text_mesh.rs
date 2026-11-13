use std::{collections::HashMap, fs, path::Path};

use nalgebra::{Vector2, Vector3};
use tools::text_mesh::{signed_area, text_mesh, triangulate};

fn square(center: Vector2<f32>, half: f32, counterclockwise: bool) -> Vec<Vector2<f32>> {
    let mut points = vec![
        center + Vector2::new(-half, -half),
        center + Vector2::new(half, -half),
        center + Vector2::new(half, half),
        center + Vector2::new(-half, half),
    ];
    if !counterclockwise {
        points.reverse();
    }
    points
}

fn filled_area(points: &[Vector2<f32>], triangles: &[[u32; 3]]) -> f32 {
    triangles
        .iter()
        .map(|t| signed_area(&t.map(|i| points[i as usize])) / 2.0)
        .sum()
}

fn barlow() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../mslicer/assets/fonts/Barlow-SemiBold.ttf");
    fs::read(path).unwrap()
}

/// Return the number of edges that do not have exactly one partner edge in the opposite direction.
fn open_edges(vertices: &[Vector3<f32>], faces: &[[u32; 3]]) -> usize {
    let key = |v: &Vector3<f32>| [v.x, v.y, v.z].map(|c| (c * 10_000.0).round() as i64);
    let mut edges: HashMap<([i64; 3], [i64; 3]), i32> = HashMap::new();
    for face in faces {
        for k in 0..3 {
            let (a, b) = (key(&vertices[face[k] as usize]), key(&vertices[face[(k + 1) % 3] as usize]));
            *edges.entry((a, b)).or_default() += 1;
        }
    }
    edges
        .iter()
        .filter(|((a, b), count)| edges.get(&(*b, *a)).copied().unwrap_or(0) != **count)
        .count()
}

fn volume(vertices: &[Vector3<f32>], faces: &[[u32; 3]]) -> f32 {
    faces
        .iter()
        .map(|f| {
            let [a, b, c] = f.map(|i| vertices[i as usize]);
            a.dot(&b.cross(&c)) / 6.0
        })
        .sum()
}

#[test]
fn square_with_a_hole_fills_only_the_ring() {
    let outer = square(Vector2::zeros(), 5.0, true);
    let hole = square(Vector2::new(1.0, 0.5), 2.0, false);
    let (points, triangles) = triangulate(&outer, &[hole]);
    assert!((filled_area(&points, &triangles) - (100.0 - 16.0)).abs() < 1e-3);
    assert!(triangles.iter().all(|t| signed_area(&t.map(|i| points[i as usize])) > 0.0));
}

#[test]
fn concave_shape_fills_its_own_area() {
    let l_shape = vec![
        Vector2::new(0.0, 0.0),
        Vector2::new(6.0, 0.0),
        Vector2::new(6.0, 2.0),
        Vector2::new(2.0, 2.0),
        Vector2::new(2.0, 6.0),
        Vector2::new(0.0, 6.0),
    ];
    let (points, triangles) = triangulate(&l_shape, &[]);
    assert_eq!(triangles.len(), 4);
    assert!((filled_area(&points, &triangles) - 20.0).abs() < 1e-4);
}

#[test]
fn text_mesh_is_closed_and_points_out() {
    let (vertices, faces) = text_mesh(&barlow(), "O8B", 10.0, 1.5).unwrap();
    assert_eq!(open_edges(&vertices, &faces), 0, "The text mesh has open edges.");
    assert!(volume(&vertices, &faces) > 0.0, "The faces point in.");
}

#[test]
fn text_mesh_has_the_requested_depth_and_size() {
    let (vertices, _) = text_mesh(&barlow(), "Ember", 8.0, 2.0).unwrap();
    let (low, high) = vertices.iter().fold(
        (Vector3::repeat(f32::MAX), Vector3::repeat(f32::MIN)),
        |(low, high), v| (low.inf(v), high.sup(v)),
    );
    assert_eq!((low.z, high.z), (0.0, 2.0));
    let (width, height) = (high.x - low.x, high.y - low.y);
    assert!(height > 4.0 && height < 8.0, "The capital E is {height} mm high.");
    assert!(width > height * 2.0, "The word is only {width} mm wide.");
}

#[test]
fn space_and_unknown_text_give_an_empty_mesh() {
    let (vertices, faces) = text_mesh(&barlow(), " ", 8.0, 1.0).unwrap();
    assert!(vertices.is_empty() && faces.is_empty());
}
