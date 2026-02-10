//! Image tracing - convert raster images to drawable paths
//!
//! This module handles:
//! - Loading image files (PNG, JPEG, etc.)
//! - Edge detection using Sobel operator
//! - Tracing edges into point sequences
//! - Normalizing coordinates to [-1, 1] range

use std::path::Path as FilePath;
use thiserror::Error;

use super::path::Path;
use super::traits::Shape;

/// Errors that can occur during image processing
#[derive(Error, Debug)]
pub enum ImageError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to decode image: {0}")]
    DecodeError(#[from] image::ImageError),

    #[error("No edges found in image")]
    NoEdges,

    #[error("Image too small (minimum 8x8)")]
    TooSmall,
}

/// Options for image tracing
pub struct ImageOptions {
    /// Edge detection threshold (0.0 to 1.0)
    pub threshold: f32,
    /// Whether to invert the image before processing
    pub invert: bool,
    /// Maximum number of points to generate
    pub max_points: usize,
    /// Minimum edge strength to consider (0.0 to 1.0)
    pub edge_min: f32,
}

impl Default for ImageOptions {
    fn default() -> Self {
        Self {
            threshold: 0.3,
            invert: false,
            max_points: 5000,
            edge_min: 0.1,
        }
    }
}

/// An image converted to drawable edge paths
pub struct ImageShape {
    /// Points along detected edges
    points: Vec<(f32, f32)>,
    /// The path for rendering
    path: Path,
    /// Original filename
    name: String,
    /// Image dimensions
    width: u32,
    height: u32,
}

impl ImageShape {
    /// Load an image from a file
    pub fn load(path: impl AsRef<FilePath>, options: &ImageOptions) -> Result<Self, ImageError> {
        let path = path.as_ref();
        let img = image::open(path)?;

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Image")
            .to_string();

        Self::from_image(img, &name, options)
    }

    /// Process an already-loaded image
    pub fn from_image(
        img: image::DynamicImage,
        name: &str,
        options: &ImageOptions,
    ) -> Result<Self, ImageError> {
        let gray = img.to_luma8();
        let (width, height) = gray.dimensions();

        if width < 8 || height < 8 {
            return Err(ImageError::TooSmall);
        }

        // Apply edge detection
        let edges = sobel_edge_detection(&gray, options);

        // Extract edge points
        let points = extract_edge_points(&edges, width, height, options);

        if points.is_empty() {
            return Err(ImageError::NoEdges);
        }

        // Sort points for better drawing order (nearest neighbor)
        let sorted_points = sort_points_nearest_neighbor(&points, options.max_points);

        // Create path from points
        let path = Path::with_options(sorted_points.clone(), false, name.to_string());

        Ok(Self {
            points: sorted_points,
            path,
            name: name.to_string(),
            width,
            height,
        })
    }

    /// Get the number of edge points
    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    /// Get the original image dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Shape for ImageShape {
    fn sample(&self, t: f32) -> (f32, f32) {
        self.path.sample(t)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn length(&self) -> f32 {
        self.path.length()
    }

    fn is_closed(&self) -> bool {
        false
    }
}

/// Apply Sobel edge detection to a grayscale image
fn sobel_edge_detection(
    img: &image::GrayImage,
    options: &ImageOptions,
) -> Vec<f32> {
    let (width, height) = img.dimensions();
    let w = width as usize;
    let h = height as usize;

    // Sobel kernels
    const GX: [[i32; 3]; 3] = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
    const GY: [[i32; 3]; 3] = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];

    let mut edges = vec![0.0f32; w * h];

    // Get pixel value, handling inversion
    let get_pixel = |x: u32, y: u32| -> f32 {
        let val = img.get_pixel(x, y).0[0] as f32 / 255.0;
        if options.invert { 1.0 - val } else { val }
    };

    // Apply Sobel operator
    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let mut gx_sum = 0.0f32;
            let mut gy_sum = 0.0f32;

            for ky in 0..3 {
                for kx in 0..3 {
                    let px = (x as i32 + kx as i32 - 1) as u32;
                    let py = (y as i32 + ky as i32 - 1) as u32;
                    let pixel = get_pixel(px, py);

                    gx_sum += pixel * GX[ky][kx] as f32;
                    gy_sum += pixel * GY[ky][kx] as f32;
                }
            }

            // Gradient magnitude
            let magnitude = (gx_sum * gx_sum + gy_sum * gy_sum).sqrt();
            edges[y as usize * w + x as usize] = magnitude;
        }
    }

    // Normalize to 0-1 range
    let max_val = edges.iter().cloned().fold(0.0f32, f32::max);
    if max_val > 0.0 {
        for edge in &mut edges {
            *edge /= max_val;
        }
    }

    edges
}

