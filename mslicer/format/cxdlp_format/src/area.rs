/// One horizontal row of lit pixels, from `x_start` to `x_end` inclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub y: u32,
    pub x_start: u32,
    pub x_end: u32,
}

impl Span {
    fn length(&self) -> u64 {
        (self.x_end - self.x_start + 1) as u64
    }

    fn touches(&self, other: &Span, reach: u32) -> bool {
        self.x_start <= other.x_end + reach && other.x_start <= self.x_end + reach
    }
}

struct DisjointSet {
    parent: Vec<usize>,
}

impl DisjointSet {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
        }
    }

    fn find(&mut self, mut index: usize) -> usize {
        while self.parent[index] != index {
            self.parent[index] = self.parent[self.parent[index]];
            index = self.parent[index];
        }
        index
    }

    fn union(&mut self, a: usize, b: usize) {
        let (root_a, root_b) = (self.find(a), self.find(b));
        if root_a != root_b {
            self.parent[root_b] = root_a;
        }
    }
}

/// Return the pixel count of the largest lit shape, with its holes filled.
///
/// The function gives the same value as the area inside the outer contour of
/// the largest shape. The `spans` must be in row order, sorted by `x_start`,
/// without overlap, and with a gap of at least one pixel between two spans of
/// one row. First, the function finds the unlit regions that connect to the
/// edge of the shapes. All other unlit pixels are holes and the function fills
/// them. Then the function joins the filled spans into shapes with
/// 8-connectivity and returns the pixel count of the largest shape.
pub fn largest_enclosed_area(width: u32, spans: &[Span]) -> u64 {
    let (Some(first), Some(last)) = (spans.first(), spans.last()) else {
        return 0;
    };
    let lit_rows = group_rows(spans, first.y, last.y);

    let unlit_rows: Vec<Vec<Span>> = lit_rows
        .iter()
        .enumerate()
        .map(|(row, lit)| complement(lit, first.y + row as u32, width))
        .collect();
    let mut unlit_sets = connect_rows(&unlit_rows, 0);
    let unlit_offsets = row_offsets(&unlit_rows);
    let last_row = unlit_rows.len() - 1;

    let mut outside = vec![false; unlit_offsets[unlit_rows.len()]];
    for (row, unlit) in unlit_rows.iter().enumerate() {
        for (index, span) in unlit.iter().enumerate() {
            let on_edge = row == 0 || row == last_row || span.x_start == 0 || span.x_end + 1 == width;
            if on_edge {
                let root = unlit_sets.find(unlit_offsets[row] + index);
                outside[root] = true;
            }
        }
    }

    let filled_rows: Vec<Vec<Span>> = lit_rows
        .iter()
        .zip(&unlit_rows)
        .enumerate()
        .map(|(row, (lit, unlit))| {
            let holes = unlit.iter().enumerate().filter_map(|(index, span)| {
                let root = unlit_sets.find(unlit_offsets[row] + index);
                (!outside[root]).then_some(*span)
            });
            let holes: Vec<Span> = holes.collect();
            merge_sorted(lit, &holes)
        })
        .collect();

    let mut filled_sets = connect_rows(&filled_rows, 1);
    let filled_offsets = row_offsets(&filled_rows);
    let mut totals = vec![0u64; filled_offsets[filled_rows.len()]];
    for (row, filled) in filled_rows.iter().enumerate() {
        for (index, span) in filled.iter().enumerate() {
            let root = filled_sets.find(filled_offsets[row] + index);
            totals[root] += span.length();
        }
    }
    totals.into_iter().max().unwrap_or(0)
}

fn group_rows(spans: &[Span], y_min: u32, y_max: u32) -> Vec<Vec<Span>> {
    let mut rows = vec![Vec::new(); (y_max - y_min + 1) as usize];
    for span in spans {
        rows[(span.y - y_min) as usize].push(*span);
    }
    rows
}

fn complement(lit: &[Span], y: u32, width: u32) -> Vec<Span> {
    let mut unlit = Vec::with_capacity(lit.len() + 1);
    let mut x = 0;
    for span in lit {
        if span.x_start > x {
            unlit.push(Span {
                y,
                x_start: x,
                x_end: span.x_start - 1,
            });
        }
        x = span.x_end + 1;
    }
    if x < width {
        unlit.push(Span {
            y,
            x_start: x,
            x_end: width - 1,
        });
    }
    unlit
}

fn merge_sorted(lit: &[Span], holes: &[Span]) -> Vec<Span> {
    let mut all: Vec<Span> = lit.iter().chain(holes).copied().collect();
    all.sort_unstable_by_key(|span| span.x_start);

    let mut merged: Vec<Span> = Vec::with_capacity(all.len());
    for span in all {
        match merged.last_mut() {
            Some(previous) if previous.x_end + 1 >= span.x_start => {
                previous.x_end = previous.x_end.max(span.x_end);
            }
            _ => merged.push(span),
        }
    }
    merged
}

fn row_offsets(rows: &[Vec<Span>]) -> Vec<usize> {
    let mut offsets = Vec::with_capacity(rows.len() + 1);
    let mut total = 0;
    offsets.push(0);
    for row in rows {
        total += row.len();
        offsets.push(total);
    }
    offsets
}

fn connect_rows(rows: &[Vec<Span>], reach: u32) -> DisjointSet {
    let offsets = row_offsets(rows);
    let mut sets = DisjointSet::new(offsets[rows.len()]);

    for row in 1..rows.len() {
        let (above, below) = (&rows[row - 1], &rows[row]);
        let (mut i, mut j) = (0, 0);
        while i < above.len() && j < below.len() {
            if above[i].touches(&below[j], reach) {
                sets.union(offsets[row - 1] + i, offsets[row] + j);
            }
            if above[i].x_end < below[j].x_end {
                i += 1;
            } else {
                j += 1;
            }
        }
    }
    sets
}
