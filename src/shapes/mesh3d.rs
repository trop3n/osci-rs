//! 3D mesh rendering for oscilloscope display
//!
//! This module handles:
//! - Loading 3D models from OBJ files
//! - Camera positioning and perspective projection
//! - Converting 3D wireframes to 2D paths for oscilloscope rendering
//!
//! ## Coordinate System
//!
//! We use a right-handed coordinate system:
//! - X: Right
//! - Y: Up
//! - Z: Towards viewer (out of screen)
//!
//! The camera looks down the negative Z axis by default.

use std::f32::consts::PI;
use std::path::Path as FilePath;

use nalgebra::{Matrix4, Point3, Vector3};
use thiserror::Error;

use super::path::Path;
use super::traits::Shape;

/// Errors that can occur during 3D mesh operations
#[derive(Error, Debug)]
pub enum MeshError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse OBJ file: {0}")]
    ParseError(String),

    #[error("No geometry found in file")]
    NoGeometry,

    #[error("Mesh has no edges")]
    NoEdges,
}

/// A 3D mesh consisting of vertices and edges
#[derive(Clone, Debug)]
pub struct Mesh {
    /// Vertex positions
    pub vertices: Vec<Point3<f32>>,
    /// Edges as pairs of vertex indices
    pub edges: Vec<(usize, usize)>,
    /// Name of the mesh
    pub name: String,
}

