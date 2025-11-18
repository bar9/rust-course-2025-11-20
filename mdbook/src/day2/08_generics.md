# Chapter 8: Generics & Type Safety

## Learning Objectives
- Master generic functions, structs, and methods
- Understand trait bounds and where clauses
- Learn const generics for compile-time parameters
- Apply type-driven design patterns
- Compare with C++ templates and .NET generics

## Introduction

Generics allow you to write flexible, reusable code that works with multiple types while maintaining type safety. Coming from C++ or .NET, you'll find Rust's generics familiar but more constrained—in a good way.

## Generic Functions

### Basic Generic Functions

```rust
// Generic function that works with any type T
fn swap<T>(a: &mut T, b: &mut T) {
    std::mem::swap(a, b);
}

// Multiple generic parameters
fn pair<T, U>(first: T, second: U) -> (T, U) {
    (first, second)
}

// Usage
fn main() {
    let mut x = 5;
    let mut y = 10;
    swap(&mut x, &mut y);
    println!("x: {}, y: {}", x, y); // x: 10, y: 5
    
    let p = pair("hello", 42);
    println!("{:?}", p); // ("hello", 42)
}
```

### Comparison with C++ and .NET

| Feature | Rust | C++ Templates | .NET Generics |
|---------|------|---------------|---------------|
| Compilation | Monomorphization | Template instantiation | Runtime generics |
| Type checking | At definition | At instantiation | At definition |
| Constraints | Trait bounds | Concepts (C++20) | Where clauses |
| Code bloat | Yes (like C++) | Yes | No |
| Performance | Zero-cost | Zero-cost | Small overhead |

## Generic Structs

```rust
// Generic struct
struct Point<T> {
    x: T,
    y: T,
}

// Different types for each field
struct Pair<T, U> {
    first: T,
    second: U,
}

// Implementation for generic struct
impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Point { x, y }
    }
}

// Implementation for specific type
impl Point<f64> {
    fn distance_from_origin(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

fn main() {
    let integer_point = Point::new(5, 10);
    let float_point = Point::new(1.0, 4.0);
    
    // Only available for Point<f64>
    println!("Distance: {}", float_point.distance_from_origin());
}
```

## Trait Bounds

Trait bounds specify what functionality a generic type must have.

```rust
use std::fmt::Display;

// T must implement Display
fn print_it<T: Display>(value: T) {
    println!("{}", value);
}

// Multiple bounds with +
fn print_and_clone<T: Display + Clone>(value: T) -> T {
    println!("{}", value);
    value.clone()
}

// Trait bounds on structs
struct Wrapper<T: Display> {
    value: T,
}

// Complex bounds
fn complex_function<T, U>(t: T, u: U) -> String
where
    T: Display + Clone,
    U: Display + Debug,
{
    format!("{} and {:?}", t.clone(), u)
}
```

## Where Clauses

Where clauses make complex bounds more readable:

```rust
use std::fmt::Debug;

// Instead of this...
fn ugly<T: Display + Clone, U: Debug + Display>(t: T, u: U) {
    // ...
}

// Write this...
fn pretty<T, U>(t: T, u: U)
where
    T: Display + Clone,
    U: Debug + Display,
{
    // Much cleaner!
}

// Particularly useful with associated types
fn process<I>(iter: I)
where
    I: Iterator,
    I::Item: Display,
{
    for item in iter {
        println!("{}", item);
    }
}
```

## Generic Enums

The most common generic enums you'll use:

```rust
// Option<T> - Rust's null replacement
enum Option<T> {
    Some(T),
    None,
}

// Result<T, E> - For error handling
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Custom generic enum
enum BinaryTree<T> {
    Empty,
    Node {
        value: T,
        left: Box<BinaryTree<T>>,
        right: Box<BinaryTree<T>>,
    },
}

impl<T> BinaryTree<T> {
    fn new() -> Self {
        BinaryTree::Empty
    }
    
    fn insert(&mut self, value: T) 
    where 
        T: Ord,
    {
        // Implementation here
    }
}
```

## Const Generics

Const generics allow you to parameterize types with constant values:

```rust
// Array wrapper with compile-time size
struct ArrayWrapper<T, const N: usize> {
    data: [T; N],
}

impl<T, const N: usize> ArrayWrapper<T, N> {
    fn new(value: T) -> Self
    where
        T: Copy,
    {
        ArrayWrapper {
            data: [value; N],
        }
    }
}

// Matrix type with compile-time dimensions
struct Matrix<T, const ROWS: usize, const COLS: usize> {
    data: [[T; COLS]; ROWS],
}

fn main() {
    let arr: ArrayWrapper<i32, 5> = ArrayWrapper::new(0);
    let matrix: Matrix<f64, 3, 4> = Matrix {
        data: [[0.0; 4]; 3],
    };
}
```

## Type Aliases and Newtype Pattern

