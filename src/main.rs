// Library modules contain methods for future milestones
#![allow(dead_code)]

//! osci-rs - Oscilloscope Music Generator
//!
//! This application converts vector graphics into XY audio signals
//! that can be displayed on an oscilloscope.
//!
//! ## Milestone 13: 3D Mesh Rendering
//! This version adds:
//! - 3D mesh loading (OBJ files)
//! - Built-in primitives (cube, tetrahedron, etc.)
//! - Camera with perspective projection
//! - Interactive rotation controls

use eframe::egui;

mod audio;
mod effects;
mod render;
mod shapes;

use audio::{AudioEngine, EffectParams, SampleBuffer};
use effects::LfoWaveform;
use render::Oscilloscope;
use shapes::{Circle, Line, Rectangle, Polygon, Path, Scene, SvgShape, SvgOptions, ImageShape, ImageOptions, TextShape, TextOptions, Mesh, Mesh3DShape, Mesh3DOptions, Camera};

/// Buffer size for audio samples
const BUFFER_SIZE: usize = 2048;

fn main() -> eframe::Result<()> {
    env_logger::init();
    log::info!("Starting osci-rs");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_title("osci-rs"),
        ..Default::default()
    };

    eframe::run_native(
        "osci-rs",
        options,
        Box::new(|cc| Ok(Box::new(OsciApp::new(cc)))),
    )
}

/// Available shape types
#[derive(Clone, Copy, PartialEq, Debug)]
enum ShapeType {
    Circle,
    Rectangle,
    Triangle,
    Square,
    Pentagon,
    Hexagon,
    Star,
    Line,
    Heart,
    Lissajous,
    Spiral,
    Svg,    // Loaded SVG file
    Image,  // Traced image file
    Text,   // Rendered text
    Mesh3D, // 3D wireframe mesh
}

impl ShapeType {
    fn all() -> &'static [ShapeType] {
        &[
            ShapeType::Circle,
            ShapeType::Rectangle,
            ShapeType::Triangle,
            ShapeType::Square,
            ShapeType::Pentagon,
            ShapeType::Hexagon,
            ShapeType::Star,
            ShapeType::Line,
            ShapeType::Heart,
            ShapeType::Lissajous,
            ShapeType::Spiral,
            ShapeType::Svg,
            ShapeType::Image,
            ShapeType::Text,
            ShapeType::Mesh3D,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            ShapeType::Circle => "Circle",
            ShapeType::Rectangle => "Rectangle",
            ShapeType::Triangle => "Triangle",
            ShapeType::Square => "Square",
            ShapeType::Pentagon => "Pentagon",
            ShapeType::Hexagon => "Hexagon",
            ShapeType::Star => "Star",
            ShapeType::Line => "Line",
            ShapeType::Heart => "Heart",
            ShapeType::Lissajous => "Lissajous",
            ShapeType::Spiral => "Spiral",
            ShapeType::Svg => "SVG File",
            ShapeType::Image => "Image File",
            ShapeType::Text => "Text",
            ShapeType::Mesh3D => "3D Mesh",
        }
    }
}

/// Editor mode - single shape or scene composition
#[derive(Clone, Copy, PartialEq, Debug)]
enum EditorMode {
    SingleShape,
    Scene,
}

/// Entry in the scene editor (for UI state)
struct SceneEntry {
    shape_type: ShapeType,
    weight: f32,
    enabled: bool,
}

impl SceneEntry {
    fn new(shape_type: ShapeType) -> Self {
        Self {
            shape_type,
            weight: 1.0,
            enabled: true,
        }
    }
}

/// Shape parameters (varies by shape type)
struct ShapeParams {
    // Common
    size: f32,

    // Rectangle specific
    width: f32,
    height: f32,

    // Star specific
    inner_radius: f32,
    points: usize,

    // Lissajous specific
    lissajous_a: f32,
    lissajous_b: f32,
    lissajous_delta: f32,

    // Spiral specific
    spiral_turns: f32,
}

/// Built-in 3D mesh primitives
#[derive(Clone, Copy, PartialEq, Debug)]
enum MeshPrimitive {
    Cube,
    Tetrahedron,
    Octahedron,
    Icosahedron,
    Custom, // Loaded OBJ file
}

impl MeshPrimitive {
    fn all() -> &'static [MeshPrimitive] {
        &[
            MeshPrimitive::Cube,
            MeshPrimitive::Tetrahedron,
            MeshPrimitive::Octahedron,
            MeshPrimitive::Icosahedron,
            MeshPrimitive::Custom,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            MeshPrimitive::Cube => "Cube",
            MeshPrimitive::Tetrahedron => "Tetrahedron",
            MeshPrimitive::Octahedron => "Octahedron",
            MeshPrimitive::Icosahedron => "Icosahedron",
            MeshPrimitive::Custom => "OBJ File",
        }
    }

    fn to_mesh(&self) -> Option<Mesh> {
        match self {
            MeshPrimitive::Cube => Some(Mesh::cube()),
            MeshPrimitive::Tetrahedron => Some(Mesh::tetrahedron()),
            MeshPrimitive::Octahedron => Some(Mesh::octahedron()),
            MeshPrimitive::Icosahedron => Some(Mesh::icosahedron()),
            MeshPrimitive::Custom => None, // Loaded from file
        }
    }
}

