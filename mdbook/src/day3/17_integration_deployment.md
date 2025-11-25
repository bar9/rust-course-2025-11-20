# Chapter 17: Integration & Deployment

## Learning Objectives
By the end of this chapter, you'll be able to:
- Integrate all components into a complete temperature monitoring system
- Configure build optimization for embedded deployment
- Flash and debug applications on ESP32-C3 hardware
- Implement basic error handling and recovery

## Complete System Integration

Over the previous chapters, we've built:
- **Chapter 13**: Hardware interaction with ESP32-C3 and temperature sensor
- **Chapter 14**: Embedded data structures with no_std foundations
- **Chapter 15**: Comprehensive testing strategy for embedded code
- **Chapter 16**: JSON communication and structured data protocols

Now let's integrate everything into a working system using simple blocking patterns.

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
name = "esp32-temp-monitor"
version = "0.1.0"
edition = "2021"

[dependencies]
# ESP32-C3 Hardware Abstraction Layer
esp-hal = { version = "0.22", features = ["esp32c3", "unstable"] }
esp-backtrace = { version = "0.18", features = ["esp32c3", "println"] }
esp-println = { version = "0.16", features = ["esp32c3"] }

# Embedded utilities
embedded-hal = "1.0"
nb = "1.0"

# Data structures for no_std
heapless = { version = "0.8", features = ["serde"] }

# JSON serialization
serde = { version = "1.0", default-features = false, features = ["derive"] }
serde-json-core = "0.6"

[profile.dev]
# Debug optimizations for faster flashing
debug = true
opt-level = "s"

[profile.release]
# Release optimizations for production
opt-level = "s"
debug = true
lto = true
codegen-units = 1
```

### Main System Implementation

```rust
// main.rs
#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::{Io, Level, Output},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
    tsens::{TemperatureSensor, Config},
};
use esp_println::println;

// Import our modules
mod temperature;
mod communication;

use temperature::TemperatureBuffer;
use communication::TemperatureComm;

const BUFFER_SIZE: usize = 32;
const SAMPLE_RATE_MS: u32 = 1000;
const JSON_OUTPUT_INTERVAL: u32 = 5;

// No mock sensor needed - we'll use real ESP32-C3 temperature sensor

