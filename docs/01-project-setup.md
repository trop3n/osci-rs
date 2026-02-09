# 01 - Project Setup

## Overview

This document covers the fundamentals of setting up a Rust project using Cargo, the Rust package manager and build system.

## Key Concepts

### Cargo - Rust's Build System

Cargo handles:
- **Dependency management** - Downloads and compiles external crates (libraries)
- **Building** - Compiles your code with the right flags
- **Testing** - Runs your test suite
- **Documentation** - Generates docs from your code comments

### Creating a Project

```bash
cargo new osci-rs
```

This creates:
```
osci-rs/
├── Cargo.toml    # Project configuration and dependencies
└── src/
    └── main.rs   # Entry point for binary applications
```

### Cargo.toml Structure

```toml
[package]
name = "osci-rs"           # Crate name
version = "0.1.0"          # Semantic versioning
edition = "2021"           # Rust edition (language version)

[dependencies]
eframe = "0.29"            # GUI framework
cpal = "0.15"              # Audio I/O
```

#### Edition
The `edition` field specifies which Rust language edition to use. Each edition can introduce syntax changes while maintaining backwards compatibility. The 2021 edition is stable and widely used.

#### Dependencies
Dependencies use [semver](https://semver.org/):
- `"0.29"` means `>=0.29.0, <0.30.0`
- `"0.29.1"` means exactly that version
- `"^0.29"` is the same as `"0.29"` (caret is default)

---

## Code Walkthrough

### The `main` Function

```rust
fn main() -> eframe::Result<()> {
    // Application code here
}
```

**Key points:**
- `fn` declares a function
- `main` is the entry point - execution starts here
- `-> eframe::Result<()>` is the return type
  - `Result<T, E>` is Rust's way of handling errors
  - `()` (unit type) means "no meaningful value" - like `void` in C
  - `eframe::Result<()>` is a type alias for `Result<(), eframe::Error>`

### The `use` Statement

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
```

`use` brings items into scope:
- `cpal::traits::DeviceTrait` - A trait from the cpal crate
- `eframe::egui` - The egui module from eframe
- `std::sync::Arc` - Arc from the standard library

### Modules and Visibility

In Rust, everything is **private by default**:
- `pub` makes items public
- `pub(crate)` makes items visible within the crate only
- No modifier = private to the current module

```rust
// Private - only this module can use it
fn helper() {}

// Public - anyone can use it
pub fn public_api() {}

// Crate-public - only this crate can use it
pub(crate) fn internal_api() {}
```

---

## Struct Definition

```rust
struct OsciApp {
    is_playing: Arc<AtomicBool>,
    _stream: Option<cpal::Stream>,
    frequency: f32,
    volume: f32,
    status: String,
}
```

**Key points:**
- `struct` defines a data structure (like a class without methods)
- Field types must be explicit
- `Arc<T>` = Atomic Reference Counted pointer (for thread-safe sharing)
- `Option<T>` = Either `Some(value)` or `None` (no null in Rust!)
- `f32` = 32-bit floating point
- `String` = Owned, growable UTF-8 string

### The Underscore Prefix

```rust
_stream: Option<cpal::Stream>,
```

The `_` prefix tells the compiler "I know this looks unused, but I need to keep it alive." The stream must be stored to prevent it from being dropped (which would stop audio playback).

---

## Implementation Blocks

```rust
impl OsciApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            is_playing: Arc::new(AtomicBool::new(false)),
            // ...
        }
    }
}
```

**Key points:**
- `impl StructName { }` adds methods to a struct
- `Self` is an alias for the struct type
- `&` means "reference to" (borrowing, not owning)
- `'_` is a lifetime placeholder (more on this later)

---

## Key Takeaways

1. **Cargo manages everything** - dependencies, building, testing
2. **Everything is private by default** - use `pub` to expose
3. **No null** - use `Option<T>` for optional values
4. **Explicit types** - Rust infers where it can, but struct fields need types
5. **`Result` for errors** - Functions that can fail return `Result<T, E>`

---

## Exercises

1. Run `cargo build` and examine the `target/` directory structure
2. Run `cargo doc --open` to see generated documentation
3. Try adding a comment with `///` above a function and regenerate docs
4. Change the edition to "2018" and see if it still compiles

---

## Links

- [The Cargo Book](https://doc.rust-lang.org/cargo/)
- [Rust Book: Packages and Crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html)
- [Rust Book: Modules](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html)
