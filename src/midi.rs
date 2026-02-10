//! MIDI input handling
//!
//! Receives MIDI CC messages and maps them to osci-rs parameters.
//! Uses a lock-free approach: the MIDI callback writes to shared atomics
//! that the UI thread reads each frame.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use midir::{MidiInput, MidiInputConnection};
use serde::{Deserialize, Serialize};

/// A parameter that can be controlled via MIDI CC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MidiParam {
    Frequency,
    Volume,
    RotationSpeed,
    ScaleLfoFreq,
    ScaleLfoMin,
    ScaleLfoMax,
    LineWidth,
    Intensity,
    Persistence,
    Zoom,
}

impl MidiParam {
    pub const ALL: &[MidiParam] = &[
        Self::Frequency,
        Self::Volume,
        Self::RotationSpeed,
        Self::ScaleLfoFreq,
        Self::ScaleLfoMin,
        Self::ScaleLfoMax,
        Self::LineWidth,
        Self::Intensity,
        Self::Persistence,
        Self::Zoom,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Frequency => "Frequency",
            Self::Volume => "Volume",
            Self::RotationSpeed => "Rotation Speed",
            Self::ScaleLfoFreq => "Scale LFO Freq",
            Self::ScaleLfoMin => "Scale LFO Min",
            Self::ScaleLfoMax => "Scale LFO Max",
            Self::LineWidth => "Line Width",
            Self::Intensity => "Intensity",
            Self::Persistence => "Persistence",
            Self::Zoom => "Zoom",
        }
    }

    /// Map a MIDI CC value (0-127) to this parameter's range
    pub fn map_value(&self, cc_value: u8) -> f32 {
        let t = cc_value as f32 / 127.0;
        let (min, max) = self.range();
        min + t * (max - min)
    }

    /// The (min, max) range for this parameter
    fn range(&self) -> (f32, f32) {
        match self {
            Self::Frequency => (20.0, 200.0),
            Self::Volume => (0.0, 1.0),
            Self::RotationSpeed => (-5.0, 5.0),
            Self::ScaleLfoFreq => (0.1, 10.0),
            Self::ScaleLfoMin => (0.1, 1.5),
            Self::ScaleLfoMax => (0.5, 2.0),
            Self::LineWidth => (0.5, 5.0),
            Self::Intensity => (0.1, 1.0),
            Self::Persistence => (0.0, 0.99),
            Self::Zoom => (0.1, 2.0),
        }
    }
}

/// A single CC-to-parameter mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiMapping {
    pub cc: u8,
    pub param: MidiParam,
}

/// Shared CC values written by the MIDI callback, read by the UI thread.
/// Index = CC number (0-127), value = last received CC value.
#[derive(Clone)]
struct SharedCcValues {
    values: Arc<[AtomicU8; 128]>,
    /// Tracks which CCs have been received since last poll.
    changed: Arc<[AtomicU8; 128]>,
}

impl SharedCcValues {
    fn new() -> Self {
        Self {
            values: Arc::new(std::array::from_fn(|_| AtomicU8::new(0))),
            changed: Arc::new(std::array::from_fn(|_| AtomicU8::new(0))),
        }
    }

    /// Called from MIDI callback thread
    fn set(&self, cc: u8, value: u8) {
        self.values[cc as usize].store(value, Ordering::Relaxed);
        self.changed[cc as usize].store(1, Ordering::Relaxed);
    }

    /// Read a CC value and clear its changed flag. Returns Some if changed since last poll.
    fn poll(&self, cc: u8) -> Option<u8> {
        if self.changed[cc as usize].swap(0, Ordering::Relaxed) != 0 {
            Some(self.values[cc as usize].load(Ordering::Relaxed))
        } else {
            None
        }
    }
}

/// MIDI input controller
pub struct MidiController {
    /// Available MIDI port names (refreshed on scan)
    pub ports: Vec<String>,

    /// Currently selected port index (for UI combo box)
    pub selected_port: usize,

    /// Active connection (None if disconnected)
    connection: Option<MidiInputConnection<()>>,

    /// Shared CC values between MIDI thread and UI
    cc_values: SharedCcValues,

    /// User-defined CC-to-parameter mappings
    pub mappings: Vec<MidiMapping>,

    /// Status message
    pub status: String,

    /// Whether currently connected
    pub is_connected: bool,

    /// CC number being learned (for MIDI learn mode)
    pub learning: Option<usize>,
}

impl MidiController {
    pub fn new() -> Self {
        let mut controller = Self {
            ports: Vec::new(),
            selected_port: 0,
            connection: None,
            cc_values: SharedCcValues::new(),
            mappings: Vec::new(),
            status: "Disconnected".to_string(),
            is_connected: false,
            learning: None,
        };
        controller.scan_ports();
        controller
    }

    /// Scan for available MIDI input ports
    pub fn scan_ports(&mut self) {
        self.ports.clear();
        match MidiInput::new("osci-rs-scan") {
            Ok(midi_in) => {
                for port in midi_in.ports().iter() {
                    let name = midi_in
                        .port_name(port)
                        .unwrap_or_else(|_| "Unknown".to_string());
                    self.ports.push(name);
                }
                if self.ports.is_empty() {
                    self.status = "No MIDI devices found".to_string();
                }
            }
            Err(e) => {
                self.status = format!("MIDI init error: {}", e);
            }
        }
    }

