# 10. Fonts and Bézier Curves

In this milestone we implemented text rendering by extracting glyph outlines from font files and converting them to drawable paths for oscilloscope display.

## Key Concepts

### Font Files and Glyphs

Fonts are collections of **glyphs** - the visual representations of characters. TrueType (.ttf) and OpenType (.otf) fonts store glyph shapes as mathematical curves rather than pixel data, making them scalable to any size.

Each glyph is defined by:
- **Contours** - closed paths that make up the glyph shape
- **Control points** - coordinates that define the curves
- **Advance width** - how far to move before the next character

```rust
use ab_glyph::{Font, FontRef, ScaleFont, OutlineCurve};

// Load font from embedded bytes
let font_data = include_bytes!("../assets/fonts/RobotoMono-Regular.ttf");
let font = FontRef::try_from_slice(font_data)?;

// Scale font to desired pixel size
let scaled_font = font.as_scaled(64.0);

// Get glyph for a character
let glyph_id = font.glyph_id('A');

// Get outline curves
if let Some(outline) = font.outline(glyph_id) {
    for curve in &outline.curves {
        // Process each curve segment
    }
}
```

### Bézier Curves

Font outlines use **Bézier curves** - smooth parametric curves defined by control points. The `ab_glyph` crate provides three curve types:

#### Line Segments
The simplest case - a straight line between two points:
```rust
OutlineCurve::Line(p0, p1)
```

#### Quadratic Bézier (TrueType)
Three control points: start, control, end. Used in TrueType fonts.

```rust
OutlineCurve::Quad(p0, p1, p2)

// Mathematical formula for point at parameter t (0..1):
// B(t) = (1-t)²·P₀ + 2(1-t)t·P₁ + t²·P₂

fn quadratic_bezier(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), t: f32) -> (f32, f32) {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;

    let x = mt2 * p0.0 + 2.0 * mt * t * p1.0 + t2 * p2.0;
    let y = mt2 * p0.1 + 2.0 * mt * t * p1.1 + t2 * p2.1;

    (x, y)
}
```

#### Cubic Bézier (OpenType/PostScript)
Four control points: start, control1, control2, end. Used in PostScript-based fonts.

```rust
OutlineCurve::Cubic(p0, p1, p2, p3)

// Mathematical formula:
// B(t) = (1-t)³·P₀ + 3(1-t)²t·P₁ + 3(1-t)t²·P₂ + t³·P₃

fn cubic_bezier(
    p0: (f32, f32), p1: (f32, f32),
    p2: (f32, f32), p3: (f32, f32),
    t: f32
) -> (f32, f32) {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    let t2 = t * t;
    let t3 = t2 * t;

    let x = mt3 * p0.0 + 3.0 * mt2 * t * p1.0 + 3.0 * mt * t2 * p2.0 + t3 * p3.0;
    let y = mt3 * p0.1 + 3.0 * mt2 * t * p1.1 + 3.0 * mt * t2 * p2.1 + t3 * p3.1;

    (x, y)
}
```

### Curve Sampling

To render curves on an oscilloscope, we **sample** them at regular intervals of the parameter `t`:

```rust
fn extract_outline_points(
    curves: &[OutlineCurve],
    offset_x: f32,
    offset_y: f32,
    scale: f32,
    curve_samples: usize,  // e.g., 8 samples per curve
) -> Vec<(f32, f32)> {
    let mut points = Vec::new();

    for curve in curves {
        match curve {
            OutlineCurve::Line(p0, p1) => {
                points.push((p0.x * scale + offset_x, p0.y * scale + offset_y));
                points.push((p1.x * scale + offset_x, p1.y * scale + offset_y));
            }
            OutlineCurve::Quad(p0, p1, p2) => {
                // Sample at curve_samples points
                for i in 0..=curve_samples {
                    let t = i as f32 / curve_samples as f32;
                    let point = quadratic_bezier(/* scaled points */, t);
                    points.push(point);
                }
            }
            OutlineCurve::Cubic(p0, p1, p2, p3) => {
                // Same sampling approach for cubic
                for i in 0..=curve_samples {
                    let t = i as f32 / curve_samples as f32;
                    let point = cubic_bezier(/* scaled points */, t);
                    points.push(point);
                }
            }
        }
    }

    points
}
```

### Text Layout

