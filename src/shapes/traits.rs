//! Shape trait definition
//!
//! The `Shape` trait is the core abstraction for drawable shapes.
//! Any type that implements `Shape` can be rendered as audio output.
//!
//! ## Trait Basics
//!
//! A trait in Rust is like an interface - it defines behavior that types
//! can implement. Unlike interfaces in other languages, traits can also
//! provide default implementations.
//!
//! ```rust
//! // Defining a trait
//! pub trait Shape {
//!     fn sample(&self, t: f32) -> (f32, f32);
//! }
//!
//! // Implementing for a type
//! impl Shape for Circle {
//!     fn sample(&self, t: f32) -> (f32, f32) {
//!         // ...
//!     }
//! }
//! ```

/// A shape that can be drawn on an oscilloscope
///
/// Shapes are defined parametrically - the `sample` method takes a parameter
/// `t` in the range [0, 1) and returns the (x, y) coordinates at that point
/// along the shape's path.
///
/// ## Parametric Representation
///
/// - `t = 0.0` → Start of the shape
/// - `t = 0.5` → Halfway through
/// - `t = 1.0` → End (wraps back to start for closed shapes)
///
/// ## Coordinate System
///
/// - X and Y range from -1.0 to 1.0
/// - (0, 0) is the center
/// - (-1, -1) is bottom-left, (1, 1) is top-right
///
/// ## Thread Safety
///
/// Shapes must be `Send + Sync` so they can be shared with the audio thread.
/// This is required because the audio callback runs on a separate thread.
pub trait Shape: Send + Sync {
    /// Sample the shape at parameter t
    ///
    /// # Arguments
    /// * `t` - Parameter in range [0, 1), representing position along the shape
    ///
    /// # Returns
    /// (x, y) coordinates, each in range [-1, 1]
    fn sample(&self, t: f32) -> (f32, f32);

    /// Get the name of this shape (for UI display)
    fn name(&self) -> &str;

    /// Get the approximate "length" of the shape
    ///
    /// This is used for uniform sampling - shapes with longer paths
    /// need more samples to look smooth.
    ///
    /// Default implementation returns 1.0 (suitable for simple shapes).
    fn length(&self) -> f32 {
        1.0
    }

    /// Whether this shape is closed (end connects to start)
    ///
    /// Closed shapes (like circles) wrap around.
    /// Open shapes (like lines) have distinct endpoints.
    fn is_closed(&self) -> bool {
        true
    }
}

/// A boxed shape for dynamic dispatch
///
/// When you need to store different shape types in the same collection,
/// use `Box<dyn Shape>`. This uses dynamic dispatch (runtime polymorphism)
/// instead of static dispatch (compile-time monomorphization).
///
/// ## Example
///
/// ```rust
/// let shapes: Vec<Box<dyn Shape>> = vec![
///     Box::new(Circle::new(0.5)),
///     Box::new(Rectangle::new(0.8, 0.4)),
/// ];
/// ```
pub type BoxedShape = Box<dyn Shape>;
