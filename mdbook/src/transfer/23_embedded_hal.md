# Chapter 23: Embedded HAL - Hardware Register Access & Volatile Memory

## Learning Objectives
- Understand why volatile access is critical for memory-mapped I/O
- Master raw register manipulation with `ptr::read_volatile` and `ptr::write_volatile`
- Use SVD2Rust to generate type-safe peripheral access crates
- Implement safe abstractions over unsafe hardware access
- Apply embedded HAL traits for portable embedded code
- Build drivers that work across different microcontrollers

## Part 1: Why Volatile Access Matters

### The Compiler Optimization Problem

When you write to a memory location in regular code, the compiler assumes it has complete control over that memory. It can optimize away "redundant" reads and writes:

```rust
// Regular memory access - compiler can optimize
fn regular_memory() {
    let mut value = 0u32;

    value = 1;  // Compiler might optimize away
    value = 2;  // Only this write matters
    value = 3;  // And this one

    let x = value;  // Reads 3
    let y = value;  // Compiler might reuse x instead of reading again
}
```

But hardware registers are different. They're not regular memory - they're windows into hardware state:

```rust
// Hardware register at address 0x4000_0000
const GPIO_OUT: *mut u32 = 0x4000_0000 as *mut u32;

unsafe fn bad_gpio_control() {
    // ❌ WRONG: Compiler might optimize these away!
    *GPIO_OUT = 0b0001;  // Turn on LED 1
    *GPIO_OUT = 0b0010;  // Turn on LED 2
    *GPIO_OUT = 0b0100;  // Turn on LED 3

    // Compiler might only emit the last write!
}

unsafe fn good_gpio_control() {
    use core::ptr;

    // ✅ CORRECT: Volatile writes are never optimized away
    ptr::write_volatile(GPIO_OUT, 0b0001);  // Turn on LED 1
    ptr::write_volatile(GPIO_OUT, 0b0010);  // Turn on LED 2
    ptr::write_volatile(GPIO_OUT, 0b0100);  // Turn on LED 3

    // All three writes will happen!
}
```

### Memory-Mapped I/O Fundamentals

In embedded systems, hardware peripherals appear as memory addresses:

```rust
// ESP32-C3 GPIO registers (simplified)
const GPIO_BASE: usize = 0x6000_4000;

// GPIO output registers
const GPIO_OUT_W1TS: *mut u32 = (GPIO_BASE + 0x0008) as *mut u32;  // Set bits
const GPIO_OUT_W1TC: *mut u32 = (GPIO_BASE + 0x000C) as *mut u32;  // Clear bits
const GPIO_OUT: *mut u32 = (GPIO_BASE + 0x0004) as *mut u32;       // Direct write
const GPIO_IN: *const u32 = (GPIO_BASE + 0x003C) as *const u32;    // Read input

unsafe fn control_gpio() {
    use core::ptr;

    // Set pin 5 high (write 1 to set)
    ptr::write_volatile(GPIO_OUT_W1TS, 1 << 5);

    // Clear pin 5 (write 1 to clear - yes, really!)
    ptr::write_volatile(GPIO_OUT_W1TC, 1 << 5);

    // Read current pin states
    let pins = ptr::read_volatile(GPIO_IN);
    let pin5_state = (pins >> 5) & 1;
}
```

### Why Each Access Must Be Volatile

Hardware registers can change at any time due to:
- External signals (button presses, sensor readings)
- Hardware state machines (timers, DMA completion)
- Interrupt handlers modifying registers
- Peripheral operations completing

```rust
// Timer register that counts up automatically
const TIMER_COUNTER: *const u32 = 0x6002_0000 as *const u32;

unsafe fn wait_for_timeout() {
    use core::ptr;

    // ❌ WRONG: Compiler might read once and cache
    let start = *TIMER_COUNTER;
    while *TIMER_COUNTER - start < 1000 {
        // Might become infinite loop if compiler optimizes!
    }

    // ✅ CORRECT: Each read goes to hardware
    let start = ptr::read_volatile(TIMER_COUNTER);
    while ptr::read_volatile(TIMER_COUNTER) - start < 1000 {
        // Will actually read the changing timer value
    }
}
```

