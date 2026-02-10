//! Path type - arbitrary sequences of points
//!
//! A Path is a flexible shape defined by a list of points.
//! It can be used for:
//! - Custom hand-drawn shapes
//! - Imported SVG paths (future)
//! - Traced image edges (future)
//! - Text glyphs (future)

use super::traits::Shape;

/// A path defined by a sequence of points
///
/// Points are connected in order. The path can be open (endpoints don't connect)
/// or closed (last point connects back to first).
#[derive(Clone, Debug)]
pub struct Path {
    /// Points along the path
    points: Vec<(f32, f32)>,
    /// Cached segment lengths for uniform sampling
    segment_lengths: Vec<f32>,
    /// Total path length
    total_length: f32,
    /// Whether the path is closed
    closed: bool,
    /// Optional name for this path
    name: String,
}

impl Path {
    /// Create a new open path from points
    pub fn new(points: Vec<(f32, f32)>) -> Self {
        Self::with_options(points, false, "Path".to_string())
    }

    /// Create a new closed path from points
    pub fn closed(points: Vec<(f32, f32)>) -> Self {
        Self::with_options(points, true, "Path".to_string())
    }

    /// Create a path with full options
    pub fn with_options(points: Vec<(f32, f32)>, closed: bool, name: String) -> Self {
        let segment_count = if closed {
            points.len()
        } else {
            points.len().saturating_sub(1)
        };

        let mut segment_lengths = Vec::with_capacity(segment_count);
        let mut total_length = 0.0;

        for i in 0..segment_count {
            let (x1, y1) = points[i];
            let (x2, y2) = points[(i + 1) % points.len()];
            let dx = x2 - x1;
            let dy = y2 - y1;
            let len = (dx * dx + dy * dy).sqrt();
            segment_lengths.push(len);
            total_length += len;
        }

        Self {
            points,
            segment_lengths,
            total_length,
            closed,
            name,
        }
    }

    /// Get the number of points in the path
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Check if the path is empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Get a reference to the points
    pub fn points(&self) -> &[(f32, f32)] {
        &self.points
    }

    /// Create a path that traces a sine wave
    pub fn sine_wave(amplitude: f32, periods: f32, num_points: usize) -> Self {
        let points: Vec<(f32, f32)> = (0..num_points)
            .map(|i| {
                let t = i as f32 / (num_points - 1) as f32;
                let x = t * 2.0 - 1.0; // -1 to 1
                let y = amplitude * (t * periods * std::f32::consts::TAU).sin();
                (x, y)
            })
            .collect();

        Self::with_options(points, false, "Sine Wave".to_string())
    }

    /// Create a Lissajous curve
    ///
    /// Lissajous curves are created by combining two perpendicular oscillations:
    /// x = A * sin(a*t + delta)
    /// y = B * sin(b*t)
    ///
    /// # Arguments
    /// * `a` - Frequency ratio for X
    /// * `b` - Frequency ratio for Y
    /// * `delta` - Phase offset for X (in radians)
    /// * `num_points` - Number of points to generate
    pub fn lissajous(a: f32, b: f32, delta: f32, num_points: usize) -> Self {
        let points: Vec<(f32, f32)> = (0..num_points)
            .map(|i| {
                let t = i as f32 / num_points as f32 * std::f32::consts::TAU;
                let x = (a * t + delta).sin();
                let y = (b * t).sin();
                (x, y)
            })
            .collect();

        Self::with_options(points, true, "Lissajous".to_string())
    }

    /// Create a spiral
    ///
    /// # Arguments
    /// * `start_radius` - Starting radius
    /// * `end_radius` - Ending radius
    /// * `turns` - Number of complete rotations
    /// * `num_points` - Number of points to generate
    pub fn spiral(start_radius: f32, end_radius: f32, turns: f32, num_points: usize) -> Self {
        let points: Vec<(f32, f32)> = (0..num_points)
            .map(|i| {
                let t = i as f32 / (num_points - 1) as f32;
                let radius = start_radius + t * (end_radius - start_radius);
                let angle = t * turns * std::f32::consts::TAU;
                let x = radius * angle.cos();
                let y = radius * angle.sin();
                (x, y)
            })
            .collect();

        Self::with_options(points, false, "Spiral".to_string())
    }

    /// Create a heart shape
    pub fn heart(scale: f32, num_points: usize) -> Self {
        let points: Vec<(f32, f32)> = (0..num_points)
            .map(|i| {
                let t = i as f32 / num_points as f32 * std::f32::consts::TAU;
                // Heart curve parametric equations
                let x = 16.0 * t.sin().powi(3);
                let y = 13.0 * t.cos()
                    - 5.0 * (2.0 * t).cos()
                    - 2.0 * (3.0 * t).cos()
                    - (4.0 * t).cos();
                // Scale to fit in [-1, 1] range
                (x * scale / 17.0, y * scale / 17.0)
            })
            .collect();

        Self::with_options(points, true, "Heart".to_string())
    }
}

impl Shape for Path {
    fn sample(&self, t: f32) -> (f32, f32) {
        if self.points.is_empty() {
            return (0.0, 0.0);
        }

        if self.points.len() == 1 {
            return self.points[0];
        }

        if self.total_length == 0.0 {
            return self.points[0];
        }

        // Find which segment we're on
        let target_dist = t * self.total_length;
        let mut accumulated = 0.0;

        for (i, &seg_len) in self.segment_lengths.iter().enumerate() {
            if accumulated + seg_len >= target_dist || i == self.segment_lengths.len() - 1 {
                // We're on this segment
                let local_t = if seg_len > 0.0 {
                    (target_dist - accumulated) / seg_len
                } else {
                    0.0
                };

                let (x1, y1) = self.points[i];
                let (x2, y2) = self.points[(i + 1) % self.points.len()];

                let x = x1 + local_t * (x2 - x1);
                let y = y1 + local_t * (y2 - y1);

                return (x, y);
            }
            accumulated += seg_len;
        }

        self.points[0]
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn length(&self) -> f32 {
        self.total_length
    }

    fn is_closed(&self) -> bool {
        self.closed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_basic() {
        let path = Path::new(vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)]);
        assert_eq!(path.len(), 3);
        assert!(!path.is_closed());
    }

    #[test]
    fn test_lissajous() {
        let lissajous = Path::lissajous(3.0, 2.0, 0.0, 100);
        assert_eq!(lissajous.len(), 100);
        assert!(lissajous.is_closed());
    }

    #[test]
    fn test_heart() {
        let heart = Path::heart(0.8, 100);
        assert_eq!(heart.len(), 100);
    }
}
