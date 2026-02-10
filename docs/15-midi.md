# 15. MIDI Control

This milestone adds MIDI CC (Control Change) input, allowing external MIDI controllers to adjust osci-rs parameters in real time.

## MIDI Concepts

### What is MIDI?

MIDI (Musical Instrument Digital Interface) is a protocol for communicating musical information between devices. For our purposes, we care about **Control Change (CC)** messages:

```
Status byte: 0xB0 | channel (0-15)
CC number:   0-127 (which knob/slider)
Value:       0-127 (position)
```

A MIDI CC message says "controller #X is now at value Y". We map these to application parameters.

### CC Value Mapping

MIDI CC values range from 0-127 (7 bits). We map these linearly to parameter ranges:

```rust
pub fn map_value(&self, cc_value: u8) -> f32 {
    let t = cc_value as f32 / 127.0;  // Normalize to 0.0..1.0
    let (min, max) = self.range();
    min + t * (max - min)             // Scale to parameter range
}
```

## Architecture

### Lock-Free CC Sharing

The MIDI callback runs on a separate OS thread. We need to share CC values with the UI thread without locks:

```rust
struct SharedCcValues {
    values: Arc<[AtomicU8; 128]>,   // Current CC values
    changed: Arc<[AtomicU8; 128]>,  // Change flags
}
```

- **MIDI thread** calls `set(cc, value)` - stores value and sets changed flag
- **UI thread** calls `poll(cc)` - reads value and clears changed flag

This uses `AtomicU8` with `Ordering::Relaxed` - sufficient because we only need eventual consistency, not strict ordering.

### Mappable Parameters

| Parameter | Range | Description |
|-----------|-------|-------------|
| Frequency | 20.0 - 200.0 | Shape trace speed (Hz) |
| Volume | 0.0 - 1.0 | Audio output level |
| RotationSpeed | -5.0 - 5.0 | Rotation effect (rad/s) |
| ScaleLfoFreq | 0.1 - 10.0 | Scale LFO frequency |
| ScaleLfoMin | 0.1 - 1.5 | Scale LFO minimum |
| ScaleLfoMax | 0.5 - 2.0 | Scale LFO maximum |
| LineWidth | 0.5 - 5.0 | Display line thickness |
| Intensity | 0.1 - 1.0 | Display brightness |
| Persistence | 0.0 - 0.99 | CRT afterglow amount |
| Zoom | 0.1 - 2.0 | Display zoom level |

### MidiController

The controller manages the full lifecycle:

```rust
pub struct MidiController {
    ports: Vec<String>,              // Available ports
    selected_port: usize,            // UI selection
    connection: Option<MidiInputConnection<()>>,
    cc_values: SharedCcValues,       // Lock-free CC storage
    mappings: Vec<MidiMapping>,      // CC -> param mappings
    learning: Option<usize>,         // MIDI learn mode
}
```

### MIDI Learn

MIDI learn simplifies mapping setup:

1. User clicks "Learn" on a mapping
2. Controller enters learn mode for that mapping index
3. Next CC message received assigns that CC number to the mapping
4. Learn mode exits automatically

```rust
pub fn poll(&mut self) -> Vec<(MidiParam, f32)> {
    if let Some(mapping_idx) = self.learning {
        // Any CC received assigns to this mapping
        for cc in 0..128u8 {
            if self.cc_values.poll(cc).is_some() {
                self.mappings[mapping_idx].cc = cc;
                self.learning = None;
                return vec![];
            }
        }
        return vec![];
    }
    // Normal: apply mapped values
    // ...
}
```

### Persistence Integration

MIDI mappings are persisted via the settings system:

```rust
// In AppSettings:
pub midi_mappings: Vec<MidiMapping>,

// MidiMapping derives Serialize/Deserialize:
#[derive(Serialize, Deserialize)]
pub struct MidiMapping {
    pub cc: u8,
    pub param: MidiParam,
}
```

## Rust Concepts

### Atomic Operations

`AtomicU8` provides thread-safe integer operations without locks:

```rust
// Store (MIDI thread):
self.values[cc].store(value, Ordering::Relaxed);

// Load + swap (UI thread):
self.changed[cc].swap(0, Ordering::Relaxed)
```

`Ordering::Relaxed` means no synchronization guarantees beyond atomicity. This is fine for CC values where we just want the latest value, not strict ordering.

### Arc for Shared Ownership

`Arc<[AtomicU8; 128]>` gives both the MIDI callback and the UI thread ownership of the same array:

```rust
let cc_values = self.cc_values.clone(); // Clone the Arc
// Move into closure (runs on MIDI thread):
move |_timestamp, message, _| {
    cc_values.set(cc, value); // Uses the cloned Arc
}
```

### Enum as HashMap Key

`MidiParam` derives `Eq` and `Hash`, allowing it to be used in `HashSet` for tracking which parameters are already mapped:

```rust
#[derive(Eq, Hash)]
pub enum MidiParam { ... }

let mapped: HashSet<MidiParam> = mappings.iter().map(|m| m.param).collect();
```

### The midir Crate

`midir` provides cross-platform MIDI I/O. Key types:

- `MidiInput::new()` - Create a MIDI input instance
- `midi_in.ports()` - List available ports
- `midi_in.connect()` - Open a port with a callback closure
- `MidiInputConnection` - Keeps the connection alive (dropped = disconnected)
