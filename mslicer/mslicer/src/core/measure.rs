use nalgebra::Vector3;

/// The points that the measure tool placed, in world space, at most two.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Measurement {
    points: Vec<Vector3<f32>>,
}

impl Measurement {
    /// Add a point. A third point clears the measurement and starts a new one.
    pub fn add(&mut self, point: Vector3<f32>) {
        if self.points.len() >= 2 {
            self.points.clear();
        }
        self.points.push(point);
    }

    pub fn clear(&mut self) {
        self.points.clear();
    }

    pub fn points(&self) -> &[Vector3<f32>] {
        &self.points
    }

    /// Return the distance between the two points and the size of the gap along each axis, or `None` before the second point.
    pub fn distance(&self) -> Option<(f32, Vector3<f32>)> {
        let [a, b] = self.points.as_slice() else {
            return None;
        };
        let delta = b - a;
        Some((delta.norm(), delta.abs()))
    }
}
