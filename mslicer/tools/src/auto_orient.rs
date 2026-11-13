use std::f32::consts::PI;

use nalgebra::{Rotation3, Vector3};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

/// The number of evenly spread directions that the search tries.
const SPHERE_DIRECTIONS: usize = 256;
/// The number of largest faces whose normals the search also tries.
const LARGEST_FACES: usize = 32;

/// The settings of the orientation search.
#[derive(Clone, Debug, PartialEq)]
pub struct OrientConfig {
    /// A face needs supports if its normal is at most this angle from straight down, in degrees.
    pub overhang_angle: f32,
    /// Faces this close to the lowest point, in millimeters, rest on the build plate.
    pub base_tolerance: f32,
    /// The cost of one millimeter of height, in square millimeters of supported area.
    pub height_weight: f32,
    /// If False, the search only measures the current orientation.
    pub allow_rotation: bool,
}

/// One orientation of the model and what it costs.
#[derive(Clone, Debug, PartialEq)]
pub struct Orientation {
    /// The Euler angles (roll, pitch, yaw) in radians, as the mesh rotation uses them.
    pub rotation: Vector3<f32>,
    /// The area of the faces that need supports, in square millimeters.
    pub supported_area: f32,
    /// The height of the model, in millimeters.
    pub height: f32,
    pub score: f32,
}

impl Default for OrientConfig {
    fn default() -> Self {
        Self {
            overhang_angle: 30.0,
            base_tolerance: 0.2,
            height_weight: 2.0,
            allow_rotation: true,
        }
    }
}

/// Return the orientation that needs the least supports, with the lowest height as the second goal.
///
/// The mesh is in its own frame, and `scale` is its uniform scale. The search
/// tries evenly spread directions and the normals of the largest faces. For
/// each direction, the function turns that direction straight down. Then the
/// function adds the area of every face that points down within the overhang
/// angle, except the faces that rest on the build plate. The score is that
/// area plus the height times the height weight.
pub fn best_orientation(
    vertices: &[Vector3<f32>],
    faces: &[[u32; 3]],
    scale: f32,
    config: &OrientConfig,
) -> Orientation {
    let triangles: Vec<[Vector3<f32>; 3]> = faces
        .iter()
        .map(|face| face.map(|i| vertices[i as usize] * scale))
        .collect();

    let candidates = if config.allow_rotation {
        candidate_directions(&triangles)
    } else {
        vec![-Vector3::z()]
    };

    candidates
        .into_par_iter()
        .map(|down| evaluate(&triangles, down, config))
        .min_by(|a, b| a.score.total_cmp(&b.score))
        .expect("There is always at least one candidate direction.")
}

fn candidate_directions(triangles: &[[Vector3<f32>; 3]]) -> Vec<Vector3<f32>> {
    let golden_angle = PI * (3.0 - 5f32.sqrt());
    let mut directions: Vec<Vector3<f32>> = (0..SPHERE_DIRECTIONS)
        .map(|i| {
            let z = 1.0 - 2.0 * (i as f32 + 0.5) / SPHERE_DIRECTIONS as f32;
            let radius = (1.0 - z * z).sqrt();
            let angle = golden_angle * i as f32;
            Vector3::new(radius * angle.cos(), radius * angle.sin(), z)
        })
        .collect();
    directions.extend([Vector3::x(), -Vector3::x(), Vector3::y(), -Vector3::y(), Vector3::z(), -Vector3::z()]);

    let mut by_area: Vec<(f32, Vector3<f32>)> = triangles
        .iter()
        .filter_map(|[a, b, c]| {
            let cross = (b - a).cross(&(c - a));
            let area = cross.norm() / 2.0;
            (area > 0.0).then(|| (area, cross / (2.0 * area)))
        })
        .collect();
    by_area.sort_by(|a, b| b.0.total_cmp(&a.0));
    directions.extend(by_area.iter().take(LARGEST_FACES).map(|(_, normal)| *normal));
    directions
}

/// Return the cost of the orientation that turns `down` in the mesh frame straight down.
fn evaluate(triangles: &[[Vector3<f32>; 3]], down: Vector3<f32>, config: &OrientConfig) -> Orientation {
    let rotation = Rotation3::rotation_between(&down, &-Vector3::z())
        .unwrap_or_else(|| Rotation3::from_axis_angle(&Vector3::x_axis(), PI));
    let up = -down;
    let heights = triangles.iter().flatten().map(|v| v.dot(&up));
    let (low, high) = heights.fold((f32::MAX, f32::MIN), |(low, high), h| (low.min(h), high.max(h)));
    let cos_limit = config.overhang_angle.to_radians().cos();

    let supported_area = triangles
        .iter()
        .filter_map(|[a, b, c]| {
            let cross = (b - a).cross(&(c - a));
            let area = cross.norm() / 2.0;
            if area == 0.0 {
                return None;
            }
            let facing_down = (cross / (2.0 * area)).dot(&down);
            let face_low = a.dot(&up).min(b.dot(&up)).min(c.dot(&up));
            let face_high = a.dot(&up).max(b.dot(&up)).max(c.dot(&up));
            let on_plate = face_high - low <= config.base_tolerance && face_low - low <= config.base_tolerance;
            (facing_down >= cos_limit && !on_plate).then_some(area)
        })
        .sum::<f32>();

    let height = high - low;
    let (roll, pitch, yaw) = rotation.euler_angles();
    Orientation {
        rotation: Vector3::new(roll, pitch, yaw),
        supported_area,
        height,
        score: supported_area + height * config.height_weight,
    }
}
