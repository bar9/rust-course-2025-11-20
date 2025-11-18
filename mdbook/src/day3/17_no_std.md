# Chapter 17: no_std & Embedded Patterns

## Learning Objectives
- Understand the difference between `core`, `alloc`, and `std` libraries
- Convert existing Rust code to work in `no_std` environments
- Use heapless data structures for memory-constrained environments
- Master const functions for compile-time computation and configuration
- Apply embedded programming patterns and best practices
- Handle resource constraints and real-time requirements

## Why no_std Matters in Embedded Systems

Embedded systems often operate under strict constraints:
- **Limited Memory**: Kilobytes, not gigabytes of RAM
- **No Operating System**: Direct hardware control without OS services
- **Real-time Requirements**: Deterministic timing and response
- **Power Constraints**: Battery-operated devices need efficiency
- **Code Size Limits**: Flash memory is precious

Rust's `no_std` approach provides zero-cost abstractions without runtime overhead, making it ideal for these environments.

## Core vs Alloc vs Std: Understanding Rust's Library Layers

Rust's standard library is composed of three layers:

```rust
#![no_std]
// Using only core library - no heap allocation, no OS dependencies

// Core is always available and provides:
use core::{
    mem, ptr, slice, str,
    option::Option,
    result::Result,
    fmt::{Debug, Display},
    iter::Iterator,
    clone::Clone,
    marker::{Copy, Send, Sync},
};

// Example of core-only function
fn find_max_core_only(slice: &[i32]) -> Option<i32> {
    if slice.is_empty() {
        return None;
    }

    let mut max = slice[0];
    for &item in slice.iter().skip(1) {
        if item > max {
            max = item;
        }
    }
    Some(max)
}

// Working with core types
fn core_types_example() {
    // Basic types work the same
    let x: i32 = 42;
    let y: Option<i32> = Some(x);
    let z: Result<i32, &str> = Ok(x);

    // Iterators work (but no collect() without alloc)
    let data = [1, 2, 3, 4, 5];
    let sum: i32 = data.iter().sum();

    // String slices work, but no String type
    let text: &str = "Hello, embedded world!";
    let first_char = text.chars().next();

    // Arrays work, but no Vec without alloc
    let mut buffer = [0u8; 64];
    buffer[0] = 42;
}
```

**Library Comparison:**

| Layer | Features | Use Case |
|-------|----------|----------|
| **core** | Basic types, iterators, traits | Minimal embedded, bootloaders |
| **alloc** | Heap allocation (Vec, String, Box) | Embedded with heap allocator |
| **std** | OS services, networking, threading | Desktop applications |

**Rust vs Other Languages:**

| Aspect | **Rust (no_std)** | **C** | **C++** |
|--------|-------------------|--------|---------|
| **Memory Safety** | Compile-time guaranteed | Manual management | RAII + manual |
| **Zero-cost Abstractions** | Built-in | Manual optimization | Template metaprogramming |
| **Standard Library** | Explicit layers (core/alloc/std) | Platform-specific libraries | STL often avoided in embedded |
| **Error Handling** | Result<T, E> with no overhead | Error codes | Exceptions (often disabled) |

## Converting to no_std: A Practical Example

Let's convert a temperature monitoring function from std to no_std:

```rust
// STD VERSION
use std::collections::HashMap;
use std::vec::Vec;

struct TemperatureMonitor {
    sensors: HashMap<String, f32>,
    history: Vec<f32>,
}

impl TemperatureMonitor {
    fn new() -> Self {
        Self {
            sensors: HashMap::new(),
            history: Vec::new(),
        }
    }

    fn add_reading(&mut self, sensor_id: String, temp: f32) {
        self.sensors.insert(sensor_id, temp);
        self.history.push(temp);
    }

    fn format_status(&self) -> String {
        format!("Sensors: {}, History: {} readings",
                self.sensors.len(), self.history.len())
    }
}
```

