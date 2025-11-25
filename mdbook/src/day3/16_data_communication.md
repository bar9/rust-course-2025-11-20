# Chapter 16: Data & Communication

## Learning Objectives
By the end of this chapter, you'll be able to:
- Use Serde for serialization in no_std embedded environments
- Send structured temperature data as JSON over USB Serial
- Implement efficient binary protocols with postcard
- Create command/response interfaces for embedded systems
- Handle communication errors gracefully in resource-constrained environments
- Design protocols optimized for IoT and embedded applications

## Why Communication Matters in Embedded Systems

Your temperature sensor is great, but isolated data isn't very useful. Modern embedded systems need to:

**Share Data:**
- Send readings to dashboards, databases, or cloud services
- Integrate with IoT platforms and monitoring systems
- Enable remote monitoring and alerting

**Accept Commands:**
- Change sampling rates or thresholds remotely
- Trigger calibration or diagnostic procedures
- Update configuration without reflashing firmware

**Interoperability:**
- Work with different programming languages and platforms
- Support standard protocols and data formats
- Enable integration with existing systems

## Serde in no_std: Serialization for Embedded

Serde is Rust's premier serialization framework, and it works great in no_std environments:

```rust
// Cargo.toml dependencies
[dependencies]
serde = { version = "1.0", default-features = false, features = ["derive"] }
serde-json-core = "0.6"  # no_std JSON support
postcard = { version = "1.0", default-features = false }  # Binary serialization
heapless = "0.8"
```

### Making Temperature Data Serializable

Let's update our temperature types to support serialization:

```rust
// src/temperature.rs - Updated with serde support
#![cfg_attr(not(test), no_std)]

use serde::{Deserialize, Serialize};
use core::fmt;

#[cfg(test)]
use std::vec::Vec;
#[cfg(not(test))]
use heapless::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Temperature {
    celsius_tenths: i16,
}

impl Temperature {
    pub const fn from_celsius(celsius: f32) -> Self {
        Self {
            celsius_tenths: (celsius * 10.0) as i16,
        }
    }

    pub fn celsius(&self) -> f32 {
        self.celsius_tenths as f32 / 10.0
    }

    pub fn fahrenheit(&self) -> f32 {
        self.celsius() * 9.0 / 5.0 + 32.0
    }

    pub const fn is_overheating(&self) -> bool {
        self.celsius_tenths > 500  // > 50°C
    }

    // Helper for JSON serialization with nice format
    pub fn to_celsius_rounded(&self) -> f32 {
        (self.celsius() * 10.0).round() / 10.0
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TemperatureReading {
    pub temperature: Temperature,
    pub timestamp_ms: u32,
    pub sensor_id: u8,  // Compact sensor identifier
}

impl TemperatureReading {
    pub fn new(temperature: Temperature, timestamp_ms: u32, sensor_id: u8) -> Self {
        Self {
            temperature,
            timestamp_ms,
            sensor_id,
        }
    }

    pub fn current_time(temperature: Temperature) -> Self {
        // In real implementation, this would get actual timestamp
        // For now, use a simple counter
        static mut TIMESTAMP: u32 = 0;
        unsafe {
            TIMESTAMP += 1000; // Simulate 1-second intervals
            Self::new(temperature, TIMESTAMP, 0)
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TemperatureStats {
    pub count: u16,          // Use u16 to save space
    pub total_count: u32,
    pub min_celsius: f32,    // Store as f32 for JSON compatibility
    pub max_celsius: f32,
    pub avg_celsius: f32,
    pub timestamp_ms: u32,
}

impl TemperatureStats {
    pub fn from_buffer<const N: usize>(
        buffer: &TemperatureBuffer<N>,
        timestamp_ms: u32
    ) -> Option<Self> {
        if buffer.len() == 0 {
            return None;
        }

        let min = buffer.min()?.celsius();
        let max = buffer.max()?.celsius();
        let avg = buffer.average()?.celsius();

        Some(Self {
            count: buffer.len() as u16,
            total_count: buffer.total_readings(),
            min_celsius: min,
            max_celsius: max,
            avg_celsius: avg,
            timestamp_ms,
        })
    }
}
```

### JSON Serialization with serde-json-core

For IoT integration, JSON is widely supported but needs special handling in no_std:

