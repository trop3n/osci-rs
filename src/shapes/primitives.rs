//! Primitive shapes - Circle, Line, Rectangle, Polygon
//!
//! These are the basic building blocks for oscilloscope graphics.

use std::f32::consts::TAU;
use super::traits::Shape;

/// A circle centered at (cx, cy) with given radius
///
/// ## Parametric Equation
/// ```text
/// x = cx + radius * cos(t * 2π)
/// y = cy + radius * sin(t * 2π)
/// ```
#[derive(Clone, Debug)]
pub struct Circle {
    /// Center X coordinate
    pub cx: f32,
    /// Center Y coordinate
    pub cy: f32,
    /// Radius (0.0 to 1.0 recommended)
    pub radius: f32,
}

impl Circle {
    /// Create a new circle at the origin with given radius
    pub fn new(radius: f32) -> Self {
        Self {
            cx: 0.0,
            cy: 0.0,
            radius,
        }
    }

    /// Create a circle at a specific position
    pub fn at(cx: f32, cy: f32, radius: f32) -> Self {
        Self { cx, cy, radius }
    }
}

impl Shape for Circle {
    fn sample(&self, t: f32) -> (f32, f32) {
        let angle = t * TAU;
        let x = self.cx + self.radius * angle.cos();
        let y = self.cy + self.radius * angle.sin();
        (x, y)
    }

    fn name(&self) -> &str {
        "Circle"
    }

    fn length(&self) -> f32 {
        TAU * self.radius
    }

    fn is_closed(&self) -> bool {
        true
    }
}

/// A line segment from (x1, y1) to (x2, y2)
///
/// ## Parametric Equation
/// ```text
/// x = x1 + t * (x2 - x1)
/// y = y1 + t * (y2 - y1)
/// ```
#[derive(Clone, Debug)]
pub struct Line {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Line {
    /// Create a new line from (x1, y1) to (x2, y2)
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Create a horizontal line at y position
    pub fn horizontal(y: f32, x_start: f32, x_end: f32) -> Self {
        Self::new(x_start, y, x_end, y)
    }

    /// Create a vertical line at x position
    pub fn vertical(x: f32, y_start: f32, y_end: f32) -> Self {
        Self::new(x, y_start, x, y_end)
    }
}

impl Shape for Line {
    fn sample(&self, t: f32) -> (f32, f32) {
        let x = self.x1 + t * (self.x2 - self.x1);
        let y = self.y1 + t * (self.y2 - self.y1);
        (x, y)
    }

    fn name(&self) -> &str {
        "Line"
    }

    fn length(&self) -> f32 {
        let dx = self.x2 - self.x1;
        let dy = self.y2 - self.y1;
        (dx * dx + dy * dy).sqrt()
    }

    fn is_closed(&self) -> bool {
        false
    }
}

/// A rectangle centered at (cx, cy) with given width and height
///
/// The rectangle is traced starting from the top-left corner,
/// going clockwise: top → right → bottom → left
#[derive(Clone, Debug)]
pub struct Rectangle {
    /// Center X coordinate
    pub cx: f32,
    /// Center Y coordinate
    pub cy: f32,
    /// Half-width (distance from center to edge)
    pub half_width: f32,
    /// Half-height (distance from center to edge)
    pub half_height: f32,
}

impl Rectangle {
    /// Create a rectangle at the origin
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            cx: 0.0,
            cy: 0.0,
            half_width: width / 2.0,
            half_height: height / 2.0,
        }
    }

    /// Create a square at the origin
    pub fn square(size: f32) -> Self {
        Self::new(size, size)
    }

    /// Create a rectangle at a specific position
    pub fn at(cx: f32, cy: f32, width: f32, height: f32) -> Self {
        Self {
            cx,
            cy,
            half_width: width / 2.0,
            half_height: height / 2.0,
        }
    }

    /// Get the corners of the rectangle
    fn corners(&self) -> [(f32, f32); 4] {
        [
            (self.cx - self.half_width, self.cy + self.half_height), // Top-left
            (self.cx + self.half_width, self.cy + self.half_height), // Top-right
            (self.cx + self.half_width, self.cy - self.half_height), // Bottom-right
            (self.cx - self.half_width, self.cy - self.half_height), // Bottom-left
        ]
    }
}

impl Shape for Rectangle {
    fn sample(&self, t: f32) -> (f32, f32) {
        let corners = self.corners();

        // Divide t into 4 segments (one per edge)
        // Each edge gets t in [0, 0.25), [0.25, 0.5), etc.
        let segment = (t * 4.0) as usize;
        let local_t = (t * 4.0).fract();

        // Clamp segment to valid range (handles t = 1.0)
        let segment = segment.min(3);

        let (x1, y1) = corners[segment];
        let (x2, y2) = corners[(segment + 1) % 4];

        // Linear interpolation between corners
        let x = x1 + local_t * (x2 - x1);
        let y = y1 + local_t * (y2 - y1);

        (x, y)
    }

    fn name(&self) -> &str {
        "Rectangle"
    }

    fn length(&self) -> f32 {
        4.0 * (self.half_width + self.half_height)
    }

    fn is_closed(&self) -> bool {
        true
    }
}

