# Chapter 14: Concurrency & Shared State
## Building Thread-Safe Applications with Arc, Mutex, and Channels

### Learning Objectives
By the end of this chapter, you'll be able to:
- Spawn and manage threads in Rust
- Share data safely between threads using Arc and Mutex
- Use channels for message passing between threads
- Choose between different synchronization primitives (Mutex vs RwLock)
- Understand when to use threads vs async programming
- Apply concurrency patterns to embedded systems
- Build thread-safe data structures
- Debug common concurrency issues

---

## Why Concurrency Matters

Modern applications need to handle multiple tasks simultaneously - reading from sensors, processing data, handling user input, and communicating over networks. Rust's ownership system makes concurrent programming safer than in most languages.

**Concurrency Comparison Across Languages:**

| Aspect | C/C++ | C# | Go | Rust |
|--------|-------|----|----|------|
| Data races | Runtime crashes | Runtime exceptions | Panic | **Compile-time prevention** |
| Memory safety | Manual management | GC overhead | GC overhead | **Zero-cost safety** |
| Deadlock prevention | Manual | Manual | Manual | **Ownership helps** |
| Performance | Fast but unsafe | Good with GC | Good with GC | **Fast and safe** |
| Learning curve | High | Medium | Low | **Medium (worth it!)** |

### The Problem with Shared Mutable State

```rust
use std::thread;
use std::time::Duration;

// This won't compile - and that's good!
fn broken_shared_counter() {
    let mut counter = 0;

    let handle1 = thread::spawn(|| {
        for _ in 0..1000 {
            counter += 1; // ❌ Error: can't capture mutable reference
        }
    });

    let handle2 = thread::spawn(|| {
        for _ in 0..1000 {
            counter += 1; // ❌ Error: can't capture mutable reference
        }
    });

    handle1.join().unwrap();
    handle2.join().unwrap();

    println!("Counter: {}", counter); // What should this be?
}
```

Rust prevents this at compile time because:
1. **Data races**: Multiple threads modifying the same data
2. **Use after free**: One thread might deallocate while another is reading
3. **Inconsistent state**: Partially updated data structures

## Arc: Atomic Reference Counting

`Arc<T>` (Atomically Reference Counted) enables multiple owners of the same data:

```rust
use std::sync::Arc;
use std::thread;

fn sharing_immutable_data() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    let mut handles = vec![];

    for i in 0..3 {
        let data_clone = Arc::clone(&data); // Cheap reference count increment
        let handle = thread::spawn(move || {
            println!("Thread {}: {:?}", i, data_clone);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Original data still accessible: {:?}", data);
}
```

**Key Points about Arc:**
- **Cheap cloning**: Only increments a counter, doesn't copy data
- **Thread-safe**: Reference counting is atomic
- **Immutable by default**: `Arc<T>` gives you `&T`, not `&mut T`
- **No garbage collection**: Automatically dropped when last reference goes away

## Mutex: Mutual Exclusion

`Mutex<T>` provides thread-safe mutable access to data:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn safe_shared_counter() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let mut num = counter_clone.lock().unwrap();
                *num += 1;
                // Lock is automatically released when `num` goes out of scope
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Counter: {}", *counter.lock().unwrap());
    // Should print "Counter: 1000" (10 threads × 100 increments)
}

// Real-world pattern: shared configuration
use std::collections::HashMap;

type Config = Arc<Mutex<HashMap<String, String>>>;

fn update_config(config: &Config, key: String, value: String) {
    let mut map = config.lock().unwrap();
    map.insert(key, value);
}

fn read_config(config: &Config, key: &str) -> Option<String> {
    let map = config.lock().unwrap();
    map.get(key).cloned()
}
```

**Mutex Best Practices:**
- **Keep critical sections small**: Don't hold locks longer than necessary
- **Avoid nested locks**: Can cause deadlocks
- **Consider RwLock**: For read-heavy workloads
- **Handle poisoning**: Use `.unwrap()` for prototypes, proper error handling in production

## RwLock: Multiple Readers, Single Writer

`RwLock<T>` allows multiple concurrent readers OR one writer:

```rust
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

