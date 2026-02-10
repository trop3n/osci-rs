# 12. Lock-Free Audio Programming

This milestone replaced mutex-based thread communication with lock-free data structures, eliminating potential audio glitches caused by priority inversion.

## The Problem with Mutexes

Audio callbacks run on real-time threads with strict timing requirements (typically every 5-20ms). If an audio callback blocks:

```
Audio Thread                    UI Thread
     |                              |
     v                              v
  try_lock()                    lock()
     |                              |
     | <--- BLOCKED waiting --------|
     |      for UI to release       |
     v                              v
  BUFFER UNDERRUN!             unlock()
  (audible click/pop)
```

Problems:
- **Priority Inversion** - High-priority audio thread waits for low-priority UI thread
- **Buffer Underruns** - Missed audio deadlines cause audible glitches
- **Unpredictable Latency** - Lock contention varies with UI activity

## Lock-Free Solution

Lock-free data structures use atomic operations instead of locks:

```
Audio Thread (Producer)         UI Thread (Consumer)
     |                              |
     v                              v
  atomic_write()               atomic_read()
     |                              |
     v                              v
  CONTINUES                    CONTINUES
  (no waiting)                 (no waiting)
```

Benefits:
- **No Blocking** - Both threads always make progress
- **Predictable Timing** - Atomic operations have bounded latency
- **No Priority Inversion** - Threads never wait for each other

## The `ringbuf` Crate

We use the `ringbuf` crate which provides SPSC (Single-Producer, Single-Consumer) ring buffers:

```rust
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};

// Create a ring buffer with capacity for 4096 samples
let rb = HeapRb::<XYSample>::new(4096);

// Split into producer and consumer halves
let (mut producer, mut consumer) = rb.split();

// Producer (audio thread) - lock-free push
producer.try_push(sample);  // Returns Err if full

// Consumer (UI thread) - lock-free pop
consumer.try_pop();  // Returns None if empty
```

### Key Properties

| Property | Description |
|----------|-------------|
| **SPSC** | Single-Producer, Single-Consumer (exactly our use case) |
| **Lock-free** | Uses atomic compare-and-swap operations |
| **Wait-free producer** | Push never blocks (drops if full) |
| **Wait-free consumer** | Pop never blocks (returns None if empty) |
| **Cache-friendly** | Contiguous memory layout |

## Our Implementation

### Architecture

```
┌─────────────────┐                    ┌─────────────────┐
│  Audio Thread   │                    │   UI Thread     │
│                 │    Lock-Free       │                 │
│  SampleProducer ├──── Ring Buffer ───►  SampleConsumer │
│                 │                    │                 │
│   push(sample)  │                    │    update()     │
└─────────────────┘                    │  get_samples()  │
                                       └─────────────────┘
```

### SampleProducer (Audio Thread)

```rust
pub struct SampleProducer {
    producer: ringbuf::HeapProd<XYSample>,
    samples_written: Arc<AtomicU64>,
}

impl SampleProducer {
    /// Push a sample - lock-free, safe for audio callbacks
    #[inline]
    pub fn push(&mut self, sample: XYSample) {
        let _ = self.producer.try_push(sample);
        self.samples_written.fetch_add(1, Ordering::Relaxed);
    }
}
```

Key points:
- `try_push` is lock-free (never blocks)
- If buffer full, sample is dropped (acceptable for visualization)
- `#[inline]` hint for performance-critical path
- `Ordering::Relaxed` is sufficient for a counter

### SampleConsumer (UI Thread)

```rust
pub struct SampleConsumer {
    consumer: ringbuf::HeapCons<XYSample>,
    snapshot: Vec<XYSample>,  // Display buffer
    write_pos: usize,
}

impl SampleConsumer {
    /// Drain ring buffer into snapshot
    pub fn update(&mut self) {
        while let Some(sample) = self.consumer.try_pop() {
            self.snapshot[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.capacity;
        }
    }

    /// Get samples for display
    pub fn get_samples(&self) -> Vec<XYSample> {
        // Return copy of snapshot (doesn't touch ring buffer)
        // ...
    }
}
```

