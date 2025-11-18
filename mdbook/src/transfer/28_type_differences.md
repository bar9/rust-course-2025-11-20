# Chapter 22: Type System Differences

## Learning Objectives
- Understand Rust's strict type system vs C++/.NET
- Learn why there are no implicit conversions
- Grasp the absence of function overloading
- Master type inference differences
- Transition from class hierarchies to algebraic data types

## No Implicit Conversions

Rust never performs implicit type conversions, unlike C++ and .NET.

### C++ Style (With Implicit Conversions)
```cpp
// C++ - These all compile
void process(double value) { /* ... */ }

int main() {
    process(42);        // int -> double
    process(3.14f);     // float -> double
    process('A');       // char -> double
    
    std::string str = "hello";
    const char* cstr = str.c_str();  // Implicit via method
}
```

### .NET Style
```csharp
// C# - Implicit conversions allowed
void Process(double value) { /* ... */ }

void Main() {
    Process(42);        // int -> double
    Process(3.14f);     // float -> double
    
    string str = "hello";
    // Implicit ToString() in many contexts
    Console.WriteLine("Value: " + 42);
}
```

### Rust Style (Explicit Everything)
```rust
// Rust - Everything must be explicit
fn process(value: f64) { /* ... */ }

fn main() {
    // process(42);         // ERROR: expected f64, found i32
    process(42.0);          // OK: f64 literal
    process(42_f64);        // OK: typed literal
    process(42 as f64);     // OK: explicit cast
    process(f64::from(42)); // OK: explicit conversion
    
    let x: i32 = 42;
    process(x as f64);      // Must explicitly cast
    
    // String conversions are explicit
    let s = String::from("hello");
    let slice: &str = &s;   // Explicit borrow
    let owned = slice.to_string(); // Explicit conversion
}
```

### Why No Implicit Conversions?

1. **Predictability**: You always know what type you have
2. **Safety**: No surprising precision loss or overflow
3. **Performance**: No hidden allocations or conversions
4. **Clarity**: Code intent is explicit

## No Function Overloading

Rust doesn't support function overloading. Use different names or traits instead.

### C++ Overloading
```cpp
// C++ - Multiple functions with same name
class Logger {
    void log(int value);
    void log(double value);
    void log(const std::string& value);
    void log(int level, const std::string& message);
};
```

### .NET Overloading
```csharp
// C# - Method overloading
public class Logger {
    public void Log(int value) { }
    public void Log(double value) { }
    public void Log(string value) { }
    public void Log(int level, string message) { }
}
```

### Rust Alternatives

#### Option 1: Different Names
```rust
struct Logger;

impl Logger {
    fn log_int(&self, value: i32) { }
    fn log_float(&self, value: f64) { }
    fn log_string(&self, value: &str) { }
    fn log_with_level(&self, level: i32, message: &str) { }
}
```

#### Option 2: Traits
```rust
trait Loggable {
    fn log(&self);
}

impl Loggable for i32 {
    fn log(&self) {
        println!("Integer: {}", self);
    }
}

impl Loggable for f64 {
    fn log(&self) {
        println!("Float: {}", self);
    }
}

impl Loggable for String {
    fn log(&self) {
        println!("String: {}", self);
    }
}

// Usage
fn log_value<T: Loggable>(value: T) {
    value.log();
}
```

#### Option 3: Enums for Variants
```rust
enum LogMessage {
    Int(i32),
    Float(f64),
    Text(String),
    WithLevel { level: i32, message: String },
}

impl LogMessage {
    fn log(&self) {
        match self {
            LogMessage::Int(v) => println!("Int: {}", v),
            LogMessage::Float(v) => println!("Float: {}", v),
            LogMessage::Text(v) => println!("Text: {}", v),
            LogMessage::WithLevel { level, message } => {
                println!("[Level {}] {}", level, message)
            }
        }
    }
}
```

## No Default Parameters

Rust doesn't have default parameters. Use builder pattern or Option types.

### C++ Default Parameters
```cpp
// C++
void connect(const string& host, int port = 80, bool secure = false) {
    // ...
}

connect("example.com");           // port=80, secure=false
connect("example.com", 443);      // secure=false
connect("example.com", 443, true);
```

### Rust Alternatives

#### Option 1: Builder Pattern
```rust
struct Connection {
    host: String,
    port: u16,
    secure: bool,
}

impl Connection {
    fn builder(host: String) -> ConnectionBuilder {
        ConnectionBuilder {
            host,
            port: 80,
            secure: false,
        }
    }
}

struct ConnectionBuilder {
    host: String,
    port: u16,
    secure: bool,
}

impl ConnectionBuilder {
    fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    
    fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }
    
    fn connect(self) -> Connection {
        Connection {
            host: self.host,
            port: self.port,
            secure: self.secure,
        }
    }
}

// Usage
let conn = Connection::builder("example.com".to_string())
    .port(443)
    .secure(true)
    .connect();
```

#### Option 2: Option Parameters
```rust
struct ConnectOptions {
    port: Option<u16>,
    secure: Option<bool>,
}

fn connect(host: &str, options: ConnectOptions) {
    let port = options.port.unwrap_or(80);
    let secure = options.secure.unwrap_or(false);
    // ...
}

// Usage
connect("example.com", ConnectOptions {
    port: Some(443),
    secure: Some(true),
});
```

#### Option 3: Multiple Functions
```rust
fn connect(host: &str) {
    connect_with_port(host, 80)
}

fn connect_with_port(host: &str, port: u16) {
    connect_full(host, port, false)
}

fn connect_full(host: &str, port: u16, secure: bool) {
    // Implementation
}
```

## Type Inference Differences

