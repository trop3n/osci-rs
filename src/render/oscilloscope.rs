//! XY Oscilloscope display widget
//!
//! This widget renders audio samples as 2D XY graphics, similar to
//! a real oscilloscope in XY mode.
//!
//! ## How it works
//!
//! - Left audio channel controls the X (horizontal) position
//! - Right audio channel controls the Y (vertical) position
//! - Samples are drawn as connected lines or points
//! - A persistence effect creates an "afterglow" like a real CRT
//!
//! ## Coordinate System
//!
//! Audio samples range from -1.0 to 1.0
//! - X: -1.0 = left edge, +1.0 = right edge
//! - Y: -1.0 = bottom edge, +1.0 = top edge

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

use crate::audio::XYSample;

/// Display settings for the oscilloscope
#[derive(Clone)]
pub struct OscilloscopeSettings {
    /// Line/point color (RGB)
    pub color: Color32,

    /// Background color
    pub background: Color32,

    /// Line thickness in pixels
    pub line_width: f32,

    /// Whether to draw lines between points (vs just points)
    pub draw_lines: bool,

    /// Intensity/brightness (0.0 to 1.0)
    pub intensity: f32,

    /// Number of samples to display
    pub sample_count: usize,

    /// Zoom/scale factor (1.0 = full range)
    pub zoom: f32,

    /// Whether to show graticule (grid lines)
    pub show_graticule: bool,

    /// Persistence decay factor (0.0 = no persistence, 0.99 = long persistence)
    pub persistence: f32,
}

impl Default for OscilloscopeSettings {
    fn default() -> Self {
        Self {
            color: Color32::from_rgb(100, 255, 100), // Phosphor green
            background: Color32::from_rgb(10, 20, 10),
            line_width: 1.5,
            draw_lines: true,
            intensity: 1.0,
            sample_count: 2048,
            zoom: 1.0,
            show_graticule: true,
            persistence: 0.85,
        }
    }
}

/// XY Oscilloscope widget
///
/// Renders audio samples as 2D graphics in the style of an analog oscilloscope.
pub struct Oscilloscope {
    /// Display settings
    pub settings: OscilloscopeSettings,

    /// Previous frame's points for persistence effect
    /// This creates the "afterglow" seen on CRT oscilloscopes
    persistence_buffer: Vec<(Pos2, f32)>, // (position, alpha)
}

impl Default for Oscilloscope {
    fn default() -> Self {
        Self::new()
    }
}

impl Oscilloscope {
    /// Create a new oscilloscope with default settings
    pub fn new() -> Self {
        Self {
            settings: OscilloscopeSettings::default(),
            persistence_buffer: Vec::with_capacity(8192),
        }
    }

    /// Create a new oscilloscope with custom settings
    pub fn with_settings(settings: OscilloscopeSettings) -> Self {
        Self {
            settings,
            persistence_buffer: Vec::with_capacity(8192),
        }
    }

    /// Convert an XY sample to screen coordinates
    ///
    /// # Arguments
    /// * `sample` - The XY sample (-1.0 to 1.0 range)
    /// * `rect` - The display rectangle
    ///
    /// # Returns
    /// Screen position in pixels
    fn sample_to_screen(&self, sample: XYSample, rect: Rect) -> Pos2 {
        let zoom = self.settings.zoom;

        // Map from [-1, 1] to [0, 1], applying zoom
        let norm_x = (sample.x / zoom + 1.0) / 2.0;
        let norm_y = (sample.y / zoom + 1.0) / 2.0;

        // Map to screen coordinates
        // Note: Y is inverted (screen Y increases downward)
        Pos2::new(
            rect.left() + norm_x * rect.width(),
            rect.bottom() - norm_y * rect.height(), // Flip Y
        )
    }

    /// Draw the oscilloscope display
    ///
    /// # Arguments
    /// * `ui` - The egui UI context
    /// * `samples` - Audio samples to display
    /// * `size` - Desired widget size (or None for available space)
    ///
    /// # Returns
    /// The response from the widget
    pub fn show(&mut self, ui: &mut egui::Ui, samples: &[XYSample], size: Option<Vec2>) -> egui::Response {
        // Determine size
        let size = size.unwrap_or_else(|| {
            let available = ui.available_size();
            let side = available.x.min(available.y).min(400.0);
            Vec2::new(side, side)
        });

        // Allocate space for the widget
        let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
        let rect = response.rect;

        // Draw background
        painter.rect_filled(rect, 4.0, self.settings.background);

        // Draw graticule (grid)
        if self.settings.show_graticule {
            self.draw_graticule(&painter, rect);
        }

        // Update persistence buffer
        self.update_persistence(samples, rect);

        // Draw persistence (afterglow)
        self.draw_persistence(&painter, rect);

        // Draw current samples
        self.draw_samples(&painter, rect, samples);

        response
    }

