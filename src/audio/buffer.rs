//! Sample buffer for sharing audio data between threads
//!
//! This module provides a thread-safe circular buffer for passing
//! audio samples from the audio thread to the UI thread.
//!
//! ## Design Notes
//!
//! For Milestone 3, we use `Arc<Mutex<T>>` for simplicity.
//! In Milestone 12, we'll replace this with a lock-free ring buffer
//! using atomics for better real-time performance.

use std::sync::{Arc, Mutex};

/// A 2D point representing an XY sample
/// Left channel = X, Right channel = Y
#[derive(Clone, Copy, Debug, Default)]
pub struct XYSample {
    pub x: f32,
    pub y: f32,
}

impl XYSample {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Thread-safe circular buffer for XY audio samples
///
/// This buffer is designed for the producer-consumer pattern:
/// - Producer (audio thread): Calls `push()` to add samples
/// - Consumer (UI thread): Calls `get_samples()` to read samples
///
/// ## Example
///
/// ```
/// let buffer = SampleBuffer::new(1024);
///
/// // In audio thread:
/// buffer.push(XYSample::new(left, right));
///
/// // In UI thread:
/// let samples = buffer.get_samples();
/// ```
pub struct SampleBuffer {
    /// The actual buffer storage, wrapped for thread safety
    /// Arc = shared ownership across threads
    /// Mutex = exclusive access (only one thread at a time)
    inner: Arc<Mutex<BufferInner>>,
}

/// Internal buffer data
struct BufferInner {
    /// Circular buffer of samples
    samples: Vec<XYSample>,
    /// Current write position
    write_pos: usize,
    /// Number of samples written (for tracking)
    samples_written: u64,
}

impl SampleBuffer {
    /// Create a new sample buffer with the given capacity
    ///
    /// # Arguments
    /// * `capacity` - Number of samples to store (typically 2048-8192)
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(BufferInner {
                samples: vec![XYSample::default(); capacity],
                write_pos: 0,
                samples_written: 0,
            })),
        }
    }

    /// Push a single XY sample into the buffer
    ///
    /// This method is called from the audio thread. It uses `try_lock()`
    /// to avoid blocking if the UI thread is reading.
    ///
    /// # Arguments
    /// * `sample` - The XY sample to push
    ///
    /// # Returns
    /// `true` if the sample was pushed, `false` if the lock couldn't be acquired
    pub fn push(&self, sample: XYSample) -> bool {
        // try_lock() returns immediately if the mutex is locked
        // This is important for audio threads - we can't wait!
        if let Ok(mut inner) = self.inner.try_lock() {
            let len = inner.samples.len();
            // Copy write_pos to a local variable first
            // This avoids borrowing `inner` both mutably (for samples) and immutably (for write_pos)
            let pos = inner.write_pos;
            inner.samples[pos] = sample;
            inner.write_pos = (pos + 1) % len;
            inner.samples_written += 1;
            true
        } else {
            // Mutex was locked by UI thread - drop this sample
            // This is acceptable for visualization (occasional dropped samples)
            false
        }
    }

    /// Push multiple XY samples into the buffer
    ///
    /// More efficient than calling `push()` repeatedly as it only
    /// acquires the lock once.
    pub fn push_slice(&self, samples: &[XYSample]) -> bool {
        if let Ok(mut inner) = self.inner.try_lock() {
            let len = inner.samples.len();
            let mut pos = inner.write_pos;
            for &sample in samples {
                inner.samples[pos] = sample;
                pos = (pos + 1) % len;
            }
            inner.write_pos = pos;
            inner.samples_written += samples.len() as u64;
            true
        } else {
            false
        }
    }

    /// Get all samples in chronological order (oldest first)
    ///
    /// This method is called from the UI thread to get samples for display.
    /// It returns a copy of the samples to avoid holding the lock.
    ///
    /// # Returns
    /// A vector of samples ordered from oldest to newest
    pub fn get_samples(&self) -> Vec<XYSample> {
        // Lock the mutex - this blocks until available
        // OK for UI thread since it's not real-time critical
        let inner = self.inner.lock().unwrap();

        let len = inner.samples.len();
        let mut result = Vec::with_capacity(len);

        // Read samples starting from write_pos (oldest) and wrapping around
        for i in 0..len {
            let idx = (inner.write_pos + i) % len;
            result.push(inner.samples[idx]);
        }

        result
    }

    /// Get the most recent N samples
    ///
    /// Useful when you want to display fewer samples than the buffer holds.
    pub fn get_recent_samples(&self, count: usize) -> Vec<XYSample> {
        let inner = self.inner.lock().unwrap();

        let len = inner.samples.len();
        let count = count.min(len);
        let mut result = Vec::with_capacity(count);

        // Start from (write_pos - count) which is the oldest of the recent samples
        let start = (inner.write_pos + len - count) % len;

        for i in 0..count {
            let idx = (start + i) % len;
            result.push(inner.samples[idx]);
        }

        result
    }

    /// Get the total number of samples written since creation
    pub fn samples_written(&self) -> u64 {
        self.inner.lock().unwrap().samples_written
    }

    /// Clear the buffer (fill with zeros)
    pub fn clear(&self) {
        if let Ok(mut inner) = self.inner.try_lock() {
            for sample in &mut inner.samples {
                *sample = XYSample::default();
            }
            inner.write_pos = 0;
        }
    }

    /// Clone the Arc to share with another thread
    ///
    /// This is how we share the buffer between audio and UI threads.
    /// Each clone increments the reference count.
    pub fn clone_ref(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

// Implement Clone manually to use Arc::clone
impl Clone for SampleBuffer {
    fn clone(&self) -> Self {
        self.clone_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_get() {
        let buffer = SampleBuffer::new(4);

        buffer.push(XYSample::new(1.0, 1.0));
        buffer.push(XYSample::new(2.0, 2.0));
        buffer.push(XYSample::new(3.0, 3.0));

        let samples = buffer.get_samples();
        assert_eq!(samples.len(), 4);
    }

    #[test]
    fn test_circular_wrap() {
        let buffer = SampleBuffer::new(3);

        // Push more samples than capacity
        buffer.push(XYSample::new(1.0, 1.0));
        buffer.push(XYSample::new(2.0, 2.0));
        buffer.push(XYSample::new(3.0, 3.0));
        buffer.push(XYSample::new(4.0, 4.0)); // Overwrites first

        let samples = buffer.get_recent_samples(3);
        assert_eq!(samples[0].x, 2.0);
        assert_eq!(samples[1].x, 3.0);
        assert_eq!(samples[2].x, 4.0);
    }
}