```rust
// src/communication.rs - JSON communication module
#![cfg_attr(not(test), no_std)]

use heapless::{String, Vec};
use serde::{Deserialize, Serialize};
use serde_json_core;

use crate::temperature::{Temperature, TemperatureReading, TemperatureStats};

/// Commands that can be sent to the temperature monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    GetStatus,
    GetLatestReading,
    GetStats,
    SetSampleRate { rate_hz: u8 },
    SetThreshold { threshold_celsius: f32 },
    Reset,
}

/// Responses from the temperature monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Status {
        uptime_ms: u32,
        sample_rate_hz: u8,
        threshold_celsius: f32,
        buffer_usage: u8,  // Percentage full
    },
    Reading(TemperatureReading),
    Stats(TemperatureStats),
    SampleRateSet(u8),
    ThresholdSet(f32),
    ResetComplete,
    Error { code: u8, message: String<32> },
}

impl Response {
    pub fn error(code: u8, message: &str) -> Self {
        let mut error_message = String::new();
        error_message.push_str(message).ok();
        Self::Error {
            code,
            message: error_message,
        }
    }
}

/// Communication handler for temperature monitor
pub struct TemperatureComm {
    sample_rate_hz: u8,
    threshold_celsius: f32,
    start_time_ms: u32,
}

impl TemperatureComm {
    pub const fn new() -> Self {
        Self {
            sample_rate_hz: 1,  // 1 Hz default
            threshold_celsius: 35.0,
            start_time_ms: 0,
        }
    }

    pub fn init(&mut self, start_time_ms: u32) {
        self.start_time_ms = start_time_ms;
    }

    /// Process a command and return appropriate response
    pub fn process_command<const N: usize>(
        &mut self,
        command: Command,
        buffer: &TemperatureBuffer<N>,
        current_time_ms: u32
    ) -> Response {
        match command {
            Command::GetStatus => {
                let uptime = current_time_ms.saturating_sub(self.start_time_ms);
                let buffer_usage = if buffer.capacity() > 0 {
                    ((buffer.len() * 100) / buffer.capacity()) as u8
                } else {
                    0
                };

                Response::Status {
                    uptime_ms: uptime,
                    sample_rate_hz: self.sample_rate_hz,
                    threshold_celsius: self.threshold_celsius,
                    buffer_usage,
                }
            }

            Command::GetLatestReading => {
                if let Some(temp) = buffer.latest() {
                    let reading = TemperatureReading::new(temp, current_time_ms, 0);
                    Response::Reading(reading)
                } else {
                    Response::error(1, "No readings available")
                }
            }

            Command::GetStats => {
                if let Some(stats) = TemperatureStats::from_buffer(buffer, current_time_ms) {
                    Response::Stats(stats)
                } else {
                    Response::error(2, "No data for statistics")
                }
            }

            Command::SetSampleRate { rate_hz } => {
                if rate_hz > 0 && rate_hz <= 10 {
                    self.sample_rate_hz = rate_hz;
                    Response::SampleRateSet(rate_hz)
                } else {
                    Response::error(3, "Rate must be 1-10 Hz")
                }
            }

            Command::SetThreshold { threshold_celsius } => {
                if threshold_celsius > 0.0 && threshold_celsius < 100.0 {
                    self.threshold_celsius = threshold_celsius;
                    Response::ThresholdSet(threshold_celsius)
                } else {
                    Response::error(4, "Threshold must be 0-100°C")
                }
            }

            Command::Reset => {
                self.start_time_ms = current_time_ms;
                self.sample_rate_hz = 1;
                self.threshold_celsius = 35.0;
                Response::ResetComplete
            }
        }
    }

    /// Serialize response to JSON string for transmission
    pub fn response_to_json(&self, response: &Response) -> Result<String<512>, ()> {
        // Use heapless String with fixed capacity
        match serde_json_core::to_string::<_, 512>(response) {
            Ok(json) => Ok(json),
            Err(_) => Err(()),
        }
    }

    /// Deserialize command from JSON string
    pub fn json_to_command(&self, json: &str) -> Result<Command, ()> {
        match serde_json_core::from_str(json) {
            Ok(command) => Ok(command),
            Err(_) => Err(()),
        }
    }

    /// Create a status response as JSON
    pub fn status_json<const N: usize>(
        &self,
        buffer: &TemperatureBuffer<N>,
        current_time_ms: u32
    ) -> String<256> {
        let status = self.process_command(
            Command::GetStatus,
            buffer,
            current_time_ms
        );

        self.response_to_json(&status)
            .unwrap_or_else(|_| {
                let mut error = String::new();
                error.push_str("{\"error\":\"serialization_failed\"}").ok();
                error
            })
    }

    /// Create latest reading as JSON
    pub fn reading_json<const N: usize>(
        &self,
        buffer: &TemperatureBuffer<N>,
        current_time_ms: u32
    ) -> String<256> {
        let reading = self.process_command(
            Command::GetLatestReading,
            buffer,
            current_time_ms
        );

        self.response_to_json(&reading)
            .unwrap_or_else(|_| {
                let mut error = String::new();
                error.push_str("{\"error\":\"no_reading\"}").ok();
                error
            })
    }

    pub fn sample_rate(&self) -> u8 {
        self.sample_rate_hz
    }

    pub fn threshold(&self) -> f32 {
        self.threshold_celsius
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temperature::TemperatureBuffer;

    #[test]
    fn test_json_serialization() {
        let temp = Temperature::from_celsius(23.5);
        let reading = TemperatureReading::new(temp, 1000, 0);

        // Test command serialization
        let command = Command::GetStatus;
        let json = serde_json_core::to_string::<_, 64>(&command).unwrap();
        assert_eq!(json, "\"GetStatus\"");

        // Test response serialization
        let response = Response::Reading(reading);
        let json = serde_json_core::to_string::<_, 256>(&response).unwrap();
        assert!(json.contains("Reading"));
        assert!(json.contains("23.5"));
    }

    #[test]
    fn test_command_processing() {
        let mut comm = TemperatureComm::new();
        comm.init(0);
        let buffer = TemperatureBuffer::<5>::new();

        // Test status command
        let status_resp = comm.process_command(Command::GetStatus, &buffer, 5000);
        if let Response::Status { uptime_ms, .. } = status_resp {
            assert_eq!(uptime_ms, 5000);
        } else {
            panic!("Expected status response");
        }

        // Test rate setting
        let rate_resp = comm.process_command(
            Command::SetSampleRate { rate_hz: 5 },
            &buffer,
            5000
        );
        assert!(matches!(rate_resp, Response::SampleRateSet(5)));
        assert_eq!(comm.sample_rate(), 5);
    }

    #[test]
    fn test_json_roundtrip() {
        let mut comm = TemperatureComm::new();

        // Test command deserialization
        let json_cmd = "\"GetStatus\"";
        let command = comm.json_to_command(json_cmd).unwrap();
        assert!(matches!(command, Command::GetStatus));

        // Test response serialization
        let response = Response::ResetComplete;
        let json_resp = comm.response_to_json(&response).unwrap();
        assert_eq!(json_resp, "\"ResetComplete\"");
    }

    #[test]
    fn test_error_handling() {
        let mut comm = TemperatureComm::new();
        let buffer = TemperatureBuffer::<5>::new();

        // Test invalid sample rate
        let response = comm.process_command(
            Command::SetSampleRate { rate_hz: 20 },  // Invalid: too high
            &buffer,
            0
        );

        if let Response::Error { code, message } = response {
            assert_eq!(code, 3);
            assert!(message.contains("Rate must be"));
        } else {
            panic!("Expected error response");
        }
    }
}
```

