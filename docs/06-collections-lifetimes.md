# Collections, Smart Pointers, and Dynamic Dispatch

This document covers how we use collections and dynamic dispatch to build the Scene composition system.

## The Problem

In Milestone 4, we could only render one shape at a time. Now we want to:
- Combine multiple shapes into a single scene
- Allocate different amounts of time to each shape
- Enable/disable shapes dynamically
- Reorder shapes at runtime

## Vec - The Workhorse Collection

Rust's `Vec<T>` is a growable array, similar to `ArrayList` in Java or `vector` in C++.

```rust
// Creating vectors
let mut shapes: Vec<Circle> = Vec::new();
shapes.push(Circle::new(0.5));
shapes.push(Circle::new(0.3));

// Or with the vec! macro
let shapes = vec![Circle::new(0.5), Circle::new(0.3)];

// Iterating
for shape in &shapes {
    println!("{}", shape.name());
}

// With index
for (i, shape) in shapes.iter().enumerate() {
    println!("{}: {}", i, shape.name());
}
```

### Key Vec Operations

| Operation | Method | Notes |
|-----------|--------|-------|
| Add to end | `push(item)` | O(1) amortized |
| Remove from end | `pop()` | Returns `Option<T>` |
| Remove at index | `remove(i)` | O(n), shifts elements |
| Swap elements | `swap(i, j)` | O(1) |
| Get length | `len()` | |
| Check empty | `is_empty()` | |
| Get by index | `get(i)` | Returns `Option<&T>` |

## The Heterogeneous Collection Problem

What if we want a vector containing different shape types?

```rust
// This won't compile!
let shapes = vec![
    Circle::new(0.5),      // Type: Circle
    Rectangle::new(1.0, 0.6), // Type: Rectangle
];
// Error: expected Circle, found Rectangle
```

Rust vectors are homogeneous - all elements must be the same type.

## Solution: Trait Objects with Box

We use `Box<dyn Shape>` to store different types that implement `Shape`:

```rust
let shapes: Vec<Box<dyn Shape>> = vec![
    Box::new(Circle::new(0.5)),
    Box::new(Rectangle::new(1.0, 0.6)),
];
```

### What's Happening Here?

1. **`Box<T>`** - A smart pointer that allocates on the heap
   - Gives us a fixed-size pointer (usize) regardless of T's size
   - Owns the data and deallocates when dropped

2. **`dyn Shape`** - A trait object
   - "Dynamic" dispatch - method calls resolved at runtime
   - Includes a vtable pointer for method lookup

3. **`Box<dyn Shape>`** - Combined
   - Fixed size (two pointers: data + vtable)
   - Can store any type implementing Shape
   - Enables runtime polymorphism

## The Scene Implementation

```rust
// src/shapes/scene.rs

pub struct SceneShape {
    shape: Box<dyn Shape>,  // Any shape type
    weight: f32,            // Time allocation weight
    enabled: bool,          // Can be toggled
}

pub struct Scene {
    shapes: Vec<SceneShape>,
    boundaries: Vec<(f32, f32, usize)>,  // Cached time segments
    name: String,
}
```

### Time Allocation Algorithm

When sampling the scene, we divide time proportionally by weight:

```rust
// With weights [2.0, 1.0, 1.0]:
// Shape 0: t ∈ [0.0, 0.5)    (2/4 = 50%)
// Shape 1: t ∈ [0.5, 0.75)   (1/4 = 25%)
// Shape 2: t ∈ [0.75, 1.0)   (1/4 = 25%)

fn sample(&self, t: f32) -> (f32, f32) {
    // Find which shape owns this time segment
    for &(start, end, idx) in &self.boundaries {
        if t >= start && t < end {
            // Remap t to [0, 1) for this shape
            let local_t = (t - start) / (end - start);
            return self.shapes[idx].shape.sample(local_t);
        }
    }
    (0.0, 0.0)
}
```

### Caching Boundaries

We precompute boundaries when shapes change, not on every sample:

```rust
fn recompute_boundaries(&mut self) {
    self.boundaries.clear();

    let total_weight: f32 = self.shapes.iter()
        .filter(|s| s.enabled)
        .map(|s| s.weight)
        .sum();

    let mut current_t = 0.0;
    for (i, shape) in self.shapes.iter().enumerate() {
        if shape.enabled {
            let duration = shape.weight / total_weight;
            self.boundaries.push((current_t, current_t + duration, i));
            current_t += duration;
        }
    }
}
```

