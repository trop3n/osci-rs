//! Effects module - transformations and modulations for shapes
//!
//! This module provides:
//! - `Effect` trait for defining transformations
//! - Transform effects: Rotate, Scale, Translate, Mirror
//! - LFO (Low Frequency Oscillator) for parameter modulation
//! - LFO-modulated effects: LfoRotate, LfoScale, LfoTranslate

mod lfo;
mod traits;
mod transform;

#[allow(unused_imports)]
pub use lfo::{Lfo, LfoRotate, LfoScale, LfoTranslate, LfoWaveform};
#[allow(unused_imports)]
pub use traits::{BoxedEffect, Effect, EffectChain};
#[allow(unused_imports)]
pub use transform::{Mirror, MirrorAxis, Rotate, Scale, Translate};
