# Chapter 13: Testing & Documentation
## Building Robust Rust Applications with Comprehensive Testing

### Learning Objectives
By the end of this chapter, you'll be able to:
- Write comprehensive unit tests, integration tests, and documentation tests
- Use test-driven development (TDD) effectively in Rust
- Mock complex dependencies using traits
- Structure projects for maximum testability
- Generate and maintain excellent documentation
- Apply testing strategies to embedded systems (when needed)
- Debug test failures efficiently
- Measure and improve test coverage

---

## Testing in Rust: The Foundation of Reliable Software

### Why Testing Matters

Testing is crucial for building reliable software, and Rust's testing framework makes it both easy and powerful. Let's start with the basics and work up to more complex scenarios.

**Testing Comparison Across Languages:**

| Aspect | C/C++ | C#/Java | Python | Rust |
|--------|-------|---------|---------|------|
| Built-in framework | No | Yes | Yes | **Yes + Zero-cost** |
| Mocking | Third-party | Frameworks | Built-in | **Trait-based** |
| Documentation tests | No | Limited | doctest | **Integrated** |
| Compile-time checks | Limited | Some | None | **Extensive** |
| Performance testing | Manual | Frameworks | Third-party | **Built-in** |

### Basic Unit Testing

```rust
// Simple function to test temperature conversions
pub fn celsius_to_fahrenheit(celsius: f32) -> f32 {
    celsius * 9.0 / 5.0 + 32.0
}

pub fn fahrenheit_to_celsius(fahrenheit: f32) -> f32 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celsius_to_fahrenheit() {
        assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
        assert_eq!(celsius_to_fahrenheit(100.0), 212.0);
        assert!((celsius_to_fahrenheit(20.0) - 68.0).abs() < 0.001);
    }

    #[test]
    fn test_fahrenheit_to_celsius() {
        assert_eq!(fahrenheit_to_celsius(32.0), 0.0);
        assert_eq!(fahrenheit_to_celsius(212.0), 100.0);
        assert!((fahrenheit_to_celsius(68.0) - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_conversion_roundtrip() {
        let temps = vec![0.0, 25.0, -40.0, 100.0];
        for temp in temps {
            let converted = fahrenheit_to_celsius(celsius_to_fahrenheit(temp));
            assert!((converted - temp).abs() < 0.001,
                    "Roundtrip failed for {}: got {}", temp, converted);
        }
    }
}
```

## Trait-Based Testing and Mocking

One of Rust's greatest strengths is using traits to create testable abstractions. This allows us to test complex logic without depending on external systems.

### The Power of Trait Abstraction

```rust
use std::fmt;

// Our core Temperature type
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Temperature {
    pub celsius: f32,
}

impl Temperature {
    pub fn new(celsius: f32) -> Self {
        Self { celsius }
    }

    pub fn from_fahrenheit(fahrenheit: f32) -> Self {
        Self {
            celsius: (fahrenheit - 32.0) * 5.0 / 9.0,
        }
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}°C", self.celsius)
    }
}

// Trait that abstracts temperature reading
pub trait TemperatureSensor {
    type Error: fmt::Debug;

    fn read_temperature(&mut self) -> Result<Temperature, Self::Error>;
    fn sensor_id(&self) -> &str;
}

// Mock implementation for testing
pub struct MockTemperatureSensor {
    id: String,
    temperature: f32,
    fail_next: bool,
    offline: bool,
}

impl MockTemperatureSensor {
    pub fn new(id: String, temperature: f32) -> Self {
        Self {
            id,
            temperature,
            fail_next: false,
            offline: false,
        }
    }

    pub fn set_temperature(&mut self, temp: f32) {
        self.temperature = temp;
    }

    pub fn set_offline(&mut self, offline: bool) {
        self.offline = offline;
    }

    pub fn fail_next_read(&mut self) {
        self.fail_next = true;
    }
}

#[derive(Debug)]
pub enum MockError {
    SensorOffline,
    ReadFailed,
}

impl TemperatureSensor for MockTemperatureSensor {
    type Error = MockError;

    fn read_temperature(&mut self) -> Result<Temperature, Self::Error> {
        if self.offline {
            return Err(MockError::SensorOffline);
        }

        if self.fail_next {
            self.fail_next = false;
            return Err(MockError::ReadFailed);
        }

        Ok(Temperature::new(self.temperature))
    }

    fn sensor_id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod sensor_tests {
    use super::*;

    #[test]
    fn mock_sensor_works() {
        let mut sensor = MockTemperatureSensor::new("test-sensor".to_string(), 25.0);

        let reading = sensor.read_temperature().unwrap();
        assert_eq!(reading.celsius, 25.0);
        assert_eq!(sensor.sensor_id(), "test-sensor");
    }

    #[test]
    fn mock_sensor_can_fail() {
        let mut sensor = MockTemperatureSensor::new("test-sensor".to_string(), 25.0);

        sensor.fail_next_read();
        let result = sensor.read_temperature();
        assert!(matches!(result, Err(MockError::ReadFailed)));

        // Should work again after failure
        let reading = sensor.read_temperature().unwrap();
        assert_eq!(reading.celsius, 25.0);
    }

    #[test]
    fn mock_sensor_can_be_offline() {
        let mut sensor = MockTemperatureSensor::new("test-sensor".to_string(), 25.0);

        sensor.set_offline(true);
        let result = sensor.read_temperature();
        assert!(matches!(result, Err(MockError::SensorOffline)));

        sensor.set_offline(false);
        let reading = sensor.read_temperature().unwrap();
        assert_eq!(reading.celsius, 25.0);
    }
}
```

