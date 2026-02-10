//! Effect trait - defines transformations applied to shapes
//!
//! Effects modify XY coordinates as they pass through the rendering pipeline.
//! They can be chained together and modulated by LFOs.

/// An effect that transforms XY coordinates
///
/// Effects are applied after shape sampling, modifying the output coordinates.
/// The `time` parameter allows time-based effects like rotation animation.
pub trait Effect: Send + Sync {
    /// Apply the effect to an XY point
    ///
    /// # Arguments
    /// * `x` - Input X coordinate (-1.0 to 1.0)
    /// * `y` - Input Y coordinate (-1.0 to 1.0)
    /// * `time` - Current time in seconds (for animation)
    ///
    /// # Returns
    /// Transformed (x, y) coordinates
    fn apply(&self, x: f32, y: f32, time: f32) -> (f32, f32);

    /// Get the name of this effect (for UI)
    fn name(&self) -> &str;

    /// Whether this effect is currently enabled
    fn is_enabled(&self) -> bool {
        true
    }
}

/// A boxed effect for dynamic dispatch
pub type BoxedEffect = Box<dyn Effect>;

/// A chain of effects applied in sequence
pub struct EffectChain {
    effects: Vec<BoxedEffect>,
}

impl EffectChain {
    /// Create an empty effect chain
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    /// Add an effect to the chain
    pub fn add<E: Effect + 'static>(&mut self, effect: E) -> &mut Self {
        self.effects.push(Box::new(effect));
        self
    }

    /// Remove an effect by index
    pub fn remove(&mut self, index: usize) -> Option<BoxedEffect> {
        if index < self.effects.len() {
            Some(self.effects.remove(index))
        } else {
            None
        }
    }

    /// Get the number of effects
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    /// Apply all effects in sequence
    pub fn apply(&self, x: f32, y: f32, time: f32) -> (f32, f32) {
        let mut result = (x, y);
        for effect in &self.effects {
            if effect.is_enabled() {
                result = effect.apply(result.0, result.1, time);
            }
        }
        result
    }

    /// Clear all effects
    pub fn clear(&mut self) {
        self.effects.clear();
    }
}

impl Default for EffectChain {
    fn default() -> Self {
        Self::new()
    }
}
