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
- 06 - Collections & Lifetimes (Coming Soon)
- 07 - Trait Objects
- 08 - Error Handling
- 09 - Image Processing
- 10 - Fonts & Bézier Curves

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
    ├── render/
    │   ├── mod.rs          # Module exports
    │   └── oscilloscope.rs # XY display widget
    └── shapes/
        ├── mod.rs          # Module exports
        ├── traits.rs       # Shape trait definition
        ├── primitives.rs   # Circle, Line, Rectangle, Polygon
        └── path.rs         # Arbitrary point sequences
```

Coming in future milestones:
- `effects/` - Transform and modulation effects
- `project/` - Save/load state
