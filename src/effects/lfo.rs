//! LFO - Low Frequency Oscillator
//!
//! LFOs generate periodic signals used to modulate effect parameters.
//! They oscillate between a minimum and maximum value at a specified frequency.

use std::f32::consts::TAU;

/// LFO waveform shapes
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LfoWaveform {
    /// Smooth sine wave
    Sine,
    /// Linear triangle wave
    Triangle,
    /// Abrupt square wave
    Square,
    /// Rising sawtooth
    Sawtooth,
    /// Falling sawtooth (reverse)
    ReverseSawtooth,
}

impl LfoWaveform {
    /// Get all waveform types
    pub fn all() -> &'static [LfoWaveform] {
        &[
            LfoWaveform::Sine,
            LfoWaveform::Triangle,
            LfoWaveform::Square,
            LfoWaveform::Sawtooth,
            LfoWaveform::ReverseSawtooth,
        ]
    }

    /// Get the name of this waveform
    pub fn name(&self) -> &'static str {
        match self {
            LfoWaveform::Sine => "Sine",
            LfoWaveform::Triangle => "Triangle",
            LfoWaveform::Square => "Square",
            LfoWaveform::Sawtooth => "Sawtooth",
            LfoWaveform::ReverseSawtooth => "Rev Saw",
        }
    }

    /// Sample the waveform at phase (0.0 to 1.0)
    /// Returns value in range -1.0 to 1.0
    pub fn sample(&self, phase: f32) -> f32 {
        match self {
            LfoWaveform::Sine => (phase * TAU).sin(),

            LfoWaveform::Triangle => {
                let p = phase * 4.0;
                if p < 1.0 {
                    p
                } else if p < 3.0 {
                    2.0 - p
                } else {
                    p - 4.0
                }
            }

            LfoWaveform::Square => {
                if phase < 0.5 { 1.0 } else { -1.0 }
            }

            LfoWaveform::Sawtooth => {
                2.0 * phase - 1.0
            }

            LfoWaveform::ReverseSawtooth => {
                1.0 - 2.0 * phase
            }
        }
    }
}

/// Low Frequency Oscillator
///
/// Generates a periodic signal for modulating parameters.
#[derive(Clone)]
pub struct Lfo {
    /// Oscillation frequency in Hz
    pub frequency: f32,
    /// Waveform shape
    pub waveform: LfoWaveform,
    /// Minimum output value
    pub min: f32,
    /// Maximum output value
    pub max: f32,
    /// Phase offset (0.0 to 1.0)
    pub phase_offset: f32,
    /// Whether the LFO is enabled
    pub enabled: bool,
}

impl Lfo {
    /// Create a new LFO with default settings
    pub fn new(frequency: f32) -> Self {
        Self {
            frequency,
            waveform: LfoWaveform::Sine,
            min: -1.0,
            max: 1.0,
            phase_offset: 0.0,
            enabled: true,
        }
    }

    /// Create an LFO with specified range
    pub fn with_range(frequency: f32, min: f32, max: f32) -> Self {
        Self {
            frequency,
            waveform: LfoWaveform::Sine,
            min,
            max,
            phase_offset: 0.0,
            enabled: true,
        }
    }

    /// Set the waveform
    pub fn waveform(mut self, waveform: LfoWaveform) -> Self {
        self.waveform = waveform;
        self
    }

    /// Set the phase offset
    pub fn phase(mut self, offset: f32) -> Self {
        self.phase_offset = offset;
        self
    }

    /// Sample the LFO at a given time
    ///
    /// # Arguments
    /// * `time` - Current time in seconds
    ///
    /// # Returns
    /// Value between `min` and `max`
    pub fn sample(&self, time: f32) -> f32 {
        if !self.enabled {
            return (self.min + self.max) / 2.0; // Return center value when disabled
        }

        // Calculate phase (0.0 to 1.0)
        let phase = ((time * self.frequency) + self.phase_offset).fract();
        let phase = if phase < 0.0 { phase + 1.0 } else { phase };

        // Get waveform value (-1.0 to 1.0)
        let raw = self.waveform.sample(phase);

        // Map to output range
        let normalized = (raw + 1.0) / 2.0; // 0.0 to 1.0
        self.min + normalized * (self.max - self.min)
    }
}

impl Default for Lfo {
    fn default() -> Self {
        Self::new(1.0)
    }
}

/// An effect modulated by an LFO
///
/// This wraps an effect and modulates one of its parameters.
use super::traits::Effect;
use super::transform::Rotate;

/// Rotation effect with LFO modulation
pub struct LfoRotate {
    /// Base rotation angle
    pub base_angle: f32,
    /// LFO for angle modulation
    pub lfo: Lfo,
    /// Whether the effect is enabled
    pub enabled: bool,
}