### Binary Serialization with postcard

For bandwidth-constrained applications, binary serialization is more efficient:

```rust
// src/binary_comm.rs - Binary communication with postcard
#![cfg_attr(not(test), no_std)]

use heapless::Vec;
use serde::{Deserialize, Serialize};
use postcard;

use crate::communication::{Command, Response};

/// Binary communication handler
pub struct BinaryComm;

impl BinaryComm {
    /// Serialize command to binary format
    pub fn command_to_binary(command: &Command) -> Result<Vec<u8, 64>, postcard::Error> {
        postcard::to_vec(command)
    }

    /// Deserialize command from binary format
    pub fn binary_to_command(data: &[u8]) -> Result<Command, postcard::Error> {
        postcard::from_bytes(data)
    }

    /// Serialize response to binary format
    pub fn response_to_binary(response: &Response) -> Result<Vec<u8, 256>, postcard::Error> {
        postcard::to_vec(response)
    }

    /// Deserialize response from binary format
    pub fn binary_to_response(data: &[u8]) -> Result<Response, postcard::Error> {
        postcard::from_bytes(data)
    }

    /// Get size of serialized command
    pub fn command_size(command: &Command) -> usize {
        Self::command_to_binary(command)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Get size of serialized response
    pub fn response_size(response: &Response) -> usize {
        Self::response_to_binary(response)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temperature::{Temperature, TemperatureReading};

    #[test]
    fn test_binary_command_serialization() {
        let command = Command::SetSampleRate { rate_hz: 5 };

        // Serialize to binary
        let binary = BinaryComm::command_to_binary(&command).unwrap();

        // Deserialize back
        let deserialized = BinaryComm::binary_to_command(&binary).unwrap();

        if let Command::SetSampleRate { rate_hz } = deserialized {
            assert_eq!(rate_hz, 5);
        } else {
            panic!("Deserialization failed");
        }
    }

    #[test]
    fn test_binary_response_serialization() {
        let temp = Temperature::from_celsius(25.0);
        let reading = TemperatureReading::new(temp, 1000, 0);
        let response = Response::Reading(reading);

        // Serialize to binary
        let binary = BinaryComm::response_to_binary(&response).unwrap();

        // Should be much smaller than JSON
        println!("Binary size: {} bytes", binary.len());
        assert!(binary.len() < 20); // Much smaller than JSON

        // Deserialize back
        let deserialized = BinaryComm::binary_to_response(&binary).unwrap();

        if let Response::Reading(r) = deserialized {
            assert!((r.temperature.celsius() - 25.0).abs() < 0.1);
            assert_eq!(r.timestamp_ms, 1000);
        } else {
            panic!("Deserialization failed");
        }
    }

    #[test]
    fn test_size_comparison() {
        let temp = Temperature::from_celsius(23.5);
        let reading = TemperatureReading::new(temp, 1000, 0);
        let response = Response::Reading(reading);

        // Binary size
        let binary_size = BinaryComm::response_size(&response);

        // JSON size (approximate)
        let json = serde_json_core::to_string::<_, 256>(&response).unwrap();
        let json_size = json.len();

        println!("Binary: {} bytes, JSON: {} bytes", binary_size, json_size);
        println!("Binary is {}% smaller", ((json_size - binary_size) * 100) / json_size);

        assert!(binary_size < json_size);
        assert!(binary_size < 16); // Binary should be very compact
    }
}
```

