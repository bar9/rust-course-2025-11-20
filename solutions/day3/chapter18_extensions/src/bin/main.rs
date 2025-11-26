#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
// Removed ESP temperature sensor imports - using mock sensor

// Use the enhanced system components
use chapter18_extensions::{Temperature, TemperatureBuffer, Command, TemperatureComm};

// System configuration constants
const BUFFER_SIZE: usize = 32;
const SAMPLE_RATE_MS: u32 = 1000;
const JSON_OUTPUT_INTERVAL: u32 = 5;
const HEALTH_REPORT_INTERVAL: u32 = 20;
const OVERHEATING_THRESHOLD: f32 = 35.0;

// Mock Temperature Sensor with realistic simulation (Chapter 18 enhancement)
struct MockTemperatureSensor {
    reading_count: u32,
    base_temperature: f32,
}

impl MockTemperatureSensor {
    fn new() -> Self {
        Self {
            reading_count: 0,
            base_temperature: 22.5,
        }
    }

    fn read_celsius(&mut self) -> f32 {
        self.reading_count += 1;

        // Simulate realistic temperature variation (no_std compatible)
        let time_factor = (self.reading_count % 100) as f32 * 0.1;
        let variation = if self.reading_count % 4 == 0 {
            1.0
        } else if self.reading_count % 4 == 1 {
            0.5
        } else if self.reading_count % 4 == 2 {
            -0.5
        } else {
            -1.0
        } * 2.0;

        // Occasionally simulate higher temps for testing (every 50 readings)
        let spike = if self.reading_count % 50 == 0 { 15.0 } else { 0.0 };

        self.base_temperature + variation + spike
    }

    fn set_base_temperature(&mut self, temp: f32) {
        self.base_temperature = temp;
    }
}

// Enhanced system state tracking (Chapter 18)
struct SystemState {
    reading_count: u32,
    system_time_ms: u32,
    overheating_count: u32,
    sensor_error_count: u32,
    last_temp: f32,
    start_time: Instant,
    current_threshold: f32,
    adaptive_sample_rate: u32,
    command_count: u32,
}

impl SystemState {
    fn new() -> Self {
        Self {
            reading_count: 0,
            system_time_ms: 0,
            overheating_count: 0,
            sensor_error_count: 0,
            last_temp: 0.0,
            start_time: Instant::now(),
            current_threshold: OVERHEATING_THRESHOLD,
            adaptive_sample_rate: SAMPLE_RATE_MS,
            command_count: 0,
        }
    }

    fn advance_time(&mut self) {
        self.reading_count += 1;
        self.system_time_ms += self.adaptive_sample_rate;
    }

    fn record_overheating(&mut self) {
        self.overheating_count += 1;
    }

    fn record_sensor_error(&mut self) {
        self.sensor_error_count += 1;
    }

    fn uptime_seconds(&self) -> u32 {
        self.system_time_ms / 1000
    }

    fn process_command(&mut self, command: &Command) -> bool {
        self.command_count += 1;
        match command {
            Command::SetThreshold { threshold_celsius } => {
                if *threshold_celsius > 0.0 && *threshold_celsius < 100.0 {
                    self.current_threshold = *threshold_celsius;
                    true
                } else {
                    false
                }
            }
            Command::SetSampleRate { rate_hz } => {
                if *rate_hz > 0 && *rate_hz <= 10 {
                    self.adaptive_sample_rate = 1000 / (*rate_hz as u32);
                    true
                } else {
                    false
                }
            }
            Command::Reset => {
                self.overheating_count = 0;
                self.sensor_error_count = 0;
                self.command_count = 0;
                self.current_threshold = OVERHEATING_THRESHOLD;
                self.adaptive_sample_rate = SAMPLE_RATE_MS;
                true
            }
            _ => true, // Other commands are read-only
        }
    }

