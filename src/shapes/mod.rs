//! Shapes module - defines drawable shapes for oscilloscope output
//!
//! This module provides:
//! - `Shape` trait for abstracting over different shape types
//! - Primitive shapes: Circle, Line, Rectangle, etc.
//! - Path type for arbitrary point sequences
//! - Scene type for composing multiple shapes
//! - SVG import for loading vector graphics
//! - Image tracing for converting raster images to paths
//! - Text rendering for converting text to paths
//! - 3D mesh rendering with wireframe projection

mod image;
mod mesh3d;
mod path;
mod primitives;
mod scene;
mod svg;
mod text;
mod traits;

#[allow(unused_imports)]
pub use image::{ImageError, ImageOptions, ImageShape};
#[allow(unused_imports)]
pub use mesh3d::{Camera, Mesh, Mesh3DOptions, Mesh3DShape, MeshError};
pub use path::Path;
pub use primitives::{Circle, Line, Polygon, Rectangle};
#[allow(unused_imports)]
pub use scene::{Scene, SceneShape};
#[allow(unused_imports)]
pub use svg::{SvgError, SvgOptions, SvgShape};
#[allow(unused_imports)]
pub use text::{TextError, TextOptions, TextShape};
pub use traits::Shape;

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
#[allow(dead_code)]
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
