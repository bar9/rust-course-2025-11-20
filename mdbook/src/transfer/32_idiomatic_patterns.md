# Chapter 27: Idiomatic Rust Patterns for Production Systems

## Learning Objectives
- Master idiomatic Rust patterns and conventions
- Understand "the Rust way" vs C++/.NET approaches
- Learn common patterns from the Rust community
- Write Rust that feels natural to Rust developers
- Recognize and apply design patterns specific to Rust

## The Rust Philosophy

### Core Principles
1. **Explicit over implicit** - Make intentions clear
2. **Composition over inheritance** - Use traits and generics
3. **Zero-cost abstractions** - Don't pay for what you don't use
4. **Fail fast and loudly** - Catch errors at compile time
5. **Ownership clarity** - Make ownership obvious

## Ownership Patterns

### The Newtype Pattern

```rust
// Wrap primitive types for type safety
struct Kilometers(f64);
struct Miles(f64);

impl Kilometers {
    fn to_miles(&self) -> Miles {
        Miles(self.0 * 0.621371)
    }
}

// Prevents mixing units
fn calculate_fuel_efficiency(distance: Kilometers, fuel: Liters) -> KmPerLiter {
    KmPerLiter(distance.0 / fuel.0)
}

// Compare with C++
// typedef double Kilometers;  // Just an alias, no type safety
// using Miles = double;       // Same type, can mix them up
```

### Builder Pattern

```rust
// Idiomatic builder for complex structs
#[derive(Debug, Default)]
pub struct ServerConfig {
    host: String,
    port: u16,
    max_connections: usize,
    timeout: Duration,
}

#[derive(Default)]
pub struct ServerConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    max_connections: Option<usize>,
    timeout: Option<Duration>,
}

impl ServerConfigBuilder {
    pub fn new() -> Self {
        Default::default()
    }
    
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }
    
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    
    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = Some(max);
        self
    }
    
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    pub fn build(self) -> Result<ServerConfig, &'static str> {
        Ok(ServerConfig {
            host: self.host.ok_or("host is required")?,
            port: self.port.unwrap_or(8080),
            max_connections: self.max_connections.unwrap_or(100),
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
        })
    }
}

// Usage
let config = ServerConfigBuilder::new()
    .host("localhost")
    .port(3000)
    .timeout(Duration::from_secs(60))
    .build()?;
```

### RAII Guards

```rust
// Idiomatic RAII pattern
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(prefix: &str) -> io::Result<Self> {
        let path = std::env::temp_dir().join(format!("{}-{}", prefix, uuid()));
        std::fs::create_dir(&path)?;
        Ok(TempDir { path })
    }
    
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

// Usage - directory automatically cleaned up
{
    let temp = TempDir::new("test")?;
    std::fs::write(temp.path().join("file.txt"), b"data")?;
    // Directory deleted when temp goes out of scope
}
```

## Error Handling Patterns

### Custom Error Types

```rust
// Idiomatic error handling
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },
}

// Result type alias for convenience
pub type Result<T> = std::result::Result<T, AppError>;

// Usage
fn process_file(path: &Path) -> Result<Data> {
    let contents = std::fs::read_to_string(path)?;  // Automatic conversion
    let parsed: i32 = contents.trim().parse()?;      // Automatic conversion
    
    if parsed < 0 {
        return Err(AppError::InvalidInput {
            message: "Value must be positive".to_string(),
        });
    }
    
    Ok(Data::new(parsed))
}
```

### Early Returns

```rust
// Idiomatic: Early return with ?
fn process(input: &str) -> Result<String> {
    let trimmed = input.trim();
    
    if trimmed.is_empty() {
        return Err(Error::EmptyInput);
    }
    
    let parsed = trimmed.parse::<i32>()?;
    let validated = validate(parsed)?;
    let result = compute(validated)?;
    
    Ok(format!("Result: {}", result))
}

// Not idiomatic: Nested error handling
fn process_nested(input: &str) -> Result<String> {
    match input.trim() {
        trimmed if !trimmed.is_empty() => {
            match trimmed.parse::<i32>() {
                Ok(parsed) => {
                    match validate(parsed) {
                        Ok(validated) => {
                            match compute(validated) {
                                Ok(result) => Ok(format!("Result: {}", result)),
                                Err(e) => Err(e),
                            }
                        }
                        Err(e) => Err(e),
                    }
                }
                Err(e) => Err(e.into()),
            }
        }
        _ => Err(Error::EmptyInput),
    }
}
```