### C++ Auto (C++11+)
```cpp
// C++ - auto deduction
auto x = 42;           // int
auto y = 42.0;         // double
auto z = "hello";      // const char*
auto vec = std::vector<int>{1, 2, 3}; // std::vector<int>

// Templates with auto
template<typename T>
auto add(T a, T b) -> decltype(a + b) {
    return a + b;
}
```

### .NET var
```csharp
// C# - var inference
var x = 42;            // int
var y = 42.0;          // double
var z = "hello";       // string
var list = new List<int> { 1, 2, 3 }; // List<int>

// Cannot use var for:
// - Field declarations
// - Method parameters
// - Return types (except lambdas)
```

### Rust Type Inference
```rust
// Rust - let inference
let x = 42;            // i32 (default integer type)
let y = 42.0;          // f64 (default float type)
let z = "hello";       // &str
let mut vec = Vec::new(); // Type determined by usage

vec.push(1);           // Now vec: Vec<i32>

// Partial type annotation
let numbers: Vec<_> = (0..10).collect(); // Infer element type

// Turbofish for disambiguation
let parsed = "42".parse::<i32>().unwrap();

// Inference flows through expressions
fn process(x: i32) -> i32 { x * 2 }
let result = process(21); // result: i32
```

### Key Differences

| Feature | C++ | .NET | Rust |
|---------|-----|------|------|
| Local inference | `auto` | `var` | `let` |
| Return type inference | `auto` (C++14) | No | Yes (impl Trait) |
| Generic inference | Template deduction | Type inference | Type inference |
| Default numeric | Platform-dependent | int/double | i32/f64 |
| Inference scope | Local | Local | Flow-based |

## Algebraic Data Types vs Classes

### Traditional OOP Hierarchy
```cpp
// C++ - Class hierarchy
class Shape {
public:
    virtual double area() = 0;
};

class Circle : public Shape {
    double radius;
public:
    double area() override { return 3.14 * radius * radius; }
};

class Rectangle : public Shape {
    double width, height;
public:
    double area() override { return width * height; }
};
```

### Rust Algebraic Data Types
```rust
// Rust - Enum with variants
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
            Shape::Rectangle { width, height } => width * height,
            Shape::Triangle { base, height } => 0.5 * base * height,
        }
    }
}

// Exhaustive pattern matching
fn describe(shape: &Shape) -> String {
    match shape {
        Shape::Circle { radius } => format!("Circle with radius {}", radius),
        Shape::Rectangle { width, height } => {
            format!("Rectangle {}x{}", width, height)
        }
        Shape::Triangle { base, height } => {
            format!("Triangle with base {} and height {}", base, height)
        }
    }
}
```

### Benefits of Algebraic Data Types

1. **Exhaustiveness**: Compiler ensures all cases handled
2. **Closed Set**: All variants known at compile time
3. **No Null**: No uninitialized objects
4. **Pattern Matching**: Powerful destructuring
5. **Memory Efficient**: Size of largest variant + discriminant

## Pattern Matching vs Switch

### C++ Switch
```cpp
// C++ - Limited to integral types
switch(value) {
    case 1:
        handle_one();
        break;
    case 2:
    case 3:
        handle_two_or_three();
        break;
    default:
        handle_other();
}
```

### Rust Pattern Matching
```rust
// Rust - Works with any type
match value {
    1 => handle_one(),
    2 | 3 => handle_two_or_three(),
    4..=10 => handle_range(),
    x if x > 100 => handle_large(x),
    _ => handle_other(),
}

// Destructuring in patterns
match person {
    Person { age: 0..=17, name } => println!("Minor: {}", name),
    Person { age: 18..=64, name } => println!("Adult: {}", name),
    Person { age: 65.., name } => println!("Senior: {}", name),
    Person { age, .. } => println!("Invalid age: {}", age),
}

// Option matching
match optional_value {
    Some(x) if x > 0 => println!("Positive: {}", x),
    Some(x) => println!("Non-positive: {}", x),
    None => println!("No value"),
}
```

## Common Migration Patterns

### From Nullable to Option
```cpp
// C++
Person* find_person(int id) {
    if (found) return &person;
    return nullptr;
}
```

```rust
// Rust
fn find_person(id: i32) -> Option<Person> {
    if found { Some(person) } else { None }
}
```

### From Exceptions to Result
```cpp
// C++
int divide(int a, int b) {
    if (b == 0) throw std::runtime_error("Division by zero");
    return a / b;
}
```

```rust
// Rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}
```

## Exercises

### Exercise 21.1: Remove Implicit Conversions
Convert this C++ code to Rust, making all conversions explicit:
```cpp
void process_temperature(double celsius) { }

int main() {
    process_temperature(98.6f);
    process_temperature(37);
    process_temperature('A');
}
```

### Exercise 21.2: Replace Overloading
Redesign this overloaded C++ interface in Rust:
```cpp
class Database {
    void insert(int id, string name);
    void insert(User user);
    void insert(vector<User> users);
};
```

### Exercise 21.3: Algebraic Data Type
Convert this class hierarchy to a Rust enum:
```cpp
class Event {
public:
    virtual void handle() = 0;
};
class ClickEvent : public Event { };
class KeyEvent : public Event { };
class TimerEvent : public Event { };
```

## Key Takeaways

✅ **No implicit conversions** - Every type change is explicit and visible

✅ **No function overloading** - Use traits, enums, or different names

✅ **No default parameters** - Use builders or Option types

✅ **Powerful type inference** - But always predictable and local

✅ **Algebraic data types** - More powerful than class hierarchies

✅ **Pattern matching** - Far beyond switch statements

---

Next: [Chapter 29: Traits vs OOP](./29_traits_vs_oop.md)
