# 14. Settings Persistence

This milestone adds automatic save/load of application settings, so your configuration persists between sessions.

## Serialization Concepts

### Serde Framework

Rust's **serde** (Serialize/Deserialize) is the standard framework for converting data structures to and from various formats. It uses derive macros to generate serialization code at compile time:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct AppSettings {
    volume: f32,
    frequency: f32,
    show_settings: bool,
}
```

The `#[derive(Serialize, Deserialize)]` attribute generates all the necessary code to convert the struct to/from JSON, TOML, YAML, and many other formats.

### Serde Attributes

Key serde attributes used in this project:

```rust
#[serde(default)]  // Use Default::default() for missing fields
```

This is critical for forward compatibility - when new settings are added in future versions, existing config files still load correctly. Missing fields get their default values instead of causing parse errors.

### JSON Format

We use `serde_json` for human-readable settings files:

```json
{
  "editor_mode": "SingleShape",
  "selected_shape": "Circle",
  "frequency": 80.0,
  "volume": 0.8,
  "line_width": 1.5,
  "color_r": 100,
  "color_g": 255,
  "color_b": 100
}
```

JSON was chosen because:
- Human-readable and editable
- Well-understood format
- Easy to debug
- `serde_json` is the most mature serde backend

## Architecture

### Settings Path

Settings are stored in the platform-specific config directory:

```rust
fn settings_path() -> PathBuf {
    let mut path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    path.push("osci-rs");
    path.push("settings.json");
    path
}
```

This resolves to:
- **Linux**: `~/.config/osci-rs/settings.json`
- **macOS**: `~/Library/Application Support/osci-rs/settings.json`
- **Windows**: `C:\Users\<user>\AppData\Roaming\osci-rs\settings.json`

### Load/Save Pattern

```rust
impl AppSettings {
    pub fn load() -> Self {
        // Try to read file, fall back to defaults
        match std::fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents)
                .unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        // Create directory if needed, write pretty JSON
        std::fs::create_dir_all(parent)?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
    }
}
```

### from_app / apply Pattern

Two methods bridge between the settings struct and the live application:

- **`from_app(&app)`** - Snapshot current app state into a settings struct
- **`apply(&self, &mut app)`** - Apply saved settings to the running app

This separation keeps serialization concerns out of the main app struct.

### Drop for Auto-Save

Rust's `Drop` trait runs when a value goes out of scope. We use it for auto-saving:

```rust
impl Drop for OsciApp {
    fn drop(&mut self) {
        AppSettings::from_app(self).save();
    }
}
```

This ensures settings are saved when the app closes, regardless of how it exits.

## What Gets Persisted

| Category | Fields |
|----------|--------|
| Editor | mode, selected shape, settings visibility |
| Shape params | size, width, height, inner_radius, points, lissajous, spiral |
| Audio | frequency, volume |
| Effects | rotation, scale LFO (freq, min, max, waveform) |
| Display | line width, draw lines, intensity, zoom, graticule, persistence |
| Color | foreground RGB, background RGB |
| Text | text input string |
| 3D | mesh primitive |
| MIDI | CC-to-parameter mappings |

**Not persisted**: loaded SVG/Image/Mesh files (path-dependent), scene entries (ephemeral), transient errors.

## Rust Concepts

### Enum Serialization

Serde serializes Rust enums as their variant names:

```rust
#[derive(Serialize, Deserialize)]
enum ShapeType {
    Circle,      // -> "Circle"
    Rectangle,   // -> "Rectangle"
    Lissajous,   // -> "Lissajous"
}
```

This works cleanly for C-style enums. The derive macro handles all variants automatically.

### Color32 Workaround

`egui::Color32` doesn't implement Serialize/Deserialize, so we store colors as separate u8 fields:

```rust
// In AppSettings:
pub color_r: u8,
pub color_g: u8,
pub color_b: u8,

// In apply():
app.oscilloscope.settings.color =
    egui::Color32::from_rgb(self.color_r, self.color_g, self.color_b);
```

This is a common pattern when working with third-party types that don't support serde.

### Graceful Degradation

The load function never panics:

```rust
pub fn load() -> Self {
    // File missing? -> defaults
    // Parse error? -> defaults
    // New fields? -> #[serde(default)] fills them in
}
```

This means the app always starts, even with corrupted or outdated config files.
