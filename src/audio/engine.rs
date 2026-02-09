//! Audio engine - handles cpal audio output
//!
//! This module provides a high-level interface for audio output,
//! abstracting the cpal setup and stream management.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use super::buffer::{SampleBuffer, XYSample};
use crate::shapes::Shape;

/// Audio engine configuration
pub struct AudioConfig {
    /// How many times per second to trace the shape (Hz)
    pub frequency: f32,
    /// Output volume (0.0 to 1.0)
    pub volume: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            frequency: 80.0,  // 80 Hz = 80 traces per second
            volume: 0.8,
        }
    }
}

/// Pre-sampled shape data for the audio thread
struct ShapeData {
    /// The sampled XY points
    samples: Vec<XYSample>,
    /// Name of the current shape
    name: String,
}

impl Default for ShapeData {
    fn default() -> Self {
        Self {
            samples: Vec::new(),
            name: "None".to_string(),
        }
    }
}

/// High-level audio output engine
///
/// Manages the cpal audio stream and provides methods for
/// controlling playback.
pub struct AudioEngine {
    /// Whether audio is currently playing
    is_playing: Arc<AtomicBool>,

    /// The audio output stream (kept alive to continue playback)
    stream: Option<cpal::Stream>,

    /// Buffer for sharing samples with the UI
    buffer: SampleBuffer,

    /// Current configuration
    pub config: AudioConfig,

    /// Pre-sampled shape data shared with audio thread
    shape_data: Arc<Mutex<ShapeData>>,

    /// Current sample index (for audio thread)
    sample_index: Arc<AtomicUsize>,

    /// Status message
    pub status: String,

    /// Sample rate of the output device
    sample_rate: f32,

    /// Number of audio samples per shape trace
    samples_per_shape: usize,
}

impl AudioEngine {
    /// Create a new audio engine
    ///
    /// # Arguments
    /// * `buffer` - Shared sample buffer for visualization
    pub fn new(buffer: SampleBuffer) -> Self {
        Self {
            is_playing: Arc::new(AtomicBool::new(false)),
            stream: None,
            buffer,
            config: AudioConfig::default(),
            shape_data: Arc::new(Mutex::new(ShapeData::default())),
            sample_index: Arc::new(AtomicUsize::new(0)),
            status: "Ready".to_string(),
            sample_rate: 48000.0,
            samples_per_shape: 600, // 48000 / 80 = 600 samples per shape at 80Hz
        }
    }

