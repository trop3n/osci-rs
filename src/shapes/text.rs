//! Text rendering - convert text to drawable paths
//!
//! This module handles:
//! - Loading font files (TTF, OTF)
//! - Extracting glyph outlines
//! - Converting Bézier curves to point sequences
//! - Text layout and positioning

use ab_glyph::{Font, FontRef, ScaleFont, OutlineCurve};
use std::path::Path as FilePath;
use thiserror::Error;

use super::path::Path;
use super::traits::Shape;

/// Errors that can occur during text rendering
#[derive(Error, Debug)]
pub enum TextError {
    #[error("Failed to read font file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse font: {0}")]
    FontError(String),

    #[error("Text is empty")]
    EmptyText,

    #[error("No glyphs could be rendered")]
    NoGlyphs,
}

/// Options for text rendering
pub struct TextOptions {
    /// Font size in pixels (before normalization)
    pub size: f32,
    /// Number of points per curve segment
    pub curve_samples: usize,
    /// Letter spacing multiplier (1.0 = normal)
    pub letter_spacing: f32,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self {
            size: 64.0,
            curve_samples: 8,
            letter_spacing: 1.0,
        }
    }
}

/// A text string converted to drawable paths
pub struct TextShape {
    /// All points from the text outline
    points: Vec<(f32, f32)>,
    /// The path for rendering
    path: Path,
    /// The original text
    text: String,
}

impl TextShape {
    /// Create text shape from a string using the embedded default font
    pub fn new(text: &str, options: &TextOptions) -> Result<Self, TextError> {
        // Use embedded Roboto Mono font
        let font_data = include_bytes!("../../assets/fonts/RobotoMono-Regular.ttf");
        Self::with_font_data(text, font_data, options)
    }

    /// Create text shape from a string using a font file
    pub fn from_font_file(
        text: &str,
        font_path: impl AsRef<FilePath>,
        options: &TextOptions,
    ) -> Result<Self, TextError> {
        let font_data = std::fs::read(font_path)?;
        Self::with_font_data(text, &font_data, options)
    }

    /// Create text shape from font data bytes
    pub fn with_font_data(
        text: &str,
        font_data: &[u8],
        options: &TextOptions,
    ) -> Result<Self, TextError> {
        if text.is_empty() {
            return Err(TextError::EmptyText);
        }

        let font = FontRef::try_from_slice(font_data)
            .map_err(|e| TextError::FontError(e.to_string()))?;

        Self::render_text(text, &font, options)
    }

    /// Render text using a font
    fn render_text<F: Font>(text: &str, font: &F, options: &TextOptions) -> Result<Self, TextError> {
        let scaled_font = font.as_scaled(options.size);

        let mut all_points: Vec<(f32, f32)> = Vec::new();
        let mut cursor_x = 0.0f32;

        // Process each character
        for ch in text.chars() {
            let glyph_id = font.glyph_id(ch);

            // Get outline for this glyph
            if let Some(outline) = font.outline(glyph_id) {
                let glyph_points = extract_outline_points(
                    &outline.curves,
                    cursor_x,
                    0.0,
                    options.size,
                    options.curve_samples,
                );
                all_points.extend(glyph_points);
            }

            // Advance cursor
            let h_advance = scaled_font.h_advance(glyph_id);
            cursor_x += h_advance * options.letter_spacing;
        }

        if all_points.is_empty() {
            return Err(TextError::NoGlyphs);
        }

        // Normalize points to [-1, 1]
        let normalized = normalize_points(&all_points);

        // Create path
        let path = Path::with_options(normalized.clone(), false, text.to_string());

        Ok(Self {
            points: normalized,
            path,
            text: text.to_string(),
        })
    }

    /// Get the text content
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the number of points
    pub fn point_count(&self) -> usize {
        self.points.len()
    }
}

impl Shape for TextShape {
    fn sample(&self, t: f32) -> (f32, f32) {
        self.path.sample(t)
    }

    fn name(&self) -> &str {
        &self.text
    }

    fn length(&self) -> f32 {
        self.path.length()
    }

    fn is_closed(&self) -> bool {
        false
    }
}

