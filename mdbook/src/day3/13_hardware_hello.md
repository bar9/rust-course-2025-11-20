# Chapter 13: Hardware Hello - ESP32-C3 Basics

## Learning Objectives
By the end of this chapter, you'll be able to:
- Set up ESP32-C3 development environment
- Understand the ESP32-C3 hardware capabilities and built-in sensors
- Create your first embedded Rust program that blinks an LED
- Read temperature from the ESP32-C3's built-in temperature sensor
- Send data over USB Serial for monitoring
- Understand the basics of embedded program structure and entry points

## Welcome to Embedded Rust!

After learning Rust fundamentals, it's time to apply that knowledge to real hardware. The ESP32-C3 is perfect for learning embedded Rust because it has:

- **Built-in temperature sensor** - No external components needed!
- **USB Serial support** - Easy debugging and communication
- **WiFi capability** - For IoT projects
- **Rust-first tooling** - Excellent `esp-hal` and ecosystem support
- **RISC-V architecture** - Modern, open-source instruction set

**Why Start with Hardware?**

Many embedded courses start with theory, but we're jumping straight into the exciting part - making real hardware do real things! This approach helps you:
- See immediate results (LED blinking, temperature readings)
- Understand constraints early (memory, power, timing)
- Build intuition for embedded programming patterns
- Stay motivated with tangible progress

## ESP32-C3 Hardware Overview

The ESP32-C3 is a system-on-chip (SoC) that includes:

```
┌─────────────────────────────────────┐
│            ESP32-C3 SoC             │
│                                     │
│  ┌─────────────┐  ┌─────────────┐   │
│  │ RISC-V Core │  │    WiFi     │   │
│  │  160 MHz    │  │ 802.11 b/g/n│   │
│  └─────────────┘  └─────────────┘   │
│                                     │
│  ┌─────────────┐  ┌─────────────┐   │
│  │   320KB     │  │    GPIO     │   │
│  │    RAM      │  │   Pins      │   │
│  └─────────────┘  └─────────────┘   │
│                                     │
│  ┌─────────────┐  ┌─────────────┐   │
│  │   4MB       │  │Temperature  │   │
│  │   Flash     │  │   Sensor    │   │ ← We'll use this!
│  └─────────────┘  └─────────────┘   │
└─────────────────────────────────────┘
```

**Key Features for Our Project:**
- **Built-in Temperature Sensor**: Returns readings in digital format
- **USB Serial**: Built-in USB-to-serial conversion for easy debugging
- **GPIO Pin 8**: Usually connected to an LED on development boards
- **Low Power**: Can run on batteries for IoT applications

## Development Environment Setup

### Prerequisites

```bash
# Install Rust targets for ESP32-C3
rustup target add riscv32imc-unknown-none-elf

# Install probe-rs for flashing and debugging
cargo install probe-rs --features cli

# Install espflash as alternative flashing tool
cargo install espflash

# Install serial monitoring tool (optional, for serial communication)
cargo install serialport-rs
```

### Hardware Requirements
- ESP32-C3 development board (like ESP32-C3-DevKitM-1)
- USB-C cable for programming and power
- Computer with USB port

**No external sensors or components needed** - we'll use the built-in temperature sensor!

## Your First ESP32 Program: LED Blink

Let's start with the embedded equivalent of "Hello, World!" - blinking an LED:

```rust
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
};

#[entry]
fn main() -> ! {
    // Take ownership of hardware peripherals
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);

    // Configure system clock to maximum frequency
    let clocks = ClockControl::max(system.clock_control).freeze();

    // Create delay provider for timing
    let delay = Delay::new(&clocks);

    // Initialize GPIO subsystem
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Configure GPIO 8 as output (usually connected to LED)
    let mut led = Output::new(io.pins.gpio8, Level::Low);

    // Print startup message
    esp_println::println!("ESP32-C3 LED Blink Starting!");
    esp_println::println!("Hardware: ESP32-C3 at {} MHz", clocks.cpu_clock.to_MHz());

    // Main loop - runs forever
    loop {
        esp_println::println!("LED ON");
        led.set_high();
        delay.delay_millis(1000);

        esp_println::println!("LED OFF");
        led.set_low();
        delay.delay_millis(1000);
    }
}
```

### Understanding the Code

**Key Differences from Regular Rust:**
- `#![no_std]` - No standard library (no heap, no OS services)
- `#![no_main]` - No traditional main function (embedded entry point)
- `#[entry]` - Marks the embedded program entry point
- `-> !` - Function never returns (embedded programs run forever)

**Hardware Abstraction:**
- `Peripherals::take()` - Ownership of hardware (can only happen once!)
- `gpio::Output` - Type-safe GPIO pin configuration
- `delay::Delay` - Hardware timer-based delays

**Why These Patterns?**
- **Singleton Pattern**: Hardware can only have one owner
- **Type Safety**: GPIO configuration enforced at compile time
- **Zero Cost**: Abstractions compile to direct hardware access

## Reading the Built-in Temperature Sensor

Now let's read the ESP32-C3's built-in temperature sensor:

```rust
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

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

    // Initialize GPIO for LED status
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = Output::new(io.pins.gpio8, Level::Low);

    // Initialize the built-in temperature sensor
    let mut temp_sensor = TemperatureSensor::new(
        peripherals.TSENS,
        Config::default()
    ).unwrap();

    esp_println::println!("ESP32-C3 Temperature Monitor");
    esp_println::println!("Built-in sensor initialized");
    esp_println::println!("Reading temperature every 2 seconds...");
    esp_println::println!();

    let mut reading_count = 0;

    loop {
        // Stabilization delay (recommended by ESP-HAL docs)
        delay.delay_micros(200);

        // Read temperature from built-in sensor
        let temperature = temp_sensor.get_temperature();
        let temp_celsius = temperature.to_celcius();
        reading_count += 1;

        // Show reading with LED blink pattern
        if temp_celsius > 30.0 {
            // Rapid blink for high temperature
            led.set_high();
            delay.delay_millis(100);
            led.set_low();
            delay.delay_millis(100);
            led.set_high();
            delay.delay_millis(100);
            led.set_low();
        } else {
            // Single blink for normal temperature
            led.set_high();
            delay.delay_millis(200);
            led.set_low();
        }

        // Print temperature reading
        esp_println::println!(
            "Reading #{}: Temperature = {:.1}°C",
            reading_count,
            temp_celsius
        );

        // Status information
        if reading_count % 10 == 0 {
            esp_println::println!("Status: {} readings completed", reading_count);
            esp_println::println!();
        }

        delay.delay_millis(1800); // Rest of 2-second interval
    }
}
```

### Understanding Temperature Sensor Code

**New Concepts:**
- `tsens::TemperatureSensor` - Hardware abstraction for built-in sensor (requires `unstable` feature)
- `get_temperature()` - Returns Temperature struct
- `to_celcius()` - Converts to Celsius value
- **No external wiring** - Sensor is built into the chip!

**Data Flow:**
```
Hardware Sensor → ADC → Digital Value → Celsius Conversion → Your Code
```

**LED Status Patterns:**
- Normal temp (≤30°C): Single blink
- High temp (>30°C): Double rapid blink

## Building and Running on Hardware

### Project Structure

Create a new embedded project:

```bash
cargo new --bin temp_monitor
cd temp_monitor
```

Update `Cargo.toml`:

```toml
[package]
name = "temp_monitor"
version = "0.1.0"
edition = "2021"

[dependencies]
esp-backtrace = "0.18"
esp-hal = { version = "0.22", features = ["esp32c3", "unstable"] }
esp-println = { version = "0.16", features = ["esp32c3"] }

[profile.dev]
# Rust debug is too slow for embedded
opt-level = "s"

[profile.release]
codegen-units = 1
debug = 2
debug-assertions = false
incremental = false
lto = 'fat'
opt-level = 's'
overflow-checks = false
```

### Building and Flashing

```bash
# Build for ESP32-C3 target
cargo build --release

# Flash to hardware with probe-rs
probe-rs run --chip=esp32c3 target/riscv32imc-unknown-none-elf/release/temp_monitor

# Alternative: Flash with espflash
espflash flash --monitor target/riscv32imc-unknown-none-elf/release/temp_monitor
```

### Serial Monitoring

Connect to see output:

```bash
# Using screen (macOS/Linux)
screen /dev/cu.usbmodem* 115200

# Using probe-rs (shows both flashing and serial output)
probe-rs run --chip=esp32c3 target/riscv32imc-unknown-none-elf/release/temp_monitor
```

**Expected Output:**
```
ESP32-C3 Temperature Monitor
Built-in sensor initialized
Reading temperature every 2 seconds...

Reading #1: Temperature = 24.3°C
Reading #2: Temperature = 24.5°C
Reading #3: Temperature = 24.1°C
...
Reading #10: Temperature = 24.7°C
Status: 10 readings completed
```

## Understanding Embedded Program Structure

### Program Lifecycle

```rust
// 1. Hardware initialization
let peripherals = Peripherals::take();  // Get hardware ownership
let clocks = ClockControl::max(...);    // Configure clocks
let delay = Delay::new(&clocks);        // Set up timing

// 2. Peripheral configuration
let io = Io::new(...);                  // Initialize GPIO system
let mut led = Output::new(...);         // Configure specific pins
let mut temp_sensor = TemperatureSensor::new(...); // Set up sensors

// 3. Main application loop
loop {
    // Read sensors
    // Process data
    // Control outputs
    // Timing/delays
}
```

### Memory and Resource Management

**Key Constraints:**
- **320KB RAM** - All variables must fit in memory
- **No heap allocation** - Only stack and static allocation
- **No garbage collector** - Manual memory management
- **Real-time constraints** - Delays must be predictable

**Best Practices:**
- Use `delay.delay_millis()` instead of `std::thread::sleep()`
- Prefer fixed-size arrays over dynamic vectors
- Initialize all peripherals before main loop
- Keep critical timing sections short

### Error Handling in Embedded