    /// Check if audio is currently playing
    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::Relaxed)
    }

    /// Get the current shape name
    pub fn current_shape_name(&self) -> String {
        self.shape_data.lock().unwrap().name.clone()
    }

    /// Set the shape to render
    ///
    /// This pre-samples the shape and stores it for the audio thread.
    /// The number of samples is based on sample_rate / frequency.
    pub fn set_shape<S: Shape>(&mut self, shape: &S) {
        // Calculate samples per shape based on frequency
        self.samples_per_shape = (self.sample_rate / self.config.frequency) as usize;
        self.samples_per_shape = self.samples_per_shape.max(10); // Minimum 10 samples

        // Sample the shape
        let mut samples = Vec::with_capacity(self.samples_per_shape);
        for i in 0..self.samples_per_shape {
            let t = i as f32 / self.samples_per_shape as f32;
            let (x, y) = shape.sample(t);
            samples.push(XYSample::new(x * self.config.volume, y * self.config.volume));
        }

        // Update shared shape data
        if let Ok(mut data) = self.shape_data.lock() {
            data.samples = samples;
            data.name = shape.name().to_string();
        }

        // Reset sample index
        self.sample_index.store(0, Ordering::Relaxed);

        log::info!("Shape set: {} ({} samples)", shape.name(), self.samples_per_shape);
    }

    /// Start audio playback
    pub fn start(&mut self) {
        if self.stream.is_some() {
            return; // Already playing
        }

        // Check if we have a shape
        {
            let data = self.shape_data.lock().unwrap();
            if data.samples.is_empty() {
                self.status = "No shape set - select a shape first".to_string();
                return;
            }
        }

        log::info!("Starting audio engine...");

        // Get the default audio host
        let host = cpal::default_host();

        // Get the default output device
        let device = match host.default_output_device() {
            Some(d) => d,
            None => {
                self.status = "Error: No output device found".to_string();
                log::error!("No output device found");
                return;
            }
        };

        let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        log::info!("Using output device: {}", device_name);

        // Get the default output configuration
        let config = match device.default_output_config() {
            Ok(c) => c,
            Err(e) => {
                self.status = format!("Error getting config: {}", e);
                log::error!("Failed to get default output config: {}", e);
                return;
            }
        };

        log::info!("Audio config: {:?}", config);

        self.sample_rate = config.sample_rate().0 as f32;
        let channels = config.channels() as usize;

        // Clone values needed in the audio callback
        let is_playing = Arc::clone(&self.is_playing);
        let shape_data = Arc::clone(&self.shape_data);
        let sample_index = Arc::clone(&self.sample_index);
        let buffer = self.buffer.clone_ref();

        // Build the output stream
        let stream_result = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    if !is_playing.load(Ordering::Relaxed) {
                        // Output silence
                        for sample in data.iter_mut() {
                            *sample = 0.0;
                        }
                        return;
                    }

                    // Get shape samples (try_lock to avoid blocking audio thread)
                    let shape_samples = if let Ok(shape) = shape_data.try_lock() {
                        if shape.samples.is_empty() {
                            return;
                        }
                        shape.samples.clone()
                    } else {
                        // Couldn't get lock - output silence this buffer
                        for sample in data.iter_mut() {
                            *sample = 0.0;
                        }
                        return;
                    };

                    let num_shape_samples = shape_samples.len();

                    // Generate audio samples
                    for frame in data.chunks_mut(channels) {
                        // Get current sample index and wrap
                        let idx = sample_index.load(Ordering::Relaxed) % num_shape_samples;
                        let xy = shape_samples[idx];

                        // Output to audio channels (Left = X, Right = Y)
                        if channels >= 2 {
                            frame[0] = xy.x;
                            frame[1] = xy.y;
                        } else {
                            frame[0] = (xy.x + xy.y) / 2.0; // Mono mix
                        }

                        // Push to visualization buffer
                        buffer.push(xy);

                        // Advance sample index
                        sample_index.fetch_add(1, Ordering::Relaxed);
                    }
                },
                |err| log::error!("Audio stream error: {}", err),
                None,
            ),
            format => {
                self.status = format!("Unsupported sample format: {:?}", format);
                log::error!("Unsupported sample format: {:?}", format);
                return;
            }
        };

        match stream_result {
            Ok(s) => {
                if let Err(e) = s.play() {
                    self.status = format!("Error starting stream: {}", e);
                    log::error!("Failed to start stream: {}", e);
                    return;
                }

                let shape_name = self.shape_data.lock().unwrap().name.clone();
                self.is_playing.store(true, Ordering::Relaxed);
                self.stream = Some(s);
                self.status = format!(
                    "Playing: {} at {}Hz, {:.0}% volume",
                    shape_name,
                    self.config.frequency,
                    self.config.volume * 100.0
                );
                log::info!("Audio started successfully");
            }
            Err(e) => {
                self.status = format!("Error building stream: {}", e);
                log::error!("Failed to build stream: {}", e);
            }
        }
    }

    /// Stop audio playback
    pub fn stop(&mut self) {
        self.is_playing.store(false, Ordering::Relaxed);
        self.stream = None;
        self.status = "Stopped".to_string();
        log::info!("Audio stopped");
    }

    /// Toggle playback state
    pub fn toggle(&mut self) {
        if self.is_playing() {
            self.stop();
        } else {
            self.start();
        }
    }

    /// Update the shape while playing (re-samples with current settings)
    pub fn update_shape<S: Shape>(&mut self, shape: &S) {
        self.set_shape(shape);
    }
}
