//! Shapes module - defines drawable shapes for oscilloscope output
//!
//! This module provides:
//! - `Shape` trait for abstracting over different shape types
//! - Primitive shapes: Circle, Line, Rectangle, etc.
//! - Path type for arbitrary point sequences

mod traits;
mod primitives;
mod path;

pub use traits::Shape;
pub use primitives::{Circle, Line, Rectangle, Polygon};
pub use path::Path;

use crate::audio::XYSample;

/// Convert a shape to a vector of XY samples
///
/// This function samples a shape at regular intervals to produce
/// audio samples that will draw the shape on an oscilloscope.
///
/// # Arguments
/// * `shape` - The shape to sample
/// * `num_samples` - Number of samples to generate (more = smoother but slower)
///
/// # Returns
/// A vector of XY samples representing the shape
pub fn shape_to_samples<S: Shape>(shape: &S, num_samples: usize) -> Vec<XYSample> {
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        // t goes from 0.0 to 1.0 (exclusive of 1.0 to avoid duplicate endpoint)
        let t = i as f32 / num_samples as f32;
        let (x, y) = shape.sample(t);
        samples.push(XYSample::new(x, y));
    }

    samples
}
