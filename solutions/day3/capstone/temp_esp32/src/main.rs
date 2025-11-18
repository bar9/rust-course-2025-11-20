//! ESP32-C3 Temperature Monitor
//!
//! This crate demonstrates deploying the temperature monitoring system to ESP32-C3.
//! It can run in two modes:
//! - Simulation mode: Runs on desktop for testing
//! - Hardware mode: Runs on actual ESP32-C3

#![cfg_attr(not(feature = "simulation"), no_std)]
#![cfg_attr(feature = "hardware", no_main)]

use temp_embedded::{
    EmbeddedTemperatureStore, EmbeddedProtocolHandler, EmbeddedCommand, EmbeddedResponse,
    EmbeddedTemperatureReading, Temperature, READING_BUFFER_SIZE
};

#[cfg(feature = "simulation")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌡️ ESP32-C3 Temperature Monitor (Simulation Mode)");
    println!("==================================================");

    // Initialize the embedded components
    let store: EmbeddedTemperatureStore<READING_BUFFER_SIZE> =
        EmbeddedTemperatureStore::new();
    let mut protocol_handler: EmbeddedProtocolHandler<READING_BUFFER_SIZE> =
        EmbeddedProtocolHandler::new();

    protocol_handler.init(get_boot_timestamp());

    println!("✅ System initialized");
    println!("📊 Buffer capacity: {} readings", store.capacity());
    println!("⚡ Sample rate: {} Hz", temp_embedded::SAMPLE_RATE_HZ);
    println!("💾 Memory usage: ~{} bytes",
             std::mem::size_of_val(&store) + std::mem::size_of_val(&protocol_handler));

    // Demonstrate serde JSON functionality
    println!("\n=== SERDE DEMO: JSON Serialization/Deserialization ===");
    demonstrate_serde_functionality(&mut protocol_handler)?;

    // Simulate temperature monitoring loop
    simulate_monitoring_loop(&mut protocol_handler)?;

    println!("\n🎉 ESP32-C3 simulation completed successfully!");
    Ok(())
}

#[cfg(feature = "simulation")]
fn demonstrate_serde_functionality(
    protocol_handler: &mut EmbeddedProtocolHandler<READING_BUFFER_SIZE>
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Adding sample temperature readings...");

    // Add some sample readings
    let temps = [24.5, 25.2, 26.1];
    for (i, temp) in temps.iter().enumerate() {
        let temperature = Temperature::new(*temp);
        let timestamp = get_current_timestamp() + i as u32;
        protocol_handler.add_reading(temperature, timestamp)?;
    }

    println!("\n📝 Demonstrating command serialization and processing:");

    // Test different commands with JSON serialization
    let commands = [
        ("GetStatus", EmbeddedCommand::GetStatus),
        ("GetStats", EmbeddedCommand::GetStats),
        ("GetLatestReading", EmbeddedCommand::GetLatestReading),
        ("GetReadingCount", EmbeddedCommand::GetReadingCount),
    ];

    for (name, command) in &commands {
        println!("\n🔄 Command: {}", name);

        // Serialize the command to JSON
        let command_json = serde_json::to_string_pretty(command)?;
        println!("  📤 Command JSON:\n{}", command_json);

        // Process the command
        let response = protocol_handler.process_command(command.clone(), get_current_timestamp());

        // Serialize the response to JSON
        let response_json = serde_json::to_string_pretty(&response)?;
        println!("  📥 Response JSON:\n{}", response_json);
    }

    println!("\n✅ Serde demonstration completed!");
    Ok(())
}

#[cfg(feature = "simulation")]
fn simulate_monitoring_loop(
    protocol_handler: &mut EmbeddedProtocolHandler<READING_BUFFER_SIZE>
) -> Result<(), Box<dyn std::error::Error>> {
    use std::thread::sleep;
    use std::time::Duration;

    println!("\n🔄 Starting monitoring loop...");

    let mut reading_count = 0u32;

    // Simulate 50 readings (5 seconds at 10Hz)
    for cycle in 0..50 {
        // Simulate ADC reading from temperature sensor
        let adc_value = simulate_adc_reading(reading_count);
        let temperature = Temperature::from_embedded_sensor(adc_value);
        let timestamp = get_current_timestamp();

        // Add reading to the system
        if let Err(e) = protocol_handler.add_reading(temperature, timestamp) {
            eprintln!("Storage error: {}", e);
        } else {
            reading_count += 1;

            if cycle % 10 == 0 {
                print_status_update(protocol_handler, timestamp, cycle);
            }
        }

        // Wait 100ms to simulate 10Hz sampling
        sleep(Duration::from_millis(100));
    }

    // Final status report
    println!("\n📈 Final System Status:");
    print_final_statistics(protocol_handler);

    Ok(())
}

