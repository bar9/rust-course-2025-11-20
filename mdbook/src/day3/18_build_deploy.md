# Chapter 18: Build, Package & Deploy

## Learning Objectives
- Master Cargo workspaces for multi-crate embedded projects
- Use features for std/no_std conditional compilation
- Set up cross-compilation for embedded targets (ESP32-C3)
- Implement CI/CD pipelines for embedded systems
- Optimize binary size for resource-constrained environments
- Deploy to embedded hardware using modern tooling

## Cargo Workspaces for Embedded Projects

Our temperature monitoring system demonstrates how workspaces manage complexity in embedded projects with multiple targets.

### Our Capstone Workspace Structure

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "temp_core",      # Core types and traits
    "temp_store",     # Thread-safe storage
    "temp_async",     # Async monitoring
    "temp_protocol",  # Serialization protocols
    "temp_embedded",  # no_std embedded version
    "temp_esp32",     # ESP32-C3 deployment
]
resolver = "2"

# Shared dependencies across all crates
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.35", features = ["full"] }
heapless = "0.8"
postcard = { version = "1.0", features = ["alloc"] }

# Optimized release profile
[profile.release]
lto = true              # Link-time optimization
opt-level = 3           # Maximum optimization
codegen-units = 1       # Better optimization

# Embedded-specific profile
[profile.embedded]
inherits = "release"
opt-level = "z"         # Optimize for size
strip = true            # Strip debug symbols
panic = "abort"         # Smaller panic handler
```

This structure allows us to:
- **Shared Development**: All crates use the same dependency versions
- **Progressive Complexity**: Each crate builds on the previous ones
- **Multi-Target Builds**: Desktop and embedded from the same codebase
- **Optimized Profiles**: Different optimization strategies for different use cases

### Member Crate Configuration

```toml
# temp_core/Cargo.toml - Foundation crate
[package]
name = "temp_core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true, default-features = false, optional = true }

[features]
default = ["serde"]
serde = ["dep:serde"]
std = []
```

```toml
# temp_embedded/Cargo.toml - no_std version
[package]
name = "temp_embedded"
version = "0.1.0"
edition = "2021"

[dependencies]
heapless = { workspace = true }
serde = { workspace = true, default-features = false }
postcard = { workspace = true }
temp_core = { path = "../temp_core", default-features = false }

[features]
default = []
std = ["temp_core/std"]
```

### Workspace Commands for Our Project

```bash
# Build all crates for desktop
cargo build --workspace

# Test specific embedded crate
cargo test -p temp_embedded

# Build only the embedded components
cargo build -p temp_embedded -p temp_esp32

# Check all crates for both std and no_std
cargo check --workspace --all-targets
cargo check --workspace --no-default-features

# Run desktop version of temperature monitor
cargo run -p temp_protocol
```

## Features for std/no_std Conditional Compilation

Features enable the same codebase to work on both desktop and embedded systems.

### Feature Strategy in Our Temperature Monitor

```rust
// temp_core/src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

// Conditional imports based on environment
#[cfg(feature = "std")]
use std::fmt;

#[cfg(not(feature = "std"))]
use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// Conditional derive based on features
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Temperature {
    pub celsius: f32,
}

// Platform-specific implementations
impl Temperature {
    #[cfg(feature = "std")]
    pub fn from_system_sensor() -> Result<Self, std::io::Error> {
        // Read from system temperature sensor
        unimplemented!("System sensor reading")
    }

    #[cfg(not(feature = "std"))]
    pub fn from_embedded_sensor(adc_value: u16) -> Self {
        // Convert ADC reading to temperature
        let voltage = (adc_value as f32 / 4095.0) * 3.3;
        let celsius = voltage / 0.01; // 10mV/°C sensor
        Temperature { celsius }
    }
}
```

### Conditional Storage Implementation

```rust
// temp_store/src/lib.rs
#[cfg(feature = "std")]
use std::sync::{Arc, Mutex};

#[cfg(not(feature = "std"))]
use heapless::Vec;

// Different storage backends based on environment
#[cfg(feature = "std")]
pub struct TemperatureStore {
    readings: Arc<Mutex<Vec<TemperatureReading>>>,
    capacity: usize,
}

#[cfg(not(feature = "std"))]
pub struct TemperatureStore<const N: usize> {
    readings: Vec<TemperatureReading, N>,
    total_readings: u32,
}
```

### Build Configuration Examples

```bash
# Desktop build with all features
cargo build --features "std,serde,async"