## Part 2: Raw Register Access Patterns

### Basic Volatile Operations

```rust
use core::ptr;

/// Safe wrapper for a hardware register
pub struct Register<T> {
    addr: *mut T,
}

impl<T> Register<T> {
    /// Create a new register at the given address
    pub const fn new(addr: usize) -> Self {
        Self {
            addr: addr as *mut T,
        }
    }

    /// Read the current value (volatile)
    pub unsafe fn read(&self) -> T
    where
        T: Copy,
    {
        ptr::read_volatile(self.addr)
    }

    /// Write a new value (volatile)
    pub unsafe fn write(&self, value: T) {
        ptr::write_volatile(self.addr, value);
    }

    /// Modify the register with read-modify-write
    pub unsafe fn modify<F>(&self, f: F)
    where
        T: Copy,
        F: FnOnce(T) -> T,
    {
        let value = self.read();
        self.write(f(value));
    }
}

// Usage example
const PORTA_OUT: Register<u32> = Register::new(0x4000_0000);

unsafe fn toggle_pin(pin: u8) {
    PORTA_OUT.modify(|v| v ^ (1 << pin));
}
```

### Register Types and Access Patterns

Different registers have different access rules:

```rust
/// Read-only register
pub struct ReadOnly<T> {
    addr: *const T,
}

impl<T> ReadOnly<T> {
    pub const fn new(addr: usize) -> Self {
        Self { addr: addr as *const T }
    }

    pub unsafe fn read(&self) -> T
    where T: Copy
    {
        ptr::read_volatile(self.addr)
    }
}

/// Write-only register
pub struct WriteOnly<T> {
    addr: *mut T,
}

impl<T> WriteOnly<T> {
    pub const fn new(addr: usize) -> Self {
        Self { addr: addr as *mut T }
    }

    pub unsafe fn write(&self, value: T) {
        ptr::write_volatile(self.addr, value);
    }
}

/// Read-write register
pub struct ReadWrite<T> {
    addr: *mut T,
}

impl<T> ReadWrite<T> {
    pub const fn new(addr: usize) -> Self {
        Self { addr: addr as *mut T }
    }

    pub unsafe fn read(&self) -> T where T: Copy {
        ptr::read_volatile(self.addr)
    }

    pub unsafe fn write(&self, value: T) {
        ptr::write_volatile(self.addr, value);
    }

    pub unsafe fn modify<F>(&self, f: F)
    where
        T: Copy,
        F: FnOnce(T) -> T,
    {
        self.write(f(self.read()));
    }
}
```

### Bitfield Manipulation

Most hardware registers pack multiple fields into single words:

```rust
/// GPIO Configuration Register Layout (ESP32-C3 example)
/// Bits 0-1:   Drive strength
/// Bits 2-3:   Pin function
/// Bit 4:      Pull-up enable
/// Bit 5:      Pull-down enable
/// Bit 6:      Input enable
/// Bit 7:      Output enable

pub struct GpioConfig {
    reg: ReadWrite<u32>,
}

impl GpioConfig {
    const DRIVE_MASK: u32 = 0b11;
    const DRIVE_SHIFT: u32 = 0;

    const FUNC_MASK: u32 = 0b11;
    const FUNC_SHIFT: u32 = 2;

    const PULLUP_BIT: u32 = 4;
    const PULLDOWN_BIT: u32 = 5;
    const INPUT_BIT: u32 = 6;
    const OUTPUT_BIT: u32 = 7;

    pub unsafe fn set_drive_strength(&self, strength: u8) {
        self.reg.modify(|v| {
            (v & !(Self::DRIVE_MASK << Self::DRIVE_SHIFT))
                | ((strength as u32 & Self::DRIVE_MASK) << Self::DRIVE_SHIFT)
        });
    }

    pub unsafe fn enable_pullup(&self, enable: bool) {
        self.reg.modify(|v| {
            if enable {
                v | (1 << Self::PULLUP_BIT)
            } else {
                v & !(1 << Self::PULLUP_BIT)
            }
        });
    }

    pub unsafe fn set_as_output(&self) {
        self.reg.modify(|v| {
            v | (1 << Self::OUTPUT_BIT) | (1 << Self::INPUT_BIT)
        });
    }
}
```

