# 13. 3D Graphics

This milestone adds 3D mesh rendering to osci-rs, allowing you to display 3D wireframe models on an oscilloscope.

## 3D Rendering Concepts

### Vertices and Edges

A 3D mesh is defined by:
- **Vertices** - Points in 3D space (x, y, z)
- **Edges** - Connections between vertices (line segments)

```rust
pub struct Mesh {
    pub vertices: Vec<Point3<f32>>,  // 3D positions
    pub edges: Vec<(usize, usize)>,  // Vertex index pairs
    pub name: String,
}
```

For oscilloscope rendering, we only need wireframes (edges), not filled surfaces. This maps directly to the line-drawing paradigm of oscilloscope graphics.

### Coordinate Systems

3D graphics uses a right-handed coordinate system:
- **X** - Right (+) / Left (-)
- **Y** - Up (+) / Down (-)
- **Z** - Toward viewer (+) / Away (-)

```
        Y
        |
        |
        +---- X
       /
      /
     Z (toward you)
```

## Perspective Projection

To display a 3D mesh on a 2D oscilloscope, we need **perspective projection** - the process of converting 3D coordinates to 2D while preserving depth cues (objects farther away appear smaller).

### The Camera

Our camera has:
- **Position** - Where the camera is in 3D space
- **Target** - Point the camera looks at
- **Field of View (FOV)** - How "wide" the camera sees

```rust
pub struct Camera {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub fov: f32,  // Degrees
}
```

### View Matrix

The **view matrix** transforms world coordinates to camera coordinates (what the camera "sees"):

```rust
impl Camera {
    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(
            &self.position,  // Eye position
            &self.target,    // Look-at point
            &self.up         // Up direction
        )
    }
}
```

The `look_at_rh` function creates a matrix that:
1. Translates the world so the camera is at the origin
2. Rotates so the camera looks down the -Z axis

### Projection Matrix

The **projection matrix** creates the perspective effect (things farther away appear smaller):

```rust
pub fn projection_matrix(&self, aspect: f32) -> Matrix4<f32> {
    Matrix4::new_perspective(
        aspect,              // Width/height ratio
        self.fov.to_radians(), // Field of view in radians
        0.1,                 // Near clip plane
        100.0,               // Far clip plane
    )
}
```

### The Full Pipeline

To project a 3D point to 2D:

```rust
// 1. Transform from world space to camera space
let view = camera.view_matrix();

// 2. Project from 3D to 2D (with perspective)
let proj = camera.projection_matrix(1.0);

// 3. Combined transformation
let mvp = proj * view;

// 4. Apply to each vertex
let clip = mvp.transform_point(&vertex);

// 5. Perspective divide (convert to normalized device coords)
let ndc_x = clip.x / clip.w;
let ndc_y = clip.y / clip.w;
```

The perspective divide is what creates the "shrinking with distance" effect - points with larger W (farther away) get divided by a larger number.

## The `nalgebra` Crate

We use `nalgebra` for linear algebra operations. Key types:

| Type | Description |
|------|-------------|
| `Point3<f32>` | A 3D point (x, y, z) |
| `Vector3<f32>` | A 3D direction/offset |
| `Matrix4<f32>` | 4x4 transformation matrix |

### Why 4x4 Matrices?

3D transformations use 4x4 matrices (instead of 3x3) for **homogeneous coordinates**. This allows us to represent translation as matrix multiplication:

```rust
// Without homogeneous coords: can't do translation with matrix
let rotated = rotation_matrix * point;  // OK
let translated = point + offset;         // Not a matrix operation!

// With homogeneous coords: everything is matrix multiplication
let point_h = Point4::new(x, y, z, 1.0);  // w=1 for points
let transformed = matrix * point_h;       // All transforms work!
```

## Loading OBJ Files

The OBJ format is a simple text format for 3D models:

```obj
# vertices
v -1.0 -1.0 1.0
v 1.0 -1.0 1.0
v 1.0 1.0 1.0
v -1.0 1.0 1.0

# faces (we extract edges from these)
f 1 2 3 4
```

We use the `tobj` crate to parse OBJ files:

```rust
impl Mesh {
    pub fn from_obj(path: impl AsRef<Path>) -> Result<Self, MeshError> {
        let (models, _) = tobj::load_obj(path.as_ref(), &tobj::GPU_LOAD_OPTIONS)?;

        // Extract vertices
        let positions = &models[0].mesh.positions;
        let vertices: Vec<Point3<f32>> = positions
            .chunks(3)
            .map(|chunk| Point3::new(chunk[0], chunk[1], chunk[2]))
            .collect();

        // Extract edges from face indices
        let indices = &models[0].mesh.indices;
        let mut edges = Vec::new();
        // ... process triangles into edges

        Ok(Mesh { vertices, edges, name })
    }
}
```