/// A polygon defined by a list of vertices
///
/// The polygon is traced by connecting consecutive vertices,
/// with the last vertex connecting back to the first.
#[derive(Clone, Debug)]
pub struct Polygon {
    /// Vertices in order (will be connected in sequence)
    vertices: Vec<(f32, f32)>,
    /// Cached edge lengths for uniform sampling
    edge_lengths: Vec<f32>,
    /// Total perimeter length
    total_length: f32,
}

impl Polygon {
    /// Create a new polygon from vertices
    ///
    /// # Panics
    /// Panics if fewer than 3 vertices are provided
    pub fn new(vertices: Vec<(f32, f32)>) -> Self {
        assert!(vertices.len() >= 3, "Polygon requires at least 3 vertices");

        let n = vertices.len();
        let mut edge_lengths = Vec::with_capacity(n);
        let mut total_length = 0.0;

        for i in 0..n {
            let (x1, y1) = vertices[i];
            let (x2, y2) = vertices[(i + 1) % n];
            let dx = x2 - x1;
            let dy = y2 - y1;
            let len = (dx * dx + dy * dy).sqrt();
            edge_lengths.push(len);
            total_length += len;
        }

        Self {
            vertices,
            edge_lengths,
            total_length,
        }
    }

    /// Create a regular polygon with n sides
    ///
    /// # Arguments
    /// * `n` - Number of sides (3 = triangle, 4 = square, etc.)
    /// * `radius` - Distance from center to vertices
    pub fn regular(n: usize, radius: f32) -> Self {
        assert!(n >= 3, "Regular polygon requires at least 3 sides");

        let vertices: Vec<(f32, f32)> = (0..n)
            .map(|i| {
                // Start from top (angle = -π/2) and go clockwise
                let angle = -std::f32::consts::FRAC_PI_2 + (i as f32 / n as f32) * TAU;
                (radius * angle.cos(), radius * angle.sin())
            })
            .collect();

        Self::new(vertices)
    }

    /// Create an equilateral triangle
    pub fn triangle(radius: f32) -> Self {
        Self::regular(3, radius)
    }

    /// Create a pentagon
    pub fn pentagon(radius: f32) -> Self {
        Self::regular(5, radius)
    }

    /// Create a hexagon
    pub fn hexagon(radius: f32) -> Self {
        Self::regular(6, radius)
    }

    /// Create a star with n points
    ///
    /// # Arguments
    /// * `n` - Number of points
    /// * `outer_radius` - Distance to outer points
    /// * `inner_radius` - Distance to inner points (between outer points)
    pub fn star(n: usize, outer_radius: f32, inner_radius: f32) -> Self {
        assert!(n >= 3, "Star requires at least 3 points");

        let total_points = n * 2;
        let vertices: Vec<(f32, f32)> = (0..total_points)
            .map(|i| {
                let angle = -std::f32::consts::FRAC_PI_2 + (i as f32 / total_points as f32) * TAU;
                let radius = if i % 2 == 0 { outer_radius } else { inner_radius };
                (radius * angle.cos(), radius * angle.sin())
            })
            .collect();

        Self::new(vertices)
    }
}

impl Shape for Polygon {
    fn sample(&self, t: f32) -> (f32, f32) {
        if self.total_length == 0.0 {
            return self.vertices[0];
        }

        // Find which edge we're on based on t
        let target_dist = t * self.total_length;
        let mut accumulated = 0.0;

        for (i, &edge_len) in self.edge_lengths.iter().enumerate() {
            if accumulated + edge_len >= target_dist || i == self.edge_lengths.len() - 1 {
                // We're on this edge
                let local_t = if edge_len > 0.0 {
                    (target_dist - accumulated) / edge_len
                } else {
                    0.0
                };

                let (x1, y1) = self.vertices[i];
                let (x2, y2) = self.vertices[(i + 1) % self.vertices.len()];

                let x = x1 + local_t * (x2 - x1);
                let y = y1 + local_t * (y2 - y1);

                return (x, y);
            }
            accumulated += edge_len;
        }

        self.vertices[0]
    }

    fn name(&self) -> &str {
        "Polygon"
    }

    fn length(&self) -> f32 {
        self.total_length
    }

    fn is_closed(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle() {
        let circle = Circle::new(0.5);

        // At t=0, should be at (0.5, 0) - rightmost point
        let (x, y) = circle.sample(0.0);
        assert!((x - 0.5).abs() < 0.001);
        assert!(y.abs() < 0.001);

        // At t=0.25, should be at (0, 0.5) - top
        let (x, y) = circle.sample(0.25);
        assert!(x.abs() < 0.001);
        assert!((y - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_line() {
        let line = Line::new(-1.0, 0.0, 1.0, 0.0);

        // At t=0, should be at start
        let (x, y) = line.sample(0.0);
        assert!((x - (-1.0)).abs() < 0.001);
        assert!(y.abs() < 0.001);

        // At t=0.5, should be at midpoint
        let (x, y) = line.sample(0.5);
        assert!(x.abs() < 0.001);
        assert!(y.abs() < 0.001);
    }

    #[test]
    fn test_rectangle() {
        let rect = Rectangle::square(1.0);

        // Verify corners are hit
        let (x, y) = rect.sample(0.0);
        assert!((x - (-0.5)).abs() < 0.001);
        assert!((y - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_polygon() {
        let triangle = Polygon::triangle(0.5);
        assert_eq!(triangle.vertices.len(), 3);

        let star = Polygon::star(5, 0.8, 0.3);
        assert_eq!(star.vertices.len(), 10);
    }
}
