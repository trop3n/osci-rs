//! Render module - UI components for visualization
//!
//! This module provides:
//! - XY oscilloscope display widget
//! - Waveform display (future)

mod oscilloscope;

#[allow(unused_imports)]
pub use oscilloscope::{Oscilloscope, OscilloscopeSettings};
