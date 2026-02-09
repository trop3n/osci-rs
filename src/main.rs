//! osci-rs - Oscilloscope Music Generator
//!
//! This application converts vector graphics into XY audio signals
//! that can be displayed on an oscilloscope.
//!
//! ## Milestone 4: Basic Shapes
//! This version adds:
//! - Shape trait for abstracting drawable shapes
//! - Primitive shapes: Circle, Line, Rectangle, Polygon
//! - Path type for arbitrary point sequences
//! - Shape selection UI

use eframe::egui;

mod audio;
mod render;
mod shapes;

use audio::{AudioEngine, SampleBuffer};
use render::Oscilloscope;
use shapes::{Circle, Line, Rectangle, Polygon, Path, Shape};

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

    // Shape selection
    selected_shape: ShapeType,
    shape_params: ShapeParams,
    shape_needs_update: bool,
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
            selected_shape: ShapeType::Circle,
            shape_params: ShapeParams::default(),
            shape_needs_update: false,
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
        }
        self.shape_needs_update = false;
    }
}

impl eframe::App for OsciApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        // Update shape if parameters changed
        if self.shape_needs_update {
            self.update_shape();
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
                .min_width(220.0)
                .show(ctx, |ui| {
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
                    }

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
                    ui.small("Milestone 4: Basic Shapes");
                });
            });
        });
    }
}
