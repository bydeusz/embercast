use std::{ops::RangeInclusive, sync::Arc, time::Instant};

use common::{
    container::Run,
    progress::Progress,
    serde::{Deserializer, Serializer},
    slice::{Layer, SliceConfig},
    units::Milimeter,
};
use nalgebra::Vector2;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::post_process::voxels::{VoxelGrid, for_each_lit_span, lit_bounds};

/// The largest voxel edge of the coarse grid, in millimeters.
const MAX_VOXEL_MM: f32 = 0.25;
/// The number of voxels across the wall that the coarse grid aims for.
const VOXELS_PER_WALL: f32 = 5.0;
/// A drain hole goes down while at least this part of its disk is lit.
const HOLE_LIT_FRACTION: f32 = 0.5;

/// Settings to make thick parts of the print hollow.
///
/// Every lit voxel that is farther than `wall_thickness` from the outside
/// becomes empty. Thin parts, such as supports and the raft, have no voxel
/// that far from the outside, so they stay solid. Optional pillars hold up
/// the roof of the cavity, and drain holes let the resin out.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hollow {
    pub enabled: bool,
    /// The wall thickness in millimeters.
    pub wall_thickness: f32,
    /// The distance between two pillars in millimeters. A value of 0 gives no pillars.
    pub pillar_spacing: f32,
    /// The width of one square pillar in millimeters.
    pub pillar_width: f32,
    /// The number of drain holes. A value of 0 gives no drain holes.
    pub drain_holes: u32,
    /// The diameter of one drain hole in millimeters.
    pub drain_hole_diameter: f32,
}

/// One vertical drain hole, in pixels and layer indices.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrainHole {
    pub center: Vector2<f32>,
    pub radius: f32,
    /// The highest layer of the hole, in the cavity.
    pub top_layer: usize,
    /// The lowest layer of the hole.
    pub bottom_layer: usize,
}

/// What the hollow step did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HollowReport {
    pub removed_pixels: u64,
    pub cavity_voxels: usize,
    pub drain_holes: Vec<DrainHole>,
}

/// The coarse voxel grid with the voxels that become the cavity.
struct Grid {
    voxels: VoxelGrid,
    cavity: Vec<bool>,
    /// True for each voxel layer that has at least one cavity voxel.
    cavity_slabs: Vec<bool>,
}

impl Hollow {
    pub fn post_slice(&self, config: &SliceConfig, layers: &mut [Layer], progress: Progress) {
        if !self.enabled {
            progress.set_finished();
            return;
        }
        let start = Instant::now();
        let report = self.apply(config, layers, &progress);
        info!(
            "Hollowed {} voxels, removed {} pixels, made {} drain holes in {:?}",
            report.cavity_voxels,
            report.removed_pixels,
            report.drain_holes.len(),
            start.elapsed()
        );
        progress.set_finished();
    }

