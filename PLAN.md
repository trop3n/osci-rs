# osci-rs - Implementation Plan

An oscilloscope music generator inspired by osci-render. Converts vector graphics to XY audio signals for oscilloscope display.

## Current Status

**Completed through Milestone 4** - The app has:
- Audio output with cpal
- XY oscilloscope display with persistence/afterglow
- Settings panel (zoom, line width, intensity, color presets)
- Shape trait system with 11 shapes (Circle, Rectangle, Triangle, Square, Pentagon, Hexagon, Star, Line, Heart, Lissajous, Spiral)
- Shape selection UI with per-shape parameters
- Modular code structure (audio/, render/, shapes/)

## Project Structure

```
osci-rs/
├── Cargo.toml
├── PLAN.md                 # This file
├── docs/                   # Learning journal (Obsidian-ready)
│   ├── 00-index.md
│   ├── 01-project-setup.md
│   ├── 02-audio-fundamentals.md
│   ├── 03-ownership-borrowing.md
│   ├── 04-egui-basics.md
│   └── 05-traits-generics.md
└── src/
    ├── main.rs             # App entry point, shape selection UI
    ├── audio/
    │   ├── mod.rs
    │   ├── buffer.rs       # SampleBuffer, XYSample (Arc<Mutex<T>>)
    │   └── engine.rs       # AudioEngine (cpal output, shape rendering)
    ├── render/
    │   ├── mod.rs
    │   └── oscilloscope.rs # XY display widget with persistence
    └── shapes/
        ├── mod.rs
        ├── traits.rs       # Shape trait definition
        ├── primitives.rs   # Circle, Line, Rectangle, Polygon
        └── path.rs         # Arbitrary point sequences (Lissajous, Spiral, Heart)
```

## Tech Stack

- **Rust** (2021 edition)
- **eframe/egui** - Immediate-mode GUI
- **cpal** - Cross-platform audio I/O

## Milestones

### Phase 1: Foundations ✅

#### Milestone 1: Hello Audio ✅
- [x] Project setup with Cargo
- [x] cpal audio output (440Hz sine wave)
- [x] Basic eframe window
- [x] `docs/01-project-setup.md`
- [x] `docs/02-audio-fundamentals.md`

#### Milestone 3: Oscilloscope Display ✅
- [x] Ring buffer for audio↔UI communication
- [x] XY oscilloscope widget
- [x] Persistence/afterglow effect
- [x] Settings panel
- [x] `docs/03-ownership-borrowing.md`
- [x] `docs/04-egui-basics.md`

---

### Phase 2: Core Features

#### Milestone 4: Basic Shapes ✅
**Goal:** Draw shapes and hear them

**Rust concepts:** Traits, generics, iterators

**Deliverables:**
- [x] `Shape` trait definition
- [x] Circle, Line, Rectangle, Polygon primitives
- [x] `Path` type (Lissajous, Spiral, Heart)
- [x] Shape→audio sample conversion
- [x] UI to select and configure shapes (11 shape types)
- [x] `docs/05-traits-generics.md`

#### Milestone 5: Scene Composition (Next)
**Goal:** Combine multiple shapes

- [ ] `Scene` struct holding multiple shapes
- [ ] Shape ordering and time allocation
- [ ] Per-shape frequency control
- [ ] Scene editor UI
- [ ] `docs/06-collections-lifetimes.md`

#### Milestone 6: Effects & Modulation
**Goal:** Add rotation, scaling, LFOs

- [ ] `Effect` trait
- [ ] Rotate, Scale, Translate effects
- [ ] LFO oscillator
- [ ] Modulation routing
- [ ] `docs/07-trait-objects.md`

#### Milestone 7: SVG Import
**Goal:** Load SVG files

- [ ] SVG file loading (`usvg`)
- [ ] Path extraction
- [ ] Path simplification
- [ ] File dialog (`rfd`)
- [ ] `docs/08-error-handling.md`

#### Milestone 8: Image Tracing
**Goal:** Convert images to paths

- [ ] Image loading (`image` crate)
- [ ] Edge detection
- [ ] Edge → path conversion
- [ ] `docs/09-image-processing.md`

#### Milestone 9: Text Rendering
**Goal:** Render text as graphics

- [ ] Font loading (`ab_glyph`)
- [ ] Text → path conversion
- [ ] `docs/10-fonts-bezier.md`

---

### Phase 4: Advanced Features

#### Milestone 12: Lock-Free Audio
- [ ] Replace `Arc<Mutex<T>>` with lock-free ring buffer
- [ ] Triple buffer for shape updates
- [ ] `docs/12-lock-free.md`

#### Milestone 13: 3D Rendering
- [ ] 3D wireframe rendering (`nalgebra`)
- [ ] OBJ file loading
- [ ] Camera and projection
- [ ] `docs/13-3d-graphics.md`

#### Milestone 14: Project Save/Load
- [ ] Serde serialization
- [ ] Save/load dialogs
- [ ] `docs/14-serialization.md`

#### Milestone 15: MIDI Control
- [ ] MIDI input (`midir`)
- [ ] Parameter mapping
- [ ] `docs/15-midi.md`

#### Milestone 16: Distribution
- [ ] Windows/macOS builds
- [ ] Performance profiling
- [ ] `docs/16-distribution.md`

---

## Dependencies (Current)

```toml
[dependencies]
eframe = "0.29"
cpal = "0.15"
log = "0.4"
env_logger = "0.11"
```

## Dependencies (Planned)

```toml
# Milestone 7+
nalgebra = "0.33"
ringbuf = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
rfd = "0.15"
usvg = "0.44"
image = "0.25"
ab_glyph = "0.2"
tobj = "4.0"
midir = "0.10"
```

---

## Running

```bash
cargo run
```

Click "Play" to generate a circle on the oscilloscope display.

## Related Project

**scope-rs** - The companion visualizer app that displays audio input as XY oscilloscope graphics.

---

## Context for AI Assistants

This project is part of a Rust learning journey. The user is a beginner learning Rust by building practical applications. Each milestone introduces new Rust concepts with accompanying documentation in the `docs/` folder.

The code prioritizes clarity and educational value over maximum performance. For example, we use `Arc<Mutex<T>>` for thread communication (simple to understand) rather than lock-free structures (which come in Milestone 12).

When continuing development:
1. Check current milestone status above
2. Review existing code structure
3. Follow the pattern of existing modules
4. Create corresponding documentation in `docs/`
5. Explain Rust concepts as they appear
