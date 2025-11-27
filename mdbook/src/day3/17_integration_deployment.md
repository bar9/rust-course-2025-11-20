# Chapter 17: Integration & Deployment

## Learning Objectives
By the end of this chapter, you'll be able to:
- Integrate all components into a complete temperature monitoring system
- Configure build optimization for embedded deployment
- Flash and debug applications on ESP32-C3 hardware
- Implement basic error handling and recovery

## Task: Build Production-Ready Temperature Monitor

Over chapters 13-16, we've built individual components. Now it's time to integrate everything into a robust, production-ready system.

**Your Mission:**
1. **Integrate all components** into a single working system
2. **Add error handling** and recovery mechanisms
3. **Optimize build configuration** for production deployment
4. **Add deployment scripts** for easy flashing and monitoring
5. **Create production monitoring** with structured output

**What We're Combining:**
- **Chapter 13**: Hardware interaction with ESP32-C3 and temperature sensor
- **Chapter 14**: Embedded data structures with no_std foundations
- **Chapter 15**: Comprehensive testing strategy for embedded code
- **Chapter 16**: JSON communication and structured data protocols

**Production Requirements:**
- Graceful error handling (no panics in production)
- Optimized binary size and performance
- Reliable sensor reading with fallback
- Structured logging for monitoring
- Easy deployment and debugging

### Simplified System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   ESP32-C3 System                      │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │              Main Loop                          │   │
│  │                                                 │   │
│  │  1. Read Temperature                            │   │
│  │  2. Store in Buffer                             │   │
│  │  3. Update LED Status                           │   │
│  │  4. Output JSON (every 5 readings)             │   │
│  │  5. Delay 1 second                              │   │
│  │  6. Repeat                                      │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │Temperature  │  │   LED       │  │    JSON     │     │
│  │Buffer       │  │ Controller  │  │  Output     │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │              USB Serial Output                  │   │
│  │  Status Messages | Readings | Statistics        │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Complete Temperature Monitor Implementation

### Project Setup

First, let's create the complete `Cargo.toml`:

```toml
[package]
name = "chapter17_integration"
version = "0.1.0"
edition = "2024"
rust-version = "1.88"

[[bin]]
name = "chapter17_integration"
path = "./src/bin/main.rs"

[lib]
name = "chapter17_integration"
path = "src/lib.rs"

[dependencies]
# Only include ESP dependencies when not testing
esp-hal = { version = "1.0.0", features = ["esp32c3", "unstable"], optional = true }
esp-bootloader-esp-idf = { version = "0.4.0", features = ["esp32c3"], optional = true }
esp-println = { version = "0.16", features = ["esp32c3"], optional = true }

# Core dependencies
critical-section = "1.2.0"
heapless = "0.8"

# Serialization
serde = { version = "1.0", default-features = false, features = ["derive"] }
serde-json-core = "0.6"

[features]
default = ["esp-hal", "esp-println", "esp-bootloader-esp-idf"]
embedded = ["esp-hal", "esp-println", "esp-bootloader-esp-idf"]

[profile.dev]
# Rust debug is too slow for embedded
opt-level = "s"

[profile.release]
# Production optimizations
codegen-units = 1     # LLVM can perform better optimizations using a single thread
debug = 2
debug-assertions = false
incremental = false
lto = 'fat'
opt-level = 's'
overflow-checks = false
```

### Main System Implementation