## Test Organization and Best Practices

### Test Module Organization

```rust
// In your lib.rs or main module
pub mod temperature {
    pub use crate::{Temperature, TemperatureSensor, MockTemperatureSensor};

    // Business logic that uses the sensor
    pub struct TemperatureMonitor<S: TemperatureSensor> {
        sensor: S,
        alert_threshold: Temperature,
    }

    impl<S: TemperatureSensor> TemperatureMonitor<S> {
        pub fn new(sensor: S, threshold_celsius: f32) -> Self {
            Self {
                sensor,
                alert_threshold: Temperature::new(threshold_celsius),
            }
        }

        pub fn check_temperature(&mut self) -> Result<bool, S::Error> {
            let current = self.sensor.read_temperature()?;
            Ok(current.celsius > self.alert_threshold.celsius)
        }

        pub fn get_sensor_id(&self) -> &str {
            self.sensor.sensor_id()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{MockTemperatureSensor, MockError};

        #[test]
        fn monitor_detects_high_temperature() {
            let sensor = MockTemperatureSensor::new("test".to_string(), 30.0);
            let mut monitor = TemperatureMonitor::new(sensor, 25.0);

            let is_alert = monitor.check_temperature().unwrap();
            assert!(is_alert);
        }

        #[test]
        fn monitor_handles_normal_temperature() {
            let sensor = MockTemperatureSensor::new("test".to_string(), 20.0);
            let mut monitor = TemperatureMonitor::new(sensor, 25.0);

            let is_alert = monitor.check_temperature().unwrap();
            assert!(!is_alert);
        }

        #[test]
        fn monitor_propagates_sensor_errors() {
            let mut sensor = MockTemperatureSensor::new("test".to_string(), 20.0);
            sensor.fail_next_read();
            let mut monitor = TemperatureMonitor::new(sensor, 25.0);

            let result = monitor.check_temperature();
            assert!(matches!(result, Err(MockError::ReadFailed)));
        }
    }
}
```

### Integration Testing

Integration tests go in the `tests/` directory and test your crate from the outside:

```rust
// tests/temperature_integration.rs
use your_crate::{Temperature, TemperatureSensor, MockTemperatureSensor};
use your_crate::temperature::TemperatureMonitor;

#[test]
fn full_temperature_monitoring_workflow() {
    let mut sensor = MockTemperatureSensor::new("integration-test".to_string(), 15.0);
    let mut monitor = TemperatureMonitor::new(sensor, 20.0);

    // Initially, temperature is below threshold
    assert!(!monitor.check_temperature().unwrap());

    // Simulate temperature rise
    // Note: We can't modify the sensor after moving it into monitor
    // This is where the real hardware implementation would change

    // In a real scenario, you might have:
    // - A sensor that reads from actual hardware
    // - A monitor that runs in a loop
    // - Integration with real systems
}
```

