use std::{collections::HashMap, f32::consts::PI, range::Range};

use common::{geometry::convex_hull, units::Milimeters};
use nalgebra::{Vector2, Vector3};
use slicer::geometry::mesh::{Mesh, MeshBuilder};

use crate::supports::{Support, SupportId};

pub type FaceMap = HashMap<SupportId, Range<u32>>;

/// The shape of new supports: the body, the tip that touches the model and the free space around the body.
#[derive(Clone, Debug, PartialEq)]
pub struct SupportPreset {
    pub radius: Milimeters,
    pub margin: Milimeters,

    pub tip_radius: Milimeters,
    pub tip_length: Milimeters,
}

/// The plate under the supports that sticks to the build plate.
///
/// The bottom of the raft is the convex hull of the support feet, made larger
/// by `offset`. The top is the hull itself, at `height` above the build
/// plate. The sloped edge makes the raft easy to remove with a scraper. A
/// `height` of 0 gives no raft.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Raft {
    pub height: f32,
    pub offset: f32,
}

impl SupportPreset {
    pub const LIGHT: Self = Self {
        radius: Milimeters::new(0.4),
        margin: Milimeters::new(0.5),
        tip_radius: Milimeters::new(0.15),
        tip_length: Milimeters::new(1.5),
    };

    pub const MEDIUM: Self = Self {
        radius: Milimeters::new(0.6),
        margin: Milimeters::new(0.5),
        tip_radius: Milimeters::new(0.2),
        tip_length: Milimeters::new(2.0),
    };

    pub const HEAVY: Self = Self {
        radius: Milimeters::new(0.8),
        margin: Milimeters::new(0.6),
        tip_radius: Milimeters::new(0.3),
        tip_length: Milimeters::new(2.5),
    };

    /// The built-in presets with their names, from light to heavy.
    pub const NAMED: [(&'static str, Self); 3] = [
        ("Light", Self::LIGHT),
        ("Medium", Self::MEDIUM),
        ("Heavy", Self::HEAVY),
    ];

    /// Return the name of the built-in preset that is equal to this preset, or `None` for custom values.
    pub fn name(&self) -> Option<&'static str> {
        Self::NAMED
            .iter()
            .find(|(_, preset)| preset == self)
            .map(|(name, _)| *name)
    }
}

impl Default for SupportPreset {
    fn default() -> Self {
        Self::MEDIUM
    }
}

impl Default for Raft {
    fn default() -> Self {
        Self {
            height: 1.0,
            offset: 1.0,
        }
    }
}

/// Return the mesh of `supports` and the faces of each support, or `None` if there are no supports.
///
/// The support points are in the frame of the mesh. The function extends
/// each support down to `plate_z`, the build plate in that frame. Each
/// support is a cone from the tip to the body, a sloped body and a vertical
/// column, with a sphere at each joint. The `resolution` is the number of
/// sides of each cylinder. If the raft has a height, the function adds the
/// raft under the support feet.
pub fn build_support_mesh<'a>(
    supports: impl Iterator<Item = &'a Support>,
    plate_z: f32,
    resolution: u32,
    raft: Raft,
) -> Option<(Mesh, FaceMap)> {
    let mut builder = MeshBuilder::new();
    let mut feet = Vec::new();
    let mut map = HashMap::new();
    let precision = resolution.max(3);

    for support in supports {
        let radius = support.radius;
        let [tip, body_top, body_bottom] = support.points;
        let foot = body_bottom.xy().push(plate_z);

        let start = builder.next_face_idx();
        builder.add_cylinder((tip, body_top), (support.tip_radius, radius), precision);
        builder.add_cylinder((body_top, body_bottom), (radius, radius), precision);
        builder.add_cylinder((body_bottom, foot), (radius, radius), precision);
        builder.add_sphere(tip, support.tip_radius, precision);
        builder.add_sphere(body_top, radius, precision);
        builder.add_sphere(body_bottom, radius, precision);
        map.insert(support.id, Range::from(start..builder.next_face_idx()));

        for i in 0..(precision * 2) {
            let angle = i as f32 / precision as f32 * PI;
            feet.push(foot.xy() + Vector2::new(angle.cos(), angle.sin()) * radius);
        }
    }

    build_raft(raft, plate_z, &feet, &mut builder);
    (!builder.is_empty()).then(|| (builder.build(), map))
}

/// Add a raft under the `points` to `builder`, from `base_z` up to `base_z + raft.height`.
///
/// The function makes a closed solid with outward faces: a bottom face, a top
/// face and one sloped side face for each edge of the convex hull. The
/// function adds nothing if the raft has no height or if there are fewer than
/// three hull points.
pub fn build_raft(raft: Raft, base_z: f32, points: &[Vector2<f32>], builder: &mut MeshBuilder) {
    let hull = convex_hull(points);
    if raft.height <= 0.0 || hull.len() < 3 {
        return;
    }

    let count = hull.len();
    let outward = |from: Vector2<f32>, to: Vector2<f32>| {
        let edge = to - from;
        Vector2::new(edge.y, -edge.x).normalize()
    };

    let first = builder.next_idx();
    for i in 0..count {
        let (previous, point, next) = (hull[(i + count - 1) % count], hull[i], hull[(i + 1) % count]);
        let direction = (outward(previous, point) + outward(point, next)).normalize();
        builder.add_vertex((point + direction * raft.offset).push(base_z));
        builder.add_vertex(point.push(base_z + raft.height));
    }

    let bottom = |i: usize| first + (i % count) as u32 * 2;
    let top = |i: usize| first + (i % count) as u32 * 2 + 1;
    for i in 1..count - 1 {
        builder.add_face([bottom(0), bottom(i + 1), bottom(i)]);
        builder.add_face([top(0), top(i), top(i + 1)]);
    }
    for i in 0..count {
        builder.add_face([bottom(i), bottom(i + 1), top(i + 1)]);
        builder.add_face([bottom(i), top(i + 1), top(i)]);
    }
}

/// Return the points of one support in world space, moved by `offset`.
pub fn offset_points(points: [Vector3<f32>; 3], offset: Vector3<f32>) -> [Vector3<f32>; 3] {
    points.map(|point| point + offset)
}