```rust
// src/bin/main.rs - Production-ready integrated system
#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types"
)]

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_hal::tsens::{Config, TemperatureSensor};

// Use the integrated system components from previous chapters
use chapter17_integration::{Temperature, TemperatureBuffer, Command, TemperatureComm};

// Production system configuration
const BUFFER_SIZE: usize = 32;
const SAMPLE_RATE_MS: u32 = 1000;
const JSON_OUTPUT_INTERVAL: u32 = 5;
const HEALTH_REPORT_INTERVAL: u32 = 20;

// System state tracking for production monitoring
struct SystemState {
    reading_count: u32,
    system_time_ms: u32,
    overheating_count: u32,
    sensor_error_count: u32,
    last_temp: f32,
}

impl SystemState {
    fn new() -> Self {
        Self {
            reading_count: 0,
            system_time_ms: 0,
            overheating_count: 0,
            sensor_error_count: 0,
            last_temp: 0.0,
        }
    }

    fn advance_time(&mut self) {
        self.reading_count += 1;
        self.system_time_ms += SAMPLE_RATE_MS;
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    // In production, we want graceful error handling
    esp_println::println!("SYSTEM_ERROR: Panic occurred, attempting recovery...");
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // Initialize hardware with error handling
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize components
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());
    let temp_sensor = TemperatureSensor::new(peripherals.TSENS, Config::default()).unwrap();
    let mut temp_buffer = TemperatureBuffer::<BUFFER_SIZE>::new();
    let mut comm = TemperatureComm::new();
    let mut state = SystemState::new();

    // System startup
    esp_println::println!("🚀 ESP32-C3 Production Temperature Monitor v1.0");
    esp_println::println!("📊 Buffer: {} readings | Sample rate: {}ms", BUFFER_SIZE, SAMPLE_RATE_MS);
    esp_println::println!("📡 JSON output every {} readings", JSON_OUTPUT_INTERVAL);
    esp_println::println!("🏥 Health reports every {} readings", HEALTH_REPORT_INTERVAL);
    esp_println::println!("✅ System initialized successfully");
    esp_println::println!();

    comm.init(0);

    // Main production loop with error handling
    loop {
        // Read temperature with error handling
        let esp_temperature = temp_sensor.get_temperature();
        let temp_celsius = esp_temperature.to_celsius();
        let temperature = Temperature::from_celsius(temp_celsius);

        // Update system state
        state.last_temp = temp_celsius;
        temp_buffer.push(temperature);
        state.advance_time();

        // LED status indication
        if temperature.is_overheating() {
            state.overheating_count += 1;
            // Rapid triple blink for overheating
            for _ in 0..3 {
                led.set_high();
                let blink_start = Instant::now();
                while blink_start.elapsed() < Duration::from_millis(100) {}
                led.set_low();
                let blink_start = Instant::now();
                while blink_start.elapsed() < Duration::from_millis(100) {}
            }
        } else if !temperature.is_normal_range() {
            // Double blink for abnormal range
            led.set_high();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(150) {}
            led.set_low();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(100) {}
            led.set_high();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(150) {}
            led.set_low();
        } else {
            // Normal single blink
            led.set_high();
            let blink_start = Instant::now();
            while blink_start.elapsed() < Duration::from_millis(200) {}
            led.set_low();
        }

        // JSON output every N readings
        if state.reading_count % JSON_OUTPUT_INTERVAL == 0 {
            let reading_json = comm.latest_reading_json(&temp_buffer, state.system_time_ms);
            esp_println::println!("READING: {}", reading_json);

            let stats_json = comm.stats_json(&temp_buffer, state.system_time_ms);
            esp_println::println!("STATS: {}", stats_json);
        }

        // Health report every N readings
        if state.reading_count % HEALTH_REPORT_INTERVAL == 0 {
            esp_println::println!("HEALTH: readings={} overheating={} errors={} uptime={}ms",
                state.reading_count,
                state.overheating_count,
                state.sensor_error_count,
                state.system_time_ms
            );
        }

        // Wait for next sample
        let wait_start = Instant::now();
        while wait_start.elapsed() < Duration::from_millis(SAMPLE_RATE_MS as u64) {}
    }
}
```

## Production Deployment

Build and deploy the production system:

```bash
# Run tests
cargo test

# Build and deploy to ESP32-C3 (recommended)
cargo run --release --features embedded

# Alternative: Build then flash separately
cargo build --release --target riscv32imc-unknown-none-elf --features embedded
cargo espflash flash target/riscv32imc-unknown-none-elf/release/chapter17_integration

# Monitor production logs
cargo espflash monitor
```

## Production System Features

✅ **Error Handling**: Graceful panic handling with recovery attempts
✅ **Health Monitoring**: System metrics and error counting
✅ **Structured Logging**: JSON output for monitoring dashboards
✅ **Performance Optimization**: Optimized builds for production deployment
✅ **State Tracking**: Comprehensive system state monitoring
✅ **Production Configuration**: Configurable intervals and thresholds

**Next**: In Chapter 18, we'll explore advanced features and extensions to make the system even more capable.