## Integrating Communication with ESP32-C3

Let's update our main application to use these communication capabilities:

```rust
// src/main.rs - ESP32 temperature monitor with communication
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

mod temperature;
mod communication;

use temperature::{Temperature, TemperatureBuffer};
use communication::{Command, Response, TemperatureComm};

const BUFFER_SIZE: usize = 20;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = Output::new(io.pins.gpio8, Level::Low);

    let temp_sensor_config = TempSensorConfig::default();
    let mut temp_sensor = TemperatureSensor::new(
        peripherals.TEMP_SENSOR,
        temp_sensor_config
    );

    // Initialize data structures
    let mut temp_buffer = TemperatureBuffer::<BUFFER_SIZE>::new();
    let mut comm = TemperatureComm::new();
    comm.init(0);  // Initialize at time 0

    esp_println::println!("🌡️ ESP32-C3 Temperature Monitor with Communication");
    esp_println::println!("📊 Buffer capacity: {} readings", temp_buffer.capacity());
    esp_println::println!("📡 JSON communication enabled");
    esp_println::println!("🔧 Send commands: status, reading, stats, reset");
    esp_println::println!();

    // Demonstrate initial JSON output
    let status_json = comm.status_json(&temp_buffer, 0);
    esp_println::println!("INITIAL_STATUS: {}", status_json);
    esp_println::println!();

    let mut reading_count = 0u32;
    let mut last_stats_output = 0u32;

    loop {
        // Get current timestamp (simplified)
        let current_time = reading_count * 1000; // 1-second intervals

        // Read temperature
        let celsius = temp_sensor.read_celsius();
        let temperature = Temperature::from_celsius(celsius);

        // Store reading
        temp_buffer.push(temperature);
        reading_count += 1;

        // LED status based on temperature
        if temperature.is_overheating() {
            // Rapid blink for overheating
            for _ in 0..3 {
                led.set_high();
                delay.delay_millis(100);
                led.set_low();
                delay.delay_millis(100);
            }
        } else {
            // Single blink for normal
            led.set_high();
            delay.delay_millis(200);
            led.set_low();
        }

        // Output current reading as JSON
        let reading_json = comm.reading_json(&temp_buffer, current_time);
        esp_println::println!("READING: {}", reading_json);

        // Output statistics every 5 readings
        if reading_count % 5 == 0 {
            let stats_resp = comm.process_command(
                Command::GetStats,
                &temp_buffer,
                current_time
            );

            if let Ok(stats_json) = comm.response_to_json(&stats_resp) {
                esp_println::println!("STATS: {}", stats_json);
            }

            let status_json = comm.status_json(&temp_buffer, current_time);
            esp_println::println!("STATUS: {}", status_json);
            esp_println::println!();
        }

        // Simulate command processing (in real application, this would read from UART/WiFi)
        if reading_count % 10 == 0 {
            demonstrate_command_processing(&mut comm, &temp_buffer, current_time);
        }

        delay.delay_millis(1000); // 1 Hz sampling
    }
}

fn demonstrate_command_processing(
    comm: &mut TemperatureComm,
    buffer: &TemperatureBuffer<BUFFER_SIZE>,
    current_time: u32
) {
    esp_println::println!("--- Command Processing Demo ---");

    // Simulate received commands
    let commands = [
        Command::GetStatus,
        Command::SetSampleRate { rate_hz: 2 },
        Command::SetThreshold { threshold_celsius: 40.0 },
    ];

    for command in commands {
        esp_println::println!("Processing command: {:?}", command);

        let response = comm.process_command(command, buffer, current_time);

        if let Ok(json) = comm.response_to_json(&response) {
            esp_println::println!("Response: {}", json);
        } else {
            esp_println::println!("Failed to serialize response");
        }
    }

    esp_println::println!("--- End Demo ---");
    esp_println::println!();
}
```

