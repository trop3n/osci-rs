use std::path::PathBuf;

use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::midi::MidiMapping;
use crate::{EditorMode, LfoWaveform, MeshPrimitive, OsciApp, ShapeType};

/// Returns the path to the settings file: `~/.config/osci-rs/settings.json`
fn settings_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("osci-rs");
    path.push("settings.json");
    path
}

/// Persisted application settings.
///
/// Serialized as JSON to the platform config directory.
/// Fields use `#[serde(default)]` so that adding new settings
/// won't break existing config files.
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    // Editor
    pub editor_mode: EditorMode,
    pub selected_shape: ShapeType,
    pub show_settings: bool,

    // Shape params
    pub size: f32,
    pub width: f32,
    pub height: f32,
    pub inner_radius: f32,
    pub points: usize,
    pub lissajous_a: f32,
    pub lissajous_b: f32,
    pub lissajous_delta: f32,
    pub spiral_turns: f32,

    // Audio
    pub frequency: f32,
    pub volume: f32,

    // Effects
    pub enable_rotation: bool,
    pub rotation_speed: f32,
    pub enable_scale_lfo: bool,
    pub scale_lfo_freq: f32,
    pub scale_lfo_min: f32,
    pub scale_lfo_max: f32,
    pub scale_lfo_waveform: LfoWaveform,

    // Display
    pub line_width: f32,
    pub draw_lines: bool,
    pub intensity: f32,
    pub zoom: f32,
    pub show_graticule: bool,
    pub persistence: f32,

    // Color (stored as u8 triples since Color32 isn't serde-friendly)
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    pub background_r: u8,
    pub background_g: u8,
    pub background_b: u8,

    // Text
    pub text_input: String,

    // 3D
    pub mesh_primitive: MeshPrimitive,

    // MIDI
    pub midi_mappings: Vec<MidiMapping>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            editor_mode: EditorMode::SingleShape,
            selected_shape: ShapeType::Circle,
            show_settings: true,

            size: 0.8,
            width: 1.2,
            height: 0.6,
            inner_radius: 0.3,
            points: 5,
            lissajous_a: 3.0,
            lissajous_b: 2.0,
            lissajous_delta: std::f32::consts::FRAC_PI_2,
            spiral_turns: 3.0,

            frequency: 80.0,
            volume: 0.8,

            enable_rotation: false,
            rotation_speed: 1.0,
            enable_scale_lfo: false,
            scale_lfo_freq: 2.0,
            scale_lfo_min: 0.8,
            scale_lfo_max: 1.2,
            scale_lfo_waveform: LfoWaveform::Sine,

            line_width: 1.5,
            draw_lines: true,
            intensity: 1.0,
            zoom: 1.0,
            show_graticule: true,
            persistence: 0.85,

            color_r: 100,
            color_g: 255,
            color_b: 100,
            background_r: 10,
            background_g: 20,
            background_b: 10,

            text_input: "Hello".to_string(),

            mesh_primitive: MeshPrimitive::Cube,

            midi_mappings: Vec::new(),
        }
    }
}

