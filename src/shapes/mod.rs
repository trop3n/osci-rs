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

mod traits;
mod primitives;
mod path;
mod scene;
mod svg;
mod image;
mod text;
mod mesh3d;

pub use traits::Shape;
pub use primitives::{Circle, Line, Rectangle, Polygon};
pub use path::Path;
#[allow(unused_imports)]
pub use scene::{Scene, SceneShape};
#[allow(unused_imports)]
pub use svg::{SvgShape, SvgError, SvgOptions};
#[allow(unused_imports)]
pub use image::{ImageShape, ImageError, ImageOptions};
#[allow(unused_imports)]
pub use text::{TextShape, TextError, TextOptions};
#[allow(unused_imports)]
pub use mesh3d::{Mesh, Mesh3DShape, Mesh3DOptions, MeshError, Camera};

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