Embedded Rust uses `Result<T, E>` even more extensively:

```rust
// Temperature sensor can fail
match temp_sensor.read_celsius() {
    Ok(temperature) => {
        esp_println::println!("Temperature: {:.1}°C", temperature);
    }
    Err(e) => {
        esp_println::println!("Sensor error: {:?}", e);
        // Could enter error state, reset, or retry
    }
}

// Alternative: Use expect() for prototype code
let temperature = temp_sensor.read_celsius()
    .expect("Temperature sensor failed");
```

## Exercise: Your First Temperature Monitor

**Time Budget: 30 minutes**

Build a basic temperature monitoring system with the ESP32-C3's built-in sensor.

### Requirements

1. **Hardware Setup**: ESP32-C3 development board connected via USB
2. **Temperature Reading**: Use built-in temperature sensor
3. **LED Status**: Visual feedback based on temperature
4. **Serial Output**: Temperature readings every 2 seconds
5. **Status Reporting**: Progress summary every 10 readings

### Starting Code

Create `src/main.rs` with this foundation:

```rust
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
    temperature_sensor::{TemperatureSensor, TempSensorConfig},
};

#[entry]
fn main() -> ! {
    // TODO: Initialize hardware

    // TODO: Set up temperature sensor

    // TODO: Main monitoring loop

    loop {
        // TODO: Read temperature

        // TODO: Control LED based on temperature

        // TODO: Print reading with status

        // TODO: Wait for next reading
    }
}
```

### Implementation Tasks

1. **Initialize Hardware** (5 minutes):
   ```rust
   let peripherals = Peripherals::take();
   let system = SystemControl::new(peripherals.SYSTEM);
   let clocks = ClockControl::max(system.clock_control).freeze();
   let delay = Delay::new(&clocks);

   let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
   let mut led = Output::new(io.pins.gpio8, Level::Low);
   ```

2. **Configure Temperature Sensor** (5 minutes):
   ```rust
   let temp_config = TempSensorConfig::default();
   let mut temp_sensor = TemperatureSensor::new(
       peripherals.TEMP_SENSOR,
       temp_config
   );
   ```

3. **Main Monitoring Loop** (15 minutes):
   - Read temperature with `temp_sensor.read_celsius()`
   - Control LED: fast blink if >25°C, slow if ≤25°C
   - Print "Reading #N: Temperature = X.X°C"
   - Status summary every 10 readings
   - 2-second intervals between readings

4. **Test on Hardware** (5 minutes):
   - Build and flash to ESP32-C3
   - Verify temperature readings and LED behavior
   - Try warming the chip with your finger

### Success Criteria

- [ ] Program compiles without warnings
- [ ] ESP32-C3 boots and shows startup message
- [ ] Temperature readings displayed every 2 seconds
- [ ] LED blinks with different patterns based on temperature
- [ ] Status summary appears every 10 readings
- [ ] Temperature values are reasonable (20-40°C typically)

### Expected Serial Output

```
ESP32-C3 Temperature Monitor
Built-in sensor initialized
Reading temperature every 2 seconds...

Reading #1: Temperature = 24.3°C
Reading #2: Temperature = 24.5°C
Reading #3: Temperature = 24.1°C
Reading #4: Temperature = 24.8°C
Reading #5: Temperature = 25.2°C  ← LED should blink faster now
...
Reading #10: Temperature = 24.7°C
Status: 10 readings completed

Reading #11: Temperature = 24.9°C
...
```

### Extension Challenges

1. **Temperature Threshold**: Make threshold adjustable via const
2. **LED Patterns**: Different patterns for different temperature ranges
3. **Statistics**: Track min/max temperatures
4. **Timing**: More precise 2-second intervals
5. **Error Handling**: Handle sensor reading failures gracefully

### Troubleshooting Tips

**Build Errors:**
- Ensure `rustup target add riscv32imc-unknown-none-elf` is installed
- Check that feature flags match your ESP32-C3 variant

**Flash Errors:**
- Install probe-rs with `cargo install probe-rs --features cli`
- Try alternative: `cargo install espflash`
- Check USB cable and connection

**No Serial Output:**
- Verify baud rate (115200)
- Try different serial monitor tools
- Check USB device enumeration

**Sensor Issues:**
- Temperature readings should be 20-40°C typically
- Values outside this range might indicate calibration issues
- Warm the chip gently with your finger to test responsiveness

## Key Takeaways

✅ **Hardware First**: Starting with real hardware creates immediate engagement and practical learning

✅ **Built-in Sensors**: ESP32-C3's temperature sensor eliminates external component complexity

✅ **Embedded Patterns**: `#[no_std]`, `#[no_main]`, and `loop` are fundamental embedded concepts

✅ **Real-time Constraints**: Understanding timing and resource limitations from the start

✅ **Type Safety**: Rust's ownership system prevents common embedded bugs even on bare metal

✅ **Immediate Feedback**: LED status and serial output provide instant verification of functionality

**Next**: In Chapter 14, we'll build proper data structures for storing and processing these temperature readings using embedded-friendly `no_std` patterns.