# Embedded build minimal features
cargo build --no-default-features --features "serde"

# ESP32 build
cargo build --target riscv32imc-esp-espidf --no-default-features
```

## Cross-Compilation for Embedded Targets

### ESP32-C3 Target Setup

```bash
# Install ESP32 toolchain
curl -LO https://github.com/esp-rs/rust-build/releases/download/v1.75.0.0/install-rust-toolchain.sh
chmod +x install-rust-toolchain.sh
./install-rust-toolchain.sh

# Install additional tools
cargo install espflash
cargo install ldproxy
```

### Cross-Compilation Configuration

```toml
# .cargo/config.toml
[build]
# Default target for local development
target = "x86_64-unknown-linux-gnu"

[target.riscv32imc-esp-espidf]
linker = "ldproxy"
runner = "espflash flash --monitor"

[target.riscv32imc-unknown-none-elf]
linker = "riscv32-esp-elf-gcc"
runner = "probe-rs run --chip esp32c3"

[env]
# ESP-IDF specific environment variables
ESP_IDF_VERSION = "v5.1.2"
```

### Multi-Target Build Scripts

```bash
# build.sh - Cross-platform build script
#!/bin/bash
set -e

echo "Building temperature monitor for multiple targets..."

# Desktop builds
echo "Building for desktop..."
cargo build --workspace --release

# Embedded simulation build
echo "Building embedded simulation..."
cargo build -p temp_embedded --no-default-features --release

# ESP32-C3 build
echo "Building for ESP32-C3..."
cargo build -p temp_esp32 \
    --target riscv32imc-esp-espidf \
    --no-default-features \
    --features "embedded" \
    --release

echo "All builds completed successfully!"
```

## Binary Size Optimization for Embedded

### Embedded-Specific Profile

```toml
# Cargo.toml
[profile.embedded]
inherits = "release"
opt-level = "z"           # Optimize for size over speed
lto = true               # Link-time optimization
codegen-units = 1        # Better optimization
strip = "symbols"        # Remove debug symbols
panic = "abort"          # Smaller panic handler
overflow-checks = false  # Disable overflow checks

[profile.embedded.package.temp_esp32]
opt-level = "s"          # Slightly less aggressive for main binary
```

### Size Analysis Tools

```bash
# Analyze binary size breakdown
cargo bloat --release --target riscv32imc-esp-espidf -p temp_esp32

# Size comparison between profiles
cargo size --release -- --format=sysv
cargo size --profile embedded -- --format=sysv

# Memory usage analysis
cargo nm --release | grep -E "(\.text|\.data|\.bss)" | sort
```

### Code Size Optimization Techniques

```rust
// Use const fn for compile-time computation
const fn calculate_buffer_size(sensor_count: usize) -> usize {
    sensor_count * 64 // 64 readings per sensor
}

// Prefer arrays over Vec when size is known
const SENSORS: usize = 3;
const BUFFER_SIZE: usize = calculate_buffer_size(SENSORS);
static mut READINGS_BUFFER: [TemperatureReading; BUFFER_SIZE] = [TemperatureReading::empty(); BUFFER_SIZE];

// Use #[inline] judiciously
#[inline]
fn critical_fast_path(temp: f32) -> bool {
    temp > 50.0 // Critical temperature check
}