## Communication Protocols for IoT

### Message Framing for Serial Communication

When sending data over serial, you need message boundaries:

```rust
// Simple line-based protocol
pub struct SerialProtocol;

impl SerialProtocol {
    /// Frame a JSON message with newline delimiter
    pub fn frame_json(json: &str) -> heapless::String<600> {
        let mut framed = heapless::String::new();
        framed.push_str(json).ok();
        framed.push('\n').ok();
        framed
    }

    /// Frame binary data with length prefix
    pub fn frame_binary(data: &[u8]) -> heapless::Vec<u8, 300> {
        let mut framed = heapless::Vec::new();

        // Add 2-byte length prefix (little-endian)
        let len = data.len() as u16;
        framed.push((len & 0xFF) as u8).ok();
        framed.push(((len >> 8) & 0xFF) as u8).ok();

        // Add data
        framed.extend_from_slice(data).ok();

        framed
    }
}
```

### WiFi and HTTP Integration

For IoT applications, you might send data over WiFi:

```rust
// Future enhancement: HTTP client for ESP32-C3
// This would integrate with esp-wifi crate

pub struct HttpClient;

impl HttpClient {
    pub fn post_temperature_json(_url: &str, _json: &str) -> Result<(), ()> {
        // Implementation would use esp-wifi to send HTTP POST
        // with JSON payload to a server endpoint
        Ok(())
    }
}
```

## Exercise: Add Communication to Temperature Monitor

**Time Budget: 25 minutes**

Add JSON communication capabilities to your temperature monitoring system.

### Requirements

1. **Serde Integration**: Make temperature types serializable
2. **Command Processing**: Handle commands to get status, readings, stats
3. **JSON Output**: Send structured data over USB Serial
4. **Error Handling**: Graceful handling of serialization and command errors
5. **Real-time Output**: Stream temperature data in parseable format

### Tasks

1. **Add Serde Dependencies** (5 minutes):
   ```toml
   # Add to Cargo.toml
   [dependencies]
   serde = { version = "1.0", default-features = false, features = ["derive"] }
   serde-json-core = "0.6"
   heapless = "0.8"
   ```

2. **Update Temperature Types** (8 minutes):
   ```rust
   // Add Serialize, Deserialize to Temperature
   #[derive(Serialize, Deserialize, ...)]
   pub struct Temperature { ... }

   // Create TemperatureReading struct
   #[derive(Serialize, Deserialize)]
   pub struct TemperatureReading {
       pub temperature: Temperature,
       pub timestamp_ms: u32,
   }
   ```