impl Mesh {
    /// Create a new empty mesh
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            name: name.into(),
        }
    }

    /// Create a mesh from vertices and edges
    pub fn from_data(
        vertices: Vec<Point3<f32>>,
        edges: Vec<(usize, usize)>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            vertices,
            edges,
            name: name.into(),
        }
    }

    /// Load a mesh from an OBJ file
    pub fn from_obj(path: impl AsRef<FilePath>) -> Result<Self, MeshError> {
        let path = path.as_ref();
        let (models, _materials) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: false,
                single_index: true,
                ..Default::default()
            },
        )
        .map_err(|e| MeshError::ParseError(e.to_string()))?;

        if models.is_empty() {
            return Err(MeshError::NoGeometry);
        }

        // Combine all models into one mesh
        let mut vertices = Vec::new();
        let mut edges = Vec::new();
        let mut vertex_offset = 0;

        for model in &models {
            let mesh = &model.mesh;

            // Add vertices
            for i in (0..mesh.positions.len()).step_by(3) {
                vertices.push(Point3::new(
                    mesh.positions[i],
                    mesh.positions[i + 1],
                    mesh.positions[i + 2],
                ));
            }

            // Extract edges from faces
            // OBJ indices are stored in mesh.indices
            let indices = &mesh.indices;
            let face_arities = &mesh.face_arities;

            let mut idx = 0;
            for &arity in face_arities {
                let arity = arity as usize;
                // Add edges for this face
                for i in 0..arity {
                    let v1 = indices[idx + i] as usize + vertex_offset;
                    let v2 = indices[idx + (i + 1) % arity] as usize + vertex_offset;
                    // Avoid duplicate edges by only adding if v1 < v2
                    if v1 < v2 {
                        edges.push((v1, v2));
                    } else if v1 > v2 {
                        edges.push((v2, v1));
                    }
                }
                idx += arity;
            }

            vertex_offset = vertices.len();
        }

        // Remove duplicate edges
        edges.sort();
        edges.dedup();

        if edges.is_empty() {
            return Err(MeshError::NoEdges);
        }

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("mesh")
            .to_string();

        Ok(Self {
            vertices,
            edges,
            name,
        })
    }

    /// Create a unit cube centered at origin
    pub fn cube() -> Self {
        let s = 0.5;
        let vertices = vec![
            Point3::new(-s, -s, -s),
            Point3::new(s, -s, -s),
            Point3::new(s, s, -s),
            Point3::new(-s, s, -s),
            Point3::new(-s, -s, s),
            Point3::new(s, -s, s),
            Point3::new(s, s, s),
            Point3::new(-s, s, s),
        ];

        let edges = vec![
            // Bottom face
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            // Top face
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            // Vertical edges
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];

        Self::from_data(vertices, edges, "Cube")
    }

    /// Create a tetrahedron
    pub fn tetrahedron() -> Self {
        let a = 1.0 / (2.0_f32).sqrt();
        let vertices = vec![
            Point3::new(1.0, 0.0, -a),
            Point3::new(-1.0, 0.0, -a),
            Point3::new(0.0, 1.0, a),
            Point3::new(0.0, -1.0, a),
        ];

        let edges = vec![(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];

        Self::from_data(vertices, edges, "Tetrahedron")
    }

    /// Create an octahedron
    pub fn octahedron() -> Self {
        let vertices = vec![
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(-1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(0.0, -1.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, 0.0, -1.0),
        ];

        let edges = vec![
            (0, 2),
            (0, 3),
            (0, 4),
            (0, 5),
            (1, 2),
            (1, 3),
            (1, 4),
            (1, 5),
            (2, 4),
            (2, 5),
            (3, 4),
            (3, 5),
        ];

        Self::from_data(vertices, edges, "Octahedron")
    }

    /// Create an icosahedron
    pub fn icosahedron() -> Self {
        let phi = (1.0 + 5.0_f32.sqrt()) / 2.0; // Golden ratio
        let a = 1.0;
        let b = a / phi;

        let vertices = vec![
            Point3::new(0.0, b, -a),
            Point3::new(b, a, 0.0),
            Point3::new(-b, a, 0.0),
            Point3::new(0.0, b, a),
            Point3::new(0.0, -b, a),
            Point3::new(-a, 0.0, b),
            Point3::new(0.0, -b, -a),
            Point3::new(a, 0.0, -b),
            Point3::new(a, 0.0, b),
            Point3::new(-a, 0.0, -b),
            Point3::new(b, -a, 0.0),
            Point3::new(-b, -a, 0.0),
        ];

        let edges = vec![
            (0, 1),
            (0, 2),
            (0, 6),
            (0, 7),
            (0, 9),
            (1, 2),
            (1, 3),
            (1, 7),
            (1, 8),
            (2, 3),
            (2, 5),
            (2, 9),
            (3, 4),
            (3, 5),
            (3, 8),
            (4, 5),
            (4, 8),
            (4, 10),
            (4, 11),
            (5, 9),
            (5, 11),
            (6, 7),
            (6, 9),
            (6, 10),
            (6, 11),
            (7, 8),
            (7, 10),
            (8, 10),
            (9, 11),
            (10, 11),
        ];

        Self::from_data(vertices, edges, "Icosahedron")
    }

    /// Get bounding box of mesh
    pub fn bounds(&self) -> (Point3<f32>, Point3<f32>) {
        if self.vertices.is_empty() {
            return (Point3::origin(), Point3::origin());
        }

        let mut min = self.vertices[0];
        let mut max = self.vertices[0];

        for v in &self.vertices {
            min.x = min.x.min(v.x);
            min.y = min.y.min(v.y);
            min.z = min.z.min(v.z);
            max.x = max.x.max(v.x);
            max.y = max.y.max(v.y);
            max.z = max.z.max(v.z);
        }

        (min, max)
    }

    /// Center mesh at origin and normalize to unit size
    pub fn normalize(&mut self) {
        if self.vertices.is_empty() {
            return;
        }

        let (min, max) = self.bounds();
        let center = Point3::new(
            (min.x + max.x) / 2.0,
            (min.y + max.y) / 2.0,
            (min.z + max.z) / 2.0,
        );

        let size = (max.x - min.x)
            .max(max.y - min.y)
            .max(max.z - min.z);

        let scale = if size > 0.0 { 2.0 / size } else { 1.0 };

        for v in &mut self.vertices {
            v.x = (v.x - center.x) * scale;
            v.y = (v.y - center.y) * scale;
            v.z = (v.z - center.z) * scale;
        }
    }
}

/// Camera for 3D viewing
#[derive(Clone, Debug)]
pub struct Camera {
    /// Camera position
    pub position: Point3<f32>,
    /// Point the camera is looking at
    pub target: Point3<f32>,
    /// Up vector
    pub up: Vector3<f32>,
    /// Field of view in radians
    pub fov: f32,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 3.0),
            target: Point3::origin(),
            up: Vector3::y(),
            fov: PI / 4.0, // 45 degrees
            near: 0.1,
            far: 100.0,
        }
    }
}

