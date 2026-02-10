# 16. Distribution

This milestone adds CI/CD pipelines and release optimization for distributing osci-rs as native binaries.

## Release Profile

### Cargo Profile Optimization

Rust's release profile is configured in `Cargo.toml`:

```toml
[profile.release]
opt-level = 3       # Maximum optimization
lto = "fat"         # Link-Time Optimization across all crates
codegen-units = 1   # Single codegen unit for better optimization
strip = true        # Remove debug symbols from binary
panic = "abort"     # No unwinding, smaller binary
```

#### What Each Setting Does

- **`opt-level = 3`**: Maximum compiler optimizations (loop unrolling, vectorization, inlining). Slower compilation, faster runtime.

- **`lto = "fat"`**: Link-Time Optimization analyzes and optimizes across all crate boundaries. Without LTO, each crate is optimized independently. With "fat" LTO, the compiler sees all code at once and can inline across crate boundaries, eliminate dead code globally, etc.

- **`codegen-units = 1`**: By default, Rust splits each crate into multiple parallel compilation units. Reducing to 1 allows better optimization (more inlining opportunities) at the cost of longer compile times.

- **`strip = true`**: Removes debug symbols and other metadata from the final binary, significantly reducing file size.

- **`panic = "abort"`**: Instead of unwinding the stack on panic (which requires runtime support code), the program immediately aborts. This removes unwinding tables from the binary.

## CI/CD with GitHub Actions

### Continuous Integration (ci.yml)

The CI pipeline runs on every push and pull request:

```yaml
jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - cargo build
      - cargo test
      - cargo fmt -- --check
      - cargo clippy -- -D warnings
```

This catches:
- **Build errors** on all three platforms
- **Test failures** across different OS environments
- **Formatting inconsistencies** via rustfmt
- **Code quality issues** via clippy with warnings-as-errors

#### Linux Dependencies

Linux builds require system libraries for audio, GUI, and file dialogs:

```yaml
- name: Install Linux dependencies
  run: |
    sudo apt-get install -y \
      libasound2-dev \     # ALSA (audio)
      libudev-dev \        # Device management
      libxkbcommon-dev \   # Keyboard input
      libgtk-3-dev \       # File dialogs (rfd)
      libgl-dev            # OpenGL (egui rendering)
```

### Release Pipeline (release.yml)

Triggered when pushing a version tag (`v*`):

```yaml
on:
  push:
    tags: ["v*"]
```

#### Build Matrix

Each platform builds a native binary:

| Platform | Target | Artifact |
|----------|--------|----------|
| Linux | x86_64-unknown-linux-gnu | osci-rs-linux-x86_64 |
| Windows | x86_64-pc-windows-msvc | osci-rs-windows-x86_64.exe |
| macOS | aarch64-apple-darwin | osci-rs-macos-aarch64 |

#### Release Creation

After all builds complete, a GitHub Release is created with all binaries attached:

```yaml
create-release:
  needs: build-release
  steps:
    - uses: softprops/action-gh-release@v2
      with:
        generate_release_notes: true
        files: release/*
```

## Rust Concepts

### Cross-Platform Builds

Rust's cross-compilation is specified via target triples:

```
arch-vendor-os-abi
```

Examples:
- `x86_64-unknown-linux-gnu` - 64-bit Linux with glibc
- `x86_64-pc-windows-msvc` - 64-bit Windows with MSVC toolchain
- `aarch64-apple-darwin` - ARM64 macOS (Apple Silicon)

### Clippy as Quality Gate

Clippy catches common Rust anti-patterns:

```bash
cargo clippy -- -D warnings
```

The `-D warnings` flag treats all warnings as errors, enforcing clean code in CI. Common catches include:
- Collapsible if statements
- Too many function arguments
- Incorrect self conventions
- Unnecessary clones

### Cargo Cache in CI

GitHub Actions caches the Cargo registry and build artifacts:

```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

The cache key includes the OS and `Cargo.lock` hash, so dependencies are only rebuilt when they change.

## Creating a Release

To create a new release:

```bash
# Tag the current commit
git tag v0.1.0

# Push the tag to trigger the release pipeline
git push origin v0.1.0
```

The CI pipeline will:
1. Build optimized binaries for all three platforms
2. Create a GitHub Release with auto-generated release notes
3. Attach all binaries as downloadable assets
