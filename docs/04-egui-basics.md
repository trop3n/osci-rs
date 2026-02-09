# 04 - egui Basics

## Overview

egui is an **immediate-mode GUI** library for Rust. Unlike retained-mode GUIs (Qt, GTK), you rebuild the entire UI every frame - but egui makes this efficient and simple.

## Immediate Mode vs Retained Mode

### Retained Mode (Traditional)
```
1. Create window
2. Create button
3. Attach callback to button
4. Run event loop
5. Callback fires when clicked
```

### Immediate Mode (egui)
```
1. Every frame:
   - if ui.button("Click me").clicked() {
       // Handle click right here
     }
```

**Benefits of immediate mode:**
- No callback spaghetti
- UI state == app state
- Easy to understand control flow
- No widget lifecycle management

---

## The eframe Application

eframe provides the window and event loop. Your app implements `eframe::App`:

```rust
struct MyApp {
    counter: i32,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Called every frame - rebuild your entire UI here
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My App");
            if ui.button("Click me").clicked() {
                self.counter += 1;
            }
            ui.label(format!("Clicked {} times", self.counter));
        });
    }
}
```

---

## Layout System

### Panels

egui uses panels to divide the window:

```rust
// Top bar
egui::TopBottomPanel::top("top").show(ctx, |ui| {
    ui.heading("Title");
});

// Side panel (from right edge)
egui::SidePanel::right("settings").show(ctx, |ui| {
    ui.label("Settings here");
});

// Everything else
egui::CentralPanel::default().show(ctx, |ui| {
    ui.label("Main content");
});
```

### Layouts Within Panels

```rust
// Vertical (default)
ui.vertical(|ui| {
    ui.label("First");
    ui.label("Second");
});

// Horizontal
ui.horizontal(|ui| {
    ui.label("Left");
    ui.label("Right");
});

// With alignment
ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
    ui.label("At the bottom");
});
```

---

## Common Widgets

### Text

```rust
ui.heading("Big text");
ui.label("Normal text");
ui.small("Small text");
ui.monospace("Code style");
```

### Buttons

```rust
if ui.button("Click me").clicked() {
    // Handle click
}

// Disabled button
ui.add_enabled(false, egui::Button::new("Disabled"));

// Toggle button
ui.toggle_value(&mut self.show_panel, "Show Panel");
```

### Sliders

```rust
ui.add(egui::Slider::new(&mut self.value, 0.0..=100.0));

// With options
ui.add(
    egui::Slider::new(&mut self.frequency, 20.0..=2000.0)
        .suffix(" Hz")
        .logarithmic(true)
);
```

### Checkboxes

```rust
ui.checkbox(&mut self.enabled, "Enable feature");
```

### Combo Boxes (Dropdowns)

```rust
egui::ComboBox::from_id_salt("device_select")
    .selected_text(&self.selected_device)
    .show_ui(ui, |ui| {
        for device in &self.devices {
            ui.selectable_value(&mut self.selected_device, device.clone(), device);
        }
    });
```

### Collapsing Headers

```rust
ui.collapsing("Advanced Settings", |ui| {
    ui.label("Hidden content here");
});
```

---

## Custom Drawing with Painter

For graphics like our oscilloscope, we use `allocate_painter`:

```rust
// Allocate space and get a painter
let (response, painter) = ui.allocate_painter(
    egui::vec2(400.0, 400.0),  // Size
    egui::Sense::hover(),      // What input to capture
);

let rect = response.rect;

// Draw a filled rectangle (background)
painter.rect_filled(
    rect,
    4.0,  // Corner radius
    egui::Color32::from_rgb(10, 20, 10),
);

// Draw a line
painter.line_segment(
    [egui::pos2(0.0, 0.0), egui::pos2(100.0, 100.0)],
    egui::Stroke::new(2.0, egui::Color32::GREEN),
);

// Draw a circle
painter.circle_filled(
    egui::pos2(50.0, 50.0),  // Center
    10.0,                     // Radius
    egui::Color32::RED,
);
```

---

## Colors

```rust
// From RGB
let green = egui::Color32::from_rgb(100, 255, 100);

// From RGBA (with alpha)
let transparent = egui::Color32::from_rgba_unmultiplied(255, 0, 0, 128);

// Grayscale
let gray = egui::Color32::from_gray(128);

// Predefined
egui::Color32::WHITE
egui::Color32::BLACK
egui::Color32::TRANSPARENT
```

---

## Continuous Animation

egui only redraws when needed. For animations, request continuous repaints:

```rust
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Request another frame immediately
    ctx.request_repaint();

    // Now your UI updates continuously
}
```

---

## Response and Interaction

Every widget returns a `Response` with interaction info:

```rust
let response = ui.button("Click");

if response.clicked() {
    // Button was clicked
}

if response.hovered() {
    // Mouse is over the button
}

if response.double_clicked() {
    // Double-click detected
}
```

---

## Separators and Spacing

```rust
ui.separator();          // Horizontal line
ui.add_space(10.0);      // Vertical space
ui.horizontal(|ui| {
    ui.label("A");
    ui.separator();      // Vertical line in horizontal layout
    ui.label("B");
});
```

---

## ID System

egui needs unique IDs for stateful widgets. Usually automatic, but sometimes explicit:

```rust
// Automatic (based on label)
ui.button("Click");

// Explicit ID (when labels might conflict)
egui::ComboBox::from_id_salt("unique_id")
    .selected_text("Select...")
    .show_ui(ui, |ui| { /* ... */ });
```

---

## Our Oscilloscope Widget

Here's how the oscilloscope widget works:

```rust
pub fn show(&mut self, ui: &mut egui::Ui, samples: &[XYSample]) -> egui::Response {
    // 1. Allocate drawing space
    let (response, painter) = ui.allocate_painter(
        egui::vec2(400.0, 400.0),
        egui::Sense::hover(),
    );
    let rect = response.rect;

    // 2. Draw background
    painter.rect_filled(rect, 4.0, self.settings.background);

    // 3. Draw grid
    self.draw_graticule(&painter, rect);

    // 4. Convert samples to screen coordinates
    let points: Vec<Pos2> = samples.iter()
        .map(|s| self.sample_to_screen(*s, rect))
        .collect();

    // 5. Draw connected lines
    for window in points.windows(2) {
        painter.line_segment([window[0], window[1]], stroke);
    }

    response
}
```

---

## Key Takeaways

1. **Immediate mode** - Rebuild UI every frame, no callbacks
2. **Panels** - Divide screen into regions (top, side, center)
3. **Widgets return Response** - Check `.clicked()`, `.hovered()`, etc.
4. **Painter for custom graphics** - Draw shapes, lines, text
5. **request_repaint()** - For continuous animation

---

## Exercises

1. Add a new slider to control a parameter
2. Create a collapsing section with multiple controls
3. Draw a simple shape (square, triangle) using `painter`
4. Add hover detection to the oscilloscope display

---

## Links

- [egui Documentation](https://docs.rs/egui)
- [egui Demo](https://www.egui.rs/)
- [eframe Documentation](https://docs.rs/eframe)
- [egui Widget Gallery](https://www.egui.rs/#demo)
