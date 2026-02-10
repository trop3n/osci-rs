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

#[allow(unused_imports)]
pub use traits::{Effect, EffectChain, BoxedEffect};
#[allow(unused_imports)]
pub use transform::{Rotate, Scale, Translate, Mirror, MirrorAxis};
#[allow(unused_imports)]
pub use lfo::{Lfo, LfoWaveform, LfoRotate, LfoScale, LfoTranslate};