## Part 3: SVD Files and Code Generation

### What is SVD?

SVD (System View Description) files are XML descriptions of microcontroller peripherals. Manufacturers provide these to describe:
- Memory map layout
- Peripheral registers
- Register fields and bits
- Access permissions
- Reset values

Example SVD snippet:
```xml
<peripheral>
    <name>GPIO</name>
    <baseAddress>0x60004000</baseAddress>
    <registers>
        <register>
            <name>OUT</name>
            <addressOffset>0x0004</addressOffset>
            <description>GPIO output register</description>
            <access>read-write</access>
            <fields>
                <field>
                    <name>DATA</name>
                    <bitRange>[31:0]</bitRange>
                </field>
            </fields>
        </register>
    </registers>
</peripheral>
```

### SVD2Rust Workflow

1. **Get the SVD file** from your chip manufacturer
2. **Install svd2rust**:
   ```bash
   cargo install svd2rust
   cargo install form
   ```

3. **Generate the PAC** (Peripheral Access Crate):
   ```bash
   svd2rust -i esp32c3.svd --target riscv
   form -i lib.rs -o src/
   cargo fmt
   ```

4. **Use the generated code**:

```rust
// Generated code provides type-safe register access
use esp32c3_pac::GPIO;

fn configure_gpio(gpio: &GPIO) {
    // Type-safe register access
    gpio.out_w1ts.write(|w| unsafe { w.bits(1 << 5) });

    // Named fields with documentation
    gpio.func_out_sel_cfg[5].write(|w| {
        w.out_sel().variant(0x80)  // Connect to GPIO matrix
    });
}
```

### Generated Code Structure

SVD2Rust generates a hierarchy of types:

```rust
// Peripheral block
pub struct GPIO {
    pub bt_select: BT_SELECT,
    pub out: OUT,
    pub out_w1ts: OUT_W1TS,
    pub out_w1tc: OUT_W1TC,
    // ... more registers
}

// Register
pub struct OUT {
    register: vcell::VolatileCell<u32>,
}

impl OUT {
    // Read access
    pub fn read(&self) -> R {
        R { bits: self.register.get() }
    }

    // Write access
    pub fn write<F>(&self, f: F)
    where
        F: FnOnce(&mut W) -> &mut W,
    {
        let mut w = W::reset_value();
        f(&mut w);
        self.register.set(w.bits);
    }

    // Modify (read-modify-write)
    pub fn modify<F>(&self, f: F)
    where
        for<'w> F: FnOnce(&R, &'w mut W) -> &'w mut W,
    {
        let bits = self.register.get();
        let r = R { bits };
        let mut w = W { bits };
        f(&r, &mut w);
        self.register.set(w.bits);
    }
}
```

## Part 4: Building Safe Abstractions

### The Ownership Pattern for Peripherals

Peripherals should have single ownership to prevent conflicts:

```rust
/// Singleton peripherals structure
pub struct Peripherals {
    pub GPIO: GPIO,
    pub UART0: UART0,
    pub SPI1: SPI1,
    pub TIMER0: TIMER0,
    // ... more peripherals
}

impl Peripherals {
    /// Take ownership of peripherals (can only be called once)
    pub fn take() -> Option<Self> {
        static mut TAKEN: bool = false;

        // This is safe because we're in single-threaded context
        // and we only allow one caller to succeed
        if unsafe { TAKEN } {
            None
        } else {
            unsafe { TAKEN = true; }

            Some(Peripherals {
                GPIO: GPIO { _private: () },
                UART0: UART0 { _private: () },
                SPI1: SPI1 { _private: () },
                TIMER0: TIMER0 { _private: () },
            })
        }
    }
}

// Usage
fn main() {
    let peripherals = Peripherals::take().unwrap();
    let gpio = peripherals.GPIO;  // Now we own GPIO

    // let peripherals2 = Peripherals::take();  // Returns None!
}
```