Key points:
- `update()` drains all available samples at once
- Separate `snapshot` buffer for UI display
- `get_samples()` reads from snapshot, not ring buffer
- UI can call `get_samples()` multiple times without affecting ring buffer

### Compatibility Wrapper

For gradual migration, we provide a wrapper with the old API:

```rust
pub struct SampleBuffer {
    producer: Arc<Mutex<Option<SampleProducer>>>,
    consumer: Arc<Mutex<Option<SampleConsumer>>>,
}

impl SampleBuffer {
    /// Old API - uses Mutex but producer/consumer are lock-free internally
    pub fn push(&self, sample: XYSample) -> bool {
        if let Ok(mut guard) = self.producer.try_lock() {
            if let Some(ref mut prod) = *guard {
                prod.push(sample);  // Lock-free push
                return true;
            }
        }
        false
    }
}
```

## Atomic Operations & Memory Ordering

The ring buffer uses atomic operations with specific memory orderings:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

// Relaxed - no ordering guarantees, just atomicity
// Sufficient for simple counters
samples_written.fetch_add(1, Ordering::Relaxed);

// Acquire/Release - synchronizes data between threads
// Used internally by ringbuf for producer/consumer coordination
```

### Memory Ordering Levels

| Ordering | Use Case | Cost |
|----------|----------|------|
| `Relaxed` | Counters, statistics | Cheapest |
| `Acquire` | Consumer reading data | Medium |
| `Release` | Producer writing data | Medium |
| `SeqCst` | Full ordering (rarely needed) | Most expensive |

## Performance Considerations

### Before (Mutex)
```
Audio callback:
  1. try_lock() - may fail if UI holds lock
  2. Write sample
  3. unlock()

Worst case: Dropped samples when UI thread holds lock
```

### After (Lock-Free)
```
Audio callback:
  1. Atomic write (always succeeds)

Worst case: Buffer full, oldest samples dropped (expected behavior)
```

### Benchmarks (Typical)

| Operation | Mutex | Lock-Free |
|-----------|-------|-----------|
| Push (uncontended) | ~50ns | ~20ns |
| Push (contended) | Blocks! | ~20ns |
| Pop | ~50ns | ~15ns |

## Common Pitfalls

### 1. Using MPSC Instead of SPSC
```rust
// WRONG - Multiple producers break lock-free guarantees
let rb = HeapRb::new(1024);
let (prod, cons) = rb.split();
let prod2 = prod.clone();  // Can't clone producer!
```

### 2. Forgetting to Update Consumer
```rust
// WRONG - Snapshot never updated
let samples = consumer.get_samples();  // Always returns stale data

// RIGHT - Update first
consumer.update();
let samples = consumer.get_samples();
```

### 3. Allocating in Audio Callback
```rust
// WRONG - Vec::push may allocate
let mut samples = Vec::new();
samples.push(sample);  // Potential allocation!

// RIGHT - Pre-allocate or use fixed buffer
let _ = producer.try_push(sample);  // No allocation
```

## Key Takeaways

1. **Mutexes are dangerous in audio code** - Even `try_lock()` can fail, dropping samples

2. **SPSC ring buffers are ideal** - Perfect match for producer/consumer audio pattern

3. **Lock-free != Wait-free** - Lock-free means no locks; wait-free means bounded time

4. **Memory ordering matters** - Use the weakest ordering that's correct

5. **Separate concerns** - Ring buffer for transfer, snapshot for display

6. **Accept dropped samples** - For visualization, occasional drops are acceptable

## Links

- [ringbuf crate](https://docs.rs/ringbuf/)
- [Lock-Free Programming](https://preshing.com/20120612/an-introduction-to-lock-free-programming/)
- [Memory Ordering](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html)
- [Real-time Audio Programming](https://www.rossbencina.com/code/real-time-audio-programming-101-time-waits-for-nothing)
