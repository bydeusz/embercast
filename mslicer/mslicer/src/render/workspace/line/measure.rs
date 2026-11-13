use nalgebra::Vector3;

use crate::{
    core::App,
    interface::theme::{self, color},
    render::workspace::line::{Line, LineGenerator},
};

/// The half length of the cross on each measure point, in millimeters.
const MARKER_SIZE: f32 = 0.6;

/// The gold line between the measure points, with a small cross on each point.
pub struct MeasureDispatch {
    cached_lines: Vec<Line>,
}

impl MeasureDispatch {
    pub fn new() -> Self {
        Self {
            cached_lines: Vec::new(),
        }
    }
}

impl LineGenerator for MeasureDispatch {
    fn generate_lines(&mut self, app: &mut App) {
        let gold = Vector3::from(theme::rgb(color::ACCENT));
        let points = app.state.measure.points();

        self.cached_lines.clear();
        for point in points {
            for axis in [Vector3::x(), Vector3::y(), Vector3::z()] {
                let offset = axis * MARKER_SIZE;
                self.cached_lines.push(Line::new(point - offset, point + offset).color(gold));
            }
        }
        if let [a, b] = points {
            self.cached_lines.push(Line::new(*a, *b).color(gold));
        }
    }

    fn lines(&self) -> &[Line] {
        &self.cached_lines
    }
}