impl Default for ShapeParams {
    fn default() -> Self {
        Self {
            size: 0.8,
            width: 1.2,
            height: 0.6,
            inner_radius: 0.3,
            points: 5,
            lissajous_a: 3.0,
            lissajous_b: 2.0,
            lissajous_delta: std::f32::consts::FRAC_PI_2,
            spiral_turns: 3.0,
        }
    }
}

/// Main application state
struct OsciApp {
    buffer: SampleBuffer,
    audio: AudioEngine,
    oscilloscope: Oscilloscope,
    show_settings: bool,

    // Editor mode
    editor_mode: EditorMode,

    // Single shape selection
    selected_shape: ShapeType,
    shape_params: ShapeParams,
    shape_needs_update: bool,

    // Scene composition
    scene_entries: Vec<SceneEntry>,
    scene_shape_to_add: ShapeType,

    // SVG import
    loaded_svg: Option<SvgShape>,
    svg_options: SvgOptions,
    svg_error: Option<String>,

    // Image import
    loaded_image: Option<ImageShape>,
    image_options: ImageOptions,
    image_error: Option<String>,

    // Text rendering
    text_input: String,
    text_shape: Option<TextShape>,
    text_options: TextOptions,
    text_error: Option<String>,

    // 3D mesh rendering
    loaded_mesh: Option<Mesh>,
    mesh_shape: Option<Mesh3DShape>,
    mesh_options: Mesh3DOptions,
    mesh_camera: Camera,
    mesh_primitive: MeshPrimitive,
    mesh_error: Option<String>,

    // Effects
    enable_rotation: bool,
    rotation_speed: f32,
    enable_scale_lfo: bool,
    scale_lfo_freq: f32,
    scale_lfo_min: f32,
    scale_lfo_max: f32,
    scale_lfo_waveform: LfoWaveform,

    // Time tracking for effects
    start_time: std::time::Instant,
}