#[cfg(feature = "simulation")]
fn print_status_update(
    protocol_handler: &mut EmbeddedProtocolHandler<READING_BUFFER_SIZE>,
    timestamp: u32,
    cycle: u32
) {
    println!("\n📊 Status Update (Cycle {}):", cycle);

    // Get system status
    let status_response = protocol_handler.process_command(
        EmbeddedCommand::GetStatus,
        timestamp
    );

    if let EmbeddedResponse::Status {
        uptime_seconds,
        reading_count,
        sample_rate,
        buffer_usage
    } = status_response {
        println!("  ⏱️  Uptime: {}s", uptime_seconds);
        println!("  📊 Readings: {}", reading_count);
        println!("  📈 Sample Rate: {} Hz", sample_rate);
        println!("  💾 Buffer Usage: {}%", buffer_usage);
    }

    // Get latest reading
    let reading_response = protocol_handler.process_command(
        EmbeddedCommand::GetLatestReading,
        timestamp
    );

    if let EmbeddedResponse::Reading(reading) = reading_response {
        println!("  🌡️  Latest: {:.1}°C @ {}s",
                 reading.temperature.celsius, reading.timestamp);
    }
}

#[cfg(feature = "simulation")]
fn print_final_statistics(
    protocol_handler: &mut EmbeddedProtocolHandler<READING_BUFFER_SIZE>
) {
    let timestamp = get_current_timestamp();

    // Get comprehensive statistics
    let stats_response = protocol_handler.process_command(
        EmbeddedCommand::GetStats,
        timestamp
    );

    if let EmbeddedResponse::Stats(stats) = stats_response {
        println!("  🌡️  Temperature Range: {:.1}°C - {:.1}°C",
                 stats.min.celsius, stats.max.celsius);
        println!("  📊 Average Temperature: {:.1}°C", stats.average.celsius);
        println!("  📈 Total Readings: {}", stats.count);
    }

    // Test binary serialization
    let status_response = protocol_handler.process_command(
        EmbeddedCommand::GetStatus,
        timestamp
    );
    let binary_data = protocol_handler.serialize_response(&status_response)
        .unwrap_or_else(|_| heapless::Vec::new());

    println!("  🔧 Binary Protocol: {} bytes per command", binary_data.len());

    // Memory efficiency
    let store_size = std::mem::size_of::<EmbeddedTemperatureStore<READING_BUFFER_SIZE>>();
    let handler_size = std::mem::size_of::<EmbeddedProtocolHandler<READING_BUFFER_SIZE>>();

    println!("  💾 Memory Efficiency:");
    println!("     Store: {} bytes", store_size);
    println!("     Handler: {} bytes", handler_size);
    println!("     Total: {} bytes (fixed allocation)", store_size + handler_size);
}

// Hardware implementation for ESP32-C3
#[cfg(feature = "hardware")]
use esp_hal::{
    clock::CpuClock,
    time::{Duration, Instant},
};

#[cfg(feature = "hardware")]
use esp_println::println as esp_println;

#[cfg(feature = "hardware")]
use serde_json_core;

#[cfg(feature = "hardware")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(feature = "hardware")]
esp_bootloader_esp_idf::esp_app_desc!();