## Documentation Tests

Rust can run code examples in your documentation as tests:

```rust
/// Convert Celsius to Fahrenheit
///
/// # Examples
///
/// ```
/// use your_crate::Temperature;
///
/// let temp = Temperature::new(20.0);
/// assert!((temp.to_fahrenheit() - 68.0).abs() < 0.1);
/// ```
///
/// # Conversion Formula
///
/// The formula is: F = C × 9/5 + 32
///
/// ```
/// use your_crate::Temperature;
///
/// // Freezing point of water
/// assert_eq!(Temperature::new(0.0).to_fahrenheit(), 32.0);
///
/// // Boiling point of water
/// assert_eq!(Temperature::new(100.0).to_fahrenheit(), 212.0);
/// ```
impl Temperature {
    pub fn to_fahrenheit(&self) -> f32 {
        self.celsius * 9.0 / 5.0 + 32.0
    }
}
```

Run documentation tests with:
```bash
cargo test --doc
```

## Test-Driven Development (TDD) in Rust

TDD follows the Red-Green-Refactor cycle:

1. **Red**: Write a failing test
2. **Green**: Write minimal code to make it pass
3. **Refactor**: Improve the code while keeping tests passing

### TDD Example: Temperature Statistics

```rust
// Step 1: Write the test first (RED)
#[cfg(test)]
mod stats_tests {
    use super::*;

    #[test]
    fn temperature_stats_calculates_average() {
        let temps = vec![
            Temperature::new(10.0),
            Temperature::new(20.0),
            Temperature::new(30.0),
        ];

        let stats = TemperatureStats::from_readings(&temps);

        assert_eq!(stats.average().celsius, 20.0);
        assert_eq!(stats.min().celsius, 10.0);
        assert_eq!(stats.max().celsius, 30.0);
        assert_eq!(stats.count(), 3);
    }

    #[test]
    fn temperature_stats_handles_empty_list() {
        let temps = vec![];

        let result = TemperatureStats::from_readings(&temps);

        assert!(result.is_none());
    }
}

// Step 2: Write minimal implementation (GREEN)
pub struct TemperatureStats {
    min: Temperature,
    max: Temperature,
    average: Temperature,
    count: usize,
}

impl TemperatureStats {
    pub fn from_readings(readings: &[Temperature]) -> Option<Self> {
        if readings.is_empty() {
            return None;
        }

        let mut min_temp = readings[0].celsius;
        let mut max_temp = readings[0].celsius;
        let mut sum = 0.0;

        for reading in readings {
            let temp = reading.celsius;
            if temp < min_temp {
                min_temp = temp;
            }
            if temp > max_temp {
                max_temp = temp;
            }
            sum += temp;
        }

        let average = sum / readings.len() as f32;

        Some(Self {
            min: Temperature::new(min_temp),
            max: Temperature::new(max_temp),
            average: Temperature::new(average),
            count: readings.len(),
        })
    }

    pub fn min(&self) -> Temperature { self.min }
    pub fn max(&self) -> Temperature { self.max }
    pub fn average(&self) -> Temperature { self.average }
    pub fn count(&self) -> usize { self.count }
}

// Step 3: Refactor as needed while tests still pass
```

## Advanced Testing Techniques

### Property-Based Testing with `proptest`

```toml
[dev-dependencies]
proptest = "1.0"
```

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn temperature_conversion_roundtrip(celsius in -273.15f32..1000.0f32) {
            let temp = Temperature::new(celsius);
            let fahrenheit = temp.to_fahrenheit();
            let back_to_celsius = Temperature::from_fahrenheit(fahrenheit);

            prop_assert!((back_to_celsius.celsius - celsius).abs() < 0.001);
        }

        #[test]
        fn temperature_display_never_panics(celsius in f32::NEG_INFINITY..f32::INFINITY) {
            if celsius.is_finite() {
                let temp = Temperature::new(celsius);
                let _ = format!("{}", temp); // Should never panic
            }
        }
    }
}
```

### Benchmark Testing

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "temperature_bench"
harness = false
```

```rust
// benches/temperature_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use your_crate::Temperature;

