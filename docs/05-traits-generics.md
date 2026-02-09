# Traits and Generics in Rust

This document covers how we use traits and generics to create an extensible shape system for osci-rs.

## What Are Traits?

Traits define shared behavior. They're similar to interfaces in other languages, but with some key differences:
- Traits can provide **default implementations**
- Traits can be implemented for types you don't own (with restrictions)
- Traits enable both static dispatch (generics) and dynamic dispatch (`dyn Trait`)

## The Shape Trait

Our core abstraction is the `Shape` trait:

```rust
// src/shapes/traits.rs

pub trait Shape: Send + Sync {
    /// Sample the shape at parameter t (0.0 to 1.0)
    fn sample(&self, t: f32) -> (f32, f32);

    /// Get the name of this shape (for UI display)
    fn name(&self) -> &str;

    /// Get the approximate "length" of the shape
    fn length(&self) -> f32 {
        1.0  // Default implementation
    }

    /// Whether this shape is closed (end connects to start)
    fn is_closed(&self) -> bool {
        true  // Default implementation
    }
}
```

### Trait Bounds: `Send + Sync`

The `: Send + Sync` part is a **supertrait bound**. It means:
- Any type implementing `Shape` must also implement `Send` and `Sync`
- `Send`: Can be transferred to another thread
- `Sync`: Can be referenced from multiple threads

This is crucial because the audio thread needs to access shape data.

### Required vs Default Methods

- `sample()` and `name()` are **required** - every implementor must provide them
- `length()` and `is_closed()` have **default implementations** - implementors can override or use the default

## Implementing Traits

Here's how we implement `Shape` for a circle:

```rust
// src/shapes/primitives.rs

use std::f32::consts::TAU;

pub struct Circle {
    pub cx: f32,
    pub cy: f32,
    pub radius: f32,
}

impl Shape for Circle {
    fn sample(&self, t: f32) -> (f32, f32) {
        let angle = t * TAU;  // t=0→0, t=1→2π
        let x = self.cx + self.radius * angle.cos();
        let y = self.cy + self.radius * angle.sin();
        (x, y)
    }

    fn name(&self) -> &str {
        "Circle"
    }

    fn length(&self) -> f32 {
        TAU * self.radius  // Circumference = 2πr
    }

    // is_closed() uses the default (true)
}
```

## Generics and Trait Bounds

Generics let us write code that works with any type meeting certain criteria.

### Generic Functions

```rust
// src/audio/engine.rs

impl AudioEngine {
    pub fn set_shape<S: Shape>(&mut self, shape: &S) {
        // S can be any type that implements Shape
        for i in 0..self.samples_per_shape {
            let t = i as f32 / self.samples_per_shape as f32;
            let (x, y) = shape.sample(t);  // Works for any Shape
            // ...
        }
    }
}
```

The `<S: Shape>` syntax means:
- `S` is a type parameter (placeholder for a concrete type)
- `: Shape` is a **trait bound** - `S` must implement `Shape`

### Calling Generic Functions

```rust
let circle = Circle::new(0.8);
let rectangle = Rectangle::new(1.2, 0.6);

audio.set_shape(&circle);     // S = Circle
audio.set_shape(&rectangle);  // S = Rectangle
```

The compiler generates specialized code for each type - this is **monomorphization**.

## Static vs Dynamic Dispatch

### Static Dispatch (Generics)

```rust
fn process_shape<S: Shape>(shape: &S) {
    let point = shape.sample(0.5);
}
```

- Compiler generates separate code for each concrete type
- No runtime overhead
- Larger binary size
- Type must be known at compile time

### Dynamic Dispatch (`dyn Trait`)

```rust
fn process_shape(shape: &dyn Shape) {
    let point = shape.sample(0.5);
}
```

- Single code path for all types
- Small runtime overhead (vtable lookup)
- Smaller binary size
- Type can be determined at runtime

### When to Use Each

