//! SVG import - load and convert SVG files to drawable paths
//!
//! This module handles:
//! - Loading SVG files from disk
//! - Parsing SVG paths using usvg
//! - Converting Bézier curves to point sequences
//! - Normalizing coordinates to [-1, 1] range

use std::path::Path as FilePath;
use thiserror::Error;

use super::path::Path;
use super::traits::Shape;

/// Errors that can occur during SVG import
#[derive(Error, Debug)]
pub enum SvgError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse SVG: {0}")]
    ParseError(String),

    #[error("SVG contains no paths")]
    NoPaths,
}

/// Options for SVG import
pub struct SvgOptions {
    /// Number of points to sample per curve segment
    pub curve_samples: usize,
    /// Whether to close open paths
    pub close_paths: bool,
    /// Simplification tolerance (0 = no simplification)
    pub simplify_tolerance: f32,
}

impl Default for SvgOptions {
    fn default() -> Self {
        Self {
            curve_samples: 8,
            close_paths: false,
            simplify_tolerance: 0.0,
        }
    }
}

/// An imported SVG containing one or more paths
#[derive(Clone)]
pub struct SvgShape {
    /// All paths extracted from the SVG
    paths: Vec<Path>,
    /// Combined path for rendering
    combined: Path,
    /// Original filename
    name: String,
}

impl SvgShape {
    /// Load an SVG from a file
    pub fn load(path: impl AsRef<FilePath>, options: &SvgOptions) -> Result<Self, SvgError> {
        let path = path.as_ref();
        let svg_data = std::fs::read(path)?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("SVG")
            .to_string();

        Self::from_data(&svg_data, &name, options)
    }

    /// Parse SVG from raw data
    pub fn from_data(data: &[u8], name: &str, options: &SvgOptions) -> Result<Self, SvgError> {
        // Parse the SVG using usvg
        let tree = usvg::Tree::from_data(data, &usvg::Options::default())
            .map_err(|e| SvgError::ParseError(e.to_string()))?;

        let mut all_points: Vec<(f32, f32)> = Vec::new();
        let mut paths: Vec<Path> = Vec::new();

        // Get the viewbox for normalization
        let view_box = tree.size();
        let width = view_box.width();
        let height = view_box.height();
        let scale = width.max(height);
        let offset_x = width / 2.0;
        let offset_y = height / 2.0;

        // Helper to normalize coordinates to [-1, 1]
        let normalize = |x: f32, y: f32| -> (f32, f32) {
            let nx = (x - offset_x) / (scale / 2.0);
            let ny = -(y - offset_y) / (scale / 2.0); // Flip Y axis
            (nx.clamp(-1.0, 1.0), ny.clamp(-1.0, 1.0))
        };

        // Process a path node
        fn process_path(
            path: &usvg::Path,
            normalize: &impl Fn(f32, f32) -> (f32, f32),
            options: &SvgOptions,
            all_points: &mut Vec<(f32, f32)>,
            paths: &mut Vec<Path>,
        ) {
            let mut path_points = Vec::new();

            for segment in path.data().segments() {
                match segment {
                    usvg::tiny_skia_path::PathSegment::MoveTo(p) => {
                        // Start a new subpath
                        if !path_points.is_empty() {
                            // Save the current path
                            if path_points.len() >= 2 {
                                let p = Path::with_options(
                                    path_points.clone(),
                                    options.close_paths,
                                    "SVG Path".to_string(),
                                );
                                paths.push(p);
                                all_points.extend(&path_points);
                            }
                            path_points.clear();
                        }
                        path_points.push(normalize(p.x, p.y));
                    }
                    usvg::tiny_skia_path::PathSegment::LineTo(p) => {
                        path_points.push(normalize(p.x, p.y));
                    }
                    usvg::tiny_skia_path::PathSegment::QuadTo(p1, p2) => {
                        // Sample quadratic Bézier curve
                        if let Some(&start) = path_points.last() {
                            let ctrl = normalize(p1.x, p1.y);
                            let end = normalize(p2.x, p2.y);
                            for i in 1..=options.curve_samples {
                                let t = i as f32 / options.curve_samples as f32;
                                let point = quadratic_bezier(start, ctrl, end, t);
                                path_points.push(point);
                            }
                        }
                    }
                    usvg::tiny_skia_path::PathSegment::CubicTo(p1, p2, p3) => {
                        // Sample cubic Bézier curve
                        if let Some(&start) = path_points.last() {
                            let ctrl1 = normalize(p1.x, p1.y);
                            let ctrl2 = normalize(p2.x, p2.y);
                            let end = normalize(p3.x, p3.y);
                            for i in 1..=options.curve_samples {
                                let t = i as f32 / options.curve_samples as f32;
                                let point = cubic_bezier(start, ctrl1, ctrl2, end, t);
                                path_points.push(point);
                            }
                        }
                    }
                    usvg::tiny_skia_path::PathSegment::Close => {
                        // Close the path by connecting to the start
                        if path_points.len() >= 2 {
                            let p = Path::with_options(
                                path_points.clone(),
                                true, // closed
                                "SVG Path".to_string(),
                            );
                            paths.push(p);
                            all_points.extend(&path_points);
                        }
                        path_points.clear();
                    }
                }
            }

            // Save any remaining path
            if path_points.len() >= 2 {
                let p = Path::with_options(
                    path_points.clone(),
                    options.close_paths,
                    "SVG Path".to_string(),
                );
                paths.push(p);
                all_points.extend(&path_points);
            }
        }

        // Recursively process all nodes in a group
        fn process_group(
            group: &usvg::Group,
            normalize: &impl Fn(f32, f32) -> (f32, f32),
            options: &SvgOptions,
            all_points: &mut Vec<(f32, f32)>,
            paths: &mut Vec<Path>,
        ) {
            for child in group.children() {
                match child {
                    usvg::Node::Path(ref path) => {
                        process_path(path, normalize, options, all_points, paths);
                    }
                    usvg::Node::Group(ref subgroup) => {
                        process_group(subgroup, normalize, options, all_points, paths);
                    }
                    _ => {}
                }
            }
        }

        // Process the root group
        process_group(
            tree.root(),
            &normalize,
            options,
            &mut all_points,
            &mut paths,
        );

        if all_points.is_empty() {
            return Err(SvgError::NoPaths);
        }

        // Create combined path
        let combined = Path::with_options(all_points, false, name.to_string());

        Ok(Self {
            paths,
            combined,
            name: name.to_string(),
        })
    }