## Iterator Patterns

### Lazy Evaluation and Chaining

```rust
// Idiomatic: Iterator chains
fn process_data(items: &[Item]) -> Vec<Summary> {
    items.iter()
        .filter(|item| item.is_valid())
        .filter_map(|item| item.try_process())
        .map(|processed| Summary::from(processed))
        .collect()
}

// Not idiomatic: Manual loops
fn process_data_manual(items: &[Item]) -> Vec<Summary> {
    let mut result = Vec::new();
    for item in items {
        if item.is_valid() {
            if let Some(processed) = item.try_process() {
                result.push(Summary::from(processed));
            }
        }
    }
    result
}
```

### Custom Iterators

```rust
// Idiomatic custom iterator
struct Fibonacci {
    curr: u64,
    next: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Fibonacci { curr: 0, next: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;
    
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.curr;
        self.curr = self.next;
        self.next = current + self.next;
        Some(current)
    }
}

// Usage
let fib_numbers: Vec<u64> = Fibonacci::new()
    .take(10)
    .collect();
```

## API Design Patterns

### Taking Strings

```rust
// Idiomatic: Accept anything string-like
fn greet(name: impl AsRef<str>) {
    println!("Hello, {}!", name.as_ref());
}

// Or for storing:
struct Person {
    name: String,
}

impl Person {
    // Accept anything that can become a String
    fn new(name: impl Into<String>) -> Self {
        Person { name: name.into() }
    }
}

// Usage - all work:
greet("Alice");
greet(String::from("Bob"));
greet(&some_string);

let p1 = Person::new("Charlie");
let p2 = Person::new(String::from("David"));
```

### Returning Iterators

```rust
// Idiomatic: Return impl Iterator for flexibility
fn get_even_numbers(max: u32) -> impl Iterator<Item = u32> {
    (0..=max).filter(|n| n % 2 == 0)
}

// For more complex cases:
struct DataProcessor;

impl DataProcessor {
    fn process<'a>(&'a self, items: &'a [Item]) 
        -> impl Iterator<Item = ProcessedItem> + 'a {
        items.iter()
            .filter(|item| self.should_process(item))
            .map(move |item| self.transform(item))
    }
}
```

### Optional Parameters

```rust
// Idiomatic: Use Option for optional parameters
#[derive(Default)]
struct QueryOptions {
    limit: Option<usize>,
    offset: Option<usize>,
    sort_by: Option<String>,
}

fn query_database(options: QueryOptions) -> Result<Vec<Record>> {
    let limit = options.limit.unwrap_or(100);
    let offset = options.offset.unwrap_or(0);
    // ...
}

// Usage
query_database(QueryOptions {
    limit: Some(50),
    ..Default::default()
})?;
```

## Type System Patterns

### Type State Pattern

```rust
// Encode state in the type system
struct Locked;
struct Unlocked;

struct Safe<State = Locked> {
    treasure: String,
    _state: PhantomData<State>,
}

impl Safe<Locked> {
    fn unlock(self, combination: &str) -> Result<Safe<Unlocked>, Safe<Locked>> {
        if combination == "12345" {
            Ok(Safe {
                treasure: self.treasure,
                _state: PhantomData,
            })
        } else {
            Err(self)
        }
    }
}

impl Safe<Unlocked> {
    fn get_treasure(&self) -> &str {
        &self.treasure
    }
    
    fn lock(self) -> Safe<Locked> {
        Safe {
            treasure: self.treasure,
            _state: PhantomData,
        }
    }
}
```

### Extension Traits

```rust
// Idiomatic: Add methods to foreign types
trait VecExt<T> {
    fn get_or_insert(&mut self, index: usize, default: T) -> &mut T;
}

impl<T> VecExt<T> for Vec<T> {
    fn get_or_insert(&mut self, index: usize, default: T) -> &mut T {
        if index >= self.len() {
            self.resize_with(index + 1, || default);
        }
        &mut self[index]
    }
}

// Usage
let mut vec = vec![1, 2, 3];
*vec.get_or_insert(5, 0) = 42;
```