impl Camera {
    /// Create a new camera
    pub fn new(position: Point3<f32>, target: Point3<f32>) -> Self {
        Self {
            position,
            target,
            ..Default::default()
        }
    }

    /// Get the view matrix
    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(&self.position, &self.target, &self.up)
    }

    /// Get the projection matrix (perspective)
    pub fn projection_matrix(&self, aspect: f32) -> Matrix4<f32> {
        Matrix4::new_perspective(aspect, self.fov, self.near, self.far)
    }

    /// Get FOV in degrees (for UI display)
    pub fn fov_degrees(&self) -> f32 {
        self.fov.to_degrees()
    }

    /// Set FOV from degrees (for UI input)
    pub fn set_fov_degrees(&mut self, degrees: f32) {
        self.fov = degrees.to_radians();
    }

    /// Orbit the camera around the target
    pub fn orbit(&mut self, yaw: f32, pitch: f32) {
        let offset = self.position - self.target;
        let distance = offset.magnitude();

        // Convert to spherical coordinates
        let current_yaw = offset.z.atan2(offset.x);
        let current_pitch = (offset.y / distance).asin();

        // Apply rotation
        let new_yaw = current_yaw + yaw;
        let new_pitch = (current_pitch + pitch).clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);

        // Convert back to Cartesian
        let cos_pitch = new_pitch.cos();
        self.position = self.target
            + Vector3::new(
                distance * cos_pitch * new_yaw.cos(),
                distance * new_pitch.sin(),
                distance * cos_pitch * new_yaw.sin(),
            );
    }

    /// Zoom the camera (move closer/farther from target)
    pub fn zoom(&mut self, factor: f32) {
        let offset = self.position - self.target;
        let new_distance = (offset.magnitude() * factor).max(0.5);
        self.position = self.target + offset.normalize() * new_distance;
    }
}

/// Options for 3D mesh rendering
#[derive(Clone, Debug)]
pub struct Mesh3DOptions {
    /// Points per edge for sampling
    pub edge_samples: usize,
    /// Camera rotation speed (radians per frame)
    pub auto_rotate_speed: f32,
    /// Whether to auto-rotate
    pub auto_rotate: bool,
}

impl Default for Mesh3DOptions {
    fn default() -> Self {
        Self {
            edge_samples: 2,
            auto_rotate_speed: 0.01,
            auto_rotate: true,
        }
    }
}

/// A 3D mesh shape for oscilloscope rendering
pub struct Mesh3DShape {
    /// The 3D mesh
    mesh: Mesh,
    /// Camera for viewing
    camera: Camera,
    /// Rendering options
    options: Mesh3DOptions,
    /// Current rotation angle (for auto-rotate)
    rotation: f32,
    /// Projected 2D path
    path: Path,
    /// Cached points for Shape trait
    points: Vec<(f32, f32)>,
}

impl Mesh3DShape {
    /// Create a new 3D mesh shape
    pub fn new(mesh: Mesh, options: Mesh3DOptions) -> Self {
        let camera = Camera::default();
        let mut shape = Self {
            mesh,
            camera,
            options,
            rotation: 0.0,
            path: Path::with_options(Vec::new(), false, "mesh".to_string()),
            points: Vec::new(),
        };
        shape.update_projection();
        shape
    }

    /// Set a custom camera (builder pattern)
    pub fn with_camera(mut self, camera: Camera) -> Self {
        self.camera = camera;
        self.update_projection();
        self
    }