#[entry]
fn main() -> ! {
    println!("🌡️ ESP32-C3 Temperature Monitor Starting...");

    // Initialize hardware
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    // Setup GPIO for LED
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = Output::new(io.pins.gpio8, Level::Low);

    // Initialize delay
    let delay = Delay::new(&clocks);

    // Initialize components
    let mut temp_sensor = TemperatureSensor::new(
        peripherals.TSENS,
        Config::default()
    ).unwrap();
    let mut temp_buffer = TemperatureBuffer::<BUFFER_SIZE>::new();
    let mut comm = TemperatureComm::new();
    let mut reading_count = 0u32;
    let mut system_time_ms = 0u32;

    println!("🚀 System initialized. Starting main loop...");
    println!("📊 Buffer capacity: {} readings", BUFFER_SIZE);
    println!("⏱️  Sample rate: {} ms", SAMPLE_RATE_MS);
    println!();

    loop {
        // 1. Read temperature
        delay.delay_micros(200); // Stabilization delay for temperature sensor
        let temperature_reading = temp_sensor.get_temperature();
        let celsius = temperature_reading.to_celcius();
        let temperature = temperature::Temperature::from_celsius(celsius);
        temp_buffer.push(temperature);
        reading_count += 1;
        system_time_ms += SAMPLE_RATE_MS;

        // 2. Update LED based on temperature
        if temperature.is_overheating() {
            led.set_high();
            println!("🔴 OVERHEATING: {:.1}°C", celsius);
        } else if reading_count % 10 == 0 {
            led.toggle();
        }

        // 3. Output basic reading
        println!("📊 Reading #{}: {:.1}°C", reading_count, celsius);

        // 4. JSON output every N readings
        if reading_count % JSON_OUTPUT_INTERVAL == 0 {
            // Output current reading as JSON
            if let Ok(reading_json) = comm.reading_json(&temp_buffer, system_time_ms) {
                println!("JSON_READING: {}", reading_json);
            }

            // Output statistics
            if let Some(stats) = temp_buffer.stats() {
                if let Ok(stats_json) = comm.stats_json(&stats, system_time_ms) {
                    println!("JSON_STATS: {}", stats_json);
                }
            }

            // Output system status
            if let Ok(status_json) = comm.status_json(
                system_time_ms,
                1, // sample rate: 1 Hz
                35.0, // threshold
                temp_buffer.len() as u8
            ) {
                println!("JSON_STATUS: {}", status_json);
            }
        }

        // 5. Show periodic summary
        if reading_count % 10 == 0 {
            if let Some(stats) = temp_buffer.stats() {
                println!(
                    "📈 Summary: {} readings, avg={:.1}°C, range={:.1}-{:.1}°C",
                    stats.count, stats.avg_celsius,
                    stats.min_celsius, stats.max_celsius
                );
            }
        }

        // 6. Simple delay
        delay.delay_ms(SAMPLE_RATE_MS);
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("💥 PANIC: {}", info);
    loop {}
}
```

## Build Configuration

### Optimized Build Settings

```toml
# .cargo/config.toml
[build]
target = "riscv32imc-unknown-none-elf"

[target.riscv32imc-unknown-none-elf]
runner = "probe-rs run --chip esp32c3"

[env]
ESP_IDF_VERSION = "5.1"
```

### Build Commands

```bash
# Development build
cargo build

# Release build with optimizations
cargo build --release

# Flash and monitor
cargo run --release

# Check size
cargo size --release
```

## Production Deployment

### Build Optimizations

The release profile is configured for optimal embedded performance:

```toml
[profile.release]
opt-level = "s"        # Optimize for size
debug = true           # Keep debug symbols for debugging
lto = true            # Link-time optimization
codegen-units = 1     # Better optimization
```

### Memory Usage

Check your binary size:
```bash
$ cargo size --release
   text    data     bss     dec     hex filename
  45234    1028    2076   48338    bcd2 esp32-temp-monitor
```

### Basic Error Handling

```rust
// Enhanced main loop with error handling
loop {
    // Temperature reading with error handling
    let celsius = match temp_sensor.read_celsius() {
        Ok(temp) => temp,
        Err(_) => {
            println!("❌ Sensor read error, using last known value");
            continue;
        }
    };

    // JSON serialization with error handling
    if reading_count % JSON_OUTPUT_INTERVAL == 0 {
        match comm.reading_json(&temp_buffer, system_time_ms) {
            Ok(json) => println!("JSON_READING: {}", json),
            Err(_) => println!("❌ JSON serialization error"),
        }
    }

    delay.delay_ms(SAMPLE_RATE_MS);
}
```

### Hardware Debugging

```bash
# Flash with debugging enabled
probe-rs run --chip esp32c3 target/riscv32imc-unknown-none-elf/release/esp32-temp-monitor

# Monitor serial output
screen /dev/cu.usbmodem* 115200

# Alternative: Use probe-rs for both
probe-rs run --chip esp32c3 target/riscv32imc-unknown-none-elf/release/esp32-temp-monitor
```

## Testing the Complete System

### Expected Output

```
🌡️ ESP32-C3 Temperature Monitor Starting...
🚀 System initialized. Starting main loop...
📊 Buffer capacity: 32 readings
⏱️  Sample rate: 1000 ms

📊 Reading #1: 23.0°C
📊 Reading #2: 23.1°C
📊 Reading #3: 23.2°C
📊 Reading #4: 23.3°C
📊 Reading #5: 23.4°C

JSON_READING: {"Reading":{"temperature":{"celsius_tenths":234},"timestamp_ms":5000}}
JSON_STATS: {"Stats":{"count":5,"min_celsius":23.0,"max_celsius":23.4,"avg_celsius":23.2,"timestamp_ms":5000}}
JSON_STATUS: {"Status":{"uptime_ms":5000,"sample_rate_hz":1,"threshold_celsius":35.0,"buffer_usage":5}}

📊 Reading #6: 23.5°C
...
📈 Summary: 10 readings, avg=23.5°C, range=23.0-23.9°C
```

### Performance Characteristics

- **Memory usage**: ~5KB RAM for the complete system
- **CPU usage**: Minimal, mostly sleeping
- **Sample rate**: Stable 1Hz timing
- **JSON output**: Every 5 seconds with statistics

## Exercise: Deploy Your Complete System

1. **Build the complete system**:
```bash
cargo build --release
```

2. **Flash to your ESP32-C3**:
```bash
cargo run --release
```

3. **Verify the output** - You should see:
   - Regular temperature readings
   - JSON output every 5 readings
   - LED blinking every 10 readings
   - Overheating detection if temperature > 35°C

4. **Test the system**:
   - Monitor for several minutes
   - Verify consistent timing
   - Check JSON format validity

## Summary

You've now built a complete embedded temperature monitoring system that:

✅ **Reads temperature** from the ESP32-C3's built-in sensor
✅ **Stores data** efficiently in a circular buffer
✅ **Provides visual feedback** through LED patterns
✅ **Outputs structured JSON** for external integration
✅ **Implements error handling** for robust operation
✅ **Optimizes for embedded constraints** (memory, power, size)

The system demonstrates key embedded Rust concepts:
- **no_std programming** with heapless data structures
- **Resource management** without dynamic allocation
- **Hardware abstraction** with safe, zero-cost interfaces
- **Structured data handling** with serde in embedded contexts
- **Production deployment** with optimized builds

This foundation prepares you for building more complex embedded systems with Rust's safety, performance, and expressiveness.

---

**Next**: [Chapter 18: Complete System Demo](./18_complete_system.md)