    /// Make the layers hollow and return what the function did.
    ///
    /// First, the function finds the lit area of all layers and samples it on
    /// a coarse voxel grid. Then the function calculates for each lit voxel
    /// the distance to the nearest unlit voxel. Voxels farther than the wall
    /// thickness are the cavity. The function chooses the drain holes at the
    /// lowest layer of the cavity. Last, the function clears the cavity and
    /// the drain holes in each layer, and keeps the pillars.
    pub fn apply(&self, config: &SliceConfig, layers: &mut [Layer], progress: &Progress) -> HollowReport {
        let resolution = config.platform_resolution.cast::<usize>();
        let Some(bounds) = lit_bounds(layers, resolution.x) else {
            return HollowReport::default();
        };

        let pixel_mm = config.pixel_size().map(|x| x.get::<Milimeter>());
        let layer_mm = config.slice_height.get::<Milimeter>();
        let voxel_mm = (self.wall_thickness / VOXELS_PER_WALL).clamp(pixel_mm.x, MAX_VOXEL_MM.max(pixel_mm.x));
        let step_xy = ((voxel_mm / pixel_mm.x).round() as usize).max(1);
        let step_z = ((voxel_mm / layer_mm).round() as usize).max(1);
        let voxel = [step_xy as f32 * pixel_mm.x, step_xy as f32 * pixel_mm.y, step_z as f32 * layer_mm];

        let (voxels, lit) = VoxelGrid::sample(layers, resolution.x, bounds, step_xy, step_z);
        let distance = squared_distance_to_outside(&lit, voxels.size, voxel);
        let mut grid = Grid {
            voxels,
            cavity: Vec::new(),
            cavity_slabs: Vec::new(),
        };
        let largest_voxel = voxel.into_iter().fold(0.0, f32::max);
        let threshold = (self.wall_thickness + largest_voxel).powi(2);
        grid.set_cavity(distance.into_iter().map(|d| d > threshold).collect());
        let cavity_voxels = grid.cavity.iter().filter(|x| **x).count();
        if cavity_voxels == 0 {
            return HollowReport::default();
        }

        let radius = self.drain_hole_diameter / 2.0 / pixel_mm.x;
        let max_depth = ((self.wall_thickness * 2.0 + 1.0) / layer_mm).ceil() as usize;
        let drain_holes = (grid.drain_hole_centers(self.drain_holes as usize).into_iter())
            .map(|(center, top_layer)| {
                let bottom_layer = hole_bottom(layers, resolution.x, center, radius, top_layer, max_depth);
                DrainHole {
                    center,
                    radius,
                    top_layer,
                    bottom_layer,
                }
            })
            .collect::<Vec<_>>();

        let pillars = (self.pillar_spacing > 0.0).then(|| {
            let spacing = ((self.pillar_spacing / pixel_mm.x).round() as usize).max(2);
            let width = ((self.pillar_width / pixel_mm.x).round() as usize).clamp(1, spacing - 1);
            (spacing, width)
        });

        progress.set_total(layers.len() as u64);
        let removed = layers
            .par_iter_mut()
            .enumerate()
            .map(|(index, layer)| {
                let holes: Vec<&DrainHole> = (drain_holes.iter())
                    .filter(|hole| (hole.bottom_layer..=hole.top_layer).contains(&index))
                    .collect();
                let removed = grid.clear_layer(layer, index, resolution.x, pillars, &holes);
                progress.add_complete(1);
                removed
            })
            .sum();

        HollowReport {
            removed_pixels: removed,
            cavity_voxels,
            drain_holes,
        }
    }

    pub fn serialize<T: Serializer>(&self, ser: &mut T) {
        ser.write_bool(self.enabled);
        ser.write_f32_be(self.wall_thickness);
        ser.write_f32_be(self.pillar_spacing);
        ser.write_f32_be(self.pillar_width);
        ser.write_u32_be(self.drain_holes);
        ser.write_f32_be(self.drain_hole_diameter);
    }

    /// Read settings that [`Hollow::serialize`] wrote, and limit each value to a usable range.
    pub fn deserialize<T: Deserializer>(des: &mut T) -> Self {
        let defaults = Self::default();
        let enabled = des.read_bool();
        let wall_thickness = read_clamped(des, 0.2..=20.0, defaults.wall_thickness);
        let pillar_spacing = read_clamped(des, 0.0..=100.0, defaults.pillar_spacing);
        let pillar_width = read_clamped(des, 0.1..=20.0, defaults.pillar_width);
        let drain_holes = des.read_u32_be().min(16);
        let drain_hole_diameter = read_clamped(des, 0.5..=20.0, defaults.drain_hole_diameter);
        Self {
            enabled,
            wall_thickness,
            pillar_spacing,
            pillar_width,
            drain_holes,
            drain_hole_diameter,
        }
    }
}

/// Read one number and limit it to `range`, or return `fallback` if the number is not finite.
fn read_clamped<T: Deserializer>(des: &mut T, range: RangeInclusive<f32>, fallback: f32) -> f32 {
    let value = des.read_f32_be();
    if value.is_finite() {
        value.clamp(*range.start(), *range.end())
    } else {
        fallback
    }
}

impl Default for Hollow {
    fn default() -> Self {
        Self {
            enabled: false,
            wall_thickness: 2.0,
            pillar_spacing: 0.0,
            pillar_width: 1.0,
            drain_holes: 2,
            drain_hole_diameter: 2.5,
        }
    }
}

impl Grid {
    fn set_cavity(&mut self, cavity: Vec<bool>) {
        let slab = self.voxels.size[0] * self.voxels.size[1];
        self.cavity_slabs = cavity.chunks(slab).map(|cells| cells.iter().any(|x| *x)).collect();
        self.cavity = cavity;
    }

    fn layer_has_cavity(&self, layer: usize) -> bool {
        let Some(z) = layer.checked_sub(self.voxels.first_layer) else {
            return false;
        };
        self.cavity_slabs.get(z / self.voxels.step_z + 1).copied().unwrap_or(false)
    }