```rust
// NO_STD VERSION
#![no_std]

use heapless::{FnvIndexMap, Vec, String};

struct EmbeddedTemperatureMonitor {
    sensors: FnvIndexMap<heapless::String<16>, f32, 8>, // Max 8 sensors
    history: Vec<f32, 100>, // Max 100 readings
}

impl EmbeddedTemperatureMonitor {
    const fn new() -> Self {
        Self {
            sensors: FnvIndexMap::new(),
            history: Vec::new(),
        }
    }

    fn add_reading(&mut self, sensor_id: &str, temp: f32) -> Result<(), &'static str> {
        // Convert to heapless string
        let mut key = heapless::String::new();
        key.push_str(sensor_id).map_err(|_| "Sensor ID too long")?;

        self.sensors.insert(key, temp).map_err(|_| "Too many sensors")?;
        self.history.push(temp).map_err(|_| "History full")?;

        Ok(())
    }

    fn format_status(&self) -> heapless::String<64> {
        let mut status = heapless::String::new();
        status.push_str("Sensors: ").ok();
        push_number(&mut status, self.sensors.len() as i32);
        status.push_str(", History: ").ok();
        push_number(&mut status, self.history.len() as i32);
        status.push_str(" readings").ok();
        status
    }

    fn get_average(&self) -> Option<f32> {
        if self.history.is_empty() {
            return None;
        }

        let sum: f32 = self.history.iter().sum();
        Some(sum / self.history.len() as f32)
    }
}

// Helper function for formatting numbers without std::format!
fn push_number(s: &mut heapless::String<64>, mut num: i32) {
    if num == 0 {
        s.push('0').ok();
        return;
    }

    if num < 0 {
        s.push('-').ok();
        num = -num;
    }

    let mut digits = heapless::Vec::<u8, 16>::new();
    while num > 0 {
        digits.push((num % 10) as u8).ok();
        num /= 10;
    }

    for &digit in digits.iter().rev() {
        s.push((b'0' + digit) as char).ok();
    }
}
```

## Using Alloc Without Std

When you need heap allocation but not OS services, `alloc` provides the middle ground:

```rust
#![no_std]
extern crate alloc;

use alloc::{
    vec::Vec,
    string::String,
    boxed::Box,
    collections::BTreeMap,
    format,
};

// Global allocator required for alloc
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Initialize heap in embedded context
fn init_heap() {
    const HEAP_SIZE: usize = 1024;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

    unsafe {
        ALLOCATOR.lock().init(HEAP.as_mut_ptr(), HEAP_SIZE);
    }
}

// Now we can use heap-allocated types
fn alloc_examples() {
    let mut numbers = Vec::new();
    numbers.push(1);
    numbers.push(2);

    let greeting = String::from("Hello");
    let formatted = format!("{}!", greeting);

    let boxed_value = Box::new(42i32);

    // Collections that don't require hashing
    let mut map = BTreeMap::new();
    map.insert("key", "value");

    // HashMap requires std due to RandomState dependency
    // This won't compile in no_std:
    // use std::collections::HashMap; // Error!
}
```

## Heapless Data Structures

The `heapless` crate provides fixed-capacity collections perfect for embedded systems:

```rust
#![no_std]

use heapless::{
    Vec, String, FnvIndexMap,
    pool::{Pool, Node},
    spsc::{Producer, Consumer, Queue},
};

// Fixed-capacity vector - perfect for sensor readings
fn heapless_sensor_storage() {
    // Vec with maximum 32 temperature readings
    let mut temperatures: Vec<f32, 32> = Vec::new();

    temperatures.push(23.5).ok(); // Returns Result - can fail if full
    temperatures.push(24.1).ok();
    temperatures.push(23.8).ok();

    // Check capacity and usage
    assert_eq!(temperatures.len(), 3);
    assert_eq!(temperatures.capacity(), 32);

    // Calculate average
    let sum: f32 = temperatures.iter().sum();
    let average = sum / temperatures.len() as f32;

    // Convert to slice for processing
    let slice: &[f32] = &temperatures;
    process_readings(slice);
}

fn process_readings(readings: &[f32]) {
    for &temp in readings {
        if temp > 25.0 {
            // Handle high temperature
        }
    }
}

// Fixed-capacity string for sensor names
fn heapless_string_example() {
    let mut sensor_name: String<16> = String::new();

    sensor_name.push_str("temp_").ok();
    sensor_name.push_str("01").ok();

    // Safe string formatting without heap
    assert_eq!(sensor_name.as_str(), "temp_01");
}

// Hash map alternative for sensor lookup
fn sensor_registry_example() {
    let mut sensors: FnvIndexMap<&str, f32, 16> = FnvIndexMap::new();

    sensors.insert("living_room", 23.5).ok();
    sensors.insert("kitchen", 24.2).ok();
    sensors.insert("bedroom", 22.8).ok();

    if let Some(&temp) = sensors.get("living_room") {
        if temp > 25.0 {
            // Trigger cooling
        }
    }

    // Iterate over all sensors
    for (location, &temperature) in &sensors {
        println!("{}: {:.1}°C", location, temperature);
    }
}

// Memory pool for dynamic allocation without heap
static mut SENSOR_POOL_MEMORY: [Node<[u8; 64]>; 8] = [Node::new(); 8];
static SENSOR_POOL: Pool<[u8; 64]> = Pool::new();

fn init_sensor_pool() {
    unsafe {
        SENSOR_POOL.grow_exact(&mut SENSOR_POOL_MEMORY);
    }
}

fn use_sensor_buffer() -> Option<()> {
    let mut buffer = SENSOR_POOL.alloc()?; // Get buffer from pool

    // Use buffer for sensor data processing
    buffer[0] = 0x42; // Sensor command
    buffer[1] = 0x01; // Sensor ID

    // Buffer automatically returned to pool when dropped
    Some(())
}

// Lock-free queue for interrupt communication
static mut SENSOR_QUEUE: Queue<f32, 16> = Queue::new();

fn init_sensor_communication() {
    // Split queue for producer/consumer
    let (mut producer, mut consumer) = unsafe { SENSOR_QUEUE.split() };

    // Producer side (could be in interrupt)
    producer.enqueue(23.5).ok(); // Temperature reading
    producer.enqueue(24.1).ok();

    // Consumer side (main loop)
    while let Some(temperature) = consumer.dequeue() {
        process_temperature_reading(temperature);
    }
}

fn process_temperature_reading(temp: f32) {
    // Process the temperature reading
}
```

## Const Functions for Compile-time Configuration

Const functions enable zero-cost configuration and computation:

```rust
#![no_std]

// Configuration constants computed at compile time
const SYSTEM_CLOCK_HZ: u32 = 16_000_000; // 16 MHz
const UART_BAUD_RATE: u32 = 115_200;

// Const function to calculate UART register values
const fn calculate_uart_divisor(clock_hz: u32, baud_rate: u32) -> u32 {
    clock_hz / (16 * baud_rate)
}

// Computed at compile time - zero runtime cost
const UART_DIVISOR: u32 = calculate_uart_divisor(SYSTEM_CLOCK_HZ, UART_BAUD_RATE);

// Temperature sensor configuration
const fn celsius_to_adc_value(celsius: f32) -> u16 {
    // Simple linear conversion: 10mV/°C, 3.3V reference, 12-bit ADC
    let voltage = celsius * 0.01; // 10mV/°C
    let adc_value = (voltage / 3.3) * 4095.0;
    adc_value as u16
}

// Temperature thresholds computed at compile time
const TEMP_THRESHOLD_LOW: u16 = celsius_to_adc_value(5.0);   // 5°C
const TEMP_THRESHOLD_HIGH: u16 = celsius_to_adc_value(35.0); // 35°C
const TEMP_CRITICAL: u16 = celsius_to_adc_value(50.0);       // 50°C

// Const generic functions for buffer sizing
const fn next_power_of_two(n: usize) -> usize {
    if n <= 1 {
        1
    } else {
        2 * next_power_of_two((n + 1) / 2)
    }
}

// Ring buffer with compile-time size validation
struct RingBuffer<T, const N: usize> {
    buffer: [Option<T>; N],
    head: usize,
    tail: usize,
    full: bool,
}

impl<T, const N: usize> RingBuffer<T, N> {
    const fn new() -> Self {
        // Compile-time assertion
        assert!(N > 0 && (N & (N - 1)) == 0, "Buffer size must be power of two");

        const fn none<T>() -> Option<T> { None }

        RingBuffer {
            buffer: [none(); N],
            head: 0,
            tail: 0,
            full: false,
        }
    }

    const fn capacity(&self) -> usize {
        N
    }

    const fn mask(&self) -> usize {
        N - 1 // Works because N is power of 2
    }

    fn push(&mut self, item: T) -> Result<(), T> {
        if self.full {
            return Err(item);
        }

        self.buffer[self.head] = Some(item);
        self.head = (self.head + 1) & self.mask();
        self.full = self.head == self.tail;
        Ok(())
    }

    fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let item = self.buffer[self.tail].take()?;
        self.tail = (self.tail + 1) & self.mask();
        self.full = false;
        Some(item)
    }

    const fn is_empty(&self) -> bool {
        !self.full && self.head == self.tail
    }
}

// Usage with compile-time validated size
const READING_BUFFER_SIZE: usize = next_power_of_two(50); // Results in 64
static mut TEMPERATURE_BUFFER: RingBuffer<f32, READING_BUFFER_SIZE> = RingBuffer::new();
```

## Embedded Programming Patterns

Essential patterns for embedded Rust development:

