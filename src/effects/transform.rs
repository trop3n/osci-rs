//! Transform effects - Rotate, Scale, Translate
//!
//! These effects apply geometric transformations to shape coordinates.
#![allow(dead_code)]

use super::traits::Effect;

/// Rotation effect
///
/// Rotates points around the origin by a fixed angle plus optional animation.
pub struct Rotate {
    /// Base rotation angle in radians
    pub angle: f32,
    /// Rotation speed in radians per second (0 = static)
    pub speed: f32,
    /// Whether the effect is enabled
    pub enabled: bool,
}

impl Rotate {
    /// Create a static rotation
    pub fn new(angle: f32) -> Self {
        Self {
            angle,
            speed: 0.0,
            enabled: true,
        }
    }

    /// Create an animated rotation
    pub fn animated(speed: f32) -> Self {
        Self {
            angle: 0.0,
            speed,
            enabled: true,
        }
    }

    /// Create a rotation with both base angle and animation
    pub fn with_speed(angle: f32, speed: f32) -> Self {
        Self {
            angle,
            speed,
            enabled: true,
        }
    }
}

impl Effect for Rotate {
    fn apply(&self, x: f32, y: f32, time: f32) -> (f32, f32) {
        let total_angle = self.angle + self.speed * time;
        let cos_a = total_angle.cos();
        let sin_a = total_angle.sin();

        // 2D rotation matrix
        let new_x = x * cos_a - y * sin_a;
        let new_y = x * sin_a + y * cos_a;

        (new_x, new_y)
    }

    fn name(&self) -> &str {
        "Rotate"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Scale effect
///
/// Scales points relative to the origin.
pub struct Scale {
    /// X scale factor (1.0 = no change)
    pub x: f32,
    /// Y scale factor (1.0 = no change)
    pub y: f32,
    /// Whether the effect is enabled
    pub enabled: bool,
}

impl Scale {
    /// Create a uniform scale
    pub fn uniform(factor: f32) -> Self {
        Self {
            x: factor,
            y: factor,
            enabled: true,
        }
    }

    /// Create a non-uniform scale
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            enabled: true,
        }
    }
}

impl Effect for Scale {
    fn apply(&self, x: f32, y: f32, _time: f32) -> (f32, f32) {
        (x * self.x, y * self.y)
    }

    fn name(&self) -> &str {
        "Scale"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Translate effect
///
/// Moves points by a fixed offset.
pub struct Translate {
    /// X offset
    pub x: f32,
    /// Y offset
    pub y: f32,
    /// Whether the effect is enabled
    pub enabled: bool,
}

impl Translate {
    /// Create a translation
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            enabled: true,
        }
    }
}

impl Effect for Translate {
    fn apply(&self, x: f32, y: f32, _time: f32) -> (f32, f32) {
        (x + self.x, y + self.y)
    }

    fn name(&self) -> &str {
        "Translate"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Mirror effect
///
/// Mirrors points across an axis.
#[derive(Clone, Copy, PartialEq)]
pub enum MirrorAxis {
    Horizontal, // Mirror across Y axis (flip X)
    Vertical,   // Mirror across X axis (flip Y)
    Both,       // Mirror across both axes
}

pub struct Mirror {
    /// Which axis to mirror across
    pub axis: MirrorAxis,
    /// Whether the effect is enabled
    pub enabled: bool,
}

impl Mirror {
    pub fn new(axis: MirrorAxis) -> Self {
        Self {
            axis,
            enabled: true,
        }
    }

    pub fn horizontal() -> Self {
        Self::new(MirrorAxis::Horizontal)
    }

    pub fn vertical() -> Self {
        Self::new(MirrorAxis::Vertical)
    }
}

impl Effect for Mirror {
    fn apply(&self, x: f32, y: f32, _time: f32) -> (f32, f32) {
        match self.axis {
            MirrorAxis::Horizontal => (-x, y),
            MirrorAxis::Vertical => (x, -y),
            MirrorAxis::Both => (-x, -y),
        }
    }

    fn name(&self) -> &str {
        "Mirror"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, PI};

    #[test]
    fn test_rotate_90_degrees() {
        let rotate = Rotate::new(FRAC_PI_2);
        let (x, y) = rotate.apply(1.0, 0.0, 0.0);
        assert!((x - 0.0).abs() < 0.001);
        assert!((y - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_rotate_180_degrees() {
        let rotate = Rotate::new(PI);
        let (x, y) = rotate.apply(1.0, 0.0, 0.0);
        assert!((x - (-1.0)).abs() < 0.001);
        assert!(y.abs() < 0.001);
    }

    #[test]
    fn test_scale() {
        let scale = Scale::uniform(2.0);
        let (x, y) = scale.apply(0.5, 0.5, 0.0);
        assert!((x - 1.0).abs() < 0.001);
        assert!((y - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_translate() {
        let translate = Translate::new(0.5, -0.5);
        let (x, y) = translate.apply(0.0, 0.0, 0.0);
        assert!((x - 0.5).abs() < 0.001);
        assert!((y - (-0.5)).abs() < 0.001);
    }

    #[test]
    fn test_mirror() {
        let mirror = Mirror::horizontal();
        let (x, y) = mirror.apply(0.5, 0.3, 0.0);
        assert!((x - (-0.5)).abs() < 0.001);
        assert!((y - 0.3).abs() < 0.001);
    }
}