## Built-in Primitives

We provide several built-in 3D shapes:

### Cube

8 vertices, 12 edges:

```rust
pub fn cube() -> Self {
    let s = 1.0;
    let vertices = vec![
        Point3::new(-s, -s, -s), Point3::new( s, -s, -s),
        Point3::new( s,  s, -s), Point3::new(-s,  s, -s),
        Point3::new(-s, -s,  s), Point3::new( s, -s,  s),
        Point3::new( s,  s,  s), Point3::new(-s,  s,  s),
    ];
    let edges = vec![
        // Front face
        (0, 1), (1, 2), (2, 3), (3, 0),
        // Back face
        (4, 5), (5, 6), (6, 7), (7, 4),
        // Connecting edges
        (0, 4), (1, 5), (2, 6), (3, 7),
    ];
    // ...
}
```

### Platonic Solids

We also provide the **Platonic solids** - the only five regular convex polyhedra:

| Solid | Vertices | Edges | Faces |
|-------|----------|-------|-------|
| Tetrahedron | 4 | 6 | 4 triangles |
| Cube | 8 | 12 | 6 squares |
| Octahedron | 6 | 12 | 8 triangles |
| Icosahedron | 12 | 30 | 20 triangles |

The icosahedron is particularly interesting for oscilloscope display due to its high symmetry and many edges.

## Camera Controls

The camera supports orbiting around the target:

```rust
pub fn orbit(&mut self, yaw: f32, pitch: f32) {
    // Get current offset from target
    let offset = self.position - self.target;

    // Convert to spherical coordinates
    let radius = offset.magnitude();
    let theta = offset.z.atan2(offset.x) + yaw;
    let phi = (offset.y / radius).asin().clamp(-PI/2.0 + 0.1, PI/2.0 - 0.1) + pitch;

    // Convert back to Cartesian
    self.position = self.target + Vector3::new(
        radius * phi.cos() * theta.cos(),
        radius * phi.sin(),
        radius * phi.cos() * theta.sin(),
    );
}
```

And zooming (moving closer/farther):

```rust
pub fn zoom(&mut self, factor: f32) {
    let direction = self.position - self.target;
    self.position = self.target + direction * factor;
}
```

## Implementing the Shape Trait

`Mesh3DShape` implements the `Shape` trait to integrate with the audio engine:

```rust
impl Shape for Mesh3DShape {
    fn sample(&self, t: f32) -> (f32, f32) {
        // The projected 2D path is stored in self.points
        if self.points.is_empty() {
            return (0.0, 0.0);
        }

        // Sample along the edge path
        let total = self.points.len();
        let index = ((t * total as f32) as usize).min(total - 1);
        self.points[index]
    }
}
```

The key is that all 3D-to-2D projection happens in `update_projection()`, which is called when the mesh or camera changes. The `sample()` method then just returns pre-computed 2D points.

## UI Controls

The 3D mesh UI provides:

1. **Model Selection** - Choose from built-in primitives or load OBJ files
2. **Camera Orbit** - Rotate around the model (arrow buttons)
3. **Zoom** - Move closer/farther
4. **FOV** - Adjust field of view (wider = more perspective distortion)
5. **Edge Detail** - Points per edge (more = smoother lines)
6. **Reset Camera** - Return to default view

## Key Takeaways

- 3D rendering requires **perspective projection** to create depth cues
- **View matrix** positions the camera; **projection matrix** creates perspective
- The **perspective divide** (dividing by W) makes far objects smaller
- `nalgebra` provides efficient linear algebra operations
- OBJ files store vertices as positions and faces as vertex indices
- For oscilloscope rendering, we only need wireframes (edges)

## Exercises

1. Add a **dodecahedron** (12 pentagonal faces, 30 edges)
2. Implement **auto-rotation** using the frame delta time
3. Add **orthographic projection** option (no perspective distortion)
4. Create a **pyramid** primitive
5. Add **backface culling** to hide edges facing away from the camera

## Links

- [nalgebra Documentation](https://docs.rs/nalgebra)
- [tobj Crate](https://docs.rs/tobj)
- [OBJ File Format](https://en.wikipedia.org/wiki/Wavefront_.obj_file)
- [Perspective Projection](https://en.wikipedia.org/wiki/3D_projection#Perspective_projection)
- [Homogeneous Coordinates](https://en.wikipedia.org/wiki/Homogeneous_coordinates)