impl AppSettings {
    /// Load settings from disk, falling back to defaults on any error.
    pub fn load() -> Self {
        let path = settings_path();
        match std::fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(settings) => {
                    log::info!("Loaded settings from {}", path.display());
                    settings
                }
                Err(e) => {
                    log::warn!("Failed to parse settings ({}), using defaults", e);
                    Self::default()
                }
            },
            Err(e) => {
                log::info!("No settings file found ({}), using defaults", e);
                Self::default()
            }
        }
    }

    /// Save settings to disk as pretty JSON.
    pub fn save(&self) {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                log::warn!("Failed to create config directory: {}", e);
                return;
            }
        }
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    log::warn!("Failed to write settings: {}", e);
                }
            }
            Err(e) => {
                log::warn!("Failed to serialize settings: {}", e);
            }
        }
    }

    /// Extract current settings from the running application.
    pub fn from_app(app: &OsciApp) -> Self {
        Self {
            editor_mode: app.editor_mode,
            selected_shape: app.selected_shape,
            show_settings: app.show_settings,

            size: app.shape_params.size,
            width: app.shape_params.width,
            height: app.shape_params.height,
            inner_radius: app.shape_params.inner_radius,
            points: app.shape_params.points,
            lissajous_a: app.shape_params.lissajous_a,
            lissajous_b: app.shape_params.lissajous_b,
            lissajous_delta: app.shape_params.lissajous_delta,
            spiral_turns: app.shape_params.spiral_turns,

            frequency: app.audio.config.frequency,
            volume: app.audio.config.volume,

            enable_rotation: app.enable_rotation,
            rotation_speed: app.rotation_speed,
            enable_scale_lfo: app.enable_scale_lfo,
            scale_lfo_freq: app.scale_lfo_freq,
            scale_lfo_min: app.scale_lfo_min,
            scale_lfo_max: app.scale_lfo_max,
            scale_lfo_waveform: app.scale_lfo_waveform,

            line_width: app.oscilloscope.settings.line_width,
            draw_lines: app.oscilloscope.settings.draw_lines,
            intensity: app.oscilloscope.settings.intensity,
            zoom: app.oscilloscope.settings.zoom,
            show_graticule: app.oscilloscope.settings.show_graticule,
            persistence: app.oscilloscope.settings.persistence,

            color_r: app.oscilloscope.settings.color.r(),
            color_g: app.oscilloscope.settings.color.g(),
            color_b: app.oscilloscope.settings.color.b(),
            background_r: app.oscilloscope.settings.background.r(),
            background_g: app.oscilloscope.settings.background.g(),
            background_b: app.oscilloscope.settings.background.b(),

            text_input: app.text_input.clone(),

            mesh_primitive: app.mesh_primitive,

            midi_mappings: app.midi.mappings.clone(),
        }
    }

    /// Apply loaded settings to the running application.
    pub fn apply(&self, app: &mut OsciApp) {
        app.editor_mode = self.editor_mode;
        app.selected_shape = self.selected_shape;
        app.show_settings = self.show_settings;

        app.shape_params.size = self.size;
        app.shape_params.width = self.width;
        app.shape_params.height = self.height;
        app.shape_params.inner_radius = self.inner_radius;
        app.shape_params.points = self.points;
        app.shape_params.lissajous_a = self.lissajous_a;
        app.shape_params.lissajous_b = self.lissajous_b;
        app.shape_params.lissajous_delta = self.lissajous_delta;
        app.shape_params.spiral_turns = self.spiral_turns;

        app.audio.config.frequency = self.frequency;
        app.audio.config.volume = self.volume;

        app.enable_rotation = self.enable_rotation;
        app.rotation_speed = self.rotation_speed;
        app.enable_scale_lfo = self.enable_scale_lfo;
        app.scale_lfo_freq = self.scale_lfo_freq;
        app.scale_lfo_min = self.scale_lfo_min;
        app.scale_lfo_max = self.scale_lfo_max;
        app.scale_lfo_waveform = self.scale_lfo_waveform;

        app.oscilloscope.settings.line_width = self.line_width;
        app.oscilloscope.settings.draw_lines = self.draw_lines;
        app.oscilloscope.settings.intensity = self.intensity;
        app.oscilloscope.settings.zoom = self.zoom;
        app.oscilloscope.settings.show_graticule = self.show_graticule;
        app.oscilloscope.settings.persistence = self.persistence;

        app.oscilloscope.settings.color =
            egui::Color32::from_rgb(self.color_r, self.color_g, self.color_b);
        app.oscilloscope.settings.background =
            egui::Color32::from_rgb(self.background_r, self.background_g, self.background_b);

        app.text_input = self.text_input.clone();

        app.mesh_primitive = self.mesh_primitive;

        app.midi.mappings = self.midi_mappings.clone();

        app.shape_needs_update = true;
    }
}
