use common::{container::Run, slice::Layer};
use nalgebra::Vector2;

/// The lit area of all layers: lowest and highest X, Y and layer index.
#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub low: Vector2<usize>,
    pub high: Vector2<usize>,
    pub first_layer: usize,
    pub last_layer: usize,
}

/// A coarse voxel grid over the lit area of all layers, with one voxel of padding on each side.
#[derive(Clone, Debug)]
pub struct VoxelGrid {
    pub origin: Vector2<usize>,
    pub first_layer: usize,
    /// Pixels per voxel along X and Y.
    pub step_xy: usize,
    /// Layers per voxel along Z.
    pub step_z: usize,
    pub size: [usize; 3],
}

/// Call `visit` with the row, the first column and the last column of each lit span of `runs`.
pub fn for_each_lit_span(runs: &[Run], width: usize, mut visit: impl FnMut(usize, usize, usize, u8)) {
    let mut position = 0usize;
    for run in runs {
        let end = position + run.length as usize;
        if run.value != 0 {
            let mut start = position;
            while start < end {
                let (row, column) = (start / width, start % width);
                let last = (end - 1).min(row * width + width - 1);
                visit(row, column, last % width, run.value);
                start = last + 1;
            }
        }
        position = end;
    }
}

pub fn lit_bounds(layers: &[Layer], width: usize) -> Option<Bounds> {
    let mut bounds: Option<Bounds> = None;
    for (index, layer) in layers.iter().enumerate() {
        for_each_lit_span(&layer.data, width, |row, first, last, _| {
            let span_low = Vector2::new(first, row);
            let span_high = Vector2::new(last, row);
            bounds = Some(match bounds {
                None => Bounds {
                    low: span_low,
                    high: span_high,
                    first_layer: index,
                    last_layer: index,
                },
                Some(b) => Bounds {
                    low: b.low.inf(&span_low),
                    high: b.high.sup(&span_high),
                    first_layer: b.first_layer.min(index),
                    last_layer: b.last_layer.max(index),
                },
            });
        });
    }
    bounds
}

impl VoxelGrid {
    /// Return an empty grid over `bounds` and the lit voxels of that grid.
    ///
    /// The grid has one unlit voxel of padding on each side. A voxel is lit
    /// if the pixel at its center is lit, in the layer at its center.
    pub fn sample(layers: &[Layer], width: usize, bounds: Bounds, step_xy: usize, step_z: usize) -> (Self, Vec<bool>) {
        let span = bounds.high - bounds.low + Vector2::repeat(1);
        let layer_count = bounds.last_layer - bounds.first_layer + 1;
        let size = [
            span.x.div_ceil(step_xy) + 2,
            span.y.div_ceil(step_xy) + 2,
            layer_count.div_ceil(step_z) + 2,
        ];
        let mut cells = vec![false; size[0] * size[1] * size[2]];

        for k in 1..size[2] - 1 {
            let layer = (bounds.first_layer + (k - 1) * step_z + step_z / 2).min(bounds.last_layer);
            for_each_lit_span(&layers[layer].data, width, |row, first, last, _| {
                if row < bounds.low.y || row > bounds.high.y {
                    return;
                }
                let local_row = row - bounds.low.y;
                let sample_row = ((local_row / step_xy) * step_xy + step_xy / 2).min(span.y - 1);
                if sample_row != local_row {
                    return;
                }
                let j = local_row / step_xy + 1;
                let (first, last) = (first - bounds.low.x, last - bounds.low.x);
                for i in (first / step_xy)..=(last / step_xy) {
                    let center = (i * step_xy + step_xy / 2).min(span.x - 1);
                    if center >= first && center <= last {
                        cells[(k * size[1] + j) * size[0] + i + 1] = true;
                    }
                }
            });
        }

        let grid = Self {
            origin: bounds.low,
            first_layer: bounds.first_layer,
            step_xy,
            step_z,
            size,
        };
        (grid, cells)
    }

    pub fn index(&self, i: usize, j: usize, k: usize) -> usize {
        (k * self.size[1] + j) * self.size[0] + i
    }

    /// Return the voxel that holds the pixel at column `x`, row `y` of `layer`, or `None` outside the grid.
    pub fn voxel_of(&self, x: usize, y: usize, layer: usize) -> Option<(usize, usize, usize)> {
        let (x, y) = (x.checked_sub(self.origin.x)?, y.checked_sub(self.origin.y)?);
        let z = layer.checked_sub(self.first_layer)?;
        let (i, j, k) = (x / self.step_xy + 1, y / self.step_xy + 1, z / self.step_z + 1);
        (i < self.size[0] - 1 && j < self.size[1] - 1 && k < self.size[2] - 1).then_some((i, j, k))
    }

}
