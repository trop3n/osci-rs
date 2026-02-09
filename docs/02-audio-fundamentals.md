# 02 - Audio Fundamentals

## Overview

This document covers digital audio concepts and how cpal (Cross-Platform Audio Library) works in Rust.

## Digital Audio Basics

### Sample Rate

Audio is captured/played as discrete samples at regular intervals.

- **44,100 Hz** - CD quality (44,100 samples per second)
- **48,000 Hz** - Common for video/professional audio
- **96,000 Hz** - High-resolution audio

Higher sample rates can represent higher frequencies (Nyquist theorem: max frequency = sample_rate / 2).

### Sample Format

Each sample is a number representing the audio amplitude:

- **f32** - 32-bit float, range -1.0 to 1.0 (most common in modern audio)
- **i16** - 16-bit integer, range -32768 to 32767 (CD quality)
- **i32** - 32-bit integer (professional audio)

### Channels

- **Mono** - 1 channel
- **Stereo** - 2 channels (Left, Right)
- **Surround** - 5.1, 7.1, etc.

For oscilloscope music:
- **Left channel = X axis**
- **Right channel = Y axis**

### Audio Buffers

Audio is processed in chunks called **buffers** or **frames**:

```
Buffer: [L0, R0, L1, R1, L2, R2, ...]
         └─frame─┘
```

A frame contains one sample per channel. Buffer sizes are typically 256-4096 samples.

---

## cpal Architecture

### The Audio Pipeline

```
┌──────────┐     ┌──────────┐     ┌────────────┐     ┌─────────┐
│   Host   │ ──► │  Device  │ ──► │   Stream   │ ──► │ Callback│
└──────────┘     └──────────┘     └────────────┘     └─────────┘
   ALSA           Speakers         Output stream      Your code
   WASAPI         Headphones
   CoreAudio      USB interface
```

### Key Components

1. **Host** - The audio backend (ALSA on Linux, WASAPI on Windows, CoreAudio on macOS)
2. **Device** - Physical or virtual audio hardware
3. **Stream** - Active audio connection (input or output)
4. **Callback** - Your function that processes audio

---

## Code Walkthrough

### Getting the Audio Host

```rust
let host = cpal::default_host();
```

cpal automatically selects the appropriate host for your OS.

### Getting an Output Device

```rust
let device = match host.default_output_device() {
    Some(d) => d,
    None => {
        // Handle error - no audio device found
        return;
    }
};
```

`default_output_device()` returns `Option<Device>`:
- `Some(device)` if a device exists
- `None` if no device is available

### Getting Device Configuration

```rust
let config = device.default_output_config()?;
let sample_rate = config.sample_rate().0 as f32;  // e.g., 48000.0
let channels = config.channels() as usize;        // e.g., 2
```

The config tells us what format the device expects.

### Building the Output Stream

```rust
let stream = device.build_output_stream(
    &config.into(),
    move |data: &mut [f32], _info: &cpal::OutputCallbackInfo| {
        // This runs on the audio thread!
        // Fill `data` with audio samples
    },
    |err| eprintln!("Audio error: {}", err),
    None,
)?;
```

**Parameters:**
1. `&config.into()` - Stream configuration
2. Data callback - Called repeatedly to fill audio buffers
3. Error callback - Called if something goes wrong
4. Timeout - Optional timeout for the stream

---

## The Audio Callback

This is the heart of audio processing:

```rust
move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
    for frame in data.chunks_mut(channels) {
        let sample = generate_sample();
        for output in frame.iter_mut() {
            *output = sample;
        }
    }
}
```

### Critical Rules for Audio Callbacks

1. **No blocking** - Never use mutexes, file I/O, or network calls
2. **No allocations** - Don't create new Vecs, Strings, or Box
3. **Be fast** - You have microseconds to fill the buffer
4. **Be deterministic** - Same input should produce same output

Breaking these rules causes **audio glitches** (clicks, pops, dropouts).

### The `move` Keyword

```rust
move |data: &mut [f32], _| {
    // closure body
}
```

`move` transfers ownership of captured variables into the closure. This is required because the closure runs on a different thread and must own its data.

---

## Generating a Sine Wave

### The Math

A sine wave is defined by:
```
y = sin(2π × frequency × time)
```

In discrete samples:
```
y = sin(2π × phase)
phase += frequency / sample_rate
```

### Phase Accumulator

```rust
let mut phase = 0.0f32;
let phase_increment = frequency / sample_rate;

// In the callback:
let value = (phase * std::f32::consts::TAU).sin();
phase += phase_increment;
if phase >= 1.0 {
    phase -= 1.0;  // Wrap to avoid precision loss
}
```

**Why wrap at 1.0?**
- Floating point numbers lose precision at large values
- Keeping phase in [0, 1) maintains accuracy
- `TAU` (2π) converts phase to radians

### Volume Control

```rust
let value = (phase * TAU).sin() * volume;  // volume is 0.0 to 1.0
```

Multiplying by volume scales the amplitude. Keep volume ≤ 1.0 to avoid clipping (distortion from values outside -1.0 to 1.0).

---

## Thread Safety with Arc and AtomicBool

### The Problem

The audio callback runs on a separate thread. We need to communicate with it from the UI thread.

### Arc (Atomic Reference Counted)

```rust
let is_playing = Arc::new(AtomicBool::new(false));
let is_playing_clone = Arc::clone(&is_playing);
```

`Arc<T>` allows multiple owners of the same data across threads:
- `Arc::new()` creates a new Arc
- `Arc::clone()` creates another reference (cheap, just increments a counter)
- When the last Arc is dropped, the data is freed

### AtomicBool

```rust
// Write (from UI thread)
is_playing.store(true, Ordering::Relaxed);

// Read (from audio thread)
if is_playing.load(Ordering::Relaxed) {
    // generate audio
}
```

`AtomicBool` is a thread-safe boolean:
- `store()` writes a value
- `load()` reads a value
- `Ordering::Relaxed` is sufficient for simple flags

---

## Keeping the Stream Alive

```rust
struct OsciApp {
    _stream: Option<cpal::Stream>,
}
```

The stream must be stored somewhere. When a `Stream` is dropped (goes out of scope), playback stops.

```rust
// Start playing
self._stream = Some(stream);

// Stop playing
self._stream = None;  // Dropping stops the stream
```

---

## Key Takeaways

1. **Audio runs on a separate thread** - The callback must be fast and lock-free
2. **`move` captures ownership** - Required for closures that cross thread boundaries
3. **Arc for shared ownership** - Multiple references to the same data across threads
4. **Atomics for thread-safe primitives** - Simple values that can be read/written from any thread
5. **Phase accumulator for synthesis** - Efficient way to generate periodic waveforms

---

## Exercises

1. Change the frequency to 880 Hz (one octave up) and listen to the difference
2. Generate a square wave: `if phase < 0.5 { volume } else { -volume }`
3. Add a second frequency (220 Hz) and mix them: `(sine1 + sine2) / 2.0`
4. Print the buffer size in the callback to see how many samples you process at once

---

## Links

- [cpal Documentation](https://docs.rs/cpal)
- [Digital Audio Basics](https://en.wikipedia.org/wiki/Digital_audio)
- [Rust Book: Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [Rust Atomics and Locks (Book)](https://marabos.nl/atomics/)
