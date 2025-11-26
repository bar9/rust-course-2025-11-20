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
use esp_hal::tsens::{Config, TemperatureSensor};

// Use the integrated system components
use chapter17_integration::{Temperature, TemperatureBuffer, Command, TemperatureComm};

// System configuration constants
const BUFFER_SIZE: usize = 32;
const SAMPLE_RATE_MS: u32 = 1000;
const JSON_OUTPUT_INTERVAL: u32 = 5;
const HEALTH_REPORT_INTERVAL: u32 = 20;
const OVERHEATING_THRESHOLD: f32 = 35.0;

// System state tracking
struct SystemState {
    reading_count: u32,
    system_time_ms: u32,
    overheating_count: u32,
    sensor_error_count: u32,
    last_temp: f32,
    start_time: Instant,
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
        }
    }

    fn advance_time(&mut self) {
        self.reading_count += 1;
        self.system_time_ms += SAMPLE_RATE_MS;
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

    // Initialize hardware
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // GPIO setup
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    // Initialize temperature sensor with error handling
    let temp_sensor = match TemperatureSensor::new(peripherals.TSENS, Config::default()) {
        Ok(sensor) => sensor,
        Err(_) => {
            esp_println::println!("❌ Failed to initialize temperature sensor");
            loop {}
        }
    };

    // System components
    let mut temp_buffer = TemperatureBuffer::<BUFFER_SIZE>::new();
    let mut comm = TemperatureComm::new();
    let mut system_state = SystemState::new();

    // Initialize communication handler
    comm.init(0);

    // System startup information
    esp_println::println!("🔧 Hardware: ESP32-C3 @ max frequency");
    esp_println::println!("📊 Buffer capacity: {} readings", BUFFER_SIZE);
    esp_println::println!("⏱️  Sample rate: {} Hz", 1000 / SAMPLE_RATE_MS);
    esp_println::println!("🌡️ Overheating threshold: {:.1}°C", OVERHEATING_THRESHOLD);
    esp_println::println!("📡 JSON output every {} readings", JSON_OUTPUT_INTERVAL);
    esp_println::println!("💓 Health reports every {} readings", HEALTH_REPORT_INTERVAL);
    esp_println::println!("🚀 System starting...");
    esp_println::println!();

    // Initial system status
    let initial_status = comm.status_json(&temp_buffer, 0);
    esp_println::println!("INITIAL_STATUS: {}", initial_status);
    esp_println::println!();

    // === MAIN SYSTEM LOOP ===
    loop {
        // STEP 1: Read temperature with error handling
        let celsius = match read_temperature_safe(&temp_sensor) {
            Ok(temp) => {
                system_state.last_temp = temp;
                temp
            }
            Err(_) => {
                system_state.record_sensor_error();
                esp_println::println!("❌ Sensor error #{}, using last value: {:.1}°C",
                    system_state.sensor_error_count, system_state.last_temp);
                system_state.last_temp // Use last known good value
            }
        };

        let temperature = Temperature::from_celsius(celsius);
        temp_buffer.push(temperature);
        system_state.advance_time();

        // STEP 2: LED feedback with enhanced patterns
        update_led_status(&mut led, &temperature, &system_state);

        // STEP 3: Track overheating events
        if temperature.is_overheating() {
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

        // STEP 9: Precise timing control
        let wait_start = Instant::now();
        while wait_start.elapsed() < Duration::from_millis(SAMPLE_RATE_MS as u64) {}
    }
}

fn read_temperature_safe(sensor: &TemperatureSensor) -> Result<f32, ()> {
    // Small stabilization delay
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_micros(200) {}

    let esp_temperature = sensor.get_temperature();
    Ok(esp_temperature.to_celsius())
}

fn update_led_status(led: &mut Output, temperature: &Temperature, state: &SystemState) {
    if temperature.is_overheating() {
        // Rapid blink for overheating
        led.set_high();
    } else if state.sensor_error_count > 0 && state.reading_count % 4 == 0 {
        // Fast blink pattern for sensor errors
        led.toggle();
    } else if state.reading_count % 10 == 0 {
        // Slow heartbeat blink for normal operation
        led.toggle();
    }
}

fn get_status_icon(temperature: &Temperature, state: &SystemState) -> &'static str {
    if temperature.is_overheating() {
        "🔴"
    } else if state.sensor_error_count > 0 {
        "🟡"
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

    esp_println::println!("💓 HEALTH REPORT");
    esp_println::println!("  Uptime: {}s | Readings: {}", uptime, state.reading_count);
    esp_println::println!("  Buffer: {}% ({}/{}) | Memory: {} bytes",
        buffer_usage_pct, buffer.len(), BUFFER_SIZE, memory_usage);
    esp_println::println!("  Errors: {} sensor, {} overheating events",
        state.sensor_error_count, state.overheating_count);
    esp_println::println!("  Current temp: {:.1}°C", state.last_temp);

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