| Use Case | Approach |
|----------|----------|
| Performance-critical inner loops | Generics |
| Collections of mixed types | `dyn Trait` |
| Plugin systems | `dyn Trait` |
| Simple, known types | Generics |

## The Parametric Shape Model

All our shapes use **parametric equations**:

```
(x, y) = f(t) where t ∈ [0, 1)
```

This maps naturally to audio generation:
- t=0.0 → Start of shape
- t=0.5 → Halfway through
- t=1.0 → End (wraps to start for closed shapes)

### Examples

**Circle:**
```
x = cx + radius × cos(t × 2π)
y = cy + radius × sin(t × 2π)
```

**Line:**
```
x = x1 + t × (x2 - x1)
y = y1 + t × (y2 - y1)
```

**Rectangle (4 segments):**
```
segment = floor(t × 4)
local_t = fract(t × 4)
interpolate between corners[segment] and corners[segment+1]
```

## Uniform Sampling Along Path Length

For smooth rendering, we sample uniformly along the path length, not uniformly in t.

Consider a polygon with edges of different lengths:
- Uniform t: Fewer samples on short edges, more on long edges
- Uniform length: Equal sample density everywhere

```rust
// src/shapes/primitives.rs - Polygon sampling

fn sample(&self, t: f32) -> (f32, f32) {
    // Convert t to distance along perimeter
    let target_dist = t * self.total_length;
    let mut accumulated = 0.0;

    for (i, &edge_len) in self.edge_lengths.iter().enumerate() {
        if accumulated + edge_len >= target_dist {
            // We're on this edge
            let local_t = (target_dist - accumulated) / edge_len;
            // Interpolate...
        }
        accumulated += edge_len;
    }
}
```

## Creating New Shapes

To add a new shape type:

1. **Define the struct:**
```rust
pub struct Ellipse {
    pub cx: f32,
    pub cy: f32,
    pub rx: f32,  // X radius
    pub ry: f32,  // Y radius
}
```

2. **Implement constructors:**
```rust
impl Ellipse {
    pub fn new(rx: f32, ry: f32) -> Self {
        Self { cx: 0.0, cy: 0.0, rx, ry }
    }
}
```

3. **Implement the Shape trait:**
```rust
impl Shape for Ellipse {
    fn sample(&self, t: f32) -> (f32, f32) {
        let angle = t * TAU;
        (
            self.cx + self.rx * angle.cos(),
            self.cy + self.ry * angle.sin(),
        )
    }

    fn name(&self) -> &str {
        "Ellipse"
    }

    // Approximate length (ellipse perimeter is complex)
    fn length(&self) -> f32 {
        // Ramanujan's approximation
        let a = self.rx;
        let b = self.ry;
        std::f32::consts::PI * (3.0 * (a + b) - ((3.0*a + b) * (a + 3.0*b)).sqrt())
    }
}
```

## Key Takeaways

1. **Traits define shared behavior** - implement once, use everywhere
2. **Trait bounds constrain generics** - `<S: Shape>` means "S must implement Shape"
3. **`Send + Sync` enables thread safety** - required for audio callbacks
4. **Default implementations reduce boilerplate** - override only what differs
5. **Generics = static dispatch** - fast, but type known at compile time
6. **`dyn Trait` = dynamic dispatch** - flexible, small runtime cost
7. **Parametric equations** map naturally to audio sample generation

## Exercises

1. **Add an Ellipse shape** with separate X and Y radii
2. **Create a `RoundedRectangle`** that uses arcs for corners
3. **Implement a `Superellipse`** (squircle) using the parametric form:
   `x = |cos(t)|^(2/n) × sign(cos(t)) × a`
4. **Make Polygon return "Triangle", "Pentagon", etc.** in `name()` based on vertex count

## Further Reading

- [The Rust Book - Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [The Rust Book - Generics](https://doc.rust-lang.org/book/ch10-01-syntax.html)
- [Rust by Example - Traits](https://doc.rust-lang.org/rust-by-example/trait.html)
- [Parametric Equations (Wikipedia)](https://en.wikipedia.org/wiki/Parametric_equation)