```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Panic handler required for no_std
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // In production: reset system or enter safe mode
    // For debug: could use RTT or LED patterns
    loop {
        // Infinite loop or system reset
    }
}

// State machine for temperature monitoring device
#[derive(Clone, Copy, Debug, PartialEq)]
enum MonitorState {
    Initializing,
    Idle,
    Reading,
    Processing,
    Transmitting,
    Error(ErrorCode),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ErrorCode {
    SensorTimeout,
    InvalidReading,
    CommunicationError,
    OverTemperature,
}

#[derive(Clone, Copy, Debug)]
enum MonitorEvent {
    InitComplete,
    StartReading,
    ReadingComplete(f32),
    ReadingFailed(ErrorCode),
    ProcessingComplete,
    TransmissionComplete,
    TransmissionFailed,
    ErrorRecovered,
}

struct TemperatureMonitor {
    state: MonitorState,
    reading_count: u32,
    error_count: u32,
    last_reading: Option<f32>,
}

impl TemperatureMonitor {
    const fn new() -> Self {
        Self {
            state: MonitorState::Initializing,
            reading_count: 0,
            error_count: 0,
            last_reading: None,
        }
    }

    fn handle_event(&mut self, event: MonitorEvent) -> MonitorState {
        self.state = match (self.state, event) {
            (MonitorState::Initializing, MonitorEvent::InitComplete) => {
                MonitorState::Idle
            }
            (MonitorState::Idle, MonitorEvent::StartReading) => {
                self.start_sensor_reading();
                MonitorState::Reading
            }
            (MonitorState::Reading, MonitorEvent::ReadingComplete(temp)) => {
                self.last_reading = Some(temp);
                self.reading_count += 1;
                MonitorState::Processing
            }
            (MonitorState::Reading, MonitorEvent::ReadingFailed(error)) => {
                self.error_count += 1;
                MonitorState::Error(error)
            }
            (MonitorState::Processing, MonitorEvent::ProcessingComplete) => {
                if self.should_transmit() {
                    MonitorState::Transmitting
                } else {
                    MonitorState::Idle
                }
            }
            (MonitorState::Transmitting, MonitorEvent::TransmissionComplete) => {
                MonitorState::Idle
            }
            (MonitorState::Error(_), MonitorEvent::ErrorRecovered) => {
                MonitorState::Idle
            }
            // Invalid transitions maintain current state
            _ => self.state,
        };

        self.state
    }

    fn start_sensor_reading(&self) {
        // Initiate ADC conversion or I2C transaction
    }

    fn should_transmit(&self) -> bool {
        // Transmit every 10 readings or if temperature is critical
        self.reading_count % 10 == 0 ||
        self.last_reading.map_or(false, |t| t > 40.0)
    }

    fn get_status(&self) -> &'static str {
        match self.state {
            MonitorState::Initializing => "Initializing sensors...",
            MonitorState::Idle => "Ready",
            MonitorState::Reading => "Reading sensors...",
            MonitorState::Processing => "Processing data...",
            MonitorState::Transmitting => "Transmitting...",
            MonitorState::Error(ErrorCode::SensorTimeout) => "Sensor timeout",
            MonitorState::Error(ErrorCode::InvalidReading) => "Invalid reading",
            MonitorState::Error(ErrorCode::CommunicationError) => "Comm error",
            MonitorState::Error(ErrorCode::OverTemperature) => "CRITICAL: Over temp!",
        }
    }
}

// Interrupt-safe shared data
use core::cell::RefCell;
use cortex_m::interrupt::{self, Mutex};

type SharedSensorData = Mutex<RefCell<Option<f32>>>;
static SENSOR_DATA: SharedSensorData = Mutex::new(RefCell::new(None));

fn read_shared_sensor_data() -> Option<f32> {
    interrupt::free(|cs| {
        *SENSOR_DATA.borrow(cs).borrow()
    })
}

fn write_shared_sensor_data(value: f32) {
    interrupt::free(|cs| {
        *SENSOR_DATA.borrow(cs).borrow_mut() = Some(value);
    });
}

// Task scheduler with fixed-size task array
struct Task {
    period_ms: u32,
    last_run: u32,
    enabled: bool,
    function: fn(),
}

impl Task {
    const fn new(period_ms: u32, function: fn()) -> Self {
        Self {
            period_ms,
            last_run: 0,
            enabled: true,
            function,
        }
    }

    fn should_run(&self, current_time: u32) -> bool {
        self.enabled && current_time.wrapping_sub(self.last_run) >= self.period_ms
    }

    fn run(&mut self, current_time: u32) {
        (self.function)();
        self.last_run = current_time;
    }
}

// Fixed task schedule
static mut TASKS: [Task; 4] = [
    Task::new(100, sensor_task),      // 100ms - sensor reading
    Task::new(1000, heartbeat_task),  // 1s - status LED
    Task::new(5000, telemetry_task),  // 5s - data transmission
    Task::new(60000, watchdog_task),  // 60s - system health check
];

fn run_scheduler() {
    let current_time = get_system_time_ms();

    unsafe {
        for task in &mut TASKS {
            if task.should_run(current_time) {
                task.run(current_time);
            }
        }
    }
}

fn sensor_task() {
    // Read temperature sensors
}

fn heartbeat_task() {
    // Toggle status LED
}

fn telemetry_task() {
    // Send sensor data via radio/WiFi
}

fn watchdog_task() {
    // Pet watchdog, check system health
}

fn get_system_time_ms() -> u32 {
    // Return system time in milliseconds
    // Implementation depends on hardware timer
    0 // Placeholder
}
```

## Exercise: Convert Temperature System to no_std

Build a complete no_std version of our temperature monitoring system:

### Requirements

1. **Core Types**: Temperature and sensor traits that work in no_std
2. **Fixed Storage**: Use heapless collections for storing readings
3. **Protocol Handler**: Binary protocol processing without allocation
4. **Const Configuration**: Compile-time system parameters
5. **Error Handling**: Comprehensive error handling without std

### Starting Implementation