impl LfoRotate {
    /// Create a new LFO-modulated rotation
    ///
    /// # Arguments
    /// * `lfo_frequency` - LFO frequency in Hz
    /// * `angle_range` - Maximum rotation angle (will oscillate ±angle_range)
    pub fn new(lfo_frequency: f32, angle_range: f32) -> Self {
        Self {
            base_angle: 0.0,
            lfo: Lfo::with_range(lfo_frequency, -angle_range, angle_range),
            enabled: true,
        }
    }

    /// Set the LFO waveform
    pub fn waveform(mut self, waveform: LfoWaveform) -> Self {
        self.lfo.waveform = waveform;
        self
    }
}

impl Effect for LfoRotate {
    fn apply(&self, x: f32, y: f32, time: f32) -> (f32, f32) {
        let angle = self.base_angle + self.lfo.sample(time);
        let rotate = Rotate::new(angle);
        rotate.apply(x, y, time)
    }

    fn name(&self) -> &str {
        "LFO Rotate"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Scale effect with LFO modulation (pulsing)
pub struct LfoScale {
    /// Base scale factor
    pub base_scale: f32,
    /// LFO for scale modulation
    pub lfo: Lfo,
    /// Whether the effect is enabled
    pub enabled: bool,
}

impl LfoScale {
    /// Create a pulsing scale effect
    ///
    /// # Arguments
    /// * `lfo_frequency` - Pulse frequency in Hz
    /// * `min_scale` - Minimum scale factor
    /// * `max_scale` - Maximum scale factor
    pub fn new(lfo_frequency: f32, min_scale: f32, max_scale: f32) -> Self {
        Self {
            base_scale: 1.0,
            lfo: Lfo::with_range(lfo_frequency, min_scale, max_scale),
            enabled: true,
        }
    }

    /// Set the LFO waveform
    pub fn waveform(mut self, waveform: LfoWaveform) -> Self {
        self.lfo.waveform = waveform;
        self
    }
}

impl Effect for LfoScale {
    fn apply(&self, x: f32, y: f32, time: f32) -> (f32, f32) {
        let scale = self.lfo.sample(time);
        (x * scale, y * scale)
    }

    fn name(&self) -> &str {
        "LFO Scale"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Translation with LFO modulation (wobble)
pub struct LfoTranslate {
    /// LFO for X movement
    pub lfo_x: Lfo,
    /// LFO for Y movement
    pub lfo_y: Lfo,
    /// Whether the effect is enabled
    pub enabled: bool,
}

impl LfoTranslate {
    /// Create a wobble effect
    ///
    /// # Arguments
    /// * `frequency` - Wobble frequency in Hz
    /// * `amount` - Maximum displacement
    pub fn new(frequency: f32, amount: f32) -> Self {
        Self {
            lfo_x: Lfo::with_range(frequency, -amount, amount),
            lfo_y: Lfo::with_range(frequency, -amount, amount).phase(0.25), // 90° offset
            enabled: true,
        }
    }

    /// Create with separate X and Y frequencies
    pub fn separate(freq_x: f32, freq_y: f32, amount: f32) -> Self {
        Self {
            lfo_x: Lfo::with_range(freq_x, -amount, amount),
            lfo_y: Lfo::with_range(freq_y, -amount, amount),
            enabled: true,
        }
    }
}

impl Effect for LfoTranslate {
    fn apply(&self, x: f32, y: f32, time: f32) -> (f32, f32) {
        let dx = self.lfo_x.sample(time);
        let dy = self.lfo_y.sample(time);
        (x + dx, y + dy)
    }

    fn name(&self) -> &str {
        "LFO Translate"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lfo_sine() {
        let lfo = Lfo::new(1.0); // 1 Hz

        // At t=0, sine starts at 0 -> maps to center of range
        let v = lfo.sample(0.0);
        assert!((v - 0.0).abs() < 0.01);

        // At t=0.25 (quarter period), sine is at max
        let v = lfo.sample(0.25);
        assert!((v - 1.0).abs() < 0.01);

        // At t=0.75 (three-quarter period), sine is at min
        let v = lfo.sample(0.75);
        assert!((v - (-1.0)).abs() < 0.01);
    }

    #[test]
    fn test_lfo_range() {
        let lfo = Lfo::with_range(1.0, 0.0, 10.0);

        // Should oscillate between 0 and 10
        let v = lfo.sample(0.25); // Max
        assert!((v - 10.0).abs() < 0.1);

        let v = lfo.sample(0.75); // Min
        assert!((v - 0.0).abs() < 0.1);
    }

    #[test]
    fn test_lfo_square() {
        let lfo = Lfo::new(1.0).waveform(LfoWaveform::Square);

        let v = lfo.sample(0.25); // First half
        assert!((v - 1.0).abs() < 0.01);

        let v = lfo.sample(0.75); // Second half
        assert!((v - (-1.0)).abs() < 0.01);
    }
}