// Avoid large dependencies for simple operations
// Instead of regex for simple patterns
fn parse_sensor_id(input: &str) -> Option<u8> {
    if input.starts_with("temp_") && input.len() == 7 {
        input.chars().last()?.to_digit(10).map(|d| d as u8)
    } else {
        None
    }
}
```

## CI/CD for Embedded Systems

### Comprehensive GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
name: Temperature Monitor CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test Suite
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust: [stable, beta]
    steps:
    - uses: actions/checkout@v4

    - name: Install Rust toolchain
      uses: dtolnay/rust-toolchain@stable
      with:
        toolchain: ${{ matrix.rust }}
        components: rustfmt, clippy

    - name: Cache cargo dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

    - name: Check formatting
      run: cargo fmt --all -- --check

    - name: Clippy analysis
      run: cargo clippy --workspace --all-targets --all-features -- -D warnings

    - name: Test std builds
      run: cargo test --workspace --all-features

    - name: Test no_std builds
      run: cargo test -p temp_core -p temp_embedded --no-default-features

  cross-compile:
    name: Cross Compilation
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - x86_64-pc-windows-gnu
          - aarch64-unknown-linux-gnu
          - riscv32imc-unknown-none-elf
    steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        targets: ${{ matrix.target }}

    - name: Install cross-compilation tools
      if: matrix.target == 'riscv32imc-unknown-none-elf'
      run: |
        curl -LO https://github.com/esp-rs/rust-build/releases/download/v1.75.0.0/install-rust-toolchain.sh
        chmod +x install-rust-toolchain.sh
        ./install-rust-toolchain.sh --extra-crates "ldproxy"

    - name: Build for target
      run: |
        if [[ "${{ matrix.target }}" == "riscv32imc-unknown-none-elf" ]]; then
          cargo build -p temp_embedded --target ${{ matrix.target }} --no-default-features
        else
          cargo build --workspace --target ${{ matrix.target }} --release
        fi

  embedded-test:
    name: Embedded Testing
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable

    - name: Install embedded testing tools
      run: |
        cargo install probe-rs --features cli
        cargo install cargo-embed

    - name: Test embedded builds
      run: |
        cargo check -p temp_embedded --target thumbv7em-none-eabihf --no-default-features
        cargo test -p temp_embedded --lib # Run tests on host

  size-analysis:
    name: Binary Size Analysis
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - run: cargo install cargo-bloat cargo-size

    - name: Analyze binary sizes
      run: |
        cargo build --release -p temp_esp32
        cargo size --release -p temp_esp32 -- --format=sysv
        cargo bloat --release -p temp_esp32
```

### Release Pipeline

```yaml
# .github/workflows/release.yml
name: Release

on:
  release:
    types: [created]

jobs:
  distribute:
    name: Distribute Binaries
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            suffix: ""
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            suffix: ".exe"
          - os: macos-latest
            target: x86_64-apple-darwin
            suffix: ""

    steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        targets: ${{ matrix.target }}

    - name: Build release binary
      run: cargo build --release -p temp_protocol --target ${{ matrix.target }}

    - name: Upload release asset
      uses: actions/upload-release-asset@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        upload_url: ${{ github.event.release.upload_url }}
        asset_path: ./target/${{ matrix.target }}/release/temp_monitor${{ matrix.suffix }}
        asset_name: temp_monitor-${{ matrix.target }}${{ matrix.suffix }}
        asset_content_type: application/octet-stream
```

## Modern Distribution with cargo-dist

cargo-dist automates building and distributing Rust binaries across platforms.

### cargo-dist Configuration

```toml
# Cargo.toml
[workspace.metadata.dist]
# The preferred cargo-dist version to use in CI
cargo-dist-version = "0.8.0"
# CI backends to support
ci = ["github"]
# The installers to generate for each app
installers = ["shell", "powershell", "homebrew"]
# Target platforms to build apps for ("rustc -vV" to find yours)
targets = [
  "aarch64-apple-darwin",
  "x86_64-apple-darwin",
  "x86_64-pc-windows-msvc",
  "x86_64-unknown-linux-gnu"
]

# The archive format to use for windows builds (defaults to .zip)
windows-archive = ".tar.gz"
# The archive format to use for non-windows builds (defaults to .tar.xz)
unix-archive = ".tar.gz"

[workspace.metadata.dist.dependencies.apt]
cmake = '*'
```

### Initialize cargo-dist

```bash
# Initialize cargo-dist in the workspace
cargo dist init

# Generate CI configuration
cargo dist generate-ci

# Build local release
cargo dist build

# Check what cargo-dist would build
cargo dist plan
```

## Embedded Deployment Tools

### probe-rs for Hardware Debugging

```toml
# .cargo/config.toml
[target.thumbv7em-none-eabihf]
runner = "probe-rs run --chip STM32F401CCUx"

[target.riscv32imc-esp-espidf]
runner = "espflash flash --monitor"
```

### Deploy Script Example

```bash
#!/bin/bash
# deploy.sh - Deploy to ESP32-C3

set -e

CHIP="esp32c3"
PORT="/dev/ttyUSB0"

echo "Building temperature monitor for ESP32-C3..."
cargo build --release -p temp_esp32 \
    --target riscv32imc-esp-espidf \
    --no-default-features

echo "Flashing to ESP32-C3..."
espflash flash \
    --chip $CHIP \
    --port $PORT \
    --baud 921600 \
    target/riscv32imc-esp-espidf/release/temp_esp32

echo "Monitoring serial output..."
espflash monitor --port $PORT
```

### probe-rs Configuration

