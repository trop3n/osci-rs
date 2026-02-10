//! Scene - composition of multiple shapes
//!
//! A Scene combines multiple shapes into a single drawable unit.
//! Each shape is allocated a portion of the total trace time based on its weight.

use super::traits::Shape;

/// A shape entry in the scene with its configuration
pub struct SceneShape {
    /// The shape (boxed for dynamic dispatch)
    shape: Box<dyn Shape>,
    /// Weight for time allocation (higher = more time)
    weight: f32,
    /// Whether this shape is enabled
    enabled: bool,
}

impl SceneShape {
    /// Create a new scene shape entry
    pub fn new<S: Shape + 'static>(shape: S) -> Self {
        Self {
            shape: Box::new(shape),
            weight: 1.0,
            enabled: true,
        }
    }

    /// Create with a specific weight
    pub fn with_weight<S: Shape + 'static>(shape: S, weight: f32) -> Self {
        Self {
            shape: Box::new(shape),
            weight,
            enabled: true,
        }
    }

    /// Get the shape's name
    pub fn name(&self) -> &str {
        self.shape.name()
    }

    /// Get the weight
    pub fn weight(&self) -> f32 {
        self.weight
    }

    /// Set the weight
    pub fn set_weight(&mut self, weight: f32) {
        self.weight = weight.max(0.1); // Minimum weight
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Toggle enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// A scene containing multiple shapes
///
/// Shapes are drawn in sequence, with time allocated based on their weights.
/// For example, with two shapes of weight 1.0 each, the first shape is drawn
/// for t in [0, 0.5) and the second for t in [0.5, 1.0).
pub struct Scene {
    /// Shapes in the scene
    shapes: Vec<SceneShape>,
    /// Cached time boundaries for each shape (computed from weights)
    /// Each entry is (start_t, end_t, shape_index)
    boundaries: Vec<(f32, f32, usize)>,
    /// Name of the scene
    name: String,
}

impl Scene {
    /// Create an empty scene
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            shapes: Vec::new(),
            boundaries: Vec::new(),
            name: name.into(),
        }
    }

    /// Add a shape to the scene
    pub fn add<S: Shape + 'static>(&mut self, shape: S) -> &mut Self {
        self.shapes.push(SceneShape::new(shape));
        self.recompute_boundaries();
        self
    }

    /// Add a shape with a specific weight
    pub fn add_weighted<S: Shape + 'static>(&mut self, shape: S, weight: f32) -> &mut Self {
        self.shapes.push(SceneShape::with_weight(shape, weight));
        self.recompute_boundaries();
        self
    }

    /// Remove a shape by index
    pub fn remove(&mut self, index: usize) -> Option<SceneShape> {
        if index < self.shapes.len() {
            let shape = self.shapes.remove(index);
            self.recompute_boundaries();
            Some(shape)
        } else {
            None
        }
    }

    /// Get the number of shapes
    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    /// Check if scene is empty
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }

    /// Get a reference to a shape entry
    pub fn get(&self, index: usize) -> Option<&SceneShape> {
        self.shapes.get(index)
    }

    /// Get a mutable reference to a shape entry
    pub fn get_mut(&mut self, index: usize) -> Option<&mut SceneShape> {
        self.shapes.get_mut(index)
    }

    /// Iterate over shape entries
    pub fn iter(&self) -> impl Iterator<Item = &SceneShape> {
        self.shapes.iter()
    }

    /// Move a shape up in the order
    pub fn move_up(&mut self, index: usize) {
        if index > 0 && index < self.shapes.len() {
            self.shapes.swap(index, index - 1);
            self.recompute_boundaries();
        }
    }

    /// Move a shape down in the order
    pub fn move_down(&mut self, index: usize) {
        if index + 1 < self.shapes.len() {
            self.shapes.swap(index, index + 1);
            self.recompute_boundaries();
        }
    }

    /// Update a shape's weight and recompute boundaries
    pub fn set_weight(&mut self, index: usize, weight: f32) {
        if let Some(shape) = self.shapes.get_mut(index) {
            shape.set_weight(weight);
            self.recompute_boundaries();
        }
    }

    /// Recompute time boundaries based on current weights
    fn recompute_boundaries(&mut self) {
        self.boundaries.clear();

        // Calculate total weight of enabled shapes
        let total_weight: f32 = self
            .shapes
            .iter()
            .filter(|s| s.enabled)
            .map(|s| s.weight)
            .sum();

        if total_weight <= 0.0 {
            return;
        }

        // Compute boundaries
        let mut current_t = 0.0;
        for (i, shape) in self.shapes.iter().enumerate() {
            if shape.enabled {
                let duration = shape.weight / total_weight;
                self.boundaries.push((current_t, current_t + duration, i));
                current_t += duration;
            }
        }
    }

    /// Find which shape should be drawn at time t
    fn find_shape_at(&self, t: f32) -> Option<(usize, f32)> {
        for &(start, end, idx) in &self.boundaries {
            if t >= start && t < end {
                // Remap t to [0, 1) for this shape
                let local_t = (t - start) / (end - start);
                return Some((idx, local_t));
            }
        }
        // Handle t == 1.0 case
        if let Some(&(_start, end, idx)) = self.boundaries.last() {
            if (t - 1.0).abs() < 0.0001 || t >= end {
                return Some((idx, 0.999));
            }
        }
        None
    }
}