### Type-State Programming for Hardware Configuration

Use types to enforce correct hardware state transitions:

```rust
/// Pin states
pub struct Input;
pub struct Output;
pub struct Analog;

/// Pin with compile-time state
pub struct Pin<MODE> {
    pin_number: u8,
    _mode: core::marker::PhantomData<MODE>,
}

impl Pin<Input> {
    /// Read the pin state
    pub fn is_high(&self) -> bool {
        unsafe {
            let gpio = &*GPIO::ptr();
            let value = ptr::read_volatile(&gpio.in_);
            (value >> self.pin_number) & 1 == 1
        }
    }

    /// Convert to output pin
    pub fn into_output(self) -> Pin<Output> {
        unsafe {
            // Configure hardware for output
            let gpio = &*GPIO::ptr();
            gpio.enable_w1ts.write(|w| w.bits(1 << self.pin_number));
        }

        Pin {
            pin_number: self.pin_number,
            _mode: core::marker::PhantomData,
        }
    }
}

impl Pin<Output> {
    /// Set pin high
    pub fn set_high(&mut self) {
        unsafe {
            let gpio = &*GPIO::ptr();
            ptr::write_volatile(
                &gpio.out_w1ts as *const _ as *mut u32,
                1 << self.pin_number
            );
        }
    }

    /// Set pin low
    pub fn set_low(&mut self) {
        unsafe {
            let gpio = &*GPIO::ptr();
            ptr::write_volatile(
                &gpio.out_w1tc as *const _ as *mut u32,
                1 << self.pin_number
            );
        }
    }

    /// Convert to input pin
    pub fn into_input(self) -> Pin<Input> {
        unsafe {
            // Configure hardware for input
            let gpio = &*GPIO::ptr();
            gpio.enable_w1tc.write(|w| w.bits(1 << self.pin_number));
        }

        Pin {
            pin_number: self.pin_number,
            _mode: core::marker::PhantomData,
        }
    }
}

// Usage - compile-time safety!
fn demo() {
    let pin = Pin::<Input>::new(5);
    let _state = pin.is_high();  // OK: can read input
    // pin.set_high();  // Compile error: no such method for Input!

    let mut pin = pin.into_output();  // Transform to output
    pin.set_high();  // Now OK!
    // let state = pin.is_high();  // Compile error: no such method for Output!
}
```

## Part 5: Embedded HAL Traits

### The embedded-hal Ecosystem

The `embedded-hal` crate defines standard traits that work across all microcontrollers:

```rust
use embedded_hal::digital::v2::{OutputPin, InputPin};
use embedded_hal::blocking::spi::{Write, Transfer};
use embedded_hal::blocking::i2c::{Read, Write as I2cWrite};

/// Driver that works with any microcontroller!
pub struct LedDriver<P: OutputPin> {
    pin: P,
}

impl<P: OutputPin> LedDriver<P> {
    pub fn new(pin: P) -> Self {
        LedDriver { pin }
    }

    pub fn on(&mut self) -> Result<(), P::Error> {
        self.pin.set_high()
    }

    pub fn off(&mut self) -> Result<(), P::Error> {
        self.pin.set_low()
    }
}

/// SPI device driver
pub struct SpiDevice<SPI> {
    spi: SPI,
}

impl<SPI> SpiDevice<SPI>
where
    SPI: Write<u8>,
{
    pub fn send_command(&mut self, cmd: u8) -> Result<(), SPI::Error> {
        self.spi.write(&[cmd])
    }

    pub fn send_data(&mut self, data: &[u8]) -> Result<(), SPI::Error> {
        self.spi.write(data)
    }
}
```

### Implementing HAL Traits

Here's how to implement embedded-hal traits for your hardware:

```rust
use embedded_hal::digital::v2::{OutputPin, InputPin};

impl OutputPin for Pin<Output> {
    type Error = core::convert::Infallible;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe {
            ptr::write_volatile(GPIO_OUT_W1TS, 1 << self.pin_number);
        }
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe {
            ptr::write_volatile(GPIO_OUT_W1TC, 1 << self.pin_number);
        }
        Ok(())
    }
}

impl InputPin for Pin<Input> {
    type Error = core::convert::Infallible;

    fn is_high(&self) -> Result<bool, Self::Error> {
        unsafe {
            let value = ptr::read_volatile(GPIO_IN);
            Ok((value >> self.pin_number) & 1 == 1)
        }
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.is_high().map(|h| !h)
    }
}
```

## Part 6: Real-World Examples

### Example 1: Interrupt Controller with Volatile Access

```rust
/// ESP32-C3 Interrupt Controller
const INTERRUPT_BASE: usize = 0x600C_2000;

pub struct InterruptController {
    /// Interrupt enable register
    enable: ReadWrite<u32>,
    /// Interrupt pending register (read-only)
    pending: ReadOnly<u32>,
    /// Interrupt clear register (write-only)
    clear: WriteOnly<u32>,
}

impl InterruptController {
    pub const fn new() -> Self {
        Self {
            enable: ReadWrite::new(INTERRUPT_BASE + 0x00),
            pending: ReadOnly::new(INTERRUPT_BASE + 0x04),
            clear: WriteOnly::new(INTERRUPT_BASE + 0x08),
        }
    }

    /// Enable an interrupt
    pub unsafe fn enable_interrupt(&self, irq: u8) {
        self.enable.modify(|v| v | (1 << irq));
    }

    /// Check if interrupt is pending
    pub unsafe fn is_pending(&self, irq: u8) -> bool {
        (self.pending.read() >> irq) & 1 == 1
    }

    /// Clear a pending interrupt
    pub unsafe fn clear_interrupt(&self, irq: u8) {
        self.clear.write(1 << irq);
    }

    /// Handle all pending interrupts
    pub unsafe fn handle_pending(&self) {
        let pending = self.pending.read();

        // Process each pending interrupt
        for irq in 0..32 {
            if (pending >> irq) & 1 == 1 {
                // Handle interrupt
                self.handle_irq(irq);
                // Clear it
                self.clear.write(1 << irq);
            }
        }
    }

    fn handle_irq(&self, irq: u8) {
        // Dispatch to appropriate handler
        match irq {
            0 => handle_timer_interrupt(),
            1 => handle_uart_interrupt(),
            2 => handle_gpio_interrupt(),
            _ => handle_unknown_interrupt(irq),
        }
    }
}
```

### Example 2: DMA Controller with Ownership

```rust
/// DMA Channel with ownership semantics
pub struct DmaChannel<const N: usize> {
    regs: &'static mut DmaRegisters,
    _phantom: core::marker::PhantomData<()>,
}

#[repr(C)]
struct DmaRegisters {
    control: u32,
    source: u32,
    destination: u32,
    count: u32,
    status: u32,
}

impl<const N: usize> DmaChannel<N> {
    const BASE: usize = 0x6003_F000 + N * 0x100;

    /// Take ownership of DMA channel N
    pub fn take() -> Option<Self> {
        static mut TAKEN: [bool; 8] = [false; 8];

        unsafe {
            if TAKEN[N] {
                None
            } else {
                TAKEN[N] = true;
                Some(Self {
                    regs: &mut *(Self::BASE as *mut DmaRegisters),
                    _phantom: core::marker::PhantomData,
                })
            }
        }
    }

    /// Start a DMA transfer
    pub fn transfer(&mut self, src: *const u8, dst: *mut u8, len: usize) {
        unsafe {
            // Configure source and destination
            ptr::write_volatile(&mut self.regs.source, src as u32);
            ptr::write_volatile(&mut self.regs.destination, dst as u32);
            ptr::write_volatile(&mut self.regs.count, len as u32);

            // Start transfer
            ptr::write_volatile(&mut self.regs.control, 0x01);
        }
    }

    /// Check if transfer is complete
    pub fn is_complete(&self) -> bool {
        unsafe {
            ptr::read_volatile(&self.regs.status) & 0x01 != 0
        }
    }

    /// Wait for transfer to complete
    pub fn wait(&self) {
        while !self.is_complete() {
            // Could add timeout logic here
            core::hint::spin_loop();
        }
    }
}
```

