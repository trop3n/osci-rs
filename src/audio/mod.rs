//! Audio module - handles audio I/O and sample buffering
//!
//! This module provides:
//! - Ring buffer for thread-safe sample sharing
//! - Audio engine for cpal integration

mod buffer;
mod engine;

// Re-export public types
pub use buffer::{SampleBuffer, XYSample};
pub use engine::AudioEngine;