fn reader_writer_example() {
    let data = Arc::new(RwLock::new(vec![1, 2, 3, 4, 5]));
    let mut handles = vec![];

    // Spawn multiple readers
    for i in 0..3 {
        let data_clone = Arc::clone(&data);
        let handle = thread::spawn(move || {
            // Multiple readers can access simultaneously
            let read_guard = data_clone.read().unwrap();
            println!("Reader {}: sum = {}", i, read_guard.iter().sum::<i32>());
            thread::sleep(Duration::from_millis(100)); // Simulate work
        });
        handles.push(handle);
    }

    // Spawn a writer (will wait for readers to finish)
    let data_clone = Arc::clone(&data);
    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(50)); // Let readers start first
        let mut write_guard = data_clone.write().unwrap();
        write_guard.push(6);
        println!("Writer: added element 6");
    });
    handles.push(handle);

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final data: {:?}", *data.read().unwrap());
}
```

**When to use RwLock vs Mutex:**
- **RwLock**: Read-heavy workloads (config, caches, lookups)
- **Mutex**: Write-heavy or simple cases (counters, queues)
- **Performance**: RwLock has higher overhead, only benefits with many readers

## Channels: Message Passing

Channels enable communication between threads without shared state:

```rust
use std::sync::mpsc; // Multiple Producer, Single Consumer
use std::thread;
use std::time::Duration;

