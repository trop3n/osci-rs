# Image Processing in Rust

**Milestone 8: Image Tracing**

This document covers image processing fundamentals, focusing on edge detection and path tracing for oscilloscope display.

## The Image Crate

Rust's `image` crate provides comprehensive image I/O and manipulation:

```rust
use image::{DynamicImage, GrayImage};

// Load any supported format (PNG, JPEG, GIF, BMP, WebP, etc.)
let img = image::open("photo.jpg")?;

// Convert to grayscale
let gray: GrayImage = img.to_luma8();

// Access dimensions
let (width, height) = gray.dimensions();

// Access pixels
let pixel = gray.get_pixel(x, y);
let value = pixel.0[0]; // 0-255
```

## Edge Detection with Sobel Operator

The Sobel operator detects edges by computing image gradients:

### The Kernels

```
Gx (horizontal):     Gy (vertical):
-1  0  +1           -1  -2  -1
-2  0  +2            0   0   0
-1  0  +1           +1  +2  +1
```

### Implementation

```rust
fn sobel_edge_detection(img: &GrayImage) -> Vec<f32> {
    let (width, height) = img.dimensions();
    let mut edges = vec![0.0f32; (width * height) as usize];

    const GX: [[i32; 3]; 3] = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
    const GY: [[i32; 3]; 3] = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];

    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let mut gx_sum = 0.0f32;
            let mut gy_sum = 0.0f32;

            // Apply 3x3 kernel
            for ky in 0..3 {
                for kx in 0..3 {
                    let px = (x as i32 + kx as i32 - 1) as u32;
                    let py = (y as i32 + ky as i32 - 1) as u32;
                    let pixel = img.get_pixel(px, py).0[0] as f32 / 255.0;

                    gx_sum += pixel * GX[ky][kx] as f32;
                    gy_sum += pixel * GY[ky][kx] as f32;
                }
            }

            // Gradient magnitude
            let magnitude = (gx_sum * gx_sum + gy_sum * gy_sum).sqrt();
            edges[(y * width + x) as usize] = magnitude;
        }
    }

    edges
}
```

### How It Works

1. **Horizontal gradient (Gx)**: Detects vertical edges (left-right intensity changes)
2. **Vertical gradient (Gy)**: Detects horizontal edges (top-bottom intensity changes)
3. **Magnitude**: Combined edge strength = √(Gx² + Gy²)

## From Edges to Points

After edge detection, we extract and organize points for drawing:

### Thresholding

```rust
fn extract_edge_points(edges: &[f32], threshold: f32) -> Vec<(f32, f32)> {
    let mut points = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let edge_val = edges[y * width + x];
            if edge_val >= threshold {
                // Normalize to [-1, 1] range
                let nx = (x as f32 - width as f32 / 2.0) / (scale / 2.0);
                let ny = -(y as f32 - height as f32 / 2.0) / (scale / 2.0);
                points.push((nx, ny));
            }
        }
    }

    points
}
```

### Nearest Neighbor Sorting

Random point order creates chaotic beam movement. Sorting by proximity creates smoother paths:

```rust
fn sort_points_nearest_neighbor(points: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let mut result = Vec::with_capacity(points.len());
    let mut used = vec![false; points.len()];

    // Start near origin
    let mut current_idx = find_closest_to_origin(points);
    result.push(points[current_idx]);
    used[current_idx] = true;

    // Greedily pick nearest unused point
    for _ in 1..points.len() {
        let current = points[current_idx];
        let mut best_idx = None;
        let mut best_dist = f32::MAX;

        for (i, point) in points.iter().enumerate() {
            if !used[i] {
                let dist = distance_squared(current, *point);
                if dist < best_dist {
                    best_dist = dist;
                    best_idx = Some(i);
                }
            }
        }

        if let Some(idx) = best_idx {
            result.push(points[idx]);
            used[idx] = true;
            current_idx = idx;
        }
    }

    result
}
```

This is a greedy approximation of the Traveling Salesman Problem (TSP).

## Configuration Options

Our `ImageOptions` struct provides user control:

```rust
pub struct ImageOptions {
    /// Edge detection threshold (0.0 to 1.0)
    pub threshold: f32,

    /// Whether to invert the image before processing
    pub invert: bool,

    /// Maximum number of points to generate
    pub max_points: usize,

    /// Minimum edge strength to consider
    pub edge_min: f32,
}
```

### Threshold

- **Low threshold (0.1)**: More edges detected, more points, more detail
- **High threshold (0.5)**: Only strong edges, fewer points, cleaner output

### Invert

- **Normal**: Light backgrounds, dark subjects
- **Inverted**: Dark backgrounds, light subjects (like photos)

### Max Points

- More points = more detail but slower tracing
- Fewer points = faster but may lose detail
- Subsampling preserves overall shape when limiting

## Performance Considerations

### Memory Usage

```rust
// Image pixels: width × height bytes (grayscale)
// Edge buffer: width × height × 4 bytes (f32)
// Points: num_edges × 8 bytes (two f32s)
```

### Algorithm Complexity

- **Sobel**: O(width × height) - linear in image size
- **Point extraction**: O(width × height) - linear scan
- **Nearest neighbor sort**: O(n²) - quadratic in point count

For large images with many edge points, the sorting step dominates. The `max_points` limit helps control this.

## Alternative Edge Detectors

### Canny Edge Detection

More sophisticated than Sobel:
1. Gaussian blur (noise reduction)
2. Sobel gradients
3. Non-maximum suppression (thin edges)
4. Hysteresis thresholding (connect edges)

### Laplacian of Gaussian (LoG)

Detects edges as zero-crossings after Gaussian smoothing.

We chose Sobel for simplicity and performance - it works well for oscilloscope display where some noise is acceptable.

## Key Takeaways

1. **Image crate handles I/O** - Supports many formats automatically
2. **Sobel operator finds edges** - Simple 3×3 convolution kernels
3. **Thresholding filters noise** - Only keep strong edges
4. **Point sorting matters** - Nearest neighbor creates smoother paths
5. **User controls are essential** - Different images need different settings

## Exercises

1. Implement Gaussian blur as a preprocessing step
2. Add edge direction output (useful for drawing order)
3. Implement contour tracing instead of random point collection
4. Add real-time parameter adjustment with live preview

## Links

- [image crate documentation](https://docs.rs/image)
- [Sobel operator - Wikipedia](https://en.wikipedia.org/wiki/Sobel_operator)
- [Edge detection - Wikipedia](https://en.wikipedia.org/wiki/Edge_detection)
- [Canny edge detector](https://en.wikipedia.org/wiki/Canny_edge_detector)