fn bench_temperature_conversion(c: &mut Criterion) {
    c.bench_function("celsius_to_fahrenheit", |b| {
        b.iter(|| {
            let temp = Temperature::new(black_box(25.0));
            black_box(temp.to_fahrenheit())
        })
    });
}

criterion_group!(benches, bench_temperature_conversion);
criterion_main!(benches);
```

## Embedded Testing Considerations

When your code needs to run on embedded systems:

### Testing with `no_std`

```rust
#![cfg_attr(not(test), no_std)]

// Use heapless collections instead of std
use heapless::Vec;

pub struct EmbeddedTemperatureBuffer<const N: usize> {
    readings: Vec<Temperature, N>,
}

impl<const N: usize> EmbeddedTemperatureBuffer<N> {
    pub fn new() -> Self {
        Self {
            readings: Vec::new(),
        }
    }

    pub fn add_reading(&mut self, temp: Temperature) -> Result<(), ()> {
        self.readings.push(temp).map_err(|_| ())
    }

    pub fn calculate_average(&self) -> Option<Temperature> {
        if self.readings.is_empty() {
            return None;
        }

        let sum: f32 = self.readings.iter().map(|t| t.celsius).sum();
        Some(Temperature::new(sum / self.readings.len() as f32))
    }
}

#[cfg(test)]
mod embedded_tests {
    use super::*;

    #[test]
    fn embedded_buffer_works() {
        let mut buffer: EmbeddedTemperatureBuffer<3> = EmbeddedTemperatureBuffer::new();

        buffer.add_reading(Temperature::new(10.0)).unwrap();
        buffer.add_reading(Temperature::new(20.0)).unwrap();
        buffer.add_reading(Temperature::new(30.0)).unwrap();

        let avg = buffer.calculate_average().unwrap();
        assert_eq!(avg.celsius, 20.0);

        // Buffer is full - this should fail
        assert!(buffer.add_reading(Temperature::new(40.0)).is_err());
    }
}
```

## Exercise: Complete the Temperature Monitoring System

Now it's time to build the first increment of our capstone project!

### Your Task

1. Create a new Rust project: `cargo new temp_core --lib`

2. Implement the `Temperature` struct with conversion methods:
   - `new(celsius: f32) -> Self`
   - `from_fahrenheit(fahrenheit: f32) -> Self`
   - `from_kelvin(kelvin: f32) -> Self`
   - `to_fahrenheit(&self) -> f32`
   - `to_kelvin(&self) -> f32`

3. Create a `TemperatureSensor` trait with:
   - `read_temperature(&mut self) -> Result<Temperature, Self::Error>`
   - `sensor_id(&self) -> &str`

4. Implement a `MockTemperatureSensor` for testing that can:
   - Return configurable temperatures
   - Simulate sensor failures
   - Go offline/online

5. Write comprehensive tests covering:
   - All temperature conversions
   - Mock sensor behavior
   - Error conditions
   - Edge cases (like very hot/cold temperatures)

### Extension Challenges

1. **Property-Based Tests**: Use `proptest` to verify conversion roundtrips
2. **Documentation Tests**: Add examples to your documentation
3. **Benchmarks**: Measure conversion performance
4. **no_std Support**: Make your types work without the standard library

### Success Criteria

- All tests pass: `cargo test`
- No warnings: `cargo clippy`
- Good documentation: `cargo doc --open`
- Code coverage is high
- You can create mock sensors that behave predictably

This foundation will be essential as we build more complex features in the following chapters!

## Key Takeaways

✅ **Testing is built into Rust** - Use `#[test]`, `#[cfg(test)]`, and `cargo test`

✅ **Traits enable powerful mocking** - Abstract dependencies behind traits

✅ **Documentation tests keep examples current** - Code in docs is tested automatically

✅ **TDD helps design better APIs** - Write tests first to drive good interfaces

✅ **Integration tests verify real workflows** - Test from the outside in

✅ **Property tests catch edge cases** - Generate random inputs to find bugs

✅ **Embedded testing is possible** - Use `no_std` compatible patterns

The testing foundation you build today will make every subsequent feature more reliable and maintainable!