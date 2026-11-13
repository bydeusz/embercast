use common::{
    container::Run,
    slice::{EncodableLayer, ExposureConfig, SliceConfig},
    units::{Milimeter, Milimeters},
};
use nalgebra::Vector2;

use crate::{Layer, Line, MAX_HEIGHT, MAX_WIDTH, Span, largest_enclosed_area};

#[derive(Clone, Copy, Default)]
struct OpenLine {
    y_start: u16,
    y_end: u16,
    gray: u8,
}

/// Turn the row ordered runs of one layer into vertical lines.
///
/// The slicer gives the pixels row by row. The format stores them column by
/// column. The encoder keeps one open line for each column. A lit pixel
/// extends the open line of its column if the gray value is the same and the
/// pixel is directly below the line. Otherwise the encoder closes the open
/// line and starts a new line.
pub struct LayerEncoder {
    width: u32,
    height: u32,
    position: u64,
    columns: Vec<OpenLine>,
    lines: Vec<Line>,
    spans: Vec<Span>,
}

impl LayerEncoder {
    fn extend_column(&mut self, x: u32, y: u32, gray: u8) {
        let open = self.columns[x as usize];
        if open.gray == gray && open.y_end as u32 + 1 == y {
            self.columns[x as usize].y_end = y as u16;
            return;
        }
        if open.gray != 0 {
            self.lines.push(Line {
                x: x as u16,
                y_start: open.y_start,
                y_end: open.y_end,
                gray: open.gray,
            });
        }
        self.columns[x as usize] = OpenLine {
            y_start: y as u16,
            y_end: y as u16,
            gray,
        };
    }

    fn add_span(&mut self, y: u32, x_start: u32, x_end: u32) {
        match self.spans.last_mut() {
            Some(previous) if previous.y == y && previous.x_end + 1 == x_start => {
                previous.x_end = x_end;
            }
            _ => self.spans.push(Span { y, x_start, x_end }),
        }
    }

    /// Return the finished layer with the area in square millimeters times 1000.
    pub fn finish_with_pixel_area(mut self, pixel_area_mm2: f32) -> Layer {
        for (x, open) in self.columns.iter().enumerate() {
            if open.gray != 0 {
                self.lines.push(Line {
                    x: x as u16,
                    y_start: open.y_start,
                    y_end: open.y_end,
                    gray: open.gray,
                });
            }
        }
        self.lines.sort_unstable();

        let pixels = largest_enclosed_area(self.width, &self.spans);
        let largest_area = (pixels as f64 * pixel_area_mm2 as f64 * 1000.0) as u32;
        Layer {
            largest_area,
            lines: self.lines,
        }
    }
}

impl EncodableLayer for LayerEncoder {
    type Output = Layer;

    fn new(platform: Vector2<u32>) -> Self {
        assert!(platform.x <= MAX_WIDTH, "CXDLP supports at most {MAX_WIDTH} columns.");
        assert!(platform.y <= MAX_HEIGHT, "CXDLP supports at most {MAX_HEIGHT} rows.");
        Self {
            width: platform.x,
            height: platform.y,
            position: 0,
            columns: vec![OpenLine::default(); platform.x as usize],
            lines: Vec::new(),
            spans: Vec::new(),
        }
    }

    fn add_run(&mut self, length: u64, value: u8) {
        let end = (self.position + length).min(self.width as u64 * self.height as u64);
        if value == 0 {
            self.position = end;
            return;
        }

        while self.position < end {
            let y = (self.position / self.width as u64) as u32;
            let x = (self.position % self.width as u64) as u32;
            let count = (end - self.position).min((self.width - x) as u64) as u32;
            self.add_span(y, x, x + count - 1);
            for column in x..x + count {
                self.extend_column(column, y, value);
            }
            self.position += count as u64;
        }
    }

    fn finish(self, config: &SliceConfig, _exposure: &ExposureConfig, _height: Milimeters) -> Layer {
        let pixel_area = config.pixel_area().get::<Milimeter>();
        self.finish_with_pixel_area(pixel_area)
    }
}

/// Return the row ordered runs of a layer from its vertical `lines`.
///
/// The function draws the lines on an empty image of `resolution` and reads
/// the image back row by row. The function skips lines outside the image.
pub fn decode_lines(lines: &[Line], resolution: Vector2<u32>) -> Vec<Run> {
    let (width, height) = (resolution.x as usize, resolution.y as usize);
    let mut image = vec![0u8; width * height];
    for line in lines {
        let x = line.x as usize;
        if x >= width || line.y_start > line.y_end || line.y_end as usize >= height {
            continue;
        }
        for y in line.y_start as usize..=line.y_end as usize {
            image[y * width + x] = line.gray;
        }
    }

    let mut runs: Vec<Run> = Vec::new();
    for &value in &image {
        match runs.last_mut() {
            Some(run) if run.value == value => run.length += 1,
            _ => runs.push(Run::new(1, value)),
        }
    }
    runs
}
