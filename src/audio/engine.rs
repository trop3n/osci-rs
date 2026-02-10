//! Audio engine - handles cpal audio output
//!
//! This module provides a high-level interface for audio output,
//! abstracting the cpal setup and stream management.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

use super::buffer::{SampleBuffer, XYSample};
use crate::effects::{EffectChain, LfoWaveform, Rotate, LfoScale};
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

/// Effect parameters shared with audio thread
#[derive(Clone)]
pub struct EffectParams {
    /// Rotation speed in radians per second
    pub rotation_speed: f32,
    /// Whether rotation is enabled
    pub rotation_enabled: bool,
    /// Scale LFO frequency
    pub scale_lfo_freq: f32,
    /// Scale LFO minimum
    pub scale_lfo_min: f32,
    /// Scale LFO maximum
    pub scale_lfo_max: f32,
    /// Whether scale LFO is enabled
    pub scale_lfo_enabled: bool,
    /// Scale LFO waveform shape
    pub scale_lfo_waveform: LfoWaveform,
}

impl Default for EffectParams {
    fn default() -> Self {
        Self {
            rotation_speed: 0.0,
            rotation_enabled: false,
            scale_lfo_freq: 1.0,
            scale_lfo_min: 0.8,
            scale_lfo_max: 1.2,
            scale_lfo_enabled: false,
            scale_lfo_waveform: LfoWaveform::Sine,
        }
    }
}

impl EffectParams {
    /// Build an EffectChain from the current parameters
    fn build_chain(&self) -> EffectChain {
        let mut chain = EffectChain::new();

        if self.rotation_enabled && self.rotation_speed != 0.0 {
            chain.add(Rotate::animated(self.rotation_speed));
        }

        if self.scale_lfo_enabled {
            chain.add(
                LfoScale::new(self.scale_lfo_freq, self.scale_lfo_min, self.scale_lfo_max)
                    .waveform(self.scale_lfo_waveform)
            );
        }

        chain
    }
}

/// How often to push samples to the visualization buffer
/// (every Nth sample to reduce lock contention)
const VIZ_DECIMATION: usize = 8;

/// Write audio samples for any sample format
fn write_audio_samples<T: Sample + FromSample<f32>>(
    data: &mut [T],
    channels: usize,
    is_playing: &AtomicBool,
    shape_data: &RwLock<ShapeData>,
    sample_index: &AtomicUsize,
    buffer: &SampleBuffer,
    effect_params: &RwLock<EffectParams>,
    total_samples: &AtomicU64,
    sample_rate: f32,
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

    // Get current index and calculate new index after this buffer
    let start_idx = sample_index.load(Ordering::Relaxed);
    let start_total = total_samples.load(Ordering::Relaxed);
    let num_frames = data.len() / channels;

    // Try to get effect chain (use empty chain if locked)
    let chain = effect_params.try_read()
        .map(|e| e.build_chain())
        .unwrap_or_default();

    // Generate audio samples
    for (frame_num, frame) in data.chunks_mut(channels).enumerate() {
        // Calculate wrapped index for this frame
        let idx = (start_idx + frame_num) % num_shape_samples;
        let xy = shape_guard.samples[idx];

        // Calculate time for effects
        let current_sample = start_total + frame_num as u64;
        let time = current_sample as f32 / sample_rate;

        // Apply effects
        let (ex, ey) = chain.apply(xy.x, xy.y, time);

        // Output to audio channels (Left = X, Right = Y)
        if channels >= 2 {
            frame[0] = T::from_sample(ex);
            frame[1] = T::from_sample(ey);
            // Fill any extra channels with silence
            for ch in frame.iter_mut().skip(2) {
                *ch = T::EQUILIBRIUM;
            }
        } else {
            frame[0] = T::from_sample((ex + ey) / 2.0); // Mono mix
        }

        // Push effected samples to visualization buffer
        if (start_idx + frame_num) % VIZ_DECIMATION == 0 {
            buffer.push(XYSample::new(ex, ey));
        }
    }

    // Update sample index with wrap-around to prevent overflow
    let new_idx = (start_idx + num_frames) % num_shape_samples;
    sample_index.store(new_idx, Ordering::Relaxed);

    // Update total sample counter for time tracking
    total_samples.fetch_add(num_frames as u64, Ordering::Relaxed);
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

    /// Effect parameters shared with audio thread
    effect_params: Arc<RwLock<EffectParams>>,

    /// Total samples played (for time tracking in effects)
    total_samples: Arc<AtomicU64>,
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
            effect_params: Arc::new(RwLock::new(EffectParams::default())),
            total_samples: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Update effect parameters
    pub fn set_effects(&self, params: EffectParams) {
        if let Ok(mut effects) = self.effect_params.write() {
            *effects = params;
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
        let effect_params = Arc::clone(&self.effect_params);
        let total_samples = Arc::clone(&self.total_samples);
        let buffer = self.buffer.clone_ref();
        let sample_rate = self.sample_rate;

        // Build the output stream based on sample format
        let sample_format = config.sample_format();
        log::info!("Sample format: {:?}", sample_format);

        let stream_result = match sample_format {
            cpal::SampleFormat::F32 => {
                let is_playing = Arc::clone(&is_playing);
                let shape_data = Arc::clone(&shape_data);
                let sample_index = Arc::clone(&sample_index);
                let effect_params = Arc::clone(&effect_params);
                let total_samples = Arc::clone(&total_samples);
                let buffer = buffer.clone_ref();
                device.build_output_stream(
                    &config.into(),
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        write_audio_samples(
                            data, channels, &is_playing, &shape_data, &sample_index, &buffer,
                            &effect_params, &total_samples, sample_rate,
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
                let effect_params = Arc::clone(&effect_params);
                let total_samples = Arc::clone(&total_samples);
                let buffer = buffer.clone_ref();
                device.build_output_stream(
                    &config.into(),
                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        write_audio_samples(
                            data, channels, &is_playing, &shape_data, &sample_index, &buffer,
                            &effect_params, &total_samples, sample_rate,
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
                let effect_params = Arc::clone(&effect_params);
                let total_samples = Arc::clone(&total_samples);
                let buffer = buffer.clone_ref();
                device.build_output_stream(
                    &config.into(),
                    move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                        write_audio_samples(
                            data, channels, &is_playing, &shape_data, &sample_index, &buffer,
                            &effect_params, &total_samples, sample_rate,
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