impl OsciApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let buffer = SampleBuffer::new(BUFFER_SIZE);
        let mut audio = AudioEngine::new(buffer.clone_ref());

        // Set initial shape
        let circle = Circle::new(0.8);
        audio.set_shape(&circle);

        Self {
            buffer,
            audio,
            oscilloscope: Oscilloscope::new(),
            show_settings: true, // Show by default for shape selection
            editor_mode: EditorMode::SingleShape,
            selected_shape: ShapeType::Circle,
            shape_params: ShapeParams::default(),
            shape_needs_update: false,
            scene_entries: Vec::new(),
            scene_shape_to_add: ShapeType::Circle,

            // SVG import
            loaded_svg: None,
            svg_options: SvgOptions::default(),
            svg_error: None,

            // Image import
            loaded_image: None,
            image_options: ImageOptions::default(),
            image_error: None,

            // Text rendering
            text_input: "Hello".to_string(),
            text_shape: None,
            text_options: TextOptions::default(),
            text_error: None,

            // 3D mesh rendering
            loaded_mesh: None,
            mesh_shape: None,
            mesh_options: Mesh3DOptions::default(),
            mesh_camera: Camera::default(),
            mesh_primitive: MeshPrimitive::Cube,
            mesh_error: None,

            // Effects
            enable_rotation: false,
            rotation_speed: 1.0,
            enable_scale_lfo: false,
            scale_lfo_freq: 2.0,
            scale_lfo_min: 0.8,
            scale_lfo_max: 1.2,
            scale_lfo_waveform: LfoWaveform::Sine,

            start_time: std::time::Instant::now(),
        }
    }

    /// Create and set the current shape based on selection and parameters
    fn update_shape(&mut self) {
        match self.selected_shape {
            ShapeType::Circle => {
                let shape = Circle::new(self.shape_params.size);
                self.audio.set_shape(&shape);
            }
            ShapeType::Rectangle => {
                let shape = Rectangle::new(
                    self.shape_params.width,
                    self.shape_params.height,
                );
                self.audio.set_shape(&shape);
            }
            ShapeType::Triangle => {
                let shape = Polygon::triangle(self.shape_params.size);
                self.audio.set_shape(&shape);
            }
            ShapeType::Square => {
                let shape = Rectangle::square(self.shape_params.size);
                self.audio.set_shape(&shape);
            }
            ShapeType::Pentagon => {
                let shape = Polygon::pentagon(self.shape_params.size);
                self.audio.set_shape(&shape);
            }
            ShapeType::Hexagon => {
                let shape = Polygon::hexagon(self.shape_params.size);
                self.audio.set_shape(&shape);
            }
            ShapeType::Star => {
                let shape = Polygon::star(
                    self.shape_params.points,
                    self.shape_params.size,
                    self.shape_params.inner_radius,
                );
                self.audio.set_shape(&shape);
            }
            ShapeType::Line => {
                let half = self.shape_params.size / 2.0;
                let shape = Line::new(-half, -half, half, half);
                self.audio.set_shape(&shape);
            }
            ShapeType::Heart => {
                let shape = Path::heart(self.shape_params.size, 200);
                self.audio.set_shape(&shape);
            }
            ShapeType::Lissajous => {
                let shape = Path::lissajous(
                    self.shape_params.lissajous_a,
                    self.shape_params.lissajous_b,
                    self.shape_params.lissajous_delta,
                    500,
                );
                self.audio.set_shape(&shape);
            }
            ShapeType::Spiral => {
                let shape = Path::spiral(
                    0.1,
                    self.shape_params.size,
                    self.shape_params.spiral_turns,
                    300,
                );
                self.audio.set_shape(&shape);
            }
            ShapeType::Svg => {
                // Use loaded SVG if available
                if let Some(ref svg) = self.loaded_svg {
                    self.audio.set_shape(svg);
                } else {
                    // No SVG loaded, show a placeholder circle
                    let shape = Circle::new(0.5);
                    self.audio.set_shape(&shape);
                }
            }
            ShapeType::Image => {
                // Use loaded image if available
                if let Some(ref img) = self.loaded_image {
                    self.audio.set_shape(img);
                } else {
                    // No image loaded, show a placeholder circle
                    let shape = Circle::new(0.5);
                    self.audio.set_shape(&shape);
                }
            }
            ShapeType::Text => {
                // Render text if we have input
                if !self.text_input.is_empty() {
                    match TextShape::new(&self.text_input, &self.text_options) {
                        Ok(text) => {
                            self.audio.set_shape(&text);
                            self.text_shape = Some(text);
                            self.text_error = None;
                        }
                        Err(e) => {
                            self.text_error = Some(e.to_string());
                            // Show placeholder
                            let shape = Circle::new(0.5);
                            self.audio.set_shape(&shape);
                        }
                    }
                } else {
                    let shape = Circle::new(0.5);
                    self.audio.set_shape(&shape);
                }
            }
            ShapeType::Mesh3D => {
                // Get the mesh (from primitive or loaded file)
                let mesh = if self.mesh_primitive == MeshPrimitive::Custom {
                    self.loaded_mesh.clone()
                } else {
                    self.mesh_primitive.to_mesh()
                };

                if let Some(mesh) = mesh {
                    let shape = Mesh3DShape::new(mesh, self.mesh_options.clone())
                        .with_camera(self.mesh_camera.clone());
                    self.audio.set_shape(&shape);
                    self.mesh_shape = Some(shape);
                    self.mesh_error = None;
                } else {
                    // No mesh available, show placeholder
                    let shape = Circle::new(0.5);
                    self.audio.set_shape(&shape);
                }
            }
        }
        self.shape_needs_update = false;
    }

    /// Load an SVG file using file dialog
    fn load_svg_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("SVG Files", &["svg"])
            .pick_file()
        {
            match SvgShape::load(&path, &self.svg_options) {
                Ok(svg) => {
                    log::info!(
                        "Loaded SVG: {} ({} paths, {} points)",
                        path.display(),
                        svg.path_count(),
                        svg.point_count()
                    );
                    self.loaded_svg = Some(svg);
                    self.selected_shape = ShapeType::Svg;
                    self.svg_error = None;
                    self.shape_needs_update = true;
                }
                Err(e) => {
                    log::error!("Failed to load SVG: {}", e);
                    self.svg_error = Some(e.to_string());
                }
            }
        }
    }

    /// Load an image file using file dialog
    fn load_image_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Image Files", &["png", "jpg", "jpeg", "gif", "bmp", "webp"])
            .pick_file()
        {
            match ImageShape::load(&path, &self.image_options) {
                Ok(img) => {
                    let (w, h) = img.dimensions();
                    log::info!(
                        "Loaded image: {} ({}x{}, {} edge points)",
                        path.display(),
                        w, h,
                        img.point_count()
                    );
                    self.loaded_image = Some(img);
                    self.selected_shape = ShapeType::Image;
                    self.image_error = None;
                    self.shape_needs_update = true;
                }
                Err(e) => {
                    log::error!("Failed to load image: {}", e);
                    self.image_error = Some(e.to_string());
                }
            }
        }
    }

    /// Reload image with current options
    fn reload_image(&mut self) {
        // If we have a loaded image, we need to reload from file
        // For now, just trigger an update - user can reload manually
        self.shape_needs_update = true;
    }

    /// Load an OBJ file using file dialog
    fn load_obj_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("OBJ Files", &["obj"])
            .pick_file()
        {
            match Mesh::from_obj(&path) {
                Ok(mesh) => {
                    log::info!(
                        "Loaded OBJ: {} ({} vertices, {} edges)",
                        path.display(),
                        mesh.vertices.len(),
                        mesh.edges.len()
                    );
                    self.loaded_mesh = Some(mesh);
                    self.mesh_primitive = MeshPrimitive::Custom;
                    self.selected_shape = ShapeType::Mesh3D;
                    self.mesh_error = None;
                    self.shape_needs_update = true;
                }
                Err(e) => {
                    log::error!("Failed to load OBJ: {}", e);
                    self.mesh_error = Some(e.to_string());
                }
            }
        }
    }

    /// Build and set the scene from scene entries
    fn update_scene(&mut self) {
        let mut scene = Scene::new("Custom Scene");

        for entry in &self.scene_entries {
            if entry.enabled {
                // Create shape based on type (using default params for simplicity)
                match entry.shape_type {
                    ShapeType::Circle => {
                        scene.add_weighted(Circle::new(0.7), entry.weight);
                    }
                    ShapeType::Rectangle => {
                        scene.add_weighted(Rectangle::new(1.0, 0.6), entry.weight);
                    }
                    ShapeType::Triangle => {
                        scene.add_weighted(Polygon::triangle(0.7), entry.weight);
                    }
                    ShapeType::Square => {
                        scene.add_weighted(Rectangle::square(0.7), entry.weight);
                    }
                    ShapeType::Pentagon => {
                        scene.add_weighted(Polygon::pentagon(0.7), entry.weight);
                    }
                    ShapeType::Hexagon => {
                        scene.add_weighted(Polygon::hexagon(0.7), entry.weight);
                    }
                    ShapeType::Star => {
                        scene.add_weighted(Polygon::star(5, 0.7, 0.3), entry.weight);
                    }
                    ShapeType::Line => {
                        scene.add_weighted(Line::new(-0.5, -0.5, 0.5, 0.5), entry.weight);
                    }
                    ShapeType::Heart => {
                        scene.add_weighted(Path::heart(0.7, 200), entry.weight);
                    }
                    ShapeType::Lissajous => {
                        scene.add_weighted(Path::lissajous(3.0, 2.0, std::f32::consts::FRAC_PI_2, 500), entry.weight);
                    }
                    ShapeType::Spiral => {
                        scene.add_weighted(Path::spiral(0.1, 0.7, 3.0, 300), entry.weight);
                    }
                    ShapeType::Svg => {
                        if let Some(ref svg) = self.loaded_svg {
                            scene.add_weighted(svg.clone(), entry.weight);
                        } else {
                            scene.add_weighted(Circle::new(0.5), entry.weight);
                        }
                    }
                    ShapeType::Image => {
                        if let Some(ref img) = self.loaded_image {
                            scene.add_weighted(img.clone(), entry.weight);
                        } else {
                            scene.add_weighted(Circle::new(0.5), entry.weight);
                        }
                    }
                    ShapeType::Text => {
                        if let Some(ref text) = self.text_shape {
                            scene.add_weighted(text.clone(), entry.weight);
                        } else {
                            scene.add_weighted(Circle::new(0.5), entry.weight);
                        }
                    }
                    ShapeType::Mesh3D => {
                        // 3D mesh in scene - use cube as default
                        let mesh = Mesh::cube();
                        let shape = Mesh3DShape::new(mesh, Mesh3DOptions::default());
                        scene.add_weighted(shape, entry.weight);
                    }
                }
            }
        }

        if !scene.is_empty() {
            self.audio.set_shape(&scene);
        }
        self.shape_needs_update = false;
    }
}