### Example 3: Timer with Precise Volatile Access

```rust
/// Hardware timer with microsecond precision
pub struct Timer {
    /// Timer counter register (64-bit, read-only)
    counter_lo: ReadOnly<u32>,
    counter_hi: ReadOnly<u32>,
    /// Timer reload value (write-only)
    reload: WriteOnly<u32>,
    /// Timer control register
    control: ReadWrite<u32>,
}

impl Timer {
    const BASE: usize = 0x6002_0000;

    pub const fn new() -> Self {
        Self {
            counter_lo: ReadOnly::new(Self::BASE + 0x00),
            counter_hi: ReadOnly::new(Self::BASE + 0x04),
            reload: WriteOnly::new(Self::BASE + 0x08),
            control: ReadWrite::new(Self::BASE + 0x0C),
        }
    }

    /// Read 64-bit counter atomically
    pub unsafe fn read_counter(&self) -> u64 {
        // Must read in correct order to handle rollover
        loop {
            let hi1 = self.counter_hi.read();
            let lo = self.counter_lo.read();
            let hi2 = self.counter_hi.read();

            // If high didn't change, we got a consistent read
            if hi1 == hi2 {
                return ((hi1 as u64) << 32) | (lo as u64);
            }
            // Otherwise retry (rollover happened during read)
        }
    }

    /// Set timer period in microseconds
    pub unsafe fn set_period_us(&self, us: u32) {
        // Assuming 160MHz clock
        const TICKS_PER_US: u32 = 160;
        self.reload.write(us * TICKS_PER_US);
    }

    /// Enable timer with interrupt
    pub unsafe fn enable_with_interrupt(&self) {
        self.control.modify(|v| {
            v | (1 << 0)  // Enable bit
              | (1 << 1)  // Auto-reload bit
              | (1 << 2)  // Interrupt enable bit
        });
    }
}
```

## Part 7: Common Patterns and Best Practices

### Critical Sections for Atomic Operations

When modifying shared registers, use critical sections:

```rust
use critical_section;

pub fn modify_shared_register() {
    critical_section::with(|_cs| {
        unsafe {
            // No interrupts can occur here
            let value = ptr::read_volatile(SHARED_REG);
            let new_value = value | 0x10;
            ptr::write_volatile(SHARED_REG, new_value);
        }
    });
}
```

### Memory Barriers

Ensure ordering of volatile operations:

```rust
use core::sync::atomic::{fence, Ordering};

unsafe fn configure_peripheral() {
    // Write configuration
    ptr::write_volatile(CONFIG_REG, 0x1234);

    // Ensure configuration is written before enabling
    fence(Ordering::SeqCst);

    // Enable peripheral
    ptr::write_volatile(ENABLE_REG, 1);
}
```

### Error Handling for Hardware Operations

```rust
#[derive(Debug)]
pub enum HardwareError {
    Timeout,
    InvalidState,
    BusError,
}

pub fn read_with_timeout(reg: *const u32, timeout_us: u32) -> Result<u32, HardwareError> {
    let start = unsafe { TIMER.read_counter() };

    loop {
        unsafe {
            // Check for ready bit
            if ptr::read_volatile(STATUS_REG) & READY_BIT != 0 {
                return Ok(ptr::read_volatile(reg));
            }

            // Check timeout
            let elapsed = TIMER.read_counter() - start;
            if elapsed > timeout_us as u64 {
                return Err(HardwareError::Timeout);
            }
        }

        core::hint::spin_loop();
    }
}
```

## Part 8: Exercises

### Exercise 1: Basic Register Control