Rendering multiple characters requires tracking the **cursor position** and using the font's **advance width** for each glyph:

```rust
let mut cursor_x = 0.0f32;

for ch in text.chars() {
    let glyph_id = font.glyph_id(ch);

    // Get and process outline at current position
    if let Some(outline) = font.outline(glyph_id) {
        let glyph_points = extract_outline_points(
            &outline.curves,
            cursor_x,  // Offset by cursor position
            0.0,
            font_size,
            curve_samples,
        );
        all_points.extend(glyph_points);
    }

    // Advance cursor for next character
    let h_advance = scaled_font.h_advance(glyph_id);
    cursor_x += h_advance * letter_spacing;
}
```

### Coordinate Normalization

After extracting all points, we normalize them to the [-1, 1] range for consistent oscilloscope display:

```rust
fn normalize_points(points: &[(f32, f32)]) -> Vec<(f32, f32)> {
    // Find bounding box
    let (min_x, max_x) = points.iter().map(|p| p.0).fold(
        (f32::MAX, f32::MIN), |acc, x| (acc.0.min(x), acc.1.max(x))
    );
    let (min_y, max_y) = points.iter().map(|p| p.1).fold(
        (f32::MAX, f32::MIN), |acc, y| (acc.0.min(y), acc.1.max(y))
    );

    let width = max_x - min_x;
    let height = max_y - min_y;
    let scale = width.max(height);  // Maintain aspect ratio

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    points.iter().map(|&(x, y)| {
        let nx = (x - center_x) / (scale / 2.0);
        let ny = -(y - center_y) / (scale / 2.0);  // Flip Y for screen coords
        (nx.clamp(-1.0, 1.0), ny.clamp(-1.0, 1.0))
    }).collect()
}
```

## The `ab_glyph` Crate

We use `ab_glyph` for font handling. Key types:

| Type | Purpose |
|------|---------|
| `FontRef` | Zero-copy font reference from borrowed data |
| `FontVec` | Font owning its data |
| `Font` trait | Common interface for font operations |
| `ScaleFont` | Font methods at a specific pixel size |
| `GlyphId` | Identifier for a specific glyph |
| `Outline` | Contains `curves: Vec<OutlineCurve>` |
| `OutlineCurve` | Enum: `Line`, `Quad`, or `Cubic` |

## Our Implementation

### `TextShape` struct

```rust
pub struct TextShape {
    points: Vec<(f32, f32)>,  // All outline points
    path: Path,               // For Shape trait sampling
    text: String,             // Original text
}
```

### `TextOptions` configuration

```rust
pub struct TextOptions {
    pub size: f32,            // Font size in pixels (before normalization)
    pub curve_samples: usize, // Points per curve segment (default: 8)
    pub letter_spacing: f32,  // Spacing multiplier (1.0 = normal)
}
```

### Creating text shapes

```rust
// Using embedded default font
let text = TextShape::new("Hello", &TextOptions::default())?;

// Using custom font file
let text = TextShape::from_font_file(
    "Hello",
    "/path/to/font.ttf",
    &TextOptions { size: 48.0, ..Default::default() }
)?;
```

## Key Takeaways

1. **Font outlines are mathematical** - Fonts store shapes as curves, not pixels, enabling infinite scaling

2. **Bézier curves are elegant** - Just a few control points define smooth, beautiful curves

3. **Sampling trades accuracy for points** - More samples = smoother curves but more data to render

4. **Letter spacing uses advance width** - Each glyph knows how much horizontal space it needs

5. **Normalization ensures consistency** - Converting to [-1, 1] range works with our existing rendering

6. **The `ab_glyph` crate is well-designed** - Uses Rust idioms (traits, enums) for a clean API

## Exercises

1. **Variable sampling** - Modify curve sampling to use adaptive sampling based on curve length

2. **Kerning support** - Use `font.kern_unscaled()` to add kerning between character pairs

3. **Multi-line text** - Add line break support with vertical positioning

4. **Font effects** - Apply effects like outline-only or bold simulation

## Links

- [ab_glyph crate](https://docs.rs/ab_glyph/)
- [Bézier curves explained](https://pomax.github.io/bezierinfo/)
- [TrueType specification](https://developer.apple.com/fonts/TrueType-Reference-Manual/)
- [OpenType specification](https://docs.microsoft.com/typography/opentype/spec/)