impl eframe::App for OsciApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        // Update shape if parameters changed
        if self.shape_needs_update {
            match self.editor_mode {
                EditorMode::SingleShape => self.update_shape(),
                EditorMode::Scene => self.update_scene(),
            }
        }

        // Top panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("osci-rs");
                ui.separator();

                // Play/Stop button
                let button_text = if self.audio.is_playing() {
                    "⏹ Stop"
                } else {
                    "▶ Play"
                };

                if ui.button(button_text).clicked() {
                    self.audio.toggle();
                }

                ui.separator();
                ui.toggle_value(&mut self.show_settings, "⚙ Settings");
                ui.separator();
                ui.label(&self.audio.status);
            });
        });

        // Settings panel
        if self.show_settings {
            egui::SidePanel::left("settings_panel")
                .min_width(240.0)
                .show(ctx, |ui| {
                    // Mode toggle
                    ui.horizontal(|ui| {
                        ui.label("Mode:");
                        if ui.selectable_label(self.editor_mode == EditorMode::SingleShape, "Single").clicked() {
                            self.editor_mode = EditorMode::SingleShape;
                            self.shape_needs_update = true;
                        }
                        if ui.selectable_label(self.editor_mode == EditorMode::Scene, "Scene").clicked() {
                            self.editor_mode = EditorMode::Scene;
                            self.shape_needs_update = true;
                        }
                    });
                    ui.separator();

                    match self.editor_mode {
                        EditorMode::SingleShape => {
                            ui.heading("Shape");
                            ui.separator();

                            // Shape type selection
                            egui::ComboBox::from_label("Type")
                                .selected_text(self.selected_shape.name())
                                .show_ui(ui, |ui| {
                                    for shape_type in ShapeType::all() {
                                        if ui.selectable_value(
                                            &mut self.selected_shape,
                                            *shape_type,
                                            shape_type.name(),
                                        ).clicked() {
                                            self.shape_needs_update = true;
                                        }
                                    }
                                });

                            ui.separator();

                    // Shape-specific parameters
                    ui.label("Parameters:");

                    match self.selected_shape {
                        ShapeType::Circle | ShapeType::Triangle | ShapeType::Square |
                        ShapeType::Pentagon | ShapeType::Hexagon | ShapeType::Line |
                        ShapeType::Heart => {
                            if ui.add(
                                egui::Slider::new(&mut self.shape_params.size, 0.1..=1.0)
                                    .text("Size")
                            ).changed() {
                                self.shape_needs_update = true;
                            }
                        }

                        ShapeType::Rectangle => {
                            if ui.add(
                                egui::Slider::new(&mut self.shape_params.width, 0.1..=1.8)
                                    .text("Width")
                            ).changed() {
                                self.shape_needs_update = true;
                            }
                            if ui.add(
                                egui::Slider::new(&mut self.shape_params.height, 0.1..=1.8)
                                    .text("Height")
                            ).changed() {
                                self.shape_needs_update = true;
                            }
                        }

                        ShapeType::Star => {
                            if ui.add(
                                egui::Slider::new(&mut self.shape_params.points, 3..=12)
                                    .text("Points")
                            ).changed() {
                                self.shape_needs_update = true;
                            }
                            if ui.add(
                                egui::Slider::new(&mut self.shape_params.size, 0.1..=1.0)
                                    .text("Outer radius")
                            ).changed() {
                                self.shape_needs_update = true;
                            }
                            if ui.add(
                                egui::Slider::new(&mut self.shape_params.inner_radius, 0.1..=0.9)
                                    .text("Inner radius")
                            ).changed() {
                                self.shape_needs_update = true;
                            }
                        }

                        ShapeType::Lissajous => {
                            if ui.add(
                                egui::Slider::new(&mut self.shape_params.lissajous_a, 1.0..=10.0)
                                    .text("A (X freq)")
                            ).changed() {
                                self.shape_needs_update = true;
                            }
                            if ui.add(
                                egui::Slider::new(&mut self.shape_params.lissajous_b, 1.0..=10.0)
                                    .text("B (Y freq)")
                            ).changed() {
                                self.shape_needs_update = true;
                            }
                            if ui.add(
                                egui::Slider::new(&mut self.shape_params.lissajous_delta, 0.0..=std::f32::consts::PI)
                                    .text("Phase")
                            ).changed() {
                                self.shape_needs_update = true;
                            }
                        }

                        ShapeType::Spiral => {
                            if ui.add(
                                egui::Slider::new(&mut self.shape_params.size, 0.2..=1.0)
                                    .text("Radius")
                            ).changed() {
                                self.shape_needs_update = true;
                            }
                            if ui.add(
                                egui::Slider::new(&mut self.shape_params.spiral_turns, 1.0..=10.0)
                                    .text("Turns")
                            ).changed() {
                                self.shape_needs_update = true;
                            }
                        }

                        ShapeType::Svg => {
                            // SVG loading UI
                            if ui.button("Load SVG File...").clicked() {
                                self.load_svg_file();
                            }

                            // Show SVG info if loaded
                            if let Some(ref svg) = self.loaded_svg {
                                ui.label(format!("Paths: {}", svg.path_count()));
                                ui.label(format!("Points: {}", svg.point_count()));
                            } else {
                                ui.label("No SVG loaded");
                            }

                            // Show error if any
                            if let Some(ref error) = self.svg_error {
                                ui.colored_label(egui::Color32::RED, error);
                            }

                            ui.separator();
                            ui.label("SVG Options:");

                            // Curve samples
                            if ui.add(
                                egui::Slider::new(&mut self.svg_options.curve_samples, 2..=32)
                                    .text("Curve detail")
                            ).changed() {
                                // Reload SVG with new options
                                self.shape_needs_update = true;
                            }

                            // Close paths option
                            if ui.checkbox(&mut self.svg_options.close_paths, "Close open paths").changed() {
                                self.shape_needs_update = true;
                            }
                        }

                        ShapeType::Image => {
                            // Image loading UI
                            if ui.button("Load Image File...").clicked() {
                                self.load_image_file();
                            }

                            // Show image info if loaded
                            if let Some(ref img) = self.loaded_image {
                                let (w, h) = img.dimensions();
                                ui.label(format!("Size: {}x{}", w, h));
                                ui.label(format!("Edge points: {}", img.point_count()));
                            } else {
                                ui.label("No image loaded");
                            }

                            // Show error if any
                            if let Some(ref error) = self.image_error {
                                ui.colored_label(egui::Color32::RED, error);
                            }

                            ui.separator();
                            ui.label("Edge Detection:");

                            // Threshold slider
                            if ui.add(
                                egui::Slider::new(&mut self.image_options.threshold, 0.05..=0.9)
                                    .text("Threshold")
                            ).changed() {
                                self.shape_needs_update = true;
                            }

                            // Edge minimum
                            if ui.add(
                                egui::Slider::new(&mut self.image_options.edge_min, 0.0..=0.5)
                                    .text("Min edge")
                            ).changed() {
                                self.shape_needs_update = true;
                            }

                            // Max points
                            if ui.add(
                                egui::Slider::new(&mut self.image_options.max_points, 500..=20000)
                                    .text("Max points")
                                    .logarithmic(true)
                            ).changed() {
                                self.shape_needs_update = true;
                            }

                            // Invert option
                            if ui.checkbox(&mut self.image_options.invert, "Invert image").changed() {
                                self.shape_needs_update = true;
                            }

                            // Reload button
                            if self.loaded_image.is_some() && ui.button("Reload with options").clicked() {
                                // Need to reload from file - for now just show message
                                ui.label("Reload from file to apply");
                            }
                        }

                        ShapeType::Text => {
                            // Text input
                            ui.label("Enter text:");
                            let response = ui.text_edit_singleline(&mut self.text_input);
                            if response.changed() {
                                self.shape_needs_update = true;
                            }

                            // Show text info if rendered
                            if let Some(ref text) = self.text_shape {
                                ui.label(format!("Points: {}", text.point_count()));
                            }

                            // Show error if any
                            if let Some(ref error) = self.text_error {
                                ui.colored_label(egui::Color32::RED, error);
                            }

                            ui.separator();
                            ui.label("Text Options:");

                            // Font size
                            if ui.add(
                                egui::Slider::new(&mut self.text_options.size, 16.0..=128.0)
                                    .text("Font size")
                            ).changed() {
                                self.shape_needs_update = true;
                            }

                            // Curve detail
                            if ui.add(
                                egui::Slider::new(&mut self.text_options.curve_samples, 2..=16)
                                    .text("Curve detail")
                            ).changed() {
                                self.shape_needs_update = true;
                            }

                            // Letter spacing
                            if ui.add(
                                egui::Slider::new(&mut self.text_options.letter_spacing, 0.5..=2.0)
                                    .text("Letter spacing")
                            ).changed() {
                                self.shape_needs_update = true;
                            }
                        }

                        ShapeType::Mesh3D => {
                            // Mesh primitive selection
                            ui.label("Model:");
                            egui::ComboBox::from_id_salt("mesh_primitive")
                                .selected_text(self.mesh_primitive.name())
                                .show_ui(ui, |ui| {
                                    for primitive in MeshPrimitive::all() {
                                        if ui.selectable_value(
                                            &mut self.mesh_primitive,
                                            *primitive,
                                            primitive.name(),
                                        ).clicked() {
                                            self.shape_needs_update = true;
                                        }
                                    }
                                });

                            // Load OBJ button (for Custom)
                            if self.mesh_primitive == MeshPrimitive::Custom {
                                if ui.button("Load OBJ File...").clicked() {
                                    self.load_obj_file();
                                }
                            }

                            // Show mesh info
                            let mesh = if self.mesh_primitive == MeshPrimitive::Custom {
                                self.loaded_mesh.as_ref()
                            } else {
                                None
                            };
                            if let Some(mesh) = mesh {
                                ui.label(format!("Vertices: {}", mesh.vertices.len()));
                                ui.label(format!("Edges: {}", mesh.edges.len()));
                            } else if self.mesh_primitive != MeshPrimitive::Custom {
                                if let Some(m) = self.mesh_primitive.to_mesh() {
                                    ui.label(format!("Vertices: {}", m.vertices.len()));
                                    ui.label(format!("Edges: {}", m.edges.len()));
                                }
                            } else {
                                ui.label("No mesh loaded");
                            }

                            // Show error if any
                            if let Some(ref error) = self.mesh_error {
                                ui.colored_label(egui::Color32::RED, error);
                            }

                            ui.separator();
                            ui.label("Camera:");

                            // Camera orbit controls
                            ui.horizontal(|ui| {
                                if ui.button("↺").clicked() {
                                    self.mesh_camera.orbit(-0.3, 0.0);
                                    self.shape_needs_update = true;
                                }
                                if ui.button("↻").clicked() {
                                    self.mesh_camera.orbit(0.3, 0.0);
                                    self.shape_needs_update = true;
                                }
                                if ui.button("↑").clicked() {
                                    self.mesh_camera.orbit(0.0, 0.2);
                                    self.shape_needs_update = true;
                                }
                                if ui.button("↓").clicked() {
                                    self.mesh_camera.orbit(0.0, -0.2);
                                    self.shape_needs_update = true;
                                }
                            });

                            // Zoom controls
                            ui.horizontal(|ui| {
                                ui.label("Zoom:");
                                if ui.button("-").clicked() {
                                    self.mesh_camera.zoom(1.2);
                                    self.shape_needs_update = true;
                                }
                                if ui.button("+").clicked() {
                                    self.mesh_camera.zoom(0.8);
                                    self.shape_needs_update = true;
                                }
                            });

                            // FOV slider (degrees, converted to radians)
                            let mut fov_deg = self.mesh_camera.fov_degrees();
                            if ui.add(
                                egui::Slider::new(&mut fov_deg, 30.0..=120.0)
                                    .text("FOV")
                            ).changed() {
                                self.mesh_camera.set_fov_degrees(fov_deg);
                                self.shape_needs_update = true;
                            }

                            // Reset camera button
                            if ui.button("Reset Camera").clicked() {
                                self.mesh_camera = Camera::default();
                                self.shape_needs_update = true;
                            }

                            ui.separator();
                            ui.label("Rendering:");

                            // Line detail slider
                            if ui.add(
                                egui::Slider::new(&mut self.mesh_options.edge_samples, 2..=50)
                                    .text("Edge detail")
                            ).changed() {
                                self.shape_needs_update = true;
                            }
                        }
                    }
                        } // end SingleShape

                        EditorMode::Scene => {
                            ui.heading("Scene");
                            ui.separator();

                            // Add shape to scene
                            ui.horizontal(|ui| {
                                egui::ComboBox::from_id_salt("add_shape")
                                    .selected_text(self.scene_shape_to_add.name())
                                    .show_ui(ui, |ui| {
                                        for shape_type in ShapeType::all() {
                                            ui.selectable_value(
                                                &mut self.scene_shape_to_add,
                                                *shape_type,
                                                shape_type.name(),
                                            );
                                        }
                                    });
                                if ui.button("+ Add").clicked() {
                                    self.scene_entries.push(SceneEntry::new(self.scene_shape_to_add));
                                    self.shape_needs_update = true;
                                }
                            });

                            ui.separator();

                            if self.scene_entries.is_empty() {
                                ui.label("No shapes in scene. Add shapes above.");
                            } else {
                                ui.label(format!("{} shapes:", self.scene_entries.len()));

                                // List of shapes with controls
                                let mut to_remove: Option<usize> = None;
                                let mut to_move_up: Option<usize> = None;
                                let mut to_move_down: Option<usize> = None;

                                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                                    for (i, entry) in self.scene_entries.iter_mut().enumerate() {
                                        ui.horizontal(|ui| {
                                            // Enable checkbox
                                            if ui.checkbox(&mut entry.enabled, "").changed() {
                                                self.shape_needs_update = true;
                                            }

                                            // Shape name
                                            ui.label(entry.shape_type.name());

                                            // Weight slider
                                            if ui.add(
                                                egui::Slider::new(&mut entry.weight, 0.1..=3.0)
                                                    .show_value(false)
                                            ).changed() {
                                                self.shape_needs_update = true;
                                            }

                                            // Move up/down buttons
                                            if ui.small_button("▲").clicked() {
                                                to_move_up = Some(i);
                                            }
                                            if ui.small_button("▼").clicked() {
                                                to_move_down = Some(i);
                                            }

                                            // Remove button
                                            if ui.small_button("✕").clicked() {
                                                to_remove = Some(i);
                                            }
                                        });
                                    }
                                });

                                // Process deferred actions
                                if let Some(i) = to_remove {
                                    self.scene_entries.remove(i);
                                    self.shape_needs_update = true;
                                }
                                if let Some(i) = to_move_up {
                                    if i > 0 {
                                        self.scene_entries.swap(i, i - 1);
                                        self.shape_needs_update = true;
                                    }
                                }
                                if let Some(i) = to_move_down {
                                    if i + 1 < self.scene_entries.len() {
                                        self.scene_entries.swap(i, i + 1);
                                        self.shape_needs_update = true;
                                    }
                                }

                                ui.separator();

                                if ui.button("Clear All").clicked() {
                                    self.scene_entries.clear();
                                    self.shape_needs_update = true;
                                }
                            }
                        }
                    } // end match editor_mode

                    ui.separator();

                    // Audio settings
                    ui.collapsing("Audio", |ui| {
                        if ui.add(
                            egui::Slider::new(&mut self.audio.config.frequency, 20.0..=200.0)
                                .text("Speed (Hz)")
                                .logarithmic(true)
                        ).changed() {
                            self.shape_needs_update = true;
                        }

                        if ui.add(
                            egui::Slider::new(&mut self.audio.config.volume, 0.0..=1.0)
                                .text("Volume")
                        ).changed() {
                            self.shape_needs_update = true;
                        }
                    });

                    ui.separator();

                    // Effects settings
                    ui.collapsing("Effects", |ui| {
                        // Rotation effect
                        ui.checkbox(&mut self.enable_rotation, "Rotation");
                        if self.enable_rotation {
                            ui.add(
                                egui::Slider::new(&mut self.rotation_speed, -5.0..=5.0)
                                    .text("Speed (rad/s)")
                            );
                        }

                        ui.separator();

                        // Scale LFO effect
                        ui.checkbox(&mut self.enable_scale_lfo, "Pulsing Scale");
                        if self.enable_scale_lfo {
                            ui.add(
                                egui::Slider::new(&mut self.scale_lfo_freq, 0.1..=10.0)
                                    .text("Frequency (Hz)")
                            );
                            ui.add(
                                egui::Slider::new(&mut self.scale_lfo_min, 0.1..=1.5)
                                    .text("Min scale")
                            );
                            ui.add(
                                egui::Slider::new(&mut self.scale_lfo_max, 0.5..=2.0)
                                    .text("Max scale")
                            );

                            // Waveform selection
                            egui::ComboBox::from_label("Waveform")
                                .selected_text(self.scale_lfo_waveform.name())
                                .show_ui(ui, |ui| {
                                    for waveform in LfoWaveform::all() {
                                        ui.selectable_value(
                                            &mut self.scale_lfo_waveform,
                                            *waveform,
                                            waveform.name(),
                                        );
                                    }
                                });
                        }

                        // Update effect parameters on the audio engine
                        self.audio.set_effects(EffectParams {
                            rotation_speed: self.rotation_speed,
                            rotation_enabled: self.enable_rotation,
                            scale_lfo_freq: self.scale_lfo_freq,
                            scale_lfo_min: self.scale_lfo_min,
                            scale_lfo_max: self.scale_lfo_max,
                            scale_lfo_enabled: self.enable_scale_lfo,
                            scale_lfo_waveform: self.scale_lfo_waveform,
                        });
                    });

                    ui.separator();

                    // Display settings
                    ui.collapsing("Display", |ui| {
                        ui.add(egui::Slider::new(&mut self.oscilloscope.settings.zoom, 0.1..=2.0).text("Zoom"));
                        ui.add(egui::Slider::new(&mut self.oscilloscope.settings.line_width, 0.5..=5.0).text("Line width"));
                        ui.add(egui::Slider::new(&mut self.oscilloscope.settings.intensity, 0.1..=1.0).text("Intensity"));
                        ui.add(egui::Slider::new(&mut self.oscilloscope.settings.persistence, 0.0..=0.99).text("Persistence"));
                        ui.checkbox(&mut self.oscilloscope.settings.show_graticule, "Show grid");
                        ui.checkbox(&mut self.oscilloscope.settings.draw_lines, "Draw lines");

                        if ui.button("Clear trail").clicked() {
                            self.oscilloscope.clear_persistence();
                        }
                    });

                    ui.separator();

                    // Color presets
                    ui.collapsing("Color", |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Green").clicked() {
                                self.oscilloscope.settings.color = egui::Color32::from_rgb(100, 255, 100);
                                self.oscilloscope.settings.background = egui::Color32::from_rgb(10, 20, 10);
                            }
                            if ui.button("Amber").clicked() {
                                self.oscilloscope.settings.color = egui::Color32::from_rgb(255, 176, 0);
                                self.oscilloscope.settings.background = egui::Color32::from_rgb(20, 15, 5);
                            }
                            if ui.button("Blue").clicked() {
                                self.oscilloscope.settings.color = egui::Color32::from_rgb(100, 150, 255);
                                self.oscilloscope.settings.background = egui::Color32::from_rgb(10, 10, 20);
                            }
                        });
                    });
                });
        }

        // Main oscilloscope display
        egui::CentralPanel::default().show(ctx, |ui| {
            let samples = self.buffer.get_samples();
            self.oscilloscope.show(ui, &samples, None);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.small(format!("Shape: {}", self.audio.current_shape_name()));
                    ui.separator();
                    ui.small(format!("Samples: {}", samples.len()));
                    ui.separator();
                    ui.small("Milestone 13: 3D Mesh Rendering");
                });
            });
        });
    }
}
