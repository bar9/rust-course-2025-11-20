# Chapter 15: Async Programming Basics

## Learning Objectives
- Understand async/await syntax and when to use asynchronous programming
- Work with Futures trait and async runtime concepts
- Master tokio runtime and its ecosystem
- Compare async vs threads trade-offs for different use cases
- Handle errors effectively in async contexts
- Build practical async applications with concurrent I/O operations

## Introduction to Async Programming

Asynchronous programming allows handling many I/O operations concurrently without the overhead of operating system threads. Rust's async model is zero-cost and provides memory safety guarantees.

```rust
use std::time::Duration;
use tokio::time::sleep;

// Basic async function
async fn simple_async_function() {
    println!("Starting async operation");
    sleep(Duration::from_millis(100)).await;
    println!("Async operation completed");
}

// Async functions return impl Future<Output = ReturnType>
async fn async_with_return() -> String {
    sleep(Duration::from_millis(50)).await;
    "Hello from async!".to_string()
}

// Entry point for async programs
#[tokio::main]
async fn main() {
    simple_async_function().await;
    let result = async_with_return().await;
    println!("Result: {}", result);
}
```

**C++/C# Comparison:**
- **C++**: std::async, coroutines (C++20), callbacks, or third-party libraries
- **C#**: Task<T>, async/await keywords, thread pool based
- **Rust**: Zero-cost futures, compile-time async, no built-in runtime (use tokio/async-std)

## Understanding Futures

Futures are the foundation of Rust's async system - they represent values that may not be ready yet:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

// Simple custom future that completes after a duration
struct DelayFuture {
    when: Instant,
}

impl DelayFuture {
    fn new(duration: Duration) -> Self {
        DelayFuture {
            when: Instant::now() + duration,
        }
    }
}

