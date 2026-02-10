# osci-rs Learning Journal

Welcome to the osci-rs learning documentation. This series of documents accompanies the code as you build an oscilloscope music generator in Rust.

## Milestones

### Phase 1: Foundations

- [[01-project-setup|01 - Project Setup]] - Cargo, dependencies, modules
- [[02-audio-fundamentals|02 - Audio Fundamentals]] - cpal, sample rates, buffers, callbacks
- [[03-ownership-borrowing|03 - Ownership & Borrowing]] - Rust's memory safety system
- [[04-egui-basics|04 - egui Basics]] - Immediate-mode GUI, widgets, custom drawing

### Phase 2: Core Features
- [[05-traits-generics|05 - Traits & Generics]] - Shape trait, generics, parametric equations
- [[06-collections-lifetimes|06 - Collections & Dynamic Dispatch]] - Vec, Box<dyn Trait>, Scene composition
- 07 - Trait Objects (Effects & Modulation)
- [[08-error-handling|08 - Error Handling]] - Result, thiserror, SVG import, Bézier curves
- [[09-image-processing|09 - Image Processing]] - Edge detection, Sobel operator, path tracing
- [[10-fonts-bezier|10 - Fonts & Bézier Curves]] - Text rendering, glyph outlines, curve sampling

### Phase 3: Advanced (Coming Soon)

- 11 - Audio Files
- 12 - Lock-Free Programming
- 13 - 3D Graphics
- 14 - Serialization
- 15 - MIDI
- 16 - Distribution

---

## Quick Reference

### Running the App

```bash
cd osci-rs
cargo run
```

### Building for Release

```bash
cargo build --release
```

### Viewing Documentation

```bash
cargo doc --open
```

### Running with Logging

```bash
RUST_LOG=debug cargo run
```

---

## Glossary

| Term | Definition |
|------|------------|
| **Crate** | A Rust package/library |
| **Cargo** | Rust's package manager and build tool |
| **Trait** | Like an interface - defines shared behavior |
| **Arc** | Atomic Reference Counted - thread-safe shared ownership |
| **Option** | Either `Some(value)` or `None` - Rust's null alternative |
| **Result** | Either `Ok(value)` or `Err(error)` - for fallible operations |
| **Closure** | Anonymous function that can capture variables |
| **Lifetime** | How long a reference is valid |

---

## Project Structure

```
osci-rs/
├── Cargo.toml              # Dependencies and metadata
├── PLAN.md                 # Full implementation roadmap
├── docs/                   # Learning documentation (you are here)
└── src/
    ├── main.rs             # Application entry point, UI
    ├── audio/
    │   ├── mod.rs          # Module exports
    │   ├── engine.rs       # cpal output, shape rendering
    │   └── buffer.rs       # XY sample ring buffer
    ├── effects/
    │   ├── mod.rs          # Module exports
    │   ├── traits.rs       # Effect trait definition
    │   ├── transform.rs    # Rotate, Scale, Translate, Mirror
    │   └── lfo.rs          # LFO oscillators and modulated effects
    ├── render/
    │   ├── mod.rs          # Module exports
    │   └── oscilloscope.rs # XY display widget
    └── shapes/
        ├── mod.rs          # Module exports
        ├── traits.rs       # Shape trait definition
        ├── primitives.rs   # Circle, Line, Rectangle, Polygon
        ├── path.rs         # Arbitrary point sequences
        ├── scene.rs        # Multi-shape composition
        ├── svg.rs          # SVG file import
        ├── image.rs        # Image edge tracing
        └── text.rs         # Text to drawable paths
```

Coming in future milestones:
- `project/` - Save/load state
