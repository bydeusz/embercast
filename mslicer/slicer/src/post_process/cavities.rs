use std::collections::VecDeque;

use common::{
    container::Run,
    progress::Progress,
    slice::{Layer, SliceConfig},
    units::Milimeter,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::post_process::voxels::{VoxelGrid, for_each_lit_span, lit_bounds};

/// The voxel edge that the detection aims for, in millimeters.
const TARGET_VOXEL_MM: f32 = 0.1;
/// The largest number of voxels in the grid. The voxels get larger for big prints.
const MAX_VOXELS: usize = 40_000_000;
/// Cavities with less volume than this, in cubic millimeters, are not reported.
const MIN_VOLUME_MM3: f32 = 0.2;

const EMPTY: u8 = 0;
const SOLID: u8 = 1;
const FROM_SIDE: u8 = 2;
const FROM_TOP: u8 = 3;

/// The kind of closed space in a print.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CavityKind {
    /// Empty space with no way out. Resin stays inside the print.
    ResinTrap,
    /// Empty space that only opens toward the higher layers. Each lift pulls a vacuum in it.
    SuctionCup,
}

/// One connected closed space.
#[derive(Clone, Debug, PartialEq)]
pub struct Cavity {
    pub kind: CavityKind,
    pub first_layer: usize,
    pub last_layer: usize,
    pub volume_mm3: f32,
}

/// The cavities of a print and the voxels that belong to them.
pub struct CavityReport {
    pub cavities: Vec<Cavity>,
    grid: Option<VoxelGrid>,
    marks: Vec<Option<CavityKind>>,
}

impl CavityKind {
    /// Return the value that [`CavityReport::layer_marks`] uses for this kind.
    pub fn mark(self) -> u8 {
        match self {
            CavityKind::ResinTrap => 1,
            CavityKind::SuctionCup => 2,
        }
    }
}

/// Find the resin traps and suction cups in `layers`.
///
/// The function samples the layers on a voxel grid of about 0.1 mm. Then the
/// function fills the empty voxels that the sides of the grid can reach, and
/// after that the empty voxels that the top of the grid can reach. The
/// build plate closes the bottom if the print starts at layer 0. Empty voxels
/// that no fill reaches are resin traps. Empty voxels that only the fill from
/// the top reaches are suction cups. The top layer of the grid stands for the
/// vat floor, so the fill from the sides does not pass through it. Last, the function groups the voxels of
/// each kind into connected cavities.
pub fn detect_cavities(config: &SliceConfig, layers: &[Layer], progress: &Progress) -> CavityReport {
    let width = config.platform_resolution.x as usize;
    let empty_report = || CavityReport {
        cavities: Vec::new(),
        grid: None,
        marks: Vec::new(),
    };
    let Some(bounds) = lit_bounds(layers, width) else {
        return empty_report();
    };

    let pixel_mm = config.pixel_size().map(|x| x.get::<Milimeter>());
    let layer_mm = config.slice_height.get::<Milimeter>();
    let span = bounds.high - bounds.low;
    let layer_count = bounds.last_layer - bounds.first_layer + 1;
    let mut step_xy = ((TARGET_VOXEL_MM / pixel_mm.x).round() as usize).max(1);
    let step_z = |step_xy: usize| ((step_xy as f32 * pixel_mm.x / layer_mm).round() as usize).max(1);
    let voxel_count = |step_xy: usize| {
        (span.x / step_xy + 3) * (span.y / step_xy + 3) * (layer_count / step_z(step_xy) + 3)
    };
    while voxel_count(step_xy) > MAX_VOXELS {
        step_xy += 1;
    }

    let (grid, lit) = VoxelGrid::sample(layers, width, bounds, step_xy, step_z(step_xy));
    progress.set_total(3);
    let mut state: Vec<u8> = lit.iter().map(|&l| if l { SOLID } else { EMPTY }).collect();
    let [nx, ny, nz] = grid.size;
    if bounds.first_layer == 0 {
        state[..nx * ny].fill(SOLID);
    }

    let on_side = |i: usize, j: usize| i == 0 || j == 0 || i == nx - 1 || j == ny - 1;
    let top_slab = (nz - 1) * nx * ny..nz * nx * ny;
    state[top_slab.clone()].fill(SOLID);
    let side_seeds = (0..nz - 1)
        .flat_map(|k| (0..ny).flat_map(move |j| (0..nx).map(move |i| (i, j, k))))
        .filter(|&(i, j, k)| on_side(i, j) || (k == 0 && bounds.first_layer > 0))
        .map(|(i, j, k)| grid.index(i, j, k));
    flood(&mut state, grid.size, side_seeds.collect(), FROM_SIDE);
    progress.add_complete(1);
    state[top_slab.clone()].fill(EMPTY);
    let top_seeds = top_slab.clone().collect();
    flood(&mut state, grid.size, top_seeds, FROM_TOP);
    progress.add_complete(1);

    let voxel_mm3 = step_xy as f32 * pixel_mm.x * step_xy as f32 * pixel_mm.y * step_z(step_xy) as f32 * layer_mm;
    let mut marks = vec![None; state.len()];
    let mut cavities = Vec::new();
    state[top_slab].fill(SOLID);
    for start in 0..state.len() {
        let kind = match state[start] {
            EMPTY => CavityKind::ResinTrap,
            FROM_TOP => CavityKind::SuctionCup,
            _ => continue,
        };
        let value = state[start];
        let cells = component(&mut state, grid.size, start, value);
        let volume_mm3 = cells.len() as f32 * voxel_mm3;
        if volume_mm3 < MIN_VOLUME_MM3 {
            continue;
        }
        let slab = |cell: usize| cell / (nx * ny);
        let (low, high) = cells.iter().fold((usize::MAX, 0), |(low, high), &cell| {
            (low.min(slab(cell)), high.max(slab(cell)))
        });
        cells.iter().for_each(|&cell| marks[cell] = Some(kind));
        cavities.push(Cavity {
            kind,
            first_layer: grid.first_layer + (low - 1) * grid.step_z,
            last_layer: (grid.first_layer + high * grid.step_z - 1).min(bounds.last_layer),
            volume_mm3,
        });
    }
    progress.add_complete(1);

    CavityReport {
        cavities,
        grid: Some(grid),
        marks,
    }
}