    /// Draw the graticule (grid lines)
    fn draw_graticule(&self, painter: &egui::Painter, rect: Rect) {
        let grid_color = Color32::from_rgba_unmultiplied(60, 80, 60, 100);
        let axis_color = Color32::from_rgba_unmultiplied(80, 100, 80, 150);

        let stroke_grid = Stroke::new(0.5, grid_color);
        let stroke_axis = Stroke::new(1.0, axis_color);

        // Draw grid lines (10 divisions)
        for i in 0..=10 {
            let t = i as f32 / 10.0;

            // Vertical lines
            let x = rect.left() + t * rect.width();
            let stroke = if i == 5 { stroke_axis } else { stroke_grid };
            painter.line_segment([Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())], stroke);

            // Horizontal lines
            let y = rect.top() + t * rect.height();
            painter.line_segment([Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)], stroke);
        }
    }

    /// Update the persistence buffer with new samples
    fn update_persistence(&mut self, samples: &[XYSample], rect: Rect) {
        let decay = self.settings.persistence;

        // Decay existing persistence
        self.persistence_buffer.retain_mut(|(_, alpha)| {
            *alpha *= decay;
            *alpha > 0.01 // Remove very faded points
        });

        // Add new points
        for sample in samples.iter().take(self.settings.sample_count) {
            let pos = self.sample_to_screen(*sample, rect);
            // Only add if within bounds
            if rect.contains(pos) {
                self.persistence_buffer.push((pos, self.settings.intensity));
            }
        }

        // Limit buffer size to prevent memory growth
        const MAX_PERSISTENCE_POINTS: usize = 50000;
        if self.persistence_buffer.len() > MAX_PERSISTENCE_POINTS {
            let excess = self.persistence_buffer.len() - MAX_PERSISTENCE_POINTS;
            self.persistence_buffer.drain(0..excess);
        }
    }

    /// Draw the persistence effect (afterglow)
    fn draw_persistence(&self, painter: &egui::Painter, rect: Rect) {
        let base_color = self.settings.color;

        for (pos, alpha) in &self.persistence_buffer {
            if !rect.contains(*pos) {
                continue;
            }

            // Fade color based on alpha
            let color = Color32::from_rgba_unmultiplied(
                base_color.r(),
                base_color.g(),
                base_color.b(),
                (alpha * 255.0 * 0.3) as u8, // Persistence is dimmer
            );

            // Draw as small circles for a softer look
            painter.circle_filled(*pos, self.settings.line_width * 0.5, color);
        }
    }

    /// Draw the current samples
    fn draw_samples(&self, painter: &egui::Painter, rect: Rect, samples: &[XYSample]) {
        if samples.is_empty() {
            return;
        }

        let color = Color32::from_rgba_unmultiplied(
            self.settings.color.r(),
            self.settings.color.g(),
            self.settings.color.b(),
            (self.settings.intensity * 255.0) as u8,
        );

        let stroke = Stroke::new(self.settings.line_width, color);

        // Convert samples to screen coordinates
        let points: Vec<Pos2> = samples
            .iter()
            .take(self.settings.sample_count)
            .map(|s| self.sample_to_screen(*s, rect))
            .collect();

        if self.settings.draw_lines && points.len() >= 2 {
            // Draw connected line segments
            for window in points.windows(2) {
                let p1 = window[0];
                let p2 = window[1];

                // Only draw if both points are reasonably close
                // (avoid drawing long lines across the screen for discontinuities)
                let dist_sq = (p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2);
                let max_dist_sq = (rect.width() * 0.5).powi(2);

                if dist_sq < max_dist_sq {
                    painter.line_segment([p1, p2], stroke);
                }
            }
        } else {
            // Draw as points
            for pos in points {
                if rect.contains(pos) {
                    painter.circle_filled(pos, self.settings.line_width, color);
                }
            }
        }
    }

    /// Clear the persistence buffer
    pub fn clear_persistence(&mut self) {
        self.persistence_buffer.clear();
    }
}