```rust
// In temp_embedded/src/lib.rs
#![no_std]

use heapless::{Vec, String, FnvIndexMap};
use serde::{Deserialize, Serialize};

// Re-export core temperature types
pub use temp_core::Temperature;

// Fixed-capacity temperature reading
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EmbeddedTemperatureReading {
    pub temperature: Temperature,
    pub timestamp: u32, // Using u32 for embedded systems
}

impl EmbeddedTemperatureReading {
    pub fn new(temperature: Temperature, timestamp: u32) -> Self {
        Self { temperature, timestamp }
    }
}

// Fixed-capacity storage for embedded systems
pub struct EmbeddedTemperatureStore<const N: usize> {
    readings: Vec<EmbeddedTemperatureReading, N>,
    total_readings: u32,
}

impl<const N: usize> EmbeddedTemperatureStore<N> {
    pub const fn new() -> Self {
        // TODO: Initialize with fixed capacity
        unimplemented!()
    }

    pub fn add_reading(&mut self, reading: EmbeddedTemperatureReading) -> Result<(), &'static str> {
        // TODO: Add reading, handling full buffer (circular buffer behavior)
        unimplemented!()
    }

    pub fn get_latest(&self) -> Option<EmbeddedTemperatureReading> {
        // TODO: Return most recent reading
        unimplemented!()
    }

    pub fn get_stats(&self) -> EmbeddedTemperatureStats {
        // TODO: Calculate stats without heap allocation
        unimplemented!()
    }

    pub fn clear(&mut self) {
        // TODO: Clear all readings
        unimplemented!()
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub fn len(&self) -> usize {
        // TODO: Return current number of readings
        unimplemented!()
    }

    pub fn is_full(&self) -> bool {
        // TODO: Check if storage is at capacity
        unimplemented!()
    }
}

// Statistics without heap allocation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EmbeddedTemperatureStats {
    pub min: Temperature,
    pub max: Temperature,
    pub average: Temperature,
    pub count: usize,
}

// Const configuration functions
pub const fn calculate_sample_rate(desired_hz: u32, clock_hz: u32) -> u32 {
    clock_hz / desired_hz
}

pub const fn validate_buffer_size(size: usize) -> usize {
    assert!(size > 0 && size <= 1024, "Buffer size must be 1-1024");
    assert!(size & (size - 1) == 0, "Buffer size must be power of 2");
    size
}

// Configuration constants
pub const SYSTEM_CLOCK_HZ: u32 = 16_000_000;
pub const SAMPLE_RATE_HZ: u32 = 10; // 10 Hz sampling
pub const TIMER_DIVISOR: u32 = calculate_sample_rate(SAMPLE_RATE_HZ, SYSTEM_CLOCK_HZ);
pub const READING_BUFFER_SIZE: usize = validate_buffer_size(64);

// Binary protocol for embedded communication
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EmbeddedCommand {
    GetStatus,
    GetLatestReading,
    GetReadingCount,
    ClearReadings,
    SetSampleRate(u32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EmbeddedResponse {
    Status {
        uptime_seconds: u32,
        reading_count: u32,
        sample_rate: u32,
    },
    Reading(EmbeddedTemperatureReading),
    ReadingCount(u32),
    Cleared,
    SampleRateSet(u32),
    Error(u8), // Error code as u8 for compact binary encoding
}

pub struct EmbeddedProtocolHandler<const N: usize> {
    store: EmbeddedTemperatureStore<N>,
    sample_rate: u32,
    start_time: u32,
}

impl<const N: usize> EmbeddedProtocolHandler<N> {
    pub const fn new() -> Self {
        // TODO: Initialize protocol handler
        unimplemented!()
    }

    pub fn process_command(&mut self, command: EmbeddedCommand, current_time: u32) -> EmbeddedResponse {
        // TODO: Process commands and return appropriate responses
        unimplemented!()
    }

    pub fn serialize_binary(&self, response: &EmbeddedResponse) -> Result<Vec<u8, 256>, postcard::Error> {
        // TODO: Serialize response to binary using postcard
        unimplemented!()
    }

    pub fn deserialize_binary(&self, data: &[u8]) -> Result<EmbeddedCommand, postcard::Error> {
        // TODO: Deserialize command from binary
        unimplemented!()
    }

    pub fn add_reading(&mut self, temperature: Temperature, timestamp: u32) -> Result<(), &'static str> {
        let reading = EmbeddedTemperatureReading::new(temperature, timestamp);
        self.store.add_reading(reading)
    }
}

// Error types for embedded systems
#[derive(Debug, Clone, Copy)]
pub enum EmbeddedError {
    BufferFull,
    InvalidSampleRate,
    SensorTimeout,
    InvalidCommand,
    SerializationError,
}

impl EmbeddedError {
    pub const fn error_code(&self) -> u8 {
        match self {
            EmbeddedError::BufferFull => 1,
            EmbeddedError::InvalidSampleRate => 2,
            EmbeddedError::SensorTimeout => 3,
            EmbeddedError::InvalidCommand => 4,
            EmbeddedError::SerializationError => 5,
        }
    }

    pub const fn description(&self) -> &'static str {
        match self {
            EmbeddedError::BufferFull => "Buffer full",
            EmbeddedError::InvalidSampleRate => "Invalid sample rate",
            EmbeddedError::SensorTimeout => "Sensor timeout",
            EmbeddedError::InvalidCommand => "Invalid command",
            EmbeddedError::SerializationError => "Serialization error",
        }
    }
}

// Utility function for creating fixed-capacity strings
pub fn create_status_string(reading_count: u32, sample_rate: u32) -> String<128> {
    let mut status = String::new();
    status.push_str("Readings: ").ok();
    push_number(&mut status, reading_count as i32);
    status.push_str(", Rate: ").ok();
    push_number(&mut status, sample_rate as i32);
    status.push_str(" Hz").ok();
    status
}

fn push_number(s: &mut String<128>, mut num: i32) {
    // TODO: Implement number to string conversion without std::format!
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_store_basic_operations() {
        // TODO: Test basic store operations
    }

    #[test]
    fn test_embedded_store_circular_buffer() {
        // TODO: Test circular buffer behavior when full
    }

    #[test]
    fn test_const_configuration() {
        // TODO: Test compile-time configuration
    }

    #[test]
    fn test_protocol_binary_serialization() {
        // TODO: Test binary protocol serialization
    }

    #[test]
    fn test_error_handling() {
        // TODO: Test error handling without std
    }
}
```