```rust
// Type alias - just a synonym
type Kilometers = i32;
type Result<T> = std::result::Result<T, std::io::Error>;

// Newtype pattern - creates a distinct type
struct Meters(f64);
struct Seconds(f64);

impl Meters {
    fn to_feet(&self) -> f64 {
        self.0 * 3.28084
    }
}

// Prevents mixing units
fn calculate_speed(distance: Meters, time: Seconds) -> f64 {
    distance.0 / time.0
}

fn main() {
    let distance = Meters(100.0);
    let time = Seconds(9.58);
    
    // Type safety prevents this:
    // let wrong = calculate_speed(time, distance); // Error!
    
    let speed = calculate_speed(distance, time);
    println!("Speed: {} m/s", speed);
}
```

## Phantom Types

Phantom types provide compile-time guarantees without runtime cost:

```rust
use std::marker::PhantomData;

// States for a type-safe builder
struct Locked;
struct Unlocked;

struct Door<State> {
    name: String,
    _state: PhantomData<State>,
}

impl Door<Locked> {
    fn new(name: String) -> Self {
        Door {
            name,
            _state: PhantomData,
        }
    }
    
    fn unlock(self) -> Door<Unlocked> {
        Door {
            name: self.name,
            _state: PhantomData,
        }
    }
}

impl Door<Unlocked> {
    fn open(&self) {
        println!("Opening door: {}", self.name);
    }
    
    fn lock(self) -> Door<Locked> {
        Door {
            name: self.name,
            _state: PhantomData,
        }
    }
}

fn main() {
    let door = Door::<Locked>::new("Front".to_string());
    // door.open(); // Error: method not found
    
    let door = door.unlock();
    door.open(); // OK
}
```

## Advanced Pattern: Type-Driven Design

```rust
// Email validation at compile time
struct Unvalidated;
struct Validated;

struct Email<State = Unvalidated> {
    value: String,
    _state: PhantomData<State>,
}

impl Email<Unvalidated> {
    fn new(value: String) -> Self {
        Email {
            value,
            _state: PhantomData,
        }
    }
    
    fn validate(self) -> Result<Email<Validated>, String> {
        if self.value.contains('@') {
            Ok(Email {
                value: self.value,
                _state: PhantomData,
            })
        } else {
            Err("Invalid email".to_string())
        }
    }
}

impl Email<Validated> {
    fn send(&self) {
        println!("Sending email to: {}", self.value);
    }
}

// Function that only accepts validated emails
fn send_newsletter(email: &Email<Validated>) {
    email.send();
}
```

## Common Pitfalls

### 1. Over-constraining Generics
```rust
// Bad: unnecessary Clone bound
fn bad<T: Clone + Display>(value: &T) {
    println!("{}", value); // Clone not needed!
}

// Good: only required bounds
fn good<T: Display>(value: &T) {
    println!("{}", value);
}
```

### 2. Missing Lifetime Parameters
```rust
// Won't compile
// struct RefHolder<T> {
//     value: &T,
// }

// Correct
struct RefHolder<'a, T> {
    value: &'a T,
}
```

### 3. Monomorphization Bloat
```rust
// Each T creates a new function copy
fn generic<T>(value: T) -> T {
    value
}

// Consider using trait objects for large functions
fn with_trait_object(value: &dyn Display) {
    println!("{}", value);
}
```

## Exercises

### Exercise 8.1: Generic Queue
Implement a generic queue with enqueue and dequeue operations:

```rust
struct Queue<T> {
    items: Vec<T>,
}

impl<T> Queue<T> {
    fn new() -> Self {
        // TODO: Implement
        todo!()
    }
    
    fn enqueue(&mut self, item: T) {
        // TODO: Implement
        todo!()
    }
    
    fn dequeue(&mut self) -> Option<T> {
        // TODO: Implement
        todo!()
    }
}
```

### Exercise 8.2: Generic Min Function
Write a generic function that returns the minimum of two values:

```rust
fn min<T>(a: T, b: T) -> T 
where
    T: /* What bound needed? */
{
    // TODO: Implement
    todo!()
}
```

### Exercise 8.3: Builder Pattern with Phantom Types
Create a type-safe builder for a `Request` that ensures headers are set before sending:

```rust
struct NoHeaders;
struct WithHeaders;

struct RequestBuilder<State> {
    url: String,
    headers: Vec<(String, String)>,
    _state: PhantomData<State>,
}

// TODO: Implement the builder pattern
```

## Key Takeaways

✅ **Generics provide type safety without code duplication** - Write once, use with many types

✅ **Trait bounds specify required functionality** - More explicit than C++ templates

✅ **Monomorphization means zero runtime cost** - Like C++ templates, unlike .NET generics

✅ **Const generics enable compile-time computations** - Arrays and matrices with known sizes

✅ **Phantom types provide compile-time guarantees** - State machines in the type system

✅ **Type-driven design prevents bugs at compile time** - Invalid states are unrepresentable

---

Next: [Chapter 9: Enums & Pattern Matching](./09_pattern_matching.md)