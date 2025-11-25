# Chapter 24: Unsafe Rust & FFI with Bindgen

## Learning Objectives
- Understand when and why to use unsafe Rust
- Interface with C/C++ libraries using FFI
- Use bindgen for automatic binding generation
- Wrap unsafe code in safe abstractions

## When Unsafe is Necessary

Unsafe Rust allows you to:
1. Dereference raw pointers
2. Call unsafe functions
3. Access or modify mutable static variables
4. Implement unsafe traits
5. Access fields of unions

### Common Use Cases

```rust
// 1. Interfacing with C libraries
extern "C" {
    fn strlen(s: *const c_char) -> size_t;
}

// 2. Performance-critical code
unsafe fn fast_copy<T>(src: *const T, dst: *mut T, count: usize) {
    std::ptr::copy_nonoverlapping(src, dst, count);
}

// 3. Hardware register access
unsafe fn read_register(addr: usize) -> u32 {
    std::ptr::read_volatile(addr as *const u32)
}
```

## Raw Pointers

### Creating and Using Raw Pointers

```rust
fn raw_pointer_example() {
    let mut num = 5;

    // Create raw pointers
    let r1 = &num as *const i32;
    let r2 = &mut num as *mut i32;

    unsafe {
        println!("r1: {}", *r1);
        *r2 = 10;
        println!("r2: {}", *r2);
    }
}
```

### Pointer Arithmetic

```rust
unsafe fn pointer_arithmetic() {
    let arr = [1, 2, 3, 4, 5];
    let ptr = arr.as_ptr();

    // Move to second element
    let second = ptr.add(1);
    println!("Second: {}", *second);

    // Iterate using raw pointers
    let mut current = ptr;
    let end = ptr.add(arr.len());

    while current < end {
        println!("Value: {}", *current);
        current = current.add(1);
    }
}
```

## FFI with C

### Basic C Function Binding

```rust
use std::os::raw::{c_char, c_int, c_void};
use std::ffi::{CString, CStr};

// Modern Rust requires unsafe for extern blocks
unsafe extern "C" {
    fn printf(format: *const c_char, ...) -> c_int;
    fn sqrt(x: f64) -> f64;
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

fn call_c_functions() {
    unsafe {
        let result = sqrt(16.0);
        println!("sqrt(16) = {}", result);

        let format = CString::new("Hello from C: %d\n").unwrap();
        printf(format.as_ptr(), 42);
    }
}
```

### Calling Rust from C

```rust
// Modern Rust requires unsafe attribute wrapper
#[unsafe(no_mangle)]
pub extern "C" fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

#[unsafe(no_mangle)]
pub extern "C" fn process_string(s: *const c_char) -> *mut c_char {
    unsafe {
        if s.is_null() {
            return std::ptr::null_mut();
        }

        let c_str = CStr::from_ptr(s);
        let rust_string = c_str.to_string_lossy().to_uppercase();
        let c_string = CString::new(rust_string).unwrap();
        c_string.into_raw()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}
```

## C++ Interop

### Simple C++ Wrapper

```cpp
// wrapper.hpp
#ifdef __cplusplus
extern "C" {
#endif

typedef struct Point {
    double x;
    double y;
} Point;

Point* create_point(double x, double y);
void delete_point(Point* p);
double distance(const Point* p1, const Point* p2);

#ifdef __cplusplus
}
#endif
```

```rust
// Rust bindings
extern "C" {
    fn create_point(x: f64, y: f64) -> *mut Point;
    fn delete_point(p: *mut Point);
    fn distance(p1: *const Point, p2: *const Point) -> f64;
}

pub struct SafePoint {
    ptr: *mut Point,
}

impl SafePoint {
    pub fn new(x: f64, y: f64) -> Self {
        unsafe {
            SafePoint {
                ptr: create_point(x, y),
            }
        }
    }

    pub fn distance_to(&self, other: &SafePoint) -> f64 {
        unsafe {
            distance(self.ptr, other.ptr)
        }
    }
}

impl Drop for SafePoint {
    fn drop(&mut self) {
        unsafe {
            delete_point(self.ptr);
        }
    }
}
```

## Using Bindgen

Bindgen automatically generates Rust FFI bindings from C headers.

### Setup

```toml
# Cargo.toml
[build-dependencies]
bindgen = "0.69"

[dependencies]
libc = "0.2"
```

### Build Script

```rust
// build.rs
use bindgen;

fn main() {
    println!("cargo:rustc-link-lib=mylib");
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .allowlist_function("my_.*")
        .allowlist_type("MyStruct")
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
```

### Using Generated Bindings