    /// Update the camera and recalculate projection
    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
        self.update_projection();
    }

    /// Create from an OBJ file
    pub fn from_obj(
        path: impl AsRef<FilePath>,
        options: Mesh3DOptions,
    ) -> Result<Self, MeshError> {
        let mut mesh = Mesh::from_obj(path)?;
        mesh.normalize();
        Ok(Self::new(mesh, options))
    }

    /// Create a cube
    pub fn cube(options: Mesh3DOptions) -> Self {
        Self::new(Mesh::cube(), options)
    }

    /// Create a tetrahedron
    pub fn tetrahedron(options: Mesh3DOptions) -> Self {
        Self::new(Mesh::tetrahedron(), options)
    }

    /// Create an octahedron
    pub fn octahedron(options: Mesh3DOptions) -> Self {
        Self::new(Mesh::octahedron(), options)
    }

    /// Create an icosahedron
    pub fn icosahedron(options: Mesh3DOptions) -> Self {
        Self::new(Mesh::icosahedron(), options)
    }

    /// Get a reference to the camera
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Get a mutable reference to the camera
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Get a reference to the options
    pub fn options(&self) -> &Mesh3DOptions {
        &self.options
    }

    /// Get a mutable reference to the options
    pub fn options_mut(&mut self) -> &mut Mesh3DOptions {
        &mut self.options
    }

    /// Update the 2D projection
    pub fn update_projection(&mut self) {
        // Apply auto-rotation
        if self.options.auto_rotate {
            self.rotation += self.options.auto_rotate_speed;
            self.camera.orbit(self.options.auto_rotate_speed, 0.0);
        }

        // Calculate view-projection matrix
        let view = self.camera.view_matrix();
        let proj = self.camera.projection_matrix(1.0); // Square aspect
        let vp = proj * view;

        // Project all vertices
        let projected: Vec<(f32, f32)> = self
            .mesh
            .vertices
            .iter()
            .map(|v| {
                let clip = vp.transform_point(v);
                // Perspective divide and convert to [-1, 1]
                (clip.x / clip.z.abs().max(0.001), clip.y / clip.z.abs().max(0.001))
            })
            .collect();

        // Build path from edges
        let mut points = Vec::new();
        for &(i1, i2) in &self.mesh.edges {
            if i1 < projected.len() && i2 < projected.len() {
                let p1 = projected[i1];
                let p2 = projected[i2];

                // Sample points along the edge
                for i in 0..=self.options.edge_samples {
                    let t = i as f32 / self.options.edge_samples as f32;
                    let x = p1.0 + t * (p2.0 - p1.0);
                    let y = p1.1 + t * (p2.1 - p1.1);
                    // Clamp to visible range
                    points.push((x.clamp(-1.5, 1.5), y.clamp(-1.5, 1.5)));
                }
            }
        }

        self.points = points.clone();
        self.path = Path::with_options(points, false, self.mesh.name.clone());
    }

    /// Get the mesh name
    pub fn name(&self) -> &str {
        &self.mesh.name
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.mesh.vertices.len()
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.mesh.edges.len()
    }
}

impl Shape for Mesh3DShape {
    fn sample(&self, t: f32) -> (f32, f32) {
        self.path.sample(t)
    }

    fn name(&self) -> &str {
        &self.mesh.name
    }

    fn length(&self) -> f32 {
        self.path.length()
    }

    fn is_closed(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube() {
        let mesh = Mesh::cube();
        assert_eq!(mesh.vertices.len(), 8);
        assert_eq!(mesh.edges.len(), 12);
    }

    #[test]
    fn test_tetrahedron() {
        let mesh = Mesh::tetrahedron();
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.edges.len(), 6);
    }

    #[test]
    fn test_octahedron() {
        let mesh = Mesh::octahedron();
        assert_eq!(mesh.vertices.len(), 6);
        assert_eq!(mesh.edges.len(), 12);
    }

    #[test]
    fn test_icosahedron() {
        let mesh = Mesh::icosahedron();
        assert_eq!(mesh.vertices.len(), 12);
        assert_eq!(mesh.edges.len(), 30);
    }

    #[test]
    fn test_mesh_normalize() {
        let mut mesh = Mesh::cube();
        mesh.normalize();

        let (min, max) = mesh.bounds();
        // Should be roughly centered and unit-sized
        assert!(min.x >= -1.1 && max.x <= 1.1);
        assert!(min.y >= -1.1 && max.y <= 1.1);
        assert!(min.z >= -1.1 && max.z <= 1.1);
    }

    #[test]
    fn test_camera_default() {
        let cam = Camera::default();
        assert!(cam.position.z > 0.0); // Camera in front
        assert_eq!(cam.target, Point3::origin());
    }

    #[test]
    fn test_mesh3d_shape() {
        let shape = Mesh3DShape::cube(Mesh3DOptions::default());
        assert_eq!(shape.vertex_count(), 8);
        assert_eq!(shape.edge_count(), 12);

        // Should be able to sample
        let (x, y) = shape.sample(0.5);
        assert!(x.is_finite() && y.is_finite());
    }
}