    fn is_cavity(&self, x: usize, y: usize, layer: usize) -> bool {
        self.voxels.voxel_of(x, y, layer)
            .is_some_and(|(i, j, k)| self.cavity[self.voxels.index(i, j, k)])
    }

    /// Return the pixel center and the top layer of up to `count` drain holes.
    ///
    /// The holes are in the lowest voxel layer of the cavity, because that
    /// layer is nearest to the build plate. The first hole is at the cavity
    /// voxel nearest to the middle of that layer. Each next hole is at the
    /// cavity voxel farthest from the holes before it.
    fn drain_hole_centers(&self, count: usize) -> Vec<(Vector2<f32>, usize)> {
        let Some(k) = (1..self.voxels.size[2] - 1).find(|&k| {
            (0..self.voxels.size[0] * self.voxels.size[1]).any(|cell| self.cavity[k * self.voxels.size[0] * self.voxels.size[1] + cell])
        }) else {
            return Vec::new();
        };

        let cells: Vec<Vector2<f32>> = (1..self.voxels.size[1] - 1)
            .flat_map(|j| (1..self.voxels.size[0] - 1).map(move |i| (i, j)))
            .filter(|&(i, j)| self.cavity[self.voxels.index(i, j, k)])
            .map(|(i, j)| Vector2::new(i as f32, j as f32))
            .collect();
        let middle = cells.iter().sum::<Vector2<f32>>() / cells.len() as f32;

        let mut chosen: Vec<Vector2<f32>> = Vec::new();
        for _ in 0..count.min(cells.len()) {
            let next = cells.iter().copied().max_by(|a, b| {
                let score = |cell: &Vector2<f32>| match chosen.is_empty() {
                    true => -(cell - middle).norm_squared(),
                    false => chosen.iter().map(|c| (cell - c).norm_squared()).fold(f32::MAX, f32::min),
                };
                score(a).total_cmp(&score(b))
            });
            chosen.extend(next);
        }

        let top_layer = self.voxels.first_layer + (k - 1) * self.voxels.step_z + self.voxels.step_z / 2;
        chosen
            .into_iter()
            .map(|cell| {
                let center = cell.map(|v| (v - 1.0) * self.voxels.step_xy as f32 + self.voxels.step_xy as f32 / 2.0);
                (center + self.voxels.origin.cast::<f32>(), top_layer)
            })
            .collect()
    }

    /// Clear the cavity and the `holes` in one layer, keep the pillars, and return the number of cleared pixels.
    fn clear_layer(
        &self,
        layer: &mut Layer,
        index: usize,
        width: usize,
        pillars: Option<(usize, usize)>,
        holes: &[&DrainHole],
    ) -> u64 {
        if holes.is_empty() && !self.layer_has_cavity(index) {
            return 0;
        }
        let in_pillar = |x: usize, y: usize| {
            pillars.is_some_and(|(spacing, size)| {
                (x - self.voxels.origin.x) % spacing < size && (y - self.voxels.origin.y) % spacing < size
            })
        };
        let in_hole = |x: usize, y: usize| {
            let point = Vector2::new(x as f32 + 0.5, y as f32 + 0.5);
            holes.iter().any(|hole| (point - hole.center).norm_squared() <= hole.radius * hole.radius)
        };

        let mut runs: Vec<Run> = Vec::with_capacity(layer.data.len());
        let mut push = |length: u64, value: u8| match runs.last_mut() {
            Some(run) if run.value == value => run.length += length,
            _ => runs.push(Run::new(length, value)),
        };

        let mut removed = 0;
        let mut position = 0usize;
        for run in layer.data.iter() {
            let end = position + run.length as usize;
            if run.value == 0 {
                push(run.length, 0);
            } else {
                for pixel in position..end {
                    let (x, y) = (pixel % width, pixel / width);
                    let cleared = in_hole(x, y) || (self.is_cavity(x, y, index) && !in_pillar(x, y));
                    removed += cleared as u64;
                    push(1, if cleared { 0 } else { run.value });
                }
            }
            position = end;
        }

        if removed > 0 {
            let mut new_layer = Layer::new_arc(Arc::new(runs), layer.height, layer.exposure.clone());
            new_layer.defects = layer.defects;
            new_layer.unique_exposure = layer.unique_exposure;
            *layer = new_layer;
        }
        removed
    }
}