impl Future for DelayFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.when {
            Poll::Ready(())
        } else {
            // In a real implementation, you'd register with a timer
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

// Using custom future
async fn use_custom_future() {
    println!("Starting delay");
    DelayFuture::new(Duration::from_millis(100)).await;
    println!("Delay completed");
}

// Futures are lazy - they don't run until polled
async fn demonstrate_lazy_futures() {
    let future1 = async {
        println!("Future 1 executing");
        42
    };
    
    let future2 = async {
        println!("Future 2 executing");
        "hello"
    };
    
    println!("Futures created but not executed yet");
    
    // Futures only execute when awaited
    let result1 = future1.await;
    let result2 = future2.await;
    
    println!("Results: {} and {}", result1, result2);
}
```

## Tokio Runtime and Ecosystem

Tokio is Rust's most popular async runtime:

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::fs::File;
use std::error::Error;

// File I/O with tokio
async fn async_file_operations() -> Result<(), Box<dyn Error>> {
    // Reading a file asynchronously
    let mut file = File::open("example.txt").await?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).await?;
    
    println!("File contents: {} bytes", contents.len());
    
    // Writing to a file asynchronously
    let mut output_file = File::create("output.txt").await?;
    output_file.write_all(b"Hello from async Rust!").await?;
    output_file.flush().await?;
    
    Ok(())
}

// TCP server example
async fn run_tcp_server() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server running on 127.0.0.1:8080");
    
    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New client: {}", addr);
        
        // Spawn a task for each client
        tokio::spawn(async move {
            if let Err(e) = handle_client(&mut socket).await {
                println!("Error handling client {}: {}", addr, e);
            }
        });
    }
}

async fn handle_client(socket: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];
    
    loop {
        let n = socket.read(&mut buffer).await?;
        if n == 0 {
            break; // Client disconnected
        }
        
        // Echo the data back
        socket.write_all(&buffer[..n]).await?;
    }
    
    Ok(())
}

// Different runtime configurations
fn different_runtime_configs() {
    // Single-threaded runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    
    rt.block_on(async {
        println!("Running on single-threaded runtime");
        tokio::time::sleep(Duration::from_millis(100)).await;
    });
    
    // Multi-threaded runtime with custom configuration
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();
    
    rt.block_on(async {
        println!("Running on multi-threaded runtime");
        tokio::time::sleep(Duration::from_millis(100)).await;
    });
}
```

## Concurrent Async Operations

Running multiple async operations concurrently:

```rust
use tokio::time::{sleep, Duration, timeout};
use std::time::Instant;

// Sequential vs concurrent execution
async fn compare_sequential_vs_concurrent() {
    let start = Instant::now();
    
    // Sequential execution
    async_task("Task 1", 100).await;
    async_task("Task 2", 150).await;
    async_task("Task 3", 200).await;
    
    println!("Sequential took: {:?}", start.elapsed());
    
    let start = Instant::now();
    
    // Concurrent execution with join!
    tokio::join!(
        async_task("Task A", 100),
        async_task("Task B", 150),
        async_task("Task C", 200)
    );
    
    println!("Concurrent took: {:?}", start.elapsed());
}

async fn async_task(name: &str, delay_ms: u64) {
    println!("Starting {}", name);
    sleep(Duration::from_millis(delay_ms)).await;
    println!("Completed {}", name);
}

// Using try_join! for error handling
async fn concurrent_with_error_handling() -> Result<(String, String, i32), Box<dyn std::error::Error>> {
    let result = tokio::try_join!(
        fetch_data_from_service_a(),
        fetch_data_from_service_b(),
        fetch_number_from_service_c()
    )?;
    
    Ok(result)
}

async fn fetch_data_from_service_a() -> Result<String, &'static str> {
    sleep(Duration::from_millis(100)).await;
    Ok("Data from service A".to_string())
}

async fn fetch_data_from_service_b() -> Result<String, &'static str> {
    sleep(Duration::from_millis(150)).await;
    Ok("Data from service B".to_string())
}

async fn fetch_number_from_service_c() -> Result<i32, &'static str> {
    sleep(Duration::from_millis(80)).await;
    Ok(42)
}

// Using select! for racing operations
async fn select_first_completion() {
    let mut task1 = Box::pin(long_running_task(1, 200));
    let mut task2 = Box::pin(long_running_task(2, 150));
    let mut task3 = Box::pin(long_running_task(3, 300));
    
    loop {
        tokio::select! {
            result = &mut task1 => {
                println!("Task 1 completed first: {}", result);
                break;
            }
            result = &mut task2 => {
                println!("Task 2 completed first: {}", result);
                break;
            }
            result = &mut task3 => {
                println!("Task 3 completed first: {}", result);
                break;
            }
            _ = sleep(Duration::from_millis(100)) => {
                println!("100ms elapsed, still waiting...");
            }
        }
    }
}

async fn long_running_task(id: u32, duration_ms: u64) -> String {
    sleep(Duration::from_millis(duration_ms)).await;
    format!("Task {} result", id)
}
```

## Spawning Tasks

Creating concurrent tasks with `tokio::spawn`:

```rust
use tokio::task;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

// Spawning multiple tasks
async fn spawn_multiple_tasks() {
    let counter = Arc::new(AtomicU32::new(0));
    let mut handles = Vec::new();
    
    // Spawn 10 tasks
    for i in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = task::spawn(async move {
            for _ in 0..100 {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                tokio::task::yield_now().await; // Yield to other tasks
            }
            println!("Task {} completed", i);
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    println!("Final counter value: {}", counter.load(Ordering::SeqCst));
}

// Task with return values
async fn spawn_tasks_with_results() {
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let handle = task::spawn(async move {
            let delay = (i + 1) * 50;
            sleep(Duration::from_millis(delay)).await;
            i * i // Return the square
        });
        handles.push(handle);
    }
    
    // Collect results
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.unwrap();
        results.push(result);
    }
    
    println!("Results: {:?}", results);
}

// Handling panicked tasks
async fn handle_task_panics() {
    let handle = task::spawn(async {
        panic!("This task panics!");
    });
    
    match handle.await {
        Ok(_) => println!("Task completed successfully"),
        Err(e) => {
            if e.is_panic() {
                println!("Task panicked: {:?}", e);
            } else if e.is_cancelled() {
                println!("Task was cancelled");
            }
        }
    }
}

// Cancelling tasks
async fn cancellation_example() {
    let handle = task::spawn(async {
        for i in 0..10 {
            println!("Working... {}", i);
            sleep(Duration::from_millis(100)).await;
        }
        "Task completed"
    });
    
    // Let it run for a bit
    sleep(Duration::from_millis(350)).await;
    
    // Cancel the task
    handle.abort();
    
    match handle.await {
        Ok(result) => println!("Task result: {}", result),
        Err(e) => {
            if e.is_cancelled() {
                println!("Task was cancelled as expected");
            }
        }
    }
}
```

## Error Handling in Async Code

Proper error handling patterns for async functions:

```rust
use tokio::io;
use std::error::Error;
use std::fmt;

// Custom error type for async operations
#[derive(Debug)]
enum AsyncError {
    Network(String),
    Timeout,
    ParseError(String),
    Io(io::Error),
}

impl fmt::Display for AsyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsyncError::Network(msg) => write!(f, "Network error: {}", msg),
            AsyncError::Timeout => write!(f, "Operation timed out"),
            AsyncError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AsyncError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl Error for AsyncError {}

impl From<io::Error> for AsyncError {
    fn from(err: io::Error) -> Self {
        AsyncError::Io(err)
    }
}

// Async function with proper error handling
async fn fetch_user_data(user_id: u32) -> Result<String, AsyncError> {
    // Simulate network request with timeout
    let result = timeout(Duration::from_millis(1000), async {
        if user_id == 0 {
            return Err(AsyncError::Network("Invalid user ID".to_string()));
        }
        
        // Simulate network delay
        sleep(Duration::from_millis(500)).await;
        
        Ok(format!("User {} data", user_id))
    }).await;
    
    match result {
        Ok(inner_result) => inner_result,
        Err(_) => Err(AsyncError::Timeout),
    }
}

// Error propagation with ?
async fn process_multiple_users(user_ids: Vec<u32>) -> Result<Vec<String>, AsyncError> {
    let mut results = Vec::new();
    
    for user_id in user_ids {
        let user_data = fetch_user_data(user_id).await?; // Error propagates
        results.push(user_data);
    }
    
    Ok(results)
}

// Collecting errors vs failing fast
async fn error_handling_strategies() {
    // Fail fast approach
    match process_multiple_users(vec![1, 2, 0, 3]).await {
        Ok(results) => println!("All succeeded: {:?}", results),
        Err(e) => println!("Failed fast: {}", e),
    }
    
    // Collect all errors approach
    let user_ids = vec![1, 2, 0, 3, 4];
    let mut results = Vec::new();
    let mut errors = Vec::new();
    
    for user_id in user_ids {
        match fetch_user_data(user_id).await {
            Ok(data) => results.push(data),
            Err(e) => errors.push((user_id, e)),
        }
    }
    
    println!("Successful results: {:?}", results);
    println!("Errors: {:?}", errors);
}

// Retry logic with exponential backoff
async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    max_retries: usize,
    base_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> std::pin::Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
    E: std::fmt::Debug,
{
    let mut attempts = 0;
    let mut delay = base_delay;
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts > max_retries {
                    return Err(e);
                }
                
                println!("Attempt {} failed: {:?}, retrying in {:?}", attempts, e, delay);
                sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }
}

// Using retry mechanism
async fn use_retry_example() -> Result<(), Box<dyn Error>> {
    let result = retry_with_backoff(
        || Box::pin(fetch_user_data(1)),
        3,
        Duration::from_millis(100),
    ).await?;
    
    println!("Retrieved data: {}", result);
    Ok(())
}
```

## Async vs Threads Trade-offs

Understanding when to use async vs threads:

```rust
use std::thread;
use std::sync::mpsc;
use tokio::sync::mpsc as async_mpsc;

// CPU-intensive work - better with threads
fn cpu_intensive_work(data: Vec<u32>) -> u64 {
    data.iter()
        .map(|&x| {
            // Simulate CPU-intensive computation
            let mut sum = 0u64;
            for i in 0..x {
                sum = sum.wrapping_add(i as u64);
            }
            sum
        })
        .sum()
}

async fn compare_cpu_work() {
    let data: Vec<u32> = (1..=1000).collect();
    let start = Instant::now();
    
    // Async version (not ideal for CPU work)
    let result1 = tokio::spawn(async move {
        cpu_intensive_work(data)
    }).await.unwrap();
    
    println!("Async CPU work took: {:?}", start.elapsed());
    
    let data: Vec<u32> = (1..=1000).collect();
    let start = Instant::now();
    
    // Thread version (better for CPU work)
    let handle = thread::spawn(move || cpu_intensive_work(data));
    let result2 = handle.join().unwrap();
    
    println!("Thread CPU work took: {:?}", start.elapsed());
    println!("Results match: {}", result1 == result2);
}

// I/O intensive work - better with async
async fn io_intensive_work() {
    let start = Instant::now();
    
    // Async I/O operations can be concurrent
    let tasks = (0..10).map(|i| {
        tokio::spawn(async move {
            // Simulate I/O delay
            sleep(Duration::from_millis(100)).await;
            format!("Result from task {}", i)
        })
    });
    
    let results = futures::future::join_all(tasks).await;
    println!("Async I/O took: {:?}", start.elapsed());
    
    // Thread version would need more resources
    let start = Instant::now();
    let handles: Vec<_> = (0..10).map(|i| {
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            format!("Result from thread {}", i)
        })
    }).collect();
    
    let _results: Vec<_> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();
    
    println!("Thread I/O took: {:?}", start.elapsed());
}

// Memory usage comparison
async fn memory_usage_comparison() {
    println!("Creating 1000 async tasks...");
    let tasks: Vec<_> = (0..1000).map(|i| {
        tokio::spawn(async move {
            sleep(Duration::from_millis(10)).await;
            i
        })
    }).collect();
    
    let start = Instant::now();
    for task in tasks {
        task.await.unwrap();
    }
    println!("1000 async tasks completed in: {:?}", start.elapsed());
    
    // Creating 1000 OS threads would be much more expensive
    println!("Note: Creating 1000 OS threads would use significantly more memory");
}
```

## Async Streams and Iterators

Working with async streams for continuous data processing:

```rust
use tokio_stream::{Stream, StreamExt};
use std::pin::Pin;
use futures::stream;

// Creating async streams
async fn work_with_streams() {
    // Stream from iterator
    let stream = stream::iter(1..=10);
    let doubled: Vec<_> = stream
        .map(|x| x * 2)
        .collect()
        .await;
    
    println!("Doubled values: {:?}", doubled);
    
    // Stream with async operations
    let async_stream = stream::iter(1..=5)
        .then(|x| async move {
            sleep(Duration::from_millis(100)).await;
            x * x
        });
    
    let squares: Vec<_> = async_stream.collect().await;
    println!("Async squares: {:?}", squares);
}

// Custom async stream
struct NumberStream {
    current: u32,
    max: u32,
    delay: Duration,
}

impl NumberStream {
    fn new(max: u32, delay: Duration) -> Self {
        NumberStream { current: 0, max, delay }
    }
}

impl Stream for NumberStream {
    type Item = u32;
    
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if self.current >= self.max {
            return Poll::Ready(None);
        }
        
        // In a real implementation, you'd use a proper timer
        let delay_future = Box::pin(sleep(self.delay));
        match delay_future.as_mut().poll(cx) {
            Poll::Ready(_) => {
                let current = self.current;
                self.current += 1;
                Poll::Ready(Some(current))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// Using custom stream
async fn use_custom_stream() {
    let mut stream = NumberStream::new(5, Duration::from_millis(200));
    
    while let Some(number) = stream.next().await {
        println!("Received: {}", number);
    }
}
```

## Common Pitfalls and Solutions

### 1. Blocking Operations in Async Context

```rust
use tokio::task;

// BAD: Blocking operation in async function
async fn bad_blocking_example() {
    // This blocks the entire async runtime!
    std::thread::sleep(Duration::from_secs(1));
    println!("This is bad!");
}

// GOOD: Use async sleep instead
async fn good_async_example() {
    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("This is good!");
}

// GOOD: Move CPU work to blocking task
async fn good_cpu_work_example() {
    let result = task::spawn_blocking(|| {
        // CPU-intensive work that would block async runtime
        let mut sum = 0u64;
        for i in 0..10_000_000 {
            sum += i;
        }
        sum
    }).await.unwrap();
    
    println!("CPU work result: {}", result);
}
```

### 2. Shared State in Async Context

```rust
use tokio::sync::{Mutex, RwLock};

// Async-aware mutex
async fn async_shared_state() {
    let data = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let data_clone = Arc::clone(&data);
        let handle = tokio::spawn(async move {
            let mut guard = data_clone.lock().await; // Note: .await on lock()
            guard.push(i);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    let final_data = data.lock().await;
    println!("Final data: {:?}", final_data);
}
```

## When to Use Async vs Threads

Choosing between async and threads depends on your use case. Here are practical guidelines:

### Use Async When:
- **I/O bound operations**: Network requests, file operations, database queries
- **Many concurrent operations**: Handling thousands of connections
- **Resource efficiency matters**: Limited memory, embedded systems
- **Coordinated operations**: Need to combine multiple I/O operations
- **Back-pressure handling**: Need to control flow between producers/consumers

### Use Threads When:
- **CPU-intensive tasks**: Mathematical calculations, image processing
- **Blocking operations**: Legacy libraries that don't support async
- **Simple parallelism**: Independent work that can be divided
- **Mixed workloads**: Some CPU work, some I/O work

### Practical Decision Framework

```rust
// ✅ Good for async - I/O bound, many concurrent operations
async fn handle_many_requests() {
    let mut tasks = Vec::new();

    for i in 0..1000 {
        let task = tokio::spawn(async move {
            // Simulated network request
            tokio::time::sleep(Duration::from_millis(100)).await;
            format!("Response {}", i)
        });
        tasks.push(task);
    }

    // All 1000 operations can run concurrently with minimal memory
    let results = futures::future::join_all(tasks).await;
    println!("Handled {} requests", results.len());
}

// ✅ Good for threads - CPU intensive work
fn cpu_intensive_parallel() {
    use std::thread;
    use std::sync::mpsc;

    let data: Vec<u64> = (0..1_000_000).collect();
    let chunk_size = data.len() / 4; // Use 4 threads

    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    for chunk in data.chunks(chunk_size) {
        let tx = tx.clone();
        let chunk = chunk.to_vec();
        let handle = thread::spawn(move || {
            // CPU-intensive calculation
            let sum: u64 = chunk.iter().map(|&x| x * x).sum();
            tx.send(sum).unwrap();
        });
        handles.push(handle);
    }

    drop(tx);
    let total: u64 = rx.iter().sum();

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Total: {}", total);
}
```

### Embedded Async Considerations

When targeting embedded systems, async has different trade-offs:

```rust
// Embassy async model for embedded
#[cfg(feature = "embedded-preview")]
mod embassy_preview {
    // Embassy provides:
    // - Single stack async (memory efficient)
    // - Interrupt-driven wakeups
    // - Zero-allocation futures
    // - Hardware-specific optimizations

    // We'll explore this more in Chapter 17!
}

// Desktop async for comparison
async fn desktop_async_pattern() {
    // Tokio provides:
    // - Multi-threaded work-stealing scheduler
    // - Heap-allocated tasks
    // - Rich ecosystem (HTTP clients, databases, etc.)
    // - Great for high-throughput servers

    println!("Desktop async focuses on throughput and ecosystem");
}
```

## Exercise: Build Async Temperature Monitoring

Now it's time to build the third increment of our capstone project!

### Your Task: Complete temp_async

Building on the `temp_core` and `temp_store` from previous chapters, create an async temperature monitoring system.

1. **Create the temp_async crate** (if following along):
   ```bash
   cargo new temp_async --lib
   # Add dependencies: temp_core, temp_store, tokio
   ```

2. **Define `AsyncTemperatureSensor` trait**:
   ```rust
   pub trait AsyncTemperatureSensor: Send {
       type Error: std::fmt::Debug + Send;

       async fn read_temperature(&mut self) -> Result<Temperature, Self::Error>;
       fn sensor_id(&self) -> &str;
   }
   ```

3. **Implement `AsyncMockSensor`**:
   - Configurable read delays (simulate real sensor timing)
   - Failure injection for testing error handling
   - Temperature updates while running

4. **Build `AsyncTemperatureMonitor`**:
   - Uses `tokio::select!` to handle multiple concurrent operations:
     - Periodic temperature readings
     - Command processing (change intervals, get stats, etc.)
     - Graceful shutdown
   - Integrates with `TemperatureStore` from Chapter 14
   - Command pattern with `tokio::sync` channels

5. **Key async patterns to implement**:
   - **Periodic tasks**: Use `tokio::time::interval` for regular sampling
   - **Command handling**: Use `mpsc` channels for control commands
   - **Request/response**: Use `oneshot` channels for queries
   - **Concurrent sensors**: Multiple async sensors running simultaneously
   - **Graceful shutdown**: Clean termination of all async tasks

### Implementation Guide

```rust
use std::time::Duration;
use tokio::time::{sleep, interval};
use tokio::sync::{mpsc, oneshot};
use temp_core::Temperature;
use temp_store::{TemperatureReading, TemperatureStore};

// Your async sensor trait
pub trait AsyncTemperatureSensor: Send {
    type Error: std::fmt::Debug + Send;

    async fn read_temperature(&mut self) -> Result<Temperature, Self::Error>;
    fn sensor_id(&self) -> &str;
}

// Commands the monitor can handle
pub enum MonitorCommand {
    SetInterval(Duration),
    GetStats(oneshot::Sender<Option<TemperatureStats>>),
    GetLatest(oneshot::Sender<Option<TemperatureReading>>),
    Stop,
}

// The main async monitor
pub struct AsyncTemperatureMonitor {
    store: TemperatureStore,
    command_rx: mpsc::Receiver<MonitorCommand>,
}

impl AsyncTemperatureMonitor {
    pub async fn run<S: AsyncTemperatureSensor>(&mut self, mut sensor: S, initial_interval: Duration) {
        let mut sample_interval = interval(initial_interval);

        loop {
            tokio::select! {
                // Sample temperature at regular intervals
                _ = sample_interval.tick() => {
                    match sensor.read_temperature().await {
                        Ok(temp) => {
                            let reading = TemperatureReading::new(temp);
                            self.store.add_reading(reading);
                            println!("📊 {}", temp);
                        }
                        Err(e) => {
                            eprintln!("❌ Sensor error: {:?}", e);
                        }
                    }
                }

                // Handle control commands
                command = self.command_rx.recv() => {
                    match command {
                        Some(MonitorCommand::SetInterval(new_interval)) => {
                            sample_interval = interval(new_interval);
                        }
                        Some(MonitorCommand::GetStats(reply)) => {
                            let stats = self.store.calculate_stats();
                            let _ = reply.send(stats);
                        }
                        Some(MonitorCommand::Stop) => break,
                        None => break, // Channel closed
                    }
                }
            }
        }

        println!("🛑 Monitor stopped");
    }
}
```

### Testing Your Implementation

Write comprehensive tests covering:

```rust
#[tokio::test]
async fn async_sensor_works() {
    let mut sensor = AsyncMockSensor::new("test".to_string(), 25.0);

    let reading = sensor.read_temperature().await.unwrap();
    assert_eq!(reading.celsius, 25.0);
}

#[tokio::test]
async fn monitor_handles_commands() {
    let mut monitor = AsyncTemperatureMonitor::new(10);
    let handle = monitor.get_handle();

    // Start monitor in background
    let monitor_task = tokio::spawn(async move {
        let sensor = AsyncMockSensor::new("test".to_string(), 20.0);
        monitor.run(sensor, Duration::from_millis(100)).await;
    });

    // Test commands
    tokio::time::sleep(Duration::from_millis(250)).await;
    let stats = handle.get_stats().await.unwrap();
    assert!(stats.is_some());

    handle.stop().await.unwrap();
    monitor_task.await.unwrap();
}

#[tokio::test]
async fn multiple_sensors_concurrently() {
    // Test running multiple async sensors at the same time
    // Each should operate independently without blocking others
}
```

### Extension Challenges

1. **Advanced async patterns**:
   - Implement timeout handling for sensor reads
   - Add exponential backoff for failed sensor connections
   - Create async stream of temperature readings

2. **Real-time features**:
   - Temperature alerts when thresholds are exceeded
   - Rolling statistics calculations
   - Data export to different formats

3. **Performance optimization**:
   - Benchmark async vs sync implementations
   - Memory usage profiling
   - Concurrent sensor handling

### Success Criteria

- All tests pass: `cargo test -p temp_async`
- No warnings: `cargo clippy`
- Async sensor can be controlled (start/stop/change interval)
- Multiple sensors can run concurrently without blocking
- Commands are handled promptly even during sensor operations
- Graceful shutdown works properly
- Integration with previous increments works seamlessly

### Integration with Previous Work

Your async monitor builds on all previous work:

```rust
// Combining all three increments
async fn integrated_example() {
    // Chapter 13: Temperature types and traits
    let sensor = AsyncMockSensor::new("living-room".to_string(), 22.0);

    // Chapter 14: Thread-safe storage
    let store = TemperatureStore::new(100);

    // Chapter 15: Async monitoring
    let mut monitor = AsyncTemperatureMonitor::new(store);
    let handle = monitor.get_handle();

    // Run everything together
    let monitor_task = tokio::spawn(async move {
        monitor.run(sensor, Duration::from_secs(1)).await;
    });

    // Control the system
    tokio::time::sleep(Duration::from_secs(5)).await;
    let stats = handle.get_stats().await.unwrap();
    println!("📊 Current stats: {:?}", stats);

    handle.stop().await.unwrap();
    monitor_task.await.unwrap();
}
```

This async foundation will be essential when we add serialization and protocols in Chapter 16!

## Key Takeaways

✅ **Async for I/O, threads for CPU**: Choose based on your workload characteristics

✅ **tokio::select! is powerful**: Handle multiple concurrent operations cleanly

✅ **Channels enable coordination**: Use mpsc for commands, oneshot for queries

✅ **Error handling is crucial**: Async operations can fail in complex ways

✅ **Testing async code**: Use `#[tokio::test]` and test concurrent scenarios

✅ **Integration builds value**: Each increment enhances the previous ones

✅ **Embedded async is different**: Embassy optimizes for memory and power

Understanding async programming opens up high-performance concurrent systems. Next, we'll add serialization and communication protocols!