#[cfg(feature = "hardware")]
#[esp_hal::main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize temperature monitoring system
    let mut protocol_handler: EmbeddedProtocolHandler<READING_BUFFER_SIZE> =
        EmbeddedProtocolHandler::new();

    protocol_handler.init(0); // Boot time

    esp_println!("🌡️ ESP32-C3 Temperature Monitor Starting");
    esp_println!("📊 Buffer capacity: {} readings", READING_BUFFER_SIZE);
    esp_println!("⚡ Sample rate: {} Hz", temp_embedded::SAMPLE_RATE_HZ);
    esp_println!("📋 JSON output format: STATUS_JSON, STATS_JSON, READING_JSON");
    esp_println!("🔧 Send JSON commands: {{\"GetStatus\"}}, {{\"GetStats\"}}, {{\"GetLatestReading\"}}");
    esp_println!("=== SERDE DEMO: Processing sample commands ===");

    // Demonstrate serde JSON command parsing and response serialization
    demonstrate_json_commands(&mut protocol_handler, 0);

    esp_println!("=== Starting continuous monitoring ===");

    // For this demo, we'll simulate temperature readings
    // In a real implementation, you would configure ADC or other temperature sensor

    let mut reading_count = 0u32;

    loop {
        // Simulate temperature reading (in real hardware, read from sensor)
        let adc_value = simulate_adc_reading_hardware(reading_count);
        let temperature = Temperature::from_embedded_sensor(adc_value);

        // Get timestamp (simple counter for this demo)
        let timestamp = get_hardware_timestamp();

        // Process reading
        if let Ok(()) = protocol_handler.add_reading(temperature, timestamp) {
            reading_count += 1;

            // Print status every 10 readings
            if reading_count % 10 == 0 {
                // Process status command
                let status_command = EmbeddedCommand::GetStatus;
                let response = protocol_handler.process_command(status_command, timestamp);

                // Serialize status response to JSON
                if let Ok(json_str) = serde_json_core::to_string::<_, 256>(&response) {
                    esp_println!("STATUS_JSON: {}", json_str);
                }

                // Show latest statistics
                let stats_command = EmbeddedCommand::GetStats;
                let stats_response = protocol_handler.process_command(stats_command, timestamp);

                // Serialize stats response to JSON
                if let Ok(json_str) = serde_json_core::to_string::<_, 256>(&stats_response) {
                    esp_println!("STATS_JSON: {}", json_str);
                }

                // Output current temperature reading
                let current_reading = EmbeddedTemperatureReading::new(temperature, timestamp);
                if let Ok(json_str) = serde_json_core::to_string::<_, 256>(&current_reading) {
                    esp_println!("READING_JSON: {}", json_str);
                }
            }
        }

        // Wait according to sample rate (100ms for 10Hz)
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(100) {}
    }
}

#[cfg(feature = "hardware")]
fn demonstrate_json_commands(protocol_handler: &mut EmbeddedProtocolHandler<READING_BUFFER_SIZE>, timestamp: u32) {
    esp_println!("📝 Demonstrating JSON command processing with serde:");

    // Add some sample readings first
    let temp1 = Temperature::from_embedded_sensor(simulate_adc_reading_hardware(0));
    let temp2 = Temperature::from_embedded_sensor(simulate_adc_reading_hardware(10));
    let temp3 = Temperature::from_embedded_sensor(simulate_adc_reading_hardware(20));

    let _ = protocol_handler.add_reading(temp1, timestamp);
    let _ = protocol_handler.add_reading(temp2, timestamp + 1);
    let _ = protocol_handler.add_reading(temp3, timestamp + 2);

    // Demonstrate various commands as JSON strings (as if received from Serial Terminal)
    let json_commands = [
        "GetStatus",
        "GetStats",
        "GetLatestReading",
        "GetReadingCount"
    ];

    for cmd_name in &json_commands {
        esp_println!("🔄 Processing command: {}", cmd_name);

        // Parse the command (in real implementation this would come from serial input)
        let command = match *cmd_name {
            "GetStatus" => EmbeddedCommand::GetStatus,
            "GetStats" => EmbeddedCommand::GetStats,
            "GetLatestReading" => EmbeddedCommand::GetLatestReading,
            "GetReadingCount" => EmbeddedCommand::GetReadingCount,
            _ => continue,
        };

        // Process the command
        let response = protocol_handler.process_command(command, timestamp + 10);

        // Serialize response to JSON using serde
        match serde_json_core::to_string::<_, 512>(&response) {
            Ok(json_response) => {
                esp_println!("✅ JSON Response: {}", json_response);
            }
            Err(_) => {
                esp_println!("❌ Failed to serialize response");
            }
        }
        esp_println!("");
    }
}

// Utility functions that work in both simulation and hardware modes

fn simulate_adc_reading(count: u32) -> u16 {
    // Simulate a temperature sensor that varies sinusoidally
    // Base temperature: 25°C, variation: ±5°C
    let base_temp = 25.0;
    let variation = libm::sinf((count as f32) * 0.1) * 5.0;
    let temp_celsius = base_temp + variation;

    // Convert to 12-bit ADC value
    // Assuming 10mV/°C sensor, 3.3V reference
    let voltage = temp_celsius * 0.01; // 10mV/°C
    let adc_value: f32 = (voltage / 3.3) * 4095.0;
    if adc_value < 0.0 { 0 } else if adc_value > 4095.0 { 4095 } else { adc_value as u16 }
}

#[cfg(feature = "hardware")]
fn simulate_adc_reading_hardware(count: u32) -> u16 {
    // Simulate a temperature sensor for hardware demo
    // Simple linear variation around 25°C
    let base_temp = 25.0;
    let variation = ((count % 100) as f32 / 10.0) - 5.0; // ±5°C variation
    let temp_celsius = base_temp + variation;

    // Convert to 12-bit ADC value
    let voltage = temp_celsius * 0.01; // 10mV/°C sensor
    let adc_value: f32 = (voltage / 3.3) * 4095.0;
    if adc_value < 0.0 { 0 } else if adc_value > 4095.0 { 4095 } else { adc_value as u16 }
}

#[cfg(feature = "simulation")]
fn get_boot_timestamp() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32
}

#[cfg(feature = "simulation")]
fn get_current_timestamp() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as u32
}

#[cfg(feature = "hardware")]
fn get_hardware_timestamp() -> u32 {
    // In hardware implementation, use a simple counter that increments with each call
    // In a real implementation, this could use the system timer or RTC
    static mut COUNTER: u32 = 0;
    unsafe {
        COUNTER += 1;
        COUNTER
    }
}

// Example of how to create a library interface for external use
pub struct ESP32TemperatureMonitor {
    store: EmbeddedTemperatureStore<READING_BUFFER_SIZE>,
    protocol_handler: EmbeddedProtocolHandler<READING_BUFFER_SIZE>,
}

impl ESP32TemperatureMonitor {
    pub fn new() -> Self {
        let mut protocol_handler = EmbeddedProtocolHandler::new();
        protocol_handler.init(0); // Boot timestamp

        Self {
            store: EmbeddedTemperatureStore::new(),
            protocol_handler,
        }
    }

    pub fn add_temperature_reading(&mut self, celsius: f32) -> Result<(), &'static str> {
        let temperature = Temperature::new(celsius);
        let timestamp = self.get_timestamp();
        self.protocol_handler.add_reading(temperature, timestamp)
    }

    pub fn get_status(&mut self) -> EmbeddedResponse {
        let timestamp = self.get_timestamp();
        self.protocol_handler.process_command(EmbeddedCommand::GetStatus, timestamp)
    }

    pub fn get_statistics(&mut self) -> EmbeddedResponse {
        let timestamp = self.get_timestamp();
        self.protocol_handler.process_command(EmbeddedCommand::GetStats, timestamp)
    }

    fn get_timestamp(&self) -> u32 {
        #[cfg(feature = "simulation")]
        {
            get_current_timestamp()
        }
        #[cfg(feature = "hardware")]
        {
            get_hardware_timestamp()
        }
    }
}

impl Default for ESP32TemperatureMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_esp32_monitor_creation() {
        let monitor = ESP32TemperatureMonitor::new();

        // Test that the monitor is properly initialized
        assert_eq!(monitor.store.len(), 0);
        assert_eq!(monitor.store.capacity(), READING_BUFFER_SIZE);
    }

    #[test]
    fn test_temperature_reading() {
        let mut monitor = ESP32TemperatureMonitor::new();

        // Add a temperature reading
        let result = monitor.add_temperature_reading(25.5);
        assert!(result.is_ok());

        // Check status includes the reading
        let status = monitor.get_status();
        if let EmbeddedResponse::Status { reading_count, .. } = status {
            assert_eq!(reading_count, 1);
        } else {
            panic!("Expected status response");
        }
    }

    #[test]
    fn test_statistics() {
        let mut monitor = ESP32TemperatureMonitor::new();

        // Add multiple readings
        monitor.add_temperature_reading(20.0).unwrap();
        monitor.add_temperature_reading(25.0).unwrap();
        monitor.add_temperature_reading(30.0).unwrap();

        // Get statistics
        let stats = monitor.get_statistics();
        if let EmbeddedResponse::Stats(stats) = stats {
            assert_eq!(stats.min.celsius, 20.0);
            assert_eq!(stats.max.celsius, 30.0);
            assert_eq!(stats.average.celsius, 25.0);
            assert_eq!(stats.count, 3);
        } else {
            panic!("Expected stats response");
        }
    }

    #[test]
    fn test_adc_simulation() {
        // Test that ADC simulation produces reasonable values
        for i in 0..100 {
            let adc_value = simulate_adc_reading(i);
            // ADC should be in 12-bit range
            assert!(adc_value <= 4095);

            // Convert back to temperature to verify range
            let temp = Temperature::from_embedded_sensor(adc_value);
            // Should be roughly 20-30°C for our simulation
            assert!(temp.celsius >= 18.0 && temp.celsius <= 32.0);
        }
    }
}