    /// Get the number of paths
    pub fn path_count(&self) -> usize {
        self.paths.len()
    }

    /// Get total point count
    pub fn point_count(&self) -> usize {
        self.combined.len()
    }

    /// Get individual paths
    pub fn paths(&self) -> &[Path] {
        &self.paths
    }
}

impl Shape for SvgShape {
    fn sample(&self, t: f32) -> (f32, f32) {
        self.combined.sample(t)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn length(&self) -> f32 {
        self.combined.length()
    }

    fn is_closed(&self) -> bool {
        self.combined.is_closed()
    }
}

/// Evaluate a quadratic Bézier curve at parameter t
fn quadratic_bezier(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), t: f32) -> (f32, f32) {
    let t2 = t * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    let x = mt2 * p0.0 + 2.0 * mt * t * p1.0 + t2 * p2.0;
    let y = mt2 * p0.1 + 2.0 * mt * t * p1.1 + t2 * p2.1;

    (x, y)
}

/// Evaluate a cubic Bézier curve at parameter t
fn cubic_bezier(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    t: f32,
) -> (f32, f32) {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    let x = mt3 * p0.0 + 3.0 * mt2 * t * p1.0 + 3.0 * mt * t2 * p2.0 + t3 * p3.0;
    let y = mt3 * p0.1 + 3.0 * mt2 * t * p1.1 + 3.0 * mt * t2 * p2.1 + t3 * p3.1;

    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadratic_bezier() {
        let p0 = (0.0, 0.0);
        let p1 = (0.5, 1.0);
        let p2 = (1.0, 0.0);

        // At t=0, should be at p0
        let (x, y) = quadratic_bezier(p0, p1, p2, 0.0);
        assert!((x - 0.0).abs() < 0.001);
        assert!((y - 0.0).abs() < 0.001);

        // At t=1, should be at p2
        let (x, y) = quadratic_bezier(p0, p1, p2, 1.0);
        assert!((x - 1.0).abs() < 0.001);
        assert!((y - 0.0).abs() < 0.001);

        // At t=0.5, should be at midpoint lifted by control point
        let (x, y) = quadratic_bezier(p0, p1, p2, 0.5);
        assert!((x - 0.5).abs() < 0.001);
        assert!((y - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_cubic_bezier() {
        let p0 = (0.0, 0.0);
        let p1 = (0.33, 1.0);
        let p2 = (0.66, 1.0);
        let p3 = (1.0, 0.0);

        // At t=0, should be at p0
        let (x, y) = cubic_bezier(p0, p1, p2, p3, 0.0);
        assert!((x - 0.0).abs() < 0.001);
        assert!((y - 0.0).abs() < 0.001);

        // At t=1, should be at p3
        let (x, y) = cubic_bezier(p0, p1, p2, p3, 1.0);
        assert!((x - 1.0).abs() < 0.001);
        assert!((y - 0.0).abs() < 0.001);
    }
}