```rust
// src/lib.rs
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Safe wrapper around generated bindings
pub struct SafeMyStruct {
    inner: MyStruct,
}

impl SafeMyStruct {
    pub fn new() -> Option<Self> {
        unsafe {
            let inner = my_struct_create();
            if my_struct_is_valid(&inner) {
                Some(SafeMyStruct { inner })
            } else {
                None
            }
        }
    }

    pub fn process(&mut self, data: &[u8]) -> Result<Vec<u8>, String> {
        unsafe {
            let result_ptr = my_struct_process(
                &mut self.inner,
                data.as_ptr(),
                data.len()
            );

            if result_ptr.is_null() {
                return Err("Processing failed".to_string());
            }

            let len = my_struct_result_length(result_ptr);
            let slice = std::slice::from_raw_parts(result_ptr as *const u8, len);
            let result = slice.to_vec();

            my_struct_free_result(result_ptr);
            Ok(result)
        }
    }
}

impl Drop for SafeMyStruct {
    fn drop(&mut self) {
        unsafe {
            my_struct_destroy(&mut self.inner);
        }
    }
}
```

### Advanced Bindgen Configuration

```rust
// build.rs
let bindings = bindgen::Builder::default()
    .header("complex.h")
    // Include additional headers
    .clang_arg("-Ivendor/include")
    // Generate bindings only for specific items
    .allowlist_function("api_.*")
    .allowlist_type("Api.*")
    .allowlist_var("API_.*")
    // Block problematic items
    .blocklist_function("internal_.*")
    // Add derives to generated structs
    .derive_default(true)
    .derive_debug(true)
    .derive_copy(false)
    // Customize layout
    .layout_tests(false)
    .generate_comments(true)
    .generate()
    .expect("Unable to generate bindings");
```

## Safety Contracts: From Unsafe Foundation to Safe Abstraction

Safety contracts are the bridge between unsafe low-level operations and safe high-level APIs. The pattern is:
1. **Document exact safety requirements** for unsafe operations
2. **Build safe abstractions** that maintain all invariants
3. **Provide safe APIs** where contract violations are impossible

### Step 1: Documented Unsafe Operations

```rust
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

/// Allocates uninitialized memory for `count` elements of type `T`
///
/// # Safety
/// - Caller must ensure `count > 0`
/// - Caller must call `deallocate_array` with the same count and layout
/// - Returned pointer must not be used after deallocation
/// - Memory is uninitialized - caller must initialize before reading
unsafe fn allocate_array<T>(count: usize) -> *mut T {
    assert!(count > 0, "Count must be greater than zero");

    let layout = Layout::array::<T>(count).expect("Invalid layout");
    let ptr = alloc(layout) as *mut T;

    if ptr.is_null() {
        panic!("Allocation failed");
    }

    ptr
}

/// Deallocates memory previously allocated by `allocate_array`
///
/// # Safety
/// - `ptr` must have been returned by `allocate_array` with the same `count`
/// - `ptr` must not be used after this call
/// - This function must be called exactly once for each allocation
/// - All elements must have been properly dropped before calling this
unsafe fn deallocate_array<T>(ptr: *mut T, count: usize) {
    let layout = Layout::array::<T>(count).expect("Invalid layout");
    dealloc(ptr as *mut u8, layout);
}

/// Initializes an element at the given index in an allocated array
///
/// # Safety
/// - `ptr` must point to valid allocated memory for at least `index + 1` elements
/// - `index` must be within bounds of the allocated array
/// - The element at `index` must not be already initialized
/// - Caller is responsible for ensuring element gets properly dropped
unsafe fn initialize_element<T>(ptr: *mut T, index: usize, value: T) {
    ptr::write(ptr.add(index), value);
}
```

### Step 2: Safe Abstraction That Maintains All Invariants

```rust
/// A dynamically-allocated array that maintains safety invariants
///
/// This safe wrapper ensures:
/// - Memory is properly allocated and deallocated
/// - All elements are properly initialized before access
/// - Bounds checking prevents out-of-bounds access
/// - Proper cleanup occurs even if panics happen
pub struct SafeArray<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> SafeArray<T> {
    /// Creates a new SafeArray with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return SafeArray {
                ptr: std::ptr::NonNull::dangling().as_ptr(),
                len: 0,
                capacity: 0,
            };
        }

        let ptr = unsafe {
            // SAFETY: capacity > 0 (checked above)
            // We store the capacity and will deallocate with same count
            allocate_array::<T>(capacity)
        };

        SafeArray {
            ptr,
            len: 0,
            capacity,
        }
    }

    /// Pushes a new element to the end of the array
    ///
    /// This is safe because it:
    /// - Checks bounds before writing
    /// - Only writes to uninitialized memory (len..capacity)
    /// - Updates len after successful initialization
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len >= self.capacity {
            return Err(value); // Array is full
        }

        unsafe {
            // SAFETY:
            // - ptr is valid (allocated in constructor)
            // - len < capacity (checked above)
            // - Element at len is uninitialized (guaranteed by len invariant)
            initialize_element(self.ptr, self.len, value);
        }

        self.len += 1;
        Ok(())
    }

    /// Gets a reference to an element at the given index
    ///
    /// This is safe because it:
    /// - Performs bounds checking
    /// - Only accesses initialized memory (0..len)
    /// - Returns a proper Rust reference with lifetime tied to self
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe {
                // SAFETY:
                // - ptr is valid (allocated in constructor)
                // - index < len (checked above)
                // - Element is initialized (guaranteed by len invariant)
                Some(&*self.ptr.add(index))
            }
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
```