/// Extract points from edge detection result
fn extract_edge_points(
    edges: &[f32],
    width: u32,
    height: u32,
    options: &ImageOptions,
) -> Vec<(f32, f32)> {
    let w = width as usize;
    let h = height as usize;

    // Calculate normalization factors to map to [-1, 1]
    let scale = width.max(height) as f32;
    let offset_x = width as f32 / 2.0;
    let offset_y = height as f32 / 2.0;

    let mut points = Vec::new();

    for y in 0..h {
        for x in 0..w {
            let edge_val = edges[y * w + x];

            // Check if this pixel is above threshold
            if edge_val >= options.threshold && edge_val >= options.edge_min {
                // Normalize coordinates to [-1, 1]
                let nx = (x as f32 - offset_x) / (scale / 2.0);
                let ny = -(y as f32 - offset_y) / (scale / 2.0); // Flip Y

                points.push((nx.clamp(-1.0, 1.0), ny.clamp(-1.0, 1.0)));
            }
        }
    }

    points
}

/// Sort points using nearest neighbor algorithm for smoother drawing
fn sort_points_nearest_neighbor(points: &[(f32, f32)], max_points: usize) -> Vec<(f32, f32)> {
    if points.is_empty() {
        return Vec::new();
    }

    // If too many points, subsample first
    let working_points: Vec<(f32, f32)> = if points.len() > max_points {
        let step = points.len() / max_points;
        points.iter().step_by(step.max(1)).cloned().collect()
    } else {
        points.to_vec()
    };

    if working_points.len() <= 2 {
        return working_points;
    }

    // Nearest neighbor sorting
    let mut result = Vec::with_capacity(working_points.len());
    let mut used = vec![false; working_points.len()];

    // Start with the point closest to origin
    let mut current_idx = working_points
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let da = a.0 * a.0 + a.1 * a.1;
            let db = b.0 * b.0 + b.1 * b.1;
            da.partial_cmp(&db).unwrap()
        })
        .map(|(i, _)| i)
        .unwrap_or(0);

    result.push(working_points[current_idx]);
    used[current_idx] = true;

    // Greedily pick nearest unused point
    for _ in 1..working_points.len() {
        let current = working_points[current_idx];

        let mut best_idx = None;
        let mut best_dist = f32::MAX;

        for (i, point) in working_points.iter().enumerate() {
            if !used[i] {
                let dx = point.0 - current.0;
                let dy = point.1 - current.1;
                let dist = dx * dx + dy * dy;

                if dist < best_dist {
                    best_dist = dist;
                    best_idx = Some(i);
                }
            }
        }

        if let Some(idx) = best_idx {
            result.push(working_points[idx]);
            used[idx] = true;
            current_idx = idx;
        } else {
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sobel_basic() {
        // Create a simple 10x10 test image with a vertical edge
        let mut img = image::GrayImage::new(10, 10);

        // Left half black, right half white
        for y in 0..10 {
            for x in 0..10 {
                let val = if x < 5 { 0 } else { 255 };
                img.put_pixel(x, y, image::Luma([val]));
            }
        }

        let options = ImageOptions::default();
        let edges = sobel_edge_detection(&img, &options);

        // Should have strong edges in the middle columns
        assert!(!edges.is_empty());

        // Check that middle column has higher values than edges
        let mid_val = edges[5 * 10 + 5]; // Middle of image
        let corner_val = edges[0]; // Corner
        assert!(mid_val > corner_val);
    }

    #[test]
    fn test_nearest_neighbor_sorting() {
        let points = vec![
            (0.0, 0.0),
            (1.0, 0.0),
            (0.1, 0.0),
            (0.9, 0.0),
        ];

        let sorted = sort_points_nearest_neighbor(&points, 100);

        // Should start at origin and visit nearby points first
        assert_eq!(sorted[0], (0.0, 0.0));
        assert_eq!(sorted[1], (0.1, 0.0));
    }

    #[test]
    fn test_extract_points() {
        // Create edge data with some values above threshold
        let edges = vec![
            0.0, 0.0, 0.0, 0.0,
            0.0, 0.5, 0.5, 0.0,
            0.0, 0.5, 0.5, 0.0,
            0.0, 0.0, 0.0, 0.0,
        ];

        let options = ImageOptions {
            threshold: 0.3,
            ..Default::default()
        };

        let points = extract_edge_points(&edges, 4, 4, &options);

        // Should extract 4 points (the center 2x2)
        assert_eq!(points.len(), 4);
    }
}