fn basic_channel_example() {
    let (tx, rx) = mpsc::channel();

    // Spawn a producer thread
    thread::spawn(move || {
        let messages = vec!["hello", "from", "the", "thread"];

        for msg in messages {
            tx.send(msg).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Main thread as consumer
    for received in rx {
        println!("Received: {}", received);
    }
}

// Multiple producers, single consumer
fn multiple_producers() {
    let (tx, rx) = mpsc::channel();

    for i in 0..3 {
        let tx_clone = tx.clone();
        thread::spawn(move || {
            tx_clone.send(format!("Message from thread {}", i)).unwrap();
        });
    }

    drop(tx); // Close the sending side

    for received in rx {
        println!("Got: {}", received);
    }
}

// Bounded channels for backpressure
fn bounded_channel_example() {
    let (tx, rx) = mpsc::sync_channel(2); // Buffer size of 2

    thread::spawn(move || {
        for i in 0..5 {
            println!("Sending {}", i);
            tx.send(i).unwrap(); // Will block when buffer is full
            println!("Sent {}", i);
        }
    });

    thread::sleep(Duration::from_secs(1)); // Let sender get ahead

    for received in rx {
        println!("Received: {}", received);
        thread::sleep(Duration::from_millis(500)); // Slow consumer
    }
}
```

**Channel Patterns:**
- **Fan-out**: One producer, multiple consumers (need multiple channels)
- **Fan-in**: Multiple producers, one consumer (mpsc)
- **Pipeline**: Chain of processing stages
- **Worker pool**: Fixed number of workers processing tasks

## Real-World Concurrency Patterns

### Worker Pool Pattern

```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

struct Task {
    id: usize,
    data: String,
}

impl Task {
    fn process(&self) -> String {
        // Simulate work
        thread::sleep(std::time::Duration::from_millis(100));
        format!("Processed task {}: {}", self.id, self.data)
    }
}

struct WorkerPool {
    workers: Vec<thread::JoinHandle<()>>,
    sender: mpsc::Sender<Task>,
}

impl WorkerPool {
    fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            let receiver_clone = Arc::clone(&receiver);
            let worker = thread::spawn(move || {
                loop {
                    let task = {
                        let receiver = receiver_clone.lock().unwrap();
                        receiver.recv()
                    };

                    match task {
                        Ok(task) => {
                            println!("Worker {}: {}", id, task.process());
                        }
                        Err(_) => {
                            println!("Worker {} shutting down", id);
                            break;
                        }
                    }
                }
            });

            workers.push(worker);
        }

        WorkerPool { workers, sender }
    }

    fn execute(&self, task: Task) {
        self.sender.send(task).unwrap();
    }

    fn shutdown(self) {
        drop(self.sender); // Close channel

        for worker in self.workers {
            worker.join().unwrap();
        }
    }
}

fn worker_pool_example() {
    let pool = WorkerPool::new(3);

    for i in 0..10 {
        pool.execute(Task {
            id: i,
            data: format!("task-data-{}", i),
        });
    }

    pool.shutdown();
}
```

## Embedded Concurrency Considerations

When targeting embedded systems, concurrency has different constraints:

### Interrupt Safety and Critical Sections

```rust
// On embedded systems (no_std), you often need interrupt-safe primitives
// This is pseudo-code showing the concept

#[cfg(feature = "embedded")]
mod embedded_concurrency {
    use cortex_m::interrupt;
    use heapless::spsc; // Single producer, single consumer queue

    static mut SENSOR_DATA: Option<f32> = None;

    // Interrupt service routine
    fn sensor_interrupt() {
        // Critical section - interrupts disabled
        interrupt::free(|_| {
            unsafe {
                SENSOR_DATA = Some(read_sensor_register());
            }
        });
    }

    // Main thread
    pub fn get_sensor_data() -> Option<f32> {
        interrupt::free(|_| unsafe {
            SENSOR_DATA.take()
        })
    }

    fn read_sensor_register() -> f32 {
        // Hardware register access
        25.0
    }
}
```

### Embassy Async Model (Preview)

Embassy provides cooperative async concurrency for embedded:

```rust
// This is what we'll cover in Chapter 15
#[cfg(feature = "preview")]
mod embassy_preview {
    // Embassy tasks run cooperatively
    // - Single stack (memory efficient)
    // - Interrupt-driven wakeups
    // - Zero-cost async for embedded

    // We'll learn this in detail in the next chapter!
}
```

## Exercise: Build Thread-Safe Temperature Storage

Now it's time to build the second increment of our capstone project!

### Your Task: Complete temp_store

Building on the `temp_core` from Chapter 13, create a thread-safe temperature storage system.

1. **Create the temp_store crate** (if following along):
   ```bash
   cargo new temp_store --lib
   # Add temp_core as dependency
   ```

2. **Implement `TemperatureReading`**:
   ```rust
   pub struct TemperatureReading {
       pub temperature: Temperature,
       pub timestamp: u64, // Unix timestamp
   }
   ```

3. **Create thread-safe `TemperatureStore`**:
   - Use `Arc<Mutex<Vec<TemperatureReading>>>` for storage
   - Implement circular buffer (fixed capacity, removes oldest when full)
   - Methods needed:
     - `new(capacity: usize) -> Self`
     - `add_reading(&self, reading: TemperatureReading)`
     - `get_latest(&self) -> Option<TemperatureReading>`
     - `get_all(&self) -> Vec<TemperatureReading>`
     - `calculate_stats(&self) -> Option<TemperatureStats>`
     - `clear(&self)`
     - `clone_handle(&self) -> Self` (for sharing between threads)

4. **Implement `TemperatureStats`**:
   ```rust
   pub struct TemperatureStats {
       pub min: Temperature,
       pub max: Temperature,
       pub average: Temperature,
       pub count: usize,
   }
   ```

5. **Write comprehensive tests**:
   - Basic storage operations
   - Circular buffer behavior
   - Statistics calculation
   - **Thread safety**: Multiple threads reading/writing concurrently
   - Edge cases (empty store, single reading, etc.)

### Extension Challenges

1. **Performance Optimization**:
   - Use `RwLock` instead of `Mutex` for read-heavy patterns
   - Benchmark the difference

2. **Advanced Statistics**:
   - Add standard deviation calculation
   - Add temperature trend detection (rising/falling)

3. **Configurable Storage**:
   - Support different storage strategies (circular vs growing)
   - Add memory usage monitoring

4. **Real-world Simulation**:
   - Create multiple "sensor" threads adding readings
   - Create "monitor" threads calculating stats
   - Demonstrate no data races or corruption

### Success Criteria

- All tests pass: `cargo test -p temp_store`
- No warnings: `cargo clippy`
- Thread safety test demonstrates concurrent access
- Statistics are calculated correctly
- Circular buffer maintains size limit
- Code is well-documented and follows Rust conventions

### Integration with Previous Work

Your `TemperatureStore` should work seamlessly with the `Temperature` and `TemperatureSensor` traits from Chapter 13:

```rust
// Example usage combining both increments
fn integrate_with_sensors() {
    let store = TemperatureStore::new(100);
    let mut sensor = MockTemperatureSensor::new("test".to_string(), 25.0);

    // Simulate readings from sensor
    for _ in 0..10 {
        match sensor.read_temperature() {
            Ok(temp) => {
                let reading = TemperatureReading::new(temp);
                store.add_reading(reading);
            }
            Err(e) => eprintln!("Sensor error: {:?}", e),
        }
    }

    if let Some(stats) = store.calculate_stats() {
        println!("Temperature stats: min={}, max={}, avg={}",
                 stats.min, stats.max, stats.average);
    }
}
```

This foundation will be essential when we add async programming in Chapter 15!

## Key Takeaways

✅ **Arc enables safe sharing**: Multiple owners of immutable data

✅ **Mutex provides thread-safe mutation**: Interior mutability with locking

✅ **RwLock optimizes for readers**: Multiple readers, single writer

✅ **Channels enable message passing**: Avoid shared state complexity

✅ **Worker pools scale processing**: Fixed threads, dynamic work

✅ **Critical sections for embedded**: Interrupt-safe operations

✅ **Choose the right tool**: Arc+Mutex, RwLock, channels, or async

Understanding these concurrency primitives prepares you for async programming, which we'll explore in Chapter 15!