/// Extract points from outline curves
fn extract_outline_points(
    curves: &[OutlineCurve],
    offset_x: f32,
    offset_y: f32,
    scale: f32,
    curve_samples: usize,
) -> Vec<(f32, f32)> {
    let mut points = Vec::new();

    for curve in curves {
        match curve {
            OutlineCurve::Line(p0, p1) => {
                // Add line endpoints
                let start = (p0.x * scale + offset_x, p0.y * scale + offset_y);
                let end = (p1.x * scale + offset_x, p1.y * scale + offset_y);
                points.push(start);
                points.push(end);
            }
            OutlineCurve::Quad(p0, p1, p2) => {
                // Sample quadratic Bézier
                let start = (p0.x * scale + offset_x, p0.y * scale + offset_y);
                let ctrl = (p1.x * scale + offset_x, p1.y * scale + offset_y);
                let end = (p2.x * scale + offset_x, p2.y * scale + offset_y);

                points.push(start);
                for i in 1..=curve_samples {
                    let t = i as f32 / curve_samples as f32;
                    let point = quadratic_bezier(start, ctrl, end, t);
                    points.push(point);
                }
            }
            OutlineCurve::Cubic(p0, p1, p2, p3) => {
                // Sample cubic Bézier
                let start = (p0.x * scale + offset_x, p0.y * scale + offset_y);
                let ctrl1 = (p1.x * scale + offset_x, p1.y * scale + offset_y);
                let ctrl2 = (p2.x * scale + offset_x, p2.y * scale + offset_y);
                let end = (p3.x * scale + offset_x, p3.y * scale + offset_y);

                points.push(start);
                for i in 1..=curve_samples {
                    let t = i as f32 / curve_samples as f32;
                    let point = cubic_bezier(start, ctrl1, ctrl2, end, t);
                    points.push(point);
                }
            }
        }
    }

    points
}

/// Evaluate a quadratic Bézier curve at parameter t
fn quadratic_bezier(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), t: f32) -> (f32, f32) {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;

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
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    let t2 = t * t;
    let t3 = t2 * t;

    let x = mt3 * p0.0 + 3.0 * mt2 * t * p1.0 + 3.0 * mt * t2 * p2.0 + t3 * p3.0;
    let y = mt3 * p0.1 + 3.0 * mt2 * t * p1.1 + 3.0 * mt * t2 * p2.1 + t3 * p3.1;

    (x, y)
}

/// Normalize points to [-1, 1] range, centered
fn normalize_points(points: &[(f32, f32)]) -> Vec<(f32, f32)> {
    if points.is_empty() {
        return Vec::new();
    }

    // Find bounding box
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for &(x, y) in points {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }

    let width = max_x - min_x;
    let height = max_y - min_y;
    let scale = width.max(height);

    if scale <= 0.0 {
        return points.to_vec();
    }

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    points
        .iter()
        .map(|&(x, y)| {
            let nx = (x - center_x) / (scale / 2.0);
            let ny = -(y - center_y) / (scale / 2.0); // Flip Y for screen coords
            (nx.clamp(-1.0, 1.0), ny.clamp(-1.0, 1.0))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadratic_bezier() {
        let p0 = (0.0, 0.0);
        let p1 = (0.5, 1.0);
        let p2 = (1.0, 0.0);

        let (x, y) = quadratic_bezier(p0, p1, p2, 0.0);
        assert!((x - 0.0).abs() < 0.001);
        assert!((y - 0.0).abs() < 0.001);

        let (x, y) = quadratic_bezier(p0, p1, p2, 1.0);
        assert!((x - 1.0).abs() < 0.001);
        assert!((y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_cubic_bezier() {
        let p0 = (0.0, 0.0);
        let p1 = (0.33, 1.0);
        let p2 = (0.66, 1.0);
        let p3 = (1.0, 0.0);

        let (x, y) = cubic_bezier(p0, p1, p2, p3, 0.0);
        assert!((x - 0.0).abs() < 0.001);
        assert!((y - 0.0).abs() < 0.001);

        let (x, y) = cubic_bezier(p0, p1, p2, p3, 1.0);
        assert!((x - 1.0).abs() < 0.001);
        assert!((y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_points() {
        let points = vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];
        let normalized = normalize_points(&points);

        // All points should be within [-1, 1]
        for &(x, y) in &normalized {
            assert!(x >= -1.0 && x <= 1.0);
            assert!(y >= -1.0 && y <= 1.0);
        }
    }

    #[test]
    fn test_text_shape_creation() {
        let options = TextOptions::default();
        let result = TextShape::new("Hi", &options);
        assert!(result.is_ok(), "Failed to create text shape: {:?}", result.err());

        let text_shape = result.unwrap();
        assert_eq!(text_shape.text(), "Hi");
        assert!(text_shape.point_count() > 0);
    }
}
