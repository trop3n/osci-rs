# osci-rs - Implementation Plan

An oscilloscope music generator inspired by osci-render. Converts vector graphics to XY audio signals for oscilloscope display.

## Current Status

**Completed through Milestone 13** - The app has:
- Audio output with cpal (crackling fixed)
- XY oscilloscope display with persistence/afterglow
- Settings panel (zoom, line width, intensity, color presets)
- Shape trait system with 11+ shapes
- Single shape mode with per-shape parameters
- Scene composition mode (combine multiple shapes, including loaded SVG/Image/Text)
- Scene editor UI with weights, ordering, enable/disable
- Effects system using Effect trait chain (rotation, scale LFO with 5 waveforms)
- SVG file import with Bézier curve support
- Image tracing via edge detection
- Text rendering with font support
- Lock-free audio (SPSC ring buffers)
- 3D mesh rendering with perspective projection (FOV in degrees)
- OBJ file loading
- Built-in 3D primitives (cube, tetrahedron, octahedron, icosahedron)
- Modular code structure (audio/, render/, shapes/, effects/)
- Zero compiler warnings

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
│   ├── 05-traits-generics.md
│   ├── 06-collections-lifetimes.md
│   ├── 08-error-handling.md
│   ├── 09-image-processing.md
│   ├── 10-fonts-bezier.md
│   ├── 12-lock-free.md
│   └── 13-3d-graphics.md
└── src/
    ├── main.rs             # App entry point, mode toggle, scene editor
    ├── audio/
    │   ├── mod.rs
    │   ├── buffer.rs       # Lock-free SPSC ring buffer
    │   └── engine.rs       # AudioEngine (cpal output, shape rendering)
    ├── effects/
    │   ├── mod.rs
    │   ├── traits.rs       # Effect trait
    │   ├── transform.rs    # Rotate, Scale, Translate effects
    │   └── lfo.rs          # LFO oscillators
    ├── render/
    │   ├── mod.rs
    │   └── oscilloscope.rs # XY display widget with persistence
    └── shapes/
        ├── mod.rs
        ├── traits.rs       # Shape trait definition
        ├── primitives.rs   # Circle, Line, Rectangle, Polygon
        ├── path.rs         # Arbitrary point sequences
        ├── scene.rs        # Multi-shape composition
        ├── svg.rs          # SVG file import
        ├── image.rs        # Image edge tracing
        ├── text.rs         # Text to paths
        └── mesh3d.rs       # 3D mesh rendering
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

#### Milestone 5: Scene Composition ✅
**Goal:** Combine multiple shapes

- [x] `Scene` struct holding multiple shapes
- [x] Shape ordering and time allocation (weight-based)
- [x] Per-shape weight control
- [x] Scene editor UI (add, remove, reorder, enable/disable)
- [x] `docs/06-collections-lifetimes.md`

#### Milestone 6: Effects & Modulation ✅
**Goal:** Add rotation, scaling, LFOs

- [x] `Effect` trait
- [x] Rotate, Scale, Translate effects
- [x] LFO oscillator
- [x] Modulation routing (rotation, scale LFO in UI)
- [ ] `docs/07-trait-objects.md` (pending)

#### Milestone 7: SVG Import ✅
**Goal:** Load SVG files

- [x] SVG file loading (`usvg`)
- [x] Path extraction
- [x] Bézier curve sampling
- [x] File dialog (`rfd`)
- [x] `docs/08-error-handling.md`

#### Milestone 8: Image Tracing ✅
**Goal:** Convert images to paths

- [x] Image loading (`image` crate)
- [x] Sobel edge detection
- [x] Edge → path conversion
- [x] `docs/09-image-processing.md`

#### Milestone 9: Text Rendering ✅
**Goal:** Render text as graphics

- [x] Font loading (`ab_glyph`)
- [x] Text → path conversion
- [x] `docs/10-fonts-bezier.md`

---

### Phase 4: Advanced Features

#### Milestone 12: Lock-Free Audio ✅
- [x] Replace `Arc<Mutex<T>>` with lock-free ring buffer (`ringbuf`)
- [x] SPSC producer/consumer pattern
- [x] `docs/12-lock-free.md`

#### Milestone 13: 3D Rendering ✅
- [x] 3D wireframe rendering (`nalgebra`)
- [x] OBJ file loading (`tobj`)
- [x] Camera with perspective projection
- [x] Built-in primitives (cube, tetrahedron, octahedron, icosahedron)
- [x] Interactive camera controls (orbit, zoom, FOV)
- [x] `docs/13-3d-graphics.md`

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
usvg = "0.44"          # SVG parsing
rfd = "0.15"           # File dialogs
thiserror = "2.0"      # Error handling
image = "0.25"         # Image processing
ab_glyph = "0.2"       # Font handling
ringbuf = "0.4"        # Lock-free ring buffer
nalgebra = "0.33"      # Linear algebra for 3D
tobj = "4.0"           # OBJ file loading
```

## Dependencies (Planned)

```toml
# Milestone 14+
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
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

This project is part of a Rust learning journey. The user is learning Rust by building practical applications. Each milestone introduces new Rust concepts with accompanying documentation in the `docs/` folder.

The codebase now includes:
- Lock-free audio (SPSC ring buffers from `ringbuf`)
- Linear algebra (3D transforms with `nalgebra`)
- File format parsing (SVG with `usvg`, OBJ with `tobj`)
- Image processing (edge detection)
- Font rendering (`ab_glyph`)

When continuing development:
1. Check current milestone status above
2. Review existing code structure
3. Follow the pattern of existing modules
4. Create corresponding documentation in `docs/`
5. Explain Rust concepts as they appear