impl Shape for Scene {
    fn sample(&self, t: f32) -> (f32, f32) {
        if let Some((idx, local_t)) = self.find_shape_at(t) {
            self.shapes[idx].shape.sample(local_t)
        } else if !self.shapes.is_empty() {
            // Fallback to first shape
            self.shapes[0].shape.sample(t)
        } else {
            (0.0, 0.0)
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn length(&self) -> f32 {
        // Sum of all shape lengths
        self.shapes
            .iter()
            .filter(|s| s.enabled)
            .map(|s| s.shape.length() * s.weight)
            .sum()
    }

    fn is_closed(&self) -> bool {
        // Scene is closed if all shapes are closed
        self.shapes
            .iter()
            .filter(|s| s.enabled)
            .all(|s| s.shape.is_closed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::Circle;

    #[test]
    fn test_empty_scene() {
        let scene = Scene::new("Empty");
        assert!(scene.is_empty());
        assert_eq!(scene.sample(0.5), (0.0, 0.0));
    }

    #[test]
    fn test_single_shape() {
        let mut scene = Scene::new("Single");
        scene.add(Circle::new(0.5));

        assert_eq!(scene.len(), 1);

        // At t=0, should be at rightmost point of circle
        let (x, y) = scene.sample(0.0);
        assert!((x - 0.5).abs() < 0.01);
        assert!(y.abs() < 0.01);
    }

    #[test]
    fn test_multiple_shapes() {
        let mut scene = Scene::new("Multi");
        scene.add(Circle::new(0.5));
        scene.add(Circle::new(0.3));

        assert_eq!(scene.len(), 2);

        // First half should sample first circle
        let (x1, _) = scene.sample(0.0);
        assert!((x1 - 0.5).abs() < 0.01);

        // Second half should sample second circle
        let (x2, _) = scene.sample(0.5);
        assert!((x2 - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_weighted_shapes() {
        let mut scene = Scene::new("Weighted");
        scene.add_weighted(Circle::new(0.5), 2.0); // Gets 2/3 of time (t=0 to 0.666)
        scene.add_weighted(Circle::new(0.3), 1.0); // Gets 1/3 of time (t=0.666 to 1.0)

        // At t=0, should be at start of first circle (radius 0.5)
        // Circle at t=0 gives x=radius, y=0
        let (x0, y0) = scene.sample(0.0);
        assert!((x0 - 0.5).abs() < 0.01);
        assert!(y0.abs() < 0.01);

        // At t=0.8, we're in second circle (0.666 to 1.0)
        // local_t = (0.8 - 0.666) / (1.0 - 0.666) ≈ 0.4
        // Second circle has radius 0.3
        let (x, y) = scene.sample(0.8);
        // Should be on the second circle (distance from origin ≈ 0.3)
        let dist = (x * x + y * y).sqrt();
        assert!((dist - 0.3).abs() < 0.1);
    }
}