### Success Criteria

- All code works in `#![no_std]` environment
- Fixed-capacity storage behaves correctly when full
- Binary protocol serialization works without allocation
- Const functions provide zero-cost configuration
- Comprehensive error handling without std types
- All tests pass demonstrating embedded-ready functionality

### Extension Ideas

1. **Power Management**: Add sleep modes and wake-on-interrupt
2. **Watchdog Integration**: System reset on hang detection
3. **Interrupt Handlers**: Safe interrupt-driven sensor reading
4. **Memory Analysis**: Verify stack and flash usage
5. **Hardware Abstraction**: Generic sensor traits for different platforms

## Key Takeaways

1. **Library Layers**: `core` for basics, `alloc` for heap, `std` for OS features
2. **Fixed Capacity**: Use heapless collections to avoid allocation failures
3. **Const Functions**: Compute configuration at compile time for zero overhead
4. **State Machines**: Explicit state management prevents invalid operations in resource-constrained environments
5. **Error Handling**: Custom error types with Result provide type-safe error handling without exceptions
6. **Memory Patterns**: Pools, ring buffers, and static allocation replace heap when needed
7. **Interrupt Safety**: Use `Mutex<RefCell<T>>` for interrupt-safe shared data
8. **Binary Protocols**: Compact serialization saves bandwidth and storage

**Next**: In Chapter 18, we'll explore build systems, cross-compilation, and deployment strategies for getting our embedded code running on actual hardware.