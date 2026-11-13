//! The solid build plate under the models: a dark slab with the word FRONT at its front edge.

use nalgebra::{Vector2, Vector3};
use slicer::geometry::mesh::Mesh;
use tools::text_mesh::text_mesh;
use wgpu::Device;

use crate::{interface::theme::LABEL_FONT_FILE, render::util::MeshBuffers};

/// The thickness of the plate under the build volume, in millimeters.
pub const PLATE_THICKNESS: f32 = 4.0;
/// The height of the top of the plate, in millimeters. The top is a little under zero, so the grid lines stay in front of it.
pub const PLATE_TOP: f32 = -0.05;
/// The height of the letters of the front marker, in millimeters.
const LABEL_SIZE: f32 = 5.0;
/// The thickness of the letters of the front marker, in millimeters.
const LABEL_DEPTH: f32 = 0.2;
/// The distance from the front edge of the plate to the letters, in millimeters.
const LABEL_MARGIN: f32 = 3.0;

/// The GPU buffers of the plate for one platform size.
pub struct PlateBuffers {
    pub size: Vector2<f32>,
    pub slab: (MeshBuffers, u32),
    pub label: Option<(MeshBuffers, u32)>,
}

impl PlateBuffers {
    /// Make the buffers of the slab and the front marker for a platform of `size` millimeters.
    pub fn new(device: &Device, size: Vector2<f32>) -> Self {
        let slab = plate_mesh(size);
        let slab = (
            MeshBuffers::get_for(device, &slab),
            slab.face_count() as u32,
        );
        let label = front_label(size).map(|label| {
            (
                MeshBuffers::get_for(device, &label),
                label.face_count() as u32,
            )
        });
        Self { size, slab, label }
    }
}

/// Return the vertices and faces of the box between the corners `min` and `max`.
///
/// Each side has two triangles. The faces of each side turn counterclockwise
/// when the viewer looks at the outside of the box.
pub fn box_mesh(min: Vector3<f32>, max: Vector3<f32>) -> (Vec<Vector3<f32>>, Vec<[u32; 3]>) {
    let vertices = (0..8)
        .map(|corner| {
            Vector3::new(
                if corner & 1 == 0 { min.x } else { max.x },
                if corner & 2 == 0 { min.y } else { max.y },
                if corner & 4 == 0 { min.z } else { max.z },
            )
        })
        .collect();
    let faces = vec![
        [0, 2, 3],
        [0, 3, 1],
        [4, 5, 7],
        [4, 7, 6],
        [0, 1, 5],
        [0, 5, 4],
        [2, 6, 7],
        [2, 7, 3],
        [0, 4, 6],
        [0, 6, 2],
        [1, 3, 7],
        [1, 7, 5],
    ];
    (vertices, faces)
}

/// Return the mesh of the slab under a platform of `size` millimeters, centered on the origin.
pub fn plate_mesh(size: Vector2<f32>) -> Mesh {
    let half = size / 2.0;
    let (vertices, faces) = box_mesh(
        Vector3::new(-half.x, -half.y, PLATE_TOP - PLATE_THICKNESS),
        Vector3::new(half.x, half.y, PLATE_TOP),
    );
    Mesh::new(vertices, faces)
}

/// Return the mesh of the word FRONT, flat on the plate and centered near its front edge.
///
/// The front edge is the edge at negative Y, which faces the default
/// camera. The function returns `None` when the font gives no outline.
pub fn front_label(size: Vector2<f32>) -> Option<Mesh> {
    let (mut vertices, faces) =
        text_mesh(LABEL_FONT_FILE, "FRONT", LABEL_SIZE, LABEL_DEPTH).ok()?;
    if vertices.is_empty() {
        return None;
    }

    let (min, max) = vertices.iter().fold(
        (Vector3::repeat(f32::MAX), Vector3::repeat(f32::MIN)),
        |(min, max), vertex| (min.inf(vertex), max.sup(vertex)),
    );
    let target = Vector3::new(
        0.0,
        -size.y / 2.0 + LABEL_MARGIN + (max.y - min.y) / 2.0,
        PLATE_TOP,
    );
    let offset = target - Vector3::new((min.x + max.x) / 2.0, (min.y + max.y) / 2.0, min.z);
    vertices.iter_mut().for_each(|vertex| *vertex += offset);
    Some(Mesh::new(vertices, faces))
}