## Conversion Patterns

### From and Into

```rust
// Idiomatic conversions
#[derive(Debug)]
struct Email(String);

impl From<String> for Email {
    fn from(s: String) -> Self {
        Email(s)
    }
}

impl From<&str> for Email {
    fn from(s: &str) -> Self {
        Email(s.to_string())
    }
}

// Automatically get Into
fn send_email(email: impl Into<Email>) {
    let email = email.into();
    // ...
}

// Usage
send_email("alice@example.com");
send_email(String::from("bob@example.com"));
```

### TryFrom for Fallible Conversions

```rust
use std::convert::TryFrom;

struct PositiveInteger(i32);

impl TryFrom<i32> for PositiveInteger {
    type Error = &'static str;
    
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > 0 {
            Ok(PositiveInteger(value))
        } else {
            Err("Value must be positive")
        }
    }
}
```

## Common Anti-Patterns to Avoid

### 1. Unnecessary Cloning
```rust
// Bad: Cloning when borrowing would work
fn bad(data: Vec<String>) -> String {
    let cloned = data.clone();  // Unnecessary!
    process(&cloned)
}

// Good: Borrow instead
fn good(data: &[String]) -> String {
    process(data)
}
```

### 2. Stringly-Typed APIs
```rust
// Bad: Using strings for everything
fn set_status(status: &str) {
    match status {
        "active" => { /* ... */ }
        "inactive" => { /* ... */ }
        _ => panic!("Invalid status"),
    }
}

// Good: Use enums
enum Status {
    Active,
    Inactive,
}

fn set_status(status: Status) {
    match status {
        Status::Active => { /* ... */ }
        Status::Inactive => { /* ... */ }
    }
}
```

### 3. Nested Options/Results
```rust
// Bad: Option<Option<T>> or Result<Result<T, E>, E>
fn bad() -> Option<Option<Data>> {
    // ...
}

// Good: Flatten or use custom type
fn good() -> Option<Data> {
    // ...
}
```

## Rust vs C++/.NET Patterns

### Resource Management
| Pattern | C++ | .NET | Rust |
|---------|-----|------|------|
| Resource cleanup | Destructor | IDisposable/using | Drop trait |
| Shared ownership | shared_ptr | Reference counting | Rc/Arc |
| Weak references | weak_ptr | WeakReference | Weak |
| Move semantics | Move constructor | N/A | Default behavior |

### Error Handling
| Pattern | C++ | .NET | Rust |
|---------|-----|------|------|
| Error propagation | Exceptions/error codes | Exceptions | Result + ? |
| Null handling | nullptr checks | null checks/nullable | Option<T> |
| Assertions | assert macro | Debug.Assert | debug_assert! |

## Exercises

### Exercise 26.1: Refactor to Idiomatic
Refactor this code to be more idiomatic:
```rust
fn process_items(items: Vec<Item>) -> Vec<String> {
    let mut results = Vec::new();
    for i in 0..items.len() {
        if items[i].is_valid == true {
            let processed = items[i].process();
            results.push(processed);
        }
    }
    return results;
}
```

### Exercise 26.2: Design an API
Design an idiomatic Rust API for a configuration system that:
- Loads from multiple sources (file, env, args)
- Validates configuration
- Provides typed access to values

### Exercise 26.3: Pattern Recognition
Identify the patterns used in this code and explain why they're idiomatic:
```rust
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
```

## Key Takeaways

✅ **Embrace ownership** - Make ownership transfers explicit and clear

✅ **Use the type system** - Encode invariants in types, not runtime checks

✅ **Prefer composition** - Traits and generics over inheritance

✅ **Early returns with ?** - Linear error handling, not nested

✅ **Iterator chains** - Functional style for data transformation

✅ **Zero-cost abstractions** - High-level code with no runtime penalty

✅ **Explicit over implicit** - Make intentions clear in the code

---

This completes your Rust journey from C++/.NET. Welcome to the Rust community! 🦀