    /// Connect to the currently selected MIDI port
    pub fn connect(&mut self) {
        if self.is_connected {
            return;
        }

        let midi_in = match MidiInput::new("osci-rs") {
            Ok(m) => m,
            Err(e) => {
                self.status = format!("MIDI init error: {}", e);
                return;
            }
        };

        let ports = midi_in.ports();
        let port = match ports.get(self.selected_port) {
            Some(p) => p,
            None => {
                self.status = "Port not found".to_string();
                return;
            }
        };

        let port_name = midi_in
            .port_name(port)
            .unwrap_or_else(|_| "Unknown".to_string());

        let cc_values = self.cc_values.clone();

        match midi_in.connect(
            port,
            "osci-rs-input",
            move |_timestamp, message, _| {
                // Parse MIDI CC messages: [0xB0 | channel, cc_number, value]
                if message.len() == 3 && (message[0] & 0xF0) == 0xB0 {
                    let cc = message[1] & 0x7F;
                    let value = message[2] & 0x7F;
                    cc_values.set(cc, value);
                }
            },
            (),
        ) {
            Ok(conn) => {
                self.connection = Some(conn);
                self.is_connected = true;
                self.status = format!("Connected: {}", port_name);
                log::info!("MIDI connected: {}", port_name);
            }
            Err(e) => {
                self.status = format!("Connect error: {}", e);
                log::error!("MIDI connect error: {}", e);
            }
        }
    }

    /// Disconnect from the current MIDI port
    pub fn disconnect(&mut self) {
        if let Some(conn) = self.connection.take() {
            conn.close();
        }
        self.is_connected = false;
        self.learning = None;
        self.status = "Disconnected".to_string();
        log::info!("MIDI disconnected");
    }

    /// Toggle connection state
    pub fn toggle(&mut self) {
        if self.is_connected {
            self.disconnect();
        } else {
            self.connect();
        }
    }

    /// Poll for changed CC values and return parameter updates.
    /// Call this once per frame from the UI thread.
    pub fn poll(&mut self) -> Vec<(MidiParam, f32)> {
        let mut updates = Vec::new();

        // Check MIDI learn mode: any CC received assigns it to the learning mapping
        if let Some(mapping_idx) = self.learning {
            for cc in 0..128u8 {
                if self.cc_values.poll(cc).is_some() {
                    if let Some(mapping) = self.mappings.get_mut(mapping_idx) {
                        mapping.cc = cc;
                        log::info!("MIDI learn: CC {} -> {}", cc, mapping.param.name());
                    }
                    self.learning = None;
                    return updates;
                }
            }
            return updates;
        }

        // Normal mode: apply mapped CC values
        for mapping in &self.mappings {
            if let Some(cc_value) = self.cc_values.poll(mapping.cc) {
                let value = mapping.param.map_value(cc_value);
                updates.push((mapping.param, value));
            }
        }

        updates
    }

    /// Add a new mapping
    pub fn add_mapping(&mut self, cc: u8, param: MidiParam) {
        self.mappings.push(MidiMapping { cc, param });
    }

    /// Remove a mapping by index
    pub fn remove_mapping(&mut self, index: usize) {
        if index < self.mappings.len() {
            self.mappings.remove(index);
            if self.learning == Some(index) {
                self.learning = None;
            }
        }
    }

    /// Start MIDI learn mode for a mapping
    pub fn start_learn(&mut self, mapping_index: usize) {
        if mapping_index < self.mappings.len() {
            self.learning = Some(mapping_index);
        }
    }

    /// Cancel MIDI learn mode
    pub fn cancel_learn(&mut self) {
        self.learning = None;
    }

    /// Get available parameters not yet mapped
    pub fn unmapped_params(&self) -> Vec<MidiParam> {
        let mapped: std::collections::HashSet<MidiParam> =
            self.mappings.iter().map(|m| m.param).collect();
        MidiParam::ALL
            .iter()
            .filter(|p| !mapped.contains(p))
            .copied()
            .collect()
    }
}

/// Apply MIDI parameter updates to the app state.
pub fn apply_updates(updates: &[(MidiParam, f32)], app: &mut crate::OsciApp) {
    for &(param, value) in updates {
        match param {
            MidiParam::Frequency => {
                app.audio.config.frequency = value;
                app.shape_needs_update = true;
            }
            MidiParam::Volume => {
                app.audio.config.volume = value;
                app.shape_needs_update = true;
            }
            MidiParam::RotationSpeed => {
                app.rotation_speed = value;
            }
            MidiParam::ScaleLfoFreq => {
                app.scale_lfo_freq = value;
            }
            MidiParam::ScaleLfoMin => {
                app.scale_lfo_min = value;
            }
            MidiParam::ScaleLfoMax => {
                app.scale_lfo_max = value;
            }
            MidiParam::LineWidth => {
                app.oscilloscope.settings.line_width = value;
            }
            MidiParam::Intensity => {
                app.oscilloscope.settings.intensity = value;
            }
            MidiParam::Persistence => {
                app.oscilloscope.settings.persistence = value;
            }
            MidiParam::Zoom => {
                app.oscilloscope.settings.zoom = value;
            }
        }
    }
}