/// Fill all `EMPTY` cells that `seeds` reach through `EMPTY` cells with `value`.
fn flood(state: &mut [u8], size: [usize; 3], seeds: Vec<usize>, value: u8) {
    let mut queue: VecDeque<usize> = seeds.into_iter().filter(|&cell| state[cell] == EMPTY).collect();
    queue.iter().for_each(|&cell| state[cell] = value);
    while let Some(cell) = queue.pop_front() {
        for next in neighbours(cell, size) {
            if state[next] == EMPTY {
                state[next] = value;
                queue.push_back(next);
            }
        }
    }
}

/// Return the cells with `value` that connect to `start`, and mark them as solid so that the search sees them once.
fn component(state: &mut [u8], size: [usize; 3], start: usize, value: u8) -> Vec<usize> {
    let mut cells = vec![start];
    state[start] = SOLID;
    let mut index = 0;
    while index < cells.len() {
        for next in neighbours(cells[index], size) {
            if state[next] == value {
                state[next] = SOLID;
                cells.push(next);
            }
        }
        index += 1;
    }
    cells
}

/// Return the up to six cells that share a face with `cell`.
fn neighbours(cell: usize, [nx, ny, nz]: [usize; 3]) -> impl Iterator<Item = usize> {
    let (i, j, k) = (cell % nx, (cell / nx) % ny, cell / (nx * ny));
    [
        (i > 0).then(|| cell - 1),
        (i + 1 < nx).then(|| cell + 1),
        (j > 0).then(|| cell - nx),
        (j + 1 < ny).then(|| cell + nx),
        (k > 0).then(|| cell - nx * ny),
        (k + 1 < nz).then(|| cell + nx * ny),
    ]
    .into_iter()
    .flatten()
}

impl CavityReport {
    pub fn is_empty(&self) -> bool {
        self.cavities.is_empty()
    }

    /// Return the marks of one layer as runs over the whole screen, or `None` if the layer has no cavity.
    ///
    /// An unlit pixel in a cavity voxel gets the value of [`CavityKind::mark`].
    /// All other pixels get 0.
    pub fn layer_marks(&self, layer: &Layer, index: usize, config: &SliceConfig) -> Option<Vec<Run<u8>>> {
        let grid = self.grid.as_ref()?;
        let (width, height) = (
            config.platform_resolution.x as usize,
            config.platform_resolution.y as usize,
        );
        let k = (index.checked_sub(grid.first_layer)? / grid.step_z) + 1;
        let [nx, ny, nz] = grid.size;
        if k >= nz - 1 || !(0..nx * ny).any(|cell| self.marks[k * nx * ny + cell].is_some()) {
            return None;
        }

        let mut lit_rows: Vec<Vec<(usize, usize)>> = vec![Vec::new(); height];
        for_each_lit_span(&layer.data, width, |row, first, last, _| lit_rows[row].push((first, last)));

        let mut runs: Vec<Run<u8>> = Vec::new();
        let mut push = |length: u64, value: u8| match runs.last_mut() {
            Some(run) if run.value == value => run.length += length,
            _ => runs.push(Run::new(length, value)),
        };
        let (x0, y0) = (grid.origin.x, grid.origin.y);
        let x1 = (x0 + (nx - 2) * grid.step_xy).min(width);
        let y1 = (y0 + (ny - 2) * grid.step_xy).min(height);
        for (y, spans) in lit_rows.iter().enumerate() {
            if !(y0..y1).contains(&y) {
                push(width as u64, 0);
                continue;
            }
            push(x0 as u64, 0);
            let mut span = spans.iter().peekable();
            for x in x0..x1 {
                while span.next_if(|&&(_, last)| last < x).is_some() {}
                let is_lit = span.peek().is_some_and(|&&(first, _)| first <= x);
                let mark = (!is_lit)
                    .then(|| grid.voxel_of(x, y, index))
                    .flatten()
                    .and_then(|(i, j, k)| self.marks[grid.index(i, j, k)])
                    .map_or(0, CavityKind::mark);
                push(1, mark);
            }
            push((width - x1) as u64, 0);
        }
        runs.retain(|run| run.length > 0);
        Some(runs)
    }

    /// Return the marks of every layer that has a cavity, as pairs of layer index and runs.
    pub fn all_layer_marks(&self, layers: &[Layer], config: &SliceConfig) -> Vec<(usize, Vec<Run<u8>>)> {
        (0..layers.len())
            .into_par_iter()
            .filter_map(|index| self.layer_marks(&layers[index], index, config).map(|runs| (index, runs)))
            .collect()
    }
}
