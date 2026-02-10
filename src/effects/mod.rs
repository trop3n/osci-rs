//! Effects module - transformations and modulations for shapes
//!
//! This module provides:
//! - `Effect` trait for defining transformations
//! - Transform effects: Rotate, Scale, Translate, Mirror
//! - LFO (Low Frequency Oscillator) for parameter modulation
//! - LFO-modulated effects: LfoRotate, LfoScale, LfoTranslate

mod traits;
mod transform;
mod lfo;

pub use traits::{Effect, EffectChain, BoxedEffect};
pub use transform::{Rotate, Scale, Translate, Mirror, MirrorAxis};
pub use lfo::{Lfo, LfoWaveform, LfoRotate, LfoScale, LfoTranslate};
