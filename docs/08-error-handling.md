# Error Handling in Rust

**Milestone 7: SVG Import**

This document covers Rust's approach to error handling, focusing on the patterns we used to implement SVG file loading.

## The Result Type

Rust uses the `Result<T, E>` type for operations that can fail:

```rust
enum Result<T, E> {
    Ok(T),    // Success with value of type T
    Err(E),   // Failure with error of type E
}
```

Unlike exceptions in other languages, Rust errors are explicit values that must be handled.

## The ? Operator

The `?` operator provides concise error propagation:

```rust
// Without ?
fn load_svg(path: &Path) -> Result<SvgShape, SvgError> {
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => return Err(SvgError::IoError(e)),
    };
    // ... more code
}

// With ?
fn load_svg(path: &Path) -> Result<SvgShape, SvgError> {
    let data = std::fs::read(path)?;  // Returns early on error
    // ... more code
}
```

The `?` operator:
1. Unwraps `Ok` values, continuing execution
2. Converts and returns `Err` values immediately
3. Requires `From` trait implementation for error conversion

## Custom Error Types with thiserror

The `thiserror` crate simplifies defining custom error types:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SvgError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse SVG: {0}")]
    ParseError(String),

    #[error("SVG contains no paths")]
    NoPaths,
}
```

Key features:
- `#[derive(Error)]` - Implements `std::error::Error`
- `#[error("...")]` - Defines the display message
- `#[from]` - Implements `From` for automatic conversion
- `{0}` - Interpolates the first tuple field

## Error Conversion with From

The `#[from]` attribute generates `From` implementations:

```rust
// This is generated automatically by thiserror:
impl From<std::io::Error> for SvgError {
    fn from(err: std::io::Error) -> SvgError {
        SvgError::IoError(err)
    }
}
```

This enables the `?` operator to convert errors automatically:

```rust
// std::io::Error -> SvgError via From
let data = std::fs::read(path)?;
```

## Handling External Library Errors

External libraries often have their own error types. We convert them using `map_err`:

```rust
let tree = usvg::Tree::from_data(data, &usvg::Options::default())
    .map_err(|e| SvgError::ParseError(e.to_string()))?;
```

This pattern:
1. Calls the fallible function
2. Maps the error type if it occurs
3. Uses `?` to propagate

## Our SVG Error Hierarchy

```rust
#[derive(Error, Debug)]
pub enum SvgError {
    // Wraps std::io::Error for file operations
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    // Wraps usvg parse errors as strings
    #[error("Failed to parse SVG: {0}")]
    ParseError(String),

    // Domain-specific error
    #[error("SVG contains no paths")]
    NoPaths,
}
```

## Using Results in the Application

In the UI code, we handle errors and display them to users:

```rust
fn load_svg_file(&mut self) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("SVG Files", &["svg"])
        .pick_file()
    {
        match SvgShape::load(&path, &self.svg_options) {
            Ok(svg) => {
                self.loaded_svg = Some(svg);
                self.svg_error = None;
            }
            Err(e) => {
                self.svg_error = Some(e.to_string());
            }
        }
    }
}
```

## File Dialogs with rfd

The `rfd` (Rusty File Dialog) crate provides native file dialogs:

```rust
use rfd::FileDialog;

// Open file dialog with filter
if let Some(path) = FileDialog::new()
    .add_filter("SVG Files", &["svg"])
    .pick_file()
{
    // path is PathBuf
}
```

Features:
- Native look and feel on each platform
- File type filtering
- Synchronous and async APIs
- Returns `Option<PathBuf>` (None if cancelled)

## Bézier Curve Mathematics

SVGs use Bézier curves which we sample into point sequences:

### Quadratic Bézier (3 points)
```rust
fn quadratic_bezier(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), t: f32) -> (f32, f32) {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;

    let x = mt2 * p0.0 + 2.0 * mt * t * p1.0 + t2 * p2.0;
    let y = mt2 * p0.1 + 2.0 * mt * t * p1.1 + t2 * p2.1;
    (x, y)
}
```

### Cubic Bézier (4 points)
```rust
fn cubic_bezier(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32), t: f32) -> (f32, f32) {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    let t2 = t * t;
    let t3 = t2 * t;

    let x = mt3 * p0.0 + 3.0 * mt2 * t * p1.0 + 3.0 * mt * t2 * p2.0 + t3 * p3.0;
    let y = mt3 * p0.1 + 3.0 * mt2 * t * p1.1 + 3.0 * mt * t2 * p2.1 + t3 * p3.1;
    (x, y)
}
```

## Key Takeaways

1. **Use Result for fallible operations** - Makes errors explicit and forces handling
2. **The ? operator simplifies propagation** - Reduces boilerplate while maintaining safety
3. **thiserror creates ergonomic error types** - Minimal code for maximum functionality
4. **#[from] enables automatic conversion** - Seamless error type interoperability
5. **map_err converts non-standard errors** - Works with any library's error types
6. **Display errors to users gracefully** - Store error strings for UI display

## Exercises

1. Add a new error variant for "SVG file too complex" (> 10,000 points)
2. Implement retry logic for failed file operations
3. Add validation for curve sample count (min 2, max 64)

## Links

- [Rust Book: Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [thiserror documentation](https://docs.rs/thiserror)
- [rfd documentation](https://docs.rs/rfd)
- [usvg documentation](https://docs.rs/usvg)
- [Bézier Curves - A Primer](https://pomax.github.io/bezierinfo/)
