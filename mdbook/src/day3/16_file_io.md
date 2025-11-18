# Chapter 16: Serialization & Protocols

## Learning Objectives
- Master serialization with serde for JSON, TOML, and binary formats
- Design robust protocols for embedded and networked systems
- Understand format trade-offs: human-readable vs. binary efficiency
- Build command/response protocols with proper error handling
- Compare Rust serialization approaches with other languages
- Implement protocol versioning and backward compatibility

## Why Serialization and Protocols Matter

In embedded systems and networked applications, data must be:
- **Transmitted** between devices over serial, I2C, or network connections
- **Stored** in flash memory or external storage with minimal space
- **Debugged** with human-readable formats during development
- **Versioned** to handle firmware updates and compatibility

Rust's serde ecosystem provides powerful, zero-cost abstractions for all these needs.

## Serialization Fundamentals with Serde

### Basic Serialization with JSON

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TemperatureReading {
    sensor_id: String,
    temperature_celsius: f32,
    timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<String>,
}

fn json_serialization_example() -> Result<(), Box<dyn std::error::Error>> {
    let reading = TemperatureReading {
        sensor_id: "temp_01".to_string(),
        temperature_celsius: 23.5,
        timestamp: 1672531200, // Unix timestamp
        location: Some("living_room".to_string()),
    };

    // Serialize to JSON string
    let json = serde_json::to_string(&reading)?;
    println!("JSON: {}", json);
    // Output: {"sensor_id":"temp_01","temperature_celsius":23.5,"timestamp":1672531200,"location":"living_room"}

    // Pretty-print for debugging
    let pretty_json = serde_json::to_string_pretty(&reading)?;
    println!("Pretty JSON:\n{}", pretty_json);

    // Deserialize back
    let parsed: TemperatureReading = serde_json::from_str(&json)?;
    println!("Parsed: {:?}", parsed);

    Ok(())
}
```

### Configuration with TOML

```rust
#[derive(Serialize, Deserialize, Debug)]
struct SensorConfig {
    name: String,
    enabled: bool,
    sample_rate_ms: u64,
    thresholds: TemperatureThresholds,
    #[serde(default)]
    calibration: Option<CalibrationData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TemperatureThresholds {
    min_warning: f32,
    max_warning: f32,
    critical_shutdown: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct CalibrationData {
    offset: f32,
    scale: f32,
}

fn toml_configuration_example() -> Result<(), Box<dyn std::error::Error>> {
    let config = SensorConfig {
        name: "Main Temperature Sensor".to_string(),
        enabled: true,
        sample_rate_ms: 1000,
        thresholds: TemperatureThresholds {
            min_warning: 5.0,
            max_warning: 35.0,
            critical_shutdown: 50.0,
        },
        calibration: Some(CalibrationData {
            offset: -1.2,
            scale: 1.05,
        }),
    };

    // Serialize to TOML
    let toml_string = toml::to_string_pretty(&config)?;
    println!("TOML Configuration:\n{}", toml_string);

    // Write to file
    std::fs::write("sensor_config.toml", &toml_string)?;

    // Read from file
    let file_content = std::fs::read_to_string("sensor_config.toml")?;
    let loaded_config: SensorConfig = toml::from_str(&file_content)?;
    println!("Loaded config: {:#?}", loaded_config);

    Ok(())
}
```

### Binary Serialization with Postcard

For embedded systems, binary formats provide significant space and performance advantages:

```rust
use postcard;

fn binary_serialization_example() -> Result<(), Box<dyn std::error::Error>> {
    let readings = vec![
        TemperatureReading {
            sensor_id: "temp_01".to_string(),
            temperature_celsius: 23.5,
            timestamp: 1672531200,
            location: Some("living_room".to_string()),
        },
        TemperatureReading {
            sensor_id: "temp_02".to_string(),
            temperature_celsius: 21.8,
            timestamp: 1672531260,
            location: None,
        },
    ];

    // JSON serialization for comparison
    let json_data = serde_json::to_string(&readings)?;
    let json_size = json_data.len();

    // Binary serialization with postcard
    let binary_data = postcard::to_allocvec(&readings)?;
    let binary_size = binary_data.len();

    println!("JSON size: {} bytes", json_size);
    println!("Binary size: {} bytes", binary_size);
    println!("Space savings: {:.1}%",
             (json_size - binary_size) as f32 / json_size as f32 * 100.0);

    // Deserialize binary data
    let parsed_readings: Vec<TemperatureReading> = postcard::from_bytes(&binary_data)?;
    println!("Parsed {} readings from binary", parsed_readings.len());

    // Write binary data to file
    std::fs::write("readings.bin", &binary_data)?;

    Ok(())
}
```

**Format Comparison:**

| Format | Pros | Cons | Use Case |
|--------|------|------|----------|
| **JSON** | Human-readable, widely supported, debugging-friendly | Large size, parsing overhead | Development, APIs, configuration |
| **TOML** | Human-readable, great for config, comments supported | Config files only | Configuration files, settings |
| **Binary (postcard)** | Minimal size, fast parsing, no_std support | Not human-readable, debugging harder | Embedded storage, high-frequency data |

## Protocol Design Patterns

### Command-Response Protocol

Embedded systems often use command-response protocols for communication:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Command {
    GetStatus,
    GetReading { sensor_id: String },
    SetThreshold {
        sensor_id: String,
        min_temp: f32,
        max_temp: f32
    },
    GetHistory {
        sensor_id: String,
        last_n: usize
    },
    Calibrate {
        sensor_id: String,
        actual_temp: f32
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Response {
    Status {
        active_sensors: Vec<String>,
        uptime_seconds: u64,
        memory_usage: u32,
    },
    Reading {
        sensor_id: String,
        temperature: f32,
        timestamp: u64,
    },
    ThresholdSet {
        sensor_id: String
    },
    History {
        sensor_id: String,
        readings: Vec<TemperatureReading>,
    },
    CalibrationComplete {
        sensor_id: String,
        offset_adjustment: f32,
    },
    Error {
        code: u16,
        message: String,
    },
}

// Protocol wrapper with versioning
#[derive(Serialize, Deserialize, Debug)]
pub struct ProtocolMessage {
    pub version: u8,
    pub id: u32,  // For matching requests to responses
    pub payload: MessagePayload,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum MessagePayload {
    Command(Command),
    Response(Response),
}
```

### Protocol Implementation

```rust
use std::collections::HashMap;

pub struct ProtocolHandler {
    next_message_id: u32,
    pending_requests: HashMap<u32, std::time::Instant>,
}

impl ProtocolHandler {
    pub fn new() -> Self {
        Self {
            next_message_id: 1,
            pending_requests: HashMap::new(),
        }
    }

    pub fn create_command(&mut self, command: Command) -> ProtocolMessage {
        let id = self.next_message_id;
        self.next_message_id += 1;
        self.pending_requests.insert(id, std::time::Instant::now());

        ProtocolMessage {
            version: 1,
            id,
            payload: MessagePayload::Command(command),
        }
    }

    pub fn create_response(&self, request_id: u32, response: Response) -> ProtocolMessage {
        ProtocolMessage {
            version: 1,
            id: request_id,
            payload: MessagePayload::Response(response),
        }
    }

    pub fn serialize_json(&self, message: &ProtocolMessage) -> Result<String, serde_json::Error> {
        serde_json::to_string(message)
    }

    pub fn serialize_binary(&self, message: &ProtocolMessage) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(message)
    }

    pub fn deserialize_json(&self, data: &str) -> Result<ProtocolMessage, serde_json::Error> {
        serde_json::from_str(data)
    }

    pub fn deserialize_binary(&self, data: &[u8]) -> Result<ProtocolMessage, postcard::Error> {
        postcard::from_bytes(data)
    }

    pub fn cleanup_expired_requests(&mut self, timeout_duration: std::time::Duration) {
        let now = std::time::Instant::now();
        self.pending_requests.retain(|_, timestamp| {
            now.duration_since(*timestamp) < timeout_duration
        });
    }
}

// Usage example
fn protocol_usage_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut handler = ProtocolHandler::new();

    // Create a command
    let command = Command::GetReading {
        sensor_id: "temp_01".to_string(),
    };
    let request_msg = handler.create_command(command);

    // Serialize for transmission
    let json_data = handler.serialize_json(&request_msg)?;
    let binary_data = handler.serialize_binary(&request_msg)?;

    println!("JSON message: {}", json_data);
    println!("Binary message size: {} bytes", binary_data.len());

    // Deserialize received message
    let received_msg = handler.deserialize_json(&json_data)?;
    println!("Received message ID: {}", received_msg.id);

    // Create response
    let response = Response::Reading {
        sensor_id: "temp_01".to_string(),
        temperature: 23.5,
        timestamp: 1672531200,
    };
    let response_msg = handler.create_response(received_msg.id, response);

    println!("Response: {:?}", response_msg);

    Ok(())
}
```

## Error Handling in Protocols

Robust protocols need comprehensive error handling:

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ProtocolError {
    InvalidSensorId { sensor_id: String },
    SensorNotResponding { sensor_id: String },
    InvalidThreshold { min: f32, max: f32, reason: String },
    CalibrationFailed { sensor_id: String, reason: String },
    SystemError { code: u16, details: String },
    ProtocolVersionMismatch { expected: u8, received: u8 },
}

impl ProtocolError {
    pub fn to_response(&self) -> Response {
        match self {
            ProtocolError::InvalidSensorId { sensor_id } => Response::Error {
                code: 404,
                message: format!("Sensor '{}' not found", sensor_id),
            },
            ProtocolError::SensorNotResponding { sensor_id } => Response::Error {
                code: 503,
                message: format!("Sensor '{}' is not responding", sensor_id),
            },
            ProtocolError::InvalidThreshold { min, max, reason } => Response::Error {
                code: 400,
                message: format!("Invalid threshold min={}, max={}: {}", min, max, reason),
            },
            ProtocolError::CalibrationFailed { sensor_id, reason } => Response::Error {
                code: 422,
                message: format!("Calibration failed for '{}': {}", sensor_id, reason),
            },
            ProtocolError::SystemError { code, details } => Response::Error {
                code: *code,
                message: details.clone(),
            },
            ProtocolError::ProtocolVersionMismatch { expected, received } => Response::Error {
                code: 505,
                message: format!("Protocol version mismatch: expected {}, got {}", expected, received),
            },
        }
    }
}
```

## Rust vs Other Languages: Serialization

| Aspect | **Rust (serde)** | **C++** | **C#** |
|--------|-------------------|---------|--------|
| **Type Safety** | Compile-time guaranteed | Manual/libraries | Runtime with attributes |
| **Performance** | Zero-cost abstractions | Variable (manual/lib) | Reflection overhead |
| **Memory Safety** | Automatic bounds checking | Manual management | Garbage collected |
| **Binary Formats** | postcard, bincode, many others | Manual/protobuf | BinaryFormatter/protobuf |
| **Derive Macros** | `#[derive(Serialize)]` | Not built-in | `[Serializable]` attribute |
| **Custom Serialization** | `#[serde(with = "...")]` | Manual implementation | ISerializable interface |
| **Schema Evolution** | `#[serde(default)]`, versioning | Manual handling | Version tolerant |

**Rust Advantages:**
- Compile-time serialization code generation
- Zero-cost abstractions - no runtime overhead
- Memory safety prevents buffer overflows
- Rich ecosystem of format support

## File I/O Integration

Serialized data needs to be stored and retrieved efficiently:

```rust
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub struct DataStore {
    data_directory: std::path::PathBuf,
}

impl DataStore {
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Result<Self, std::io::Error> {
        let data_directory = data_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&data_directory)?;
        Ok(Self { data_directory })
    }

    pub fn save_readings_json(&self, readings: &[TemperatureReading]) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.data_directory.join("readings.json");
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, readings)?;
        Ok(())
    }

    pub fn load_readings_json(&self) -> Result<Vec<TemperatureReading>, Box<dyn std::error::Error>> {
        let file_path = self.data_directory.join("readings.json");
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let readings = serde_json::from_reader(reader)?;
        Ok(readings)
    }

    pub fn save_readings_binary(&self, readings: &[TemperatureReading]) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.data_directory.join("readings.bin");
        let binary_data = postcard::to_allocvec(readings)?;
        std::fs::write(file_path, binary_data)?;
        Ok(())
    }

    pub fn load_readings_binary(&self) -> Result<Vec<TemperatureReading>, Box<dyn std::error::Error>> {
        let file_path = self.data_directory.join("readings.bin");
        let binary_data = std::fs::read(file_path)?;
        let readings = postcard::from_bytes(&binary_data)?;
        Ok(readings)
    }

    pub fn save_config(&self, config: &SensorConfig) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.data_directory.join("config.toml");
        let toml_string = toml::to_string_pretty(config)?;
        std::fs::write(file_path, toml_string)?;
        Ok(())
    }

    pub fn load_config(&self) -> Result<SensorConfig, Box<dyn std::error::Error>> {
        let file_path = self.data_directory.join("config.toml");
        let toml_content = std::fs::read_to_string(file_path)?;
        let config = toml::from_str(&toml_content)?;
        Ok(config)
    }
}
```

## Exercise: Temperature Monitoring Protocol

Build a comprehensive temperature monitoring protocol that supports both JSON and binary formats:

### Requirements

1. **Protocol Definition**: Create a command/response protocol that supports:
   - Getting current sensor readings
   - Setting temperature thresholds
   - Retrieving historical data
   - System status queries

2. **Dual Format Support**: Support both JSON (for debugging) and binary (for production)

3. **Integration**: Use types from previous capstone increments (`temp_core`, `temp_store`)

4. **Error Handling**: Comprehensive error types and proper error responses

5. **Protocol Versioning**: Support for protocol version handling

### Starting Code

```rust
// In temp_protocol/src/lib.rs
use serde::{Deserialize, Serialize};
use temp_core::{Temperature, TemperatureReading};
use temp_store::TemperatureStats;

// TODO: Define your Command enum here

// TODO: Define your Response enum here

// TODO: Define your ProtocolMessage wrapper

// TODO: Implement ProtocolHandler

pub struct TemperatureProtocolHandler {
    // TODO: Add fields needed for protocol handling
}

impl TemperatureProtocolHandler {
    pub fn new() -> Self {
        // TODO: Initialize handler
        unimplemented!()
    }

    pub fn process_command(&mut self, command: Command) -> Response {
        // TODO: Process commands and return appropriate responses
        // This should integrate with your temperature monitoring system
        unimplemented!()
    }

    pub fn serialize_json(&self, message: &ProtocolMessage) -> Result<String, serde_json::Error> {
        // TODO: Serialize message to JSON
        unimplemented!()
    }

    pub fn serialize_binary(&self, message: &ProtocolMessage) -> Result<Vec<u8>, postcard::Error> {
        // TODO: Serialize message to binary format
        unimplemented!()
    }

    pub fn deserialize_json(&self, data: &str) -> Result<ProtocolMessage, serde_json::Error> {
        // TODO: Deserialize JSON message
        unimplemented!()
    }

    pub fn deserialize_binary(&self, data: &[u8]) -> Result<ProtocolMessage, postcard::Error> {
        // TODO: Deserialize binary message
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_serialization() {
        // TODO: Test that commands can be serialized to JSON and back
    }

    #[test]
    fn test_binary_vs_json_size() {
        // TODO: Compare binary and JSON serialization sizes
    }

    #[test]
    fn test_protocol_versioning() {
        // TODO: Test protocol version handling
    }

    #[test]
    fn test_error_responses() {
        // TODO: Test that errors are properly converted to error responses
    }

    #[test]
    fn test_command_processing() {
        // TODO: Test that commands produce appropriate responses
    }
}
```

### Success Criteria

- All tests pass
- Binary format is significantly smaller than JSON for typical messages
- Protocol handler can process all defined commands
- Error cases are handled gracefully
- Integration with `temp_core` and `temp_store` types works correctly

### Extension Ideas

1. **Message Compression**: Add optional compression for large data transfers
2. **Authentication**: Add basic authentication to the protocol
3. **Streaming**: Support for streaming large datasets
4. **Rate Limiting**: Add rate limiting for command processing

## Key Takeaways

1. **Format Choice Matters**: JSON for debugging, binary for production, TOML for configuration
2. **Zero-Cost Serialization**: Rust's serde provides compile-time code generation with no runtime overhead
3. **Type Safety**: Serialization is compile-time checked, preventing runtime errors
4. **Protocol Design**: Structure messages with versioning and proper error handling from the start
5. **Integration Strategy**: Build protocols that work with existing type systems
6. **Performance Awareness**: Binary formats can provide 50-80% space savings over JSON

**Next**: In Chapter 17, we'll explore no_std programming to prepare our temperature monitoring system for embedded deployment.