/// Return the lowest layer of a drain hole that starts at `top_layer`.
///
/// The hole goes down layer by layer. The hole stops above the first layer
/// where less than half of its disk is lit, because that layer is outside the
/// model. The hole also stops after `max_depth` layers.
fn hole_bottom(
    layers: &[Layer],
    width: usize,
    center: Vector2<f32>,
    radius: f32,
    top_layer: usize,
    max_depth: usize,
) -> usize {
    let top_layer = top_layer.min(layers.len() - 1);
    let mut bottom = top_layer;
    for index in (top_layer.saturating_sub(max_depth)..=top_layer).rev() {
        let (lit, total) = disk_coverage(&layers[index], width, center, radius);
        if total > 0 && (lit as f32) < total as f32 * HOLE_LIT_FRACTION && index < top_layer {
            break;
        }
        bottom = index;
    }
    bottom
}

/// Return the number of lit pixels and the number of all pixels in the disk.
fn disk_coverage(layer: &Layer, width: usize, center: Vector2<f32>, radius: f32) -> (u64, u64) {
    let low_y = (center.y - radius).floor().max(0.0) as usize;
    let high_y = (center.y + radius).ceil() as usize;
    let in_disk = |x: usize, y: usize| {
        (Vector2::new(x as f32 + 0.5, y as f32 + 0.5) - center).norm_squared() <= radius * radius
    };

    let mut lit = 0;
    for_each_lit_span(&layer.data, width, |row, first, last, _| {
        if row < low_y || row > high_y {
            return;
        }
        lit += (first..=last).filter(|&x| in_disk(x, row)).count() as u64;
    });

    let low_x = (center.x - radius).floor().max(0.0) as usize;
    let high_x = (center.x + radius).ceil() as usize;
    let total = (low_y..=high_y)
        .flat_map(|y| (low_x..=high_x).map(move |x| (x, y)))
        .filter(|&(x, y)| in_disk(x, y))
        .count() as u64;
    (lit, total)
}

/// Return for each lit cell the squared distance in millimeters to the nearest unlit cell.
///
/// The function uses the separable exact Euclidean distance transform of
/// Felzenszwalb and Huttenlocher, one axis after the other, with the voxel
/// size of each axis.
fn squared_distance_to_outside(lit: &[bool], size: [usize; 3], voxel: [f32; 3]) -> Vec<f32> {
    let mut distance: Vec<f32> = lit.iter().map(|&l| if l { f32::INFINITY } else { 0.0 }).collect();
    let strides = [1, size[0], size[0] * size[1]];

    for axis in 0..3 {
        let length = size[axis];
        let (stride, weight) = (strides[axis], voxel[axis] * voxel[axis]);
        let mut line = vec![0.0; length];
        let mut output = vec![0.0; length];
        let starts: Vec<usize> = (0..lit.len()).filter(|i| (i / stride) % length == 0).collect();
        for start in starts {
            for (n, value) in line.iter_mut().enumerate() {
                *value = distance[start + n * stride];
            }
            distance_transform_1d(&line, weight, &mut output);
            for (n, value) in output.iter().enumerate() {
                distance[start + n * stride] = *value;
            }
        }
    }
    distance
}

/// Write to `output` the lower envelope of the parabolas `f[q] + weight * (p - q)²`.
fn distance_transform_1d(f: &[f32], weight: f32, output: &mut [f32]) {
    let n = f.len();
    let mut vertices = vec![0usize; n];
    let mut boundaries = vec![0.0f32; n + 1];
    let mut k = 0usize;
    let first = (0..n).find(|&q| f[q].is_finite());
    let Some(first) = first else {
        output.iter_mut().for_each(|value| *value = f32::INFINITY);
        return;
    };
    vertices[0] = first;
    boundaries[0] = f32::NEG_INFINITY;
    boundaries[1] = f32::INFINITY;

    let intersection = |q: usize, v: usize| {
        ((f[q] + weight * (q * q) as f32) - (f[v] + weight * (v * v) as f32)) / (2.0 * weight * (q as f32 - v as f32))
    };
    for q in (first + 1)..n {
        if !f[q].is_finite() {
            continue;
        }
        let mut s = intersection(q, vertices[k]);
        while s <= boundaries[k] {
            k -= 1;
            s = intersection(q, vertices[k]);
        }
        k += 1;
        vertices[k] = q;
        boundaries[k] = s;
        boundaries[k + 1] = f32::INFINITY;
    }

    k = 0;
    for (p, value) in output.iter_mut().enumerate() {
        while boundaries[k + 1] < p as f32 {
            k += 1;
        }
        let v = vertices[k];
        let offset = p as f32 - v as f32;
        *value = weight * offset * offset + f[v];
    }
}