### Step 3: Automatic Safety Through Drop

```rust
impl<T> Drop for SafeArray<T> {
    /// Ensures proper cleanup of all resources
    ///
    /// This is safe because it:
    /// - Only drops initialized elements (0..len)
    /// - Deallocates with the same parameters used for allocation
    /// - Handles the zero-capacity case correctly
    fn drop(&mut self) {
        // Drop all initialized elements
        for i in 0..self.len {
            unsafe {
                // SAFETY: i < len, so element is initialized
                ptr::drop_in_place(self.ptr.add(i));
            }
        }

        // Deallocate memory
        if self.capacity > 0 {
            unsafe {
                // SAFETY:
                // - ptr was allocated with allocate_array(capacity)
                // - All elements have been dropped above
                // - We're calling this exactly once
                deallocate_array(self.ptr, self.capacity);
            }
        }
    }
}
```

### Step 4: The Result - Safe API

```rust
fn main() {
    // Safe usage - contract violations are impossible
    let mut safe_array = SafeArray::with_capacity(3);

    safe_array.push(10).unwrap();
    safe_array.push(20).unwrap();

    // Bounds checking prevents errors
    println!("Element at index 1: {:?}", safe_array.get(1));  // Some(20)
    println!("Out of bounds: {:?}", safe_array.get(10));      // None

    // Automatic cleanup happens when safe_array is dropped
}
```

### Key Benefits of This Pattern

- **Unsafe operations** document exact safety requirements
- **Safe abstractions** maintain all invariants internally
- **Users get memory safety** without thinking about contracts
- **Panic safety** is handled by the Drop implementation
- **Bounds checking** prevents common errors
- **Zero runtime cost** - compiles to same performance as manual unsafe code

This is the same pattern used throughout Rust's standard library: unsafe foundations with documented contracts, wrapped in safe abstractions that make contract violations impossible.

## Common Pitfalls

### 1. Dangling Pointers

```rust
// BAD: Returning pointer to local variable
fn bad_pointer() -> *const i32 {
    let x = 42;
    &x as *const i32  // x is dropped, pointer becomes invalid
}

// GOOD: Return owned data or use proper lifetimes
fn good_pointer() -> i32 {
    42  // Return the value itself
}
```

### 2. Data Races

```rust
// BAD: Unsynchronized access to static
static mut COUNTER: i32 = 0;

fn bad_increment() {
    unsafe {
        COUNTER += 1;  // Data race if called from multiple threads
    }
}

// GOOD: Use atomic types
use std::sync::atomic::{AtomicI32, Ordering};
static COUNTER: AtomicI32 = AtomicI32::new(0);

fn good_increment() {
    COUNTER.fetch_add(1, Ordering::SeqCst);
}
```

### 3. Buffer Overflows

```rust
// BAD: No bounds checking
unsafe fn bad_array_access(arr: *mut i32, index: usize, value: i32) {
    *arr.add(index) = value;  // No bounds checking!
}

// GOOD: Use safe abstractions when possible
fn good_array_access(arr: &mut [i32], index: usize, value: i32) -> Result<(), &'static str> {
    if index < arr.len() {
        arr[index] = value;
        Ok(())
    } else {
        Err("Index out of bounds")
    }
}
```

## Try It Yourself

### Exercise 1: Safe Buffer
Implement a growing buffer that safely handles reallocation:
```rust
struct GrowingBuffer {
    // TODO: Add fields
}

impl GrowingBuffer {
    fn new() -> Self { todo!() }
    fn push(&mut self, byte: u8) { todo!() }
    fn as_slice(&self) -> &[u8] { todo!() }
}
```

### Exercise 2: C Library Wrapper
Create a safe wrapper for a C string manipulation library:
```rust
extern "C" {
    fn str_reverse(s: *mut c_char);
    fn str_upper(s: *mut c_char);
}

// TODO: Create safe wrappers
```

### Exercise 3: Bindgen Practice
Use bindgen to create bindings for a simple math library and wrap the unsafe functions in safe Rust APIs.

## Best Practices

- **Minimize unsafe scope** - Keep `unsafe` blocks as small as possible
- **Document safety contracts** - Explain what makes the code safe
- **Provide safe abstractions** - Wrap unsafe code in safe interfaces
- **Use existing solutions** - Don't reinvent memory management
- **Test thoroughly** - Unsafe code needs extra testing
- **Consider Miri** - Use Miri to detect undefined behavior

---

Next: [Chapter 25: Embedded HAL - Registers, SVD2Rust & Volatile Access](./25_embedded_hal.md)