```toml
# Embed.toml
[default.probe]
protocol = "Swd"
speed = 20_000

[default.flashing]
enabled = true
restore_unwritten_bytes = false

[default.general]
chip = "STM32F401CCUx"

[default.reset]
enabled = true
halt_afterwards = false

[default.gdb]
enabled = false
```

## Language Comparison: Build & Deploy

| Aspect | **Rust** | **C/C++** | **C#** | **Go** |
|--------|----------|-----------|--------|---------|
| **Package Manager** | Built-in (Cargo) | External (vcpkg, conan) | Built-in (NuGet) | Built-in (go mod) |
| **Build System** | Cargo | CMake, Make, Ninja | MSBuild | Built-in |
| **Cross-compilation** | Native support | Complex toolchain setup | Limited | Excellent |
| **Embedded Support** | First-class | Traditional choice | Limited | Not suitable |
| **Binary Size** | <50KB optimized | <20KB (C), >1MB (C++) | >10MB with runtime | >5MB |
| **CI/CD Complexity** | Simple YAML | Complex scripts | Azure DevOps integration | Simple |
| **Deployment** | Single binary | Dependencies/static linking | Runtime required | Single binary |

## Exercise: Deploy Temperature Monitor to Production

Build a complete deployment pipeline for your temperature monitoring system that works across desktop and embedded targets.

### Requirements

Your deployment system should:

1. **Multi-Target Building**:
   - Desktop version with full std features
   - Embedded version with no_std constraints
   - ESP32-C3 version for hardware deployment

2. **Feature Configuration**:
   - `std` feature for desktop builds
   - `embedded` feature for no_std builds
   - `simulation` feature for testing without hardware

3. **Size Optimization**:
   - Optimized profiles for embedded deployment
   - Binary size analysis and reporting

4. **CI/CD Pipeline**:
   - Automated testing for both std and no_std builds
   - Cross-compilation verification
   - Release artifact generation

5. **Distribution**:
   - Desktop binaries for multiple platforms
   - Embedded firmware images
   - Installation scripts

### Starting Configuration

Your workspace should already have the basic structure. Enhance it with deployment-specific configuration:

```toml
# Cargo.toml (add these sections)
[workspace.metadata.dist]
cargo-dist-version = "0.8.0"
ci = ["github"]
targets = ["x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc", "x86_64-apple-darwin"]

[profile.embedded]
inherits = "release"
opt-level = "z"
lto = true
strip = true
panic = "abort"

[profile.simulation]
inherits = "dev"
debug = true
opt-level = 1
```

### Implementation Steps

1. **Configure Features**: Set up conditional compilation for different deployment targets

2. **Optimize Builds**: Create profiles optimized for embedded deployment

3. **Set Up Cross-Compilation**: Configure toolchains for ESP32-C3 target

4. **Create Build Scripts**: Automate building for multiple targets

5. **Implement CI/CD**: Create GitHub Actions workflow for automated testing and deployment

6. **Test Deployment**: Verify builds work on target platforms

### Success Criteria

- Desktop version runs with full features and networking
- Embedded version compiles to <50KB binary
- CI pipeline successfully builds and tests all targets
- Binary size analysis reports memory usage
- ESP32-C3 version can be flashed to hardware
- Release artifacts are automatically generated

### Extension Ideas

1. **Over-the-Air Updates**: Implement firmware update mechanism
2. **Docker Deployment**: Create containerized version for server deployment
3. **Package Distribution**: Publish crates to crates.io
4. **Hardware Simulation**: Test embedded code without physical hardware
5. **Performance Benchmarks**: Automated performance testing in CI

## Key Takeaways

1. **Workspaces Scale**: Multi-crate workspaces manage complexity in embedded projects
2. **Features Enable Flexibility**: Same codebase works across std and no_std environments
3. **Cross-Compilation is Powerful**: Build for any target from any development machine
4. **Size Matters in Embedded**: Optimization profiles crucial for resource-constrained systems
5. **CI/CD Prevents Regressions**: Automated testing catches issues across all targets
6. **Modern Tooling Simplifies Deployment**: cargo-dist and probe-rs streamline distribution
7. **Rust Shines in Embedded**: Memory safety without performance cost ideal for embedded systems

The Rust ecosystem provides unmatched tooling for building, testing, and deploying embedded systems while maintaining the same high-level abstractions used in desktop development.

**Next**: In Chapter 19, we'll bring everything together in the final capstone integration, deploying our complete temperature monitoring system to ESP32-C3 hardware.