Create a safe LED controller using volatile access:

```rust
// Your task: implement a safe LED controller
pub struct LedController {
    // Add fields
}

impl LedController {
    /// Create new LED controller for given pin
    pub fn new(pin: u8) -> Self {
        todo!("Initialize LED on given pin")
    }

    /// Turn LED on
    pub fn on(&mut self) {
        todo!("Use volatile write to set pin high")
    }

    /// Turn LED off
    pub fn off(&mut self) {
        todo!("Use volatile write to set pin low")
    }

    /// Toggle LED state
    pub fn toggle(&mut self) {
        todo!("Read current state and toggle")
    }
}
```

### Exercise 2: Implement a UART Driver

Build a basic UART driver with volatile register access:

```rust
/// UART peripheral registers
#[repr(C)]
struct UartRegisters {
    data: u32,      // Data register (read/write)
    status: u32,    // Status register (read-only)
    control: u32,   // Control register (read/write)
    baud: u32,      // Baud rate register (write-only)
}

const UART_TX_READY: u32 = 1 << 0;
const UART_RX_READY: u32 = 1 << 1;

pub struct Uart {
    regs: *mut UartRegisters,
}

impl Uart {
    /// Initialize UART with given baud rate
    pub fn init(baud_rate: u32) -> Self {
        todo!("Initialize UART with volatile writes")
    }

    /// Send a byte
    pub fn write_byte(&mut self, byte: u8) {
        todo!("Wait for TX ready, then write byte")
    }

    /// Receive a byte
    pub fn read_byte(&mut self) -> Option<u8> {
        todo!("Check RX ready, read if available")
    }

    /// Send a string
    pub fn write_str(&mut self, s: &str) {
        todo!("Send each byte of the string")
    }
}
```

### Exercise 3: Create a PAC-style Interface

Design a type-safe register interface:

```rust
/// Your task: Create a PAC-style interface for a PWM peripheral
///
/// PWM Registers:
/// - DUTY (0x00): 16-bit duty cycle value
/// - PERIOD (0x04): 16-bit period value
/// - CONTROL (0x08): Control bits [0: Enable, 1: Interrupt Enable]
/// - STATUS (0x0C): Status bits [0: Running, 1: Interrupt Pending]

// TODO: Define register types with appropriate read/write permissions

// TODO: Implement safe register access methods

// TODO: Add builder pattern for configuration

pub struct PwmConfig {
    duty: u16,
    period: u16,
    enable_interrupt: bool,
}

impl PwmConfig {
    pub fn new() -> Self {
        todo!("Create default config")
    }

    pub fn duty_cycle(mut self, percent: u8) -> Self {
        todo!("Set duty cycle as percentage")
    }

    pub fn frequency(mut self, hz: u32) -> Self {
        todo!("Calculate period from frequency")
    }
}
```

## Key Takeaways

✅ **Volatile access is mandatory** for memory-mapped I/O - regular memory access won't work

✅ **SVD2Rust generates safe abstractions** from manufacturer-provided hardware descriptions

✅ **Type-state patterns** prevent hardware misuse at compile time

✅ **Ownership patterns** ensure exclusive peripheral access

✅ **embedded-hal traits** enable portable drivers across different chips

✅ **Critical sections and barriers** ensure correct operation ordering

✅ **PACs provide the foundation** but HALs make embedded Rust ergonomic

## Summary

Hardware register access requires volatile operations because:
1. **Compiler optimizations** would break hardware communication
2. **Registers change independently** of program execution
3. **Each access has side effects** that must not be optimized away

Modern embedded Rust provides multiple abstraction layers:
- **Raw volatile pointers** - Maximum control, maximum danger
- **PACs from SVD2Rust** - Type-safe register access
- **HAL implementations** - Ergonomic, safe APIs
- **embedded-hal traits** - Portable driver ecosystem

This layered approach gives you both the power to control hardware directly and the safety to avoid common embedded programming pitfalls.

---

Next: [Chapter 24: Memory Management Paradigm Shift](./24_memory_paradigm.md)