    fn is_overheating(&self, temp: f32) -> bool {
        temp > self.current_threshold
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    esp_println::println!("💥 SYSTEM PANIC: {}", info);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    // === SYSTEM INITIALIZATION ===
    esp_println::println!("🌡️ ESP32-C3 Complete Temperature Monitor System");
    esp_println::println!("=================================================");
    esp_println::println!("🆕 Chapter 18: Enhanced System with Extensions");

    // Initialize hardware
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // GPIO setup
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    // Initialize mock temperature sensor with realistic simulation (Chapter 18 enhancement)
    let mut temp_sensor = MockTemperatureSensor::new();

    // System components
    let mut temp_buffer = TemperatureBuffer::<BUFFER_SIZE>::new();
    let mut comm = TemperatureComm::new();
    let mut system_state = SystemState::new();

    // Initialize communication handler
    comm.init(0);

    // System startup information
    esp_println::println!("🔧 Hardware: ESP32-C3 @ max frequency");
    esp_println::println!("📊 Buffer capacity: {} readings", BUFFER_SIZE);
    esp_println::println!("⏱️  Initial sample rate: {} Hz (adaptive)", 1000 / SAMPLE_RATE_MS);
    esp_println::println!("🌡️ Initial overheating threshold: {:.1}°C (configurable)", OVERHEATING_THRESHOLD);
    esp_println::println!("📡 JSON output every {} readings", JSON_OUTPUT_INTERVAL);
    esp_println::println!("💓 Health reports every {} readings", HEALTH_REPORT_INTERVAL);
    esp_println::println!("🎮 Command processing: threshold, sample rate, reset");
    esp_println::println!("🧪 Mock sensor: realistic temperature simulation");
    esp_println::println!("🚀 Enhanced system starting...");
    esp_println::println!();

    // Initial system status
    let initial_status = comm.status_json(&temp_buffer, 0);
    esp_println::println!("INITIAL_STATUS: {}", initial_status);
    esp_println::println!();

    // === MAIN SYSTEM LOOP ===
    loop {
        // STEP 1: Read temperature from mock sensor (Chapter 18)
        let celsius = temp_sensor.read_celsius();
        system_state.last_temp = celsius;

        let temperature = Temperature::from_celsius(celsius);
        temp_buffer.push(temperature);
        system_state.advance_time();

        // STEP 2: LED feedback with enhanced patterns
        update_led_status(&mut led, &temperature, &system_state);

        // STEP 3: Track overheating events (with configurable threshold)
        if system_state.is_overheating(celsius) {
            system_state.record_overheating();
        }

        // STEP 4: Console output with status indicators
        let status_icon = get_status_icon(&temperature, &system_state);
        esp_println::println!("{}📊 #{:03} | {:.1}°C | Buffer: {}/{}",
            status_icon,
            system_state.reading_count,
            celsius,
            temp_buffer.len(),
            BUFFER_SIZE
        );

        // STEP 5: JSON data output at regular intervals
        if system_state.reading_count % JSON_OUTPUT_INTERVAL == 0 {
            output_json_data(&mut comm, &temp_buffer, &system_state);
        }

        // STEP 6: Health monitoring and reporting
        if system_state.reading_count % HEALTH_REPORT_INTERVAL == 0 {
            output_health_report(&system_state, &temp_buffer);
        }

        // STEP 7: Error condition management
        handle_error_conditions(&system_state);

        // STEP 8: System demonstration features
        if system_state.reading_count % 50 == 0 {
            demonstrate_system_integration(&mut comm, &temp_buffer, &system_state);
        }

        // STEP 8a: Command processing demonstration (Chapter 18)
        if system_state.reading_count % 30 == 0 {
            demonstrate_command_processing(&mut system_state, &mut temp_sensor, &mut comm, &temp_buffer);
        }

        // STEP 9: Adaptive timing control (Chapter 18)
        let wait_start = Instant::now();
        while wait_start.elapsed() < Duration::from_millis(system_state.adaptive_sample_rate as u64) {}
    }
}

// Mock sensor doesn't need stabilization - removed read_temperature_safe

fn update_led_status(led: &mut Output, _temperature: &Temperature, state: &SystemState) {
    if state.is_overheating(state.last_temp) {
        // Rapid blink for overheating (configurable threshold)
        led.set_high();
    } else if state.command_count > 0 && state.reading_count % 3 == 0 {
        // Fast blink pattern when commands are being processed
        led.toggle();
    } else if state.reading_count % 10 == 0 {
        // Slow heartbeat blink for normal operation
        led.toggle();
    }
}

fn get_status_icon(temperature: &Temperature, state: &SystemState) -> &'static str {
    if state.is_overheating(state.last_temp) {
        "🔴"
    } else if state.command_count > 10 {
        "🟣" // Purple for active command processing
    } else if temperature.is_normal_range() {
        "🟢"
    } else {
        "🔵"
    }
}

fn output_json_data(comm: &mut TemperatureComm, buffer: &TemperatureBuffer<BUFFER_SIZE>, state: &SystemState) {
    esp_println::println!("\n--- JSON OUTPUT ---");

    // Current reading
    let reading_json = comm.reading_json(buffer, state.system_time_ms);
    esp_println::println!("READING: {}", reading_json);

    // Statistics
    let stats_command = comm.process_command(Command::GetStats, buffer, state.system_time_ms);
    if let Ok(stats_resp) = comm.response_to_json(&stats_command) {
        esp_println::println!("STATS: {}", stats_resp);
    }

    // System status
    let status_json = comm.status_json(buffer, state.system_time_ms);
    esp_println::println!("STATUS: {}", status_json);

    esp_println::println!("--- END JSON ---\n");
}