## Scene Implements Shape

The powerful insight: Scene itself implements Shape!

```rust
impl Shape for Scene {
    fn sample(&self, t: f32) -> (f32, f32) {
        // Delegate to the appropriate child shape
        if let Some((idx, local_t)) = self.find_shape_at(t) {
            self.shapes[idx].shape.sample(local_t)
        } else {
            (0.0, 0.0)
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}
```

This means scenes can contain other scenes - composability!

## Static vs Dynamic Dispatch Recap

| Aspect | Static (`<T: Shape>`) | Dynamic (`dyn Shape`) |
|--------|----------------------|----------------------|
| Resolution | Compile time | Runtime |
| Performance | Faster (inlined) | Vtable lookup |
| Binary size | Larger (monomorphized) | Smaller |
| Flexibility | Type known at compile | Type decided at runtime |
| Collections | Homogeneous only | Heterogeneous |

## The Builder Pattern

Scene uses method chaining for convenient construction:

```rust
let mut scene = Scene::new("My Scene");
scene
    .add(Circle::new(0.5))
    .add_weighted(Rectangle::new(1.0, 0.6), 2.0)
    .add(Polygon::star(5, 0.7, 0.3));
```

The pattern:
```rust
pub fn add<S: Shape + 'static>(&mut self, shape: S) -> &mut Self {
    self.shapes.push(SceneShape::new(shape));
    self.recompute_boundaries();
    self  // Return &mut Self for chaining
}
```

### The `'static` Lifetime Bound

`S: Shape + 'static` means:
- S implements Shape
- S contains no non-static references
- Required because Box<dyn Shape> needs to own the data indefinitely

## UI State Management

The scene editor maintains separate UI state:

```rust
struct SceneEntry {
    shape_type: ShapeType,  // Which shape template
    weight: f32,
    enabled: bool,
}

// UI owns the editable state
scene_entries: Vec<SceneEntry>

// Scene is rebuilt from entries when needed
fn update_scene(&mut self) {
    let mut scene = Scene::new("Custom Scene");
    for entry in &self.scene_entries {
        if entry.enabled {
            // Create shape from type and add to scene
            match entry.shape_type {
                ShapeType::Circle => scene.add_weighted(Circle::new(0.7), entry.weight),
                // ...
            }
        }
    }
    self.audio.set_shape(&scene);
}
```

This separation keeps UI responsive while audio runs independently.

## Deferred Modification Pattern

When iterating, we can't modify the collection directly:

```rust
// This won't work - can't borrow mutably while iterating
for (i, entry) in self.scene_entries.iter_mut().enumerate() {
    if should_remove(entry) {
        self.scene_entries.remove(i);  // Error!
    }
}

// Solution: collect indices, apply changes after
let mut to_remove: Option<usize> = None;
for (i, entry) in self.scene_entries.iter().enumerate() {
    if should_remove(entry) {
        to_remove = Some(i);
    }
}
if let Some(i) = to_remove {
    self.scene_entries.remove(i);
}
```

## Key Takeaways

1. **`Vec<T>`** is Rust's dynamic array - use it for ordered collections
2. **`Box<dyn Trait>`** enables heterogeneous collections via dynamic dispatch
3. **Trait objects** have a small runtime cost but enable runtime polymorphism
4. **Scenes compose shapes** - and Scene itself is a Shape (composability)
5. **Cache computed values** to avoid recalculating on every access
6. **Defer modifications** when iterating to satisfy the borrow checker
7. **`'static` bound** ensures boxed trait objects own their data

## Exercises

1. Add a "duplicate" button that copies a scene entry
2. Implement scene presets (e.g., "Logo", "Music Visualizer")
3. Add per-shape offset (x, y translation)
4. Create nested scenes - a scene containing other scenes

## Further Reading

- [The Rust Book - Common Collections](https://doc.rust-lang.org/book/ch08-00-common-collections.html)
- [The Rust Book - Trait Objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
- [Rust by Example - Box](https://doc.rust-lang.org/rust-by-example/std/box.html)
