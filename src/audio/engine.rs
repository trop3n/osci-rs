//! Audio engine - handles cpal audio output
//!
//! This module provides a high-level interface for audio output,
//! abstracting the cpal setup and stream management.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

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
///
/// Uses RwLock for better concurrency - audio thread only reads,
/// main thread writes when shape changes.
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

/// Write audio samples for any sample format
fn write_audio_samples<T: Sample + FromSample<f32>>(
    data: &mut [T],
    channels: usize,
    is_playing: &AtomicBool,
    shape_data: &RwLock<ShapeData>,
    sample_index: &AtomicUsize,
    buffer: &SampleBuffer,
) {
    // Check if we should output audio
    if !is_playing.load(Ordering::Relaxed) {
        // Output silence
        for sample in data.iter_mut() {
            *sample = T::EQUILIBRIUM;
        }
        return;
    }

    // Try to read shape data (non-blocking for audio thread)
    let shape_guard = match shape_data.try_read() {
        Ok(guard) => guard,
        Err(_) => {
            // Couldn't get lock - output silence
            for sample in data.iter_mut() {
                *sample = T::EQUILIBRIUM;
            }
            return;
        }
    };

    if shape_guard.samples.is_empty() {
        // No shape data - output silence
        for sample in data.iter_mut() {
            *sample = T::EQUILIBRIUM;
        }
        return;
    }

    let num_shape_samples = shape_guard.samples.len();

    // Generate audio samples
    for frame in data.chunks_mut(channels) {
        // Get current sample index and wrap
        let idx = sample_index.load(Ordering::Relaxed) % num_shape_samples;
        let xy = shape_guard.samples[idx];

        // Output to audio channels (Left = X, Right = Y)
        if channels >= 2 {
            frame[0] = T::from_sample(xy.x);
            frame[1] = T::from_sample(xy.y);
            // Fill any extra channels with silence
            for ch in frame.iter_mut().skip(2) {
                *ch = T::EQUILIBRIUM;
            }
        } else {
            frame[0] = T::from_sample((xy.x + xy.y) / 2.0); // Mono mix
        }

        // Push to visualization buffer
        buffer.push(xy);

        // Advance sample index
        sample_index.fetch_add(1, Ordering::Relaxed);
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

    /// Pre-sampled shape data shared with audio thread (RwLock for better concurrency)
    shape_data: Arc<RwLock<ShapeData>>,

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
            shape_data: Arc::new(RwLock::new(ShapeData::default())),
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
        self.shape_data.read().unwrap().name.clone()
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
        if let Ok(mut data) = self.shape_data.write() {
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
            let data = self.shape_data.read().unwrap();
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

        // Build the output stream based on sample format
        let sample_format = config.sample_format();
        log::info!("Sample format: {:?}", sample_format);

        let stream_result = match sample_format {
            cpal::SampleFormat::F32 => {
                let is_playing = Arc::clone(&is_playing);
                let shape_data = Arc::clone(&shape_data);
                let sample_index = Arc::clone(&sample_index);
                let buffer = buffer.clone_ref();
                device.build_output_stream(
                    &config.into(),
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        write_audio_samples(
                            data, channels, &is_playing, &shape_data, &sample_index, &buffer,
                        );
                    },
                    |err| log::error!("Audio stream error: {}", err),
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                let is_playing = Arc::clone(&is_playing);
                let shape_data = Arc::clone(&shape_data);
                let sample_index = Arc::clone(&sample_index);
                let buffer = buffer.clone_ref();
                device.build_output_stream(
                    &config.into(),
                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        write_audio_samples(
                            data, channels, &is_playing, &shape_data, &sample_index, &buffer,
                        );
                    },
                    |err| log::error!("Audio stream error: {}", err),
                    None,
                )
            }
            cpal::SampleFormat::U16 => {
                let is_playing = Arc::clone(&is_playing);
                let shape_data = Arc::clone(&shape_data);
                let sample_index = Arc::clone(&sample_index);
                let buffer = buffer.clone_ref();
                device.build_output_stream(
                    &config.into(),
                    move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                        write_audio_samples(
                            data, channels, &is_playing, &shape_data, &sample_index, &buffer,
                        );
                    },
                    |err| log::error!("Audio stream error: {}", err),
                    None,
                )
            }
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

                let shape_name = self.shape_data.read().unwrap().name.clone();
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