3. **Implement Command System** (8 minutes):
   ```rust
   #[derive(Serialize, Deserialize)]
   pub enum Command {
       GetStatus,
       GetReading,
       GetStats,
   }

   #[derive(Serialize, Deserialize)]
   pub enum Response {
       Status { /* fields */ },
       Reading(TemperatureReading),
       Stats { /* fields */ },
       Error(String),
   }
   ```

4. **Integration with Main Loop** (4 minutes):
   - Output readings as JSON every cycle
   - Output stats every 5 readings
   - Demonstrate command processing

### Expected Output

```
🌡️ ESP32-C3 Temperature Monitor with Communication
📊 Buffer capacity: 20 readings
📡 JSON communication enabled

INITIAL_STATUS: {"Status":{"uptime_ms":0,"sample_rate_hz":1,"buffer_usage":0}}

READING: {"Reading":{"temperature":{"celsius_tenths":245},"timestamp_ms":1000}}
READING: {"Reading":{"temperature":{"celsius_tenths":243},"timestamp_ms":2000}}
READING: {"Reading":{"temperature":{"celsius_tenths":247},"timestamp_ms":3000}}
READING: {"Reading":{"temperature":{"celsius_tenths":241},"timestamp_ms":4000}}
READING: {"Reading":{"temperature":{"celsius_tenths":249},"timestamp_ms":5000}}

STATS: {"Stats":{"count":5,"min_celsius":24.1,"max_celsius":24.9,"avg_celsius":24.5}}
STATUS: {"Status":{"uptime_ms":5000,"sample_rate_hz":1,"buffer_usage":25}}

--- Command Processing Demo ---
Processing command: GetStatus
Response: {"Status":{"uptime_ms":10000,"sample_rate_hz":1,"buffer_usage":50}}
Processing command: SetSampleRate { rate_hz: 2 }
Response: {"SampleRateSet":2}
--- End Demo ---
```

### Success Criteria

- [ ] Temperature data serializes correctly to JSON
- [ ] Commands process and return appropriate responses
- [ ] JSON output is valid and parseable
- [ ] Error conditions are handled gracefully
- [ ] Serial output can be parsed by external tools
- [ ] Memory usage remains bounded (no heap allocation)

### Extension Challenges

1. **Binary Protocol**: Add postcard binary serialization
2. **Command Input**: Parse commands from serial input
3. **Data Logging**: Store readings with timestamps
4. **Configuration**: Persistent settings across resets
5. **HTTP Client**: Send data to web server (with esp-wifi)

### Testing Communication

You can test the JSON output with external tools:

```python
# test_serial.py - Parse ESP32 JSON output
import serial
import json
import time

ser = serial.Serial('/dev/cu.usbmodem*', 115200)

while True:
    line = ser.readline().decode('utf-8').strip()

    if line.startswith('READING:'):
        json_data = line[8:]  # Remove "READING:" prefix
        try:
            reading = json.loads(json_data)
            temp_c = reading['Reading']['temperature']['celsius_tenths'] / 10.0
            timestamp = reading['Reading']['timestamp_ms']
            print(f"Temperature: {temp_c}°C at {timestamp}ms")
        except json.JSONDecodeError:
            print(f"Invalid JSON: {json_data}")

    elif line.startswith('STATS:'):
        json_data = line[6:]  # Remove "STATS:" prefix
        try:
            stats = json.loads(json_data)['Stats']
            print(f"Stats: {stats['count']} readings, "
                  f"avg {stats['avg_celsius']:.1f}°C, "
                  f"range {stats['min_celsius']:.1f}-{stats['max_celsius']:.1f}°C")
        except json.JSONDecodeError:
            print(f"Invalid JSON: {json_data}")
```

## Key Takeaways

✅ **Serde in no_std**: Powerful serialization without heap allocation using serde-json-core

✅ **Structured Communication**: JSON for interoperability, binary for efficiency

✅ **Command/Response Pattern**: Essential for interactive embedded systems

✅ **Error Handling**: Graceful degradation when serialization or commands fail

✅ **Real-time Streaming**: Continuous data output for monitoring and integration

✅ **IoT Ready**: Foundation for WiFi, HTTP, and cloud integration

**Next**: In Chapter 17, we'll add async capabilities using Embassy to handle multiple tasks concurrently while maintaining real-time performance.