fn output_health_report(state: &SystemState, buffer: &TemperatureBuffer<BUFFER_SIZE>) {
    let uptime = state.uptime_seconds();
    let buffer_usage_pct = (buffer.len() * 100) / BUFFER_SIZE;
    let memory_usage = core::mem::size_of::<TemperatureBuffer<BUFFER_SIZE>>();

    esp_println::println!("💓 ENHANCED HEALTH REPORT (Chapter 18)");
    esp_println::println!("  Uptime: {}s | Readings: {}", uptime, state.reading_count);
    esp_println::println!("  Buffer: {}% ({}/{}) | Memory: {} bytes",
        buffer_usage_pct, buffer.len(), BUFFER_SIZE, memory_usage);
    esp_println::println!("  Overheating events: {} (threshold: {:.1}°C)",
        state.overheating_count, state.current_threshold);
    esp_println::println!("  Current temp: {:.1}°C | Sample rate: {} ms",
        state.last_temp, state.adaptive_sample_rate);
    esp_println::println!("  Commands processed: {}", state.command_count);

    if buffer.len() >= BUFFER_SIZE {
        esp_println::println!("  ℹ️  Buffer full - circular mode active");
    }
    esp_println::println!();
}

fn handle_error_conditions(state: &SystemState) {
    if state.overheating_count >= 5 {
        esp_println::println!("🚨 WARNING: {} overheating events detected! System requires attention.",
            state.overheating_count);
    }

    if state.sensor_error_count >= 3 {
        esp_println::println!("⚠️  ALERT: {} sensor errors detected. Check sensor connection.",
            state.sensor_error_count);
    }
}

fn demonstrate_system_integration(
    comm: &mut TemperatureComm,
    buffer: &TemperatureBuffer<BUFFER_SIZE>,
    state: &SystemState
) {
    esp_println::println!("🔧 SYSTEM INTEGRATION DEMO");
    esp_println::println!("  Testing complete system functionality...");

    // Test all communication features
    let commands = [
        Command::GetStatus,
        Command::GetLatestReading,
        Command::GetStats,
    ];

    for command in commands {
        let response = comm.process_command(command, buffer, state.system_time_ms);
        if let Ok(json) = comm.response_to_json(&response) {
            esp_println::println!("  ✅ Command processed: {}", json.len());
        }
    }

    // System performance info
    esp_println::println!("  📈 Performance: {} Hz stable, {} total readings",
        1000 / SAMPLE_RATE_MS, state.reading_count);
    esp_println::println!("  🔧 Integration test complete\n");
}

// Chapter 18: Enhanced command processing demonstration
fn demonstrate_command_processing(
    state: &mut SystemState,
    sensor: &mut MockTemperatureSensor,
    comm: &mut TemperatureComm,
    buffer: &TemperatureBuffer<BUFFER_SIZE>
) {
    esp_println::println!("🎮 ENHANCED COMMAND PROCESSING DEMO (Chapter 18)");

    // Simulate different commands being received
    let commands = [
        Command::SetThreshold { threshold_celsius: 30.0 },
        Command::SetSampleRate { rate_hz: 2 },
        Command::GetStatus,
        Command::Reset,
    ];

    for command in commands {
        esp_println::println!("  📥 Processing: {:?}", command);

        // Process command with enhanced system state
        let success = state.process_command(&command);
        if success {
            esp_println::println!("  ✅ Command executed successfully");

            // Show the effect of the command
            match command {
                Command::SetThreshold { threshold_celsius } => {
                    esp_println::println!("  🌡️ Threshold updated to {:.1}°C", threshold_celsius);
                }
                Command::SetSampleRate { rate_hz } => {
                    esp_println::println!("  ⏱️  Sample rate updated to {} Hz", rate_hz);
                }
                Command::Reset => {
                    esp_println::println!("  🔄 System reset complete");
                    sensor.set_base_temperature(22.5); // Reset sensor too
                }
                _ => {}
            }
        } else {
            esp_println::println!("  ❌ Command failed - invalid parameters");
        }

        // Show updated system status
        let response = comm.process_command(Command::GetStatus, buffer, state.system_time_ms);
        if let Ok(json) = comm.response_to_json(&response) {
            esp_println::println!("  📊 Status: {}", json);
        }
    }

    esp_println::println!("  🎯 Command demo complete - {} total commands processed",
        state.command_count);
    esp_println::println!();
}
