# Chapter 21: Macros & Code Generation

## Learning Objectives
- Understand declarative macros with `macro_rules!`
- Master pattern matching syntax variants
- Learn procedural macros basics
- Generate code at compile time

## What are Macros?

Macros are code that writes other code (metaprogramming). They run at compile time, generating Rust code that gets compiled normally.

```rust
// This macro call
println!("Hello, {}!", "world");

// Expands to something like this (simplified)
std::io::_print(format_args!("Hello, {}!\n", "world"));
```

## Declarative Macros with `macro_rules!`

### Basic Syntax

```rust
macro_rules! say_hello {
    () => {
        println!("Hello!");
    };
}

say_hello!(); // Prints: Hello!
```

### Pattern Matching Types

#### 1. `item` - Items like functions, structs, modules

```rust
macro_rules! create_function {
    ($func_name:ident) => {
        fn $func_name() {
            println!("You called {}!", stringify!($func_name));
        }
    };
}

create_function!(foo);
foo(); // Prints: You called foo!
```

#### 2. `block` - Code blocks

```rust
macro_rules! time_it {
    ($block:block) => {
        let start = std::time::Instant::now();
        $block
        println!("Took: {:?}", start.elapsed());
    };
}

time_it!({
    std::thread::sleep(std::time::Duration::from_millis(100));
    println!("Work done!");
});
```

#### 3. `stmt` - Statements

```rust
macro_rules! debug_stmt {
    ($stmt:stmt) => {
        println!("Executing: {}", stringify!($stmt));
        $stmt
    };
}

debug_stmt!(let x = 42;);
```

#### 4. `expr` - Expressions

```rust
macro_rules! double {
    ($e:expr) => {
        $e * 2
    };
}

let result = double!(5 + 3); // 16
```

#### 5. `ty` - Types

```rust
macro_rules! create_struct {
    ($name:ident, $field_type:ty) => {
        struct $name {
            value: $field_type,
        }
    };
}

create_struct!(MyStruct, i32);
```

#### 6. `ident` - Identifiers

```rust
macro_rules! getter {
    ($field:ident) => {
        fn $field(&self) -> &str {
            &self.$field
        }
    };
}
```

#### 7. `path` - Paths like `std::vec::Vec`

```rust
macro_rules! use_type {
    ($path:path) => {
        let _instance: $path = Default::default();
    };
}

use_type!(std::collections::HashMap<String, i32>);
```

#### 8. `literal` - Literal values

```rust
macro_rules! print_literal {
    ($lit:literal) => {
        println!("Literal: {}", $lit);
    };
}

print_literal!("hello");
print_literal!(42);
```

#### 9. `tt` - Token trees (any valid tokens)

```rust
macro_rules! capture_tokens {
    ($($tt:tt)*) => {
        println!("Tokens: {}", stringify!($($tt)*));
    };
}

capture_tokens!(fn main() { println!("anything"); });
```

### Repetition Patterns

#### `*` - Zero or more repetitions

```rust
macro_rules! print_all {
    ($($item:expr),*) => {
        $(
            println!("{}", $item);
        )*
    };
}

print_all!(1, 2, 3, "hello");
```

#### `+` - One or more repetitions

```rust
macro_rules! sum {
    ($first:expr $(, $rest:expr)+) => {
        $first $(+ $rest)+
    };
}

let result = sum!(1, 2, 3, 4); // 10
```

#### `?` - Zero or one repetition

```rust
macro_rules! optional_print {
    ($msg:expr $(, $extra:expr)?) => {
        print!("{}", $msg);
        $(print!(" {}", $extra);)?
        println!();
    };
}

optional_print!("Hello");           // Hello
optional_print!("Hello", "World");  // Hello World
```

### Multiple Pattern Arms

```rust
macro_rules! calculate {
    (add $a:expr, $b:expr) => {
        $a + $b
    };
    (multiply $a:expr, $b:expr) => {
        $a * $b
    };
    (power $base:expr, $exp:expr) => {
        // For integer types, pow requires u32 as exponent
        ($base as i32).pow($exp as u32)
    };
}

let sum = calculate!(add 5, 3);        // 8
let product = calculate!(multiply 4, 7); // 28
let power = calculate!(power 2, 3);    // 8
```

### Practical Example: HashMap Creation

```rust
macro_rules! hashmap {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key, $value);
            )*
            map
        }
    };
}

let ages = hashmap! {
    "Alice" => 30,
    "Bob" => 25,
    "Carol" => 35,
};
```

## Procedural Macros

Procedural macros are more powerful than declarative macros. They operate on token streams using Rust code.

### Setting Up a Proc Macro Crate

```toml
# Cargo.toml
[package]
name = "my_macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true  # This makes it a proc macro crate

[dependencies]
syn = { version = "2.0", features = ["full", "extra-traits"] }
quote = "1.0"
proc-macro2 = "1.0"
```

### The Three Essential Crates

1. **`proc-macro`** - Compiler's API for procedural macros
2. **`proc-macro2`** - Wrapper that enables testing and better ergonomics
3. **`syn`** - Parsing Rust code into syntax trees
4. **`quote`** - Generating Rust code from syntax trees

### Function-like Macros

```rust
// src/lib.rs
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn make_answer(_input: TokenStream) -> TokenStream {
    let output = quote! {
        fn answer() -> u32 { 42 }
    };
    TokenStream::from(output)
}

// Usage in another crate:
use my_macros::make_answer;

make_answer!();
assert_eq!(answer(), 42);
```

### Derive Macros

#### Basic Derive Macro

```rust
// src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MyDebug)]
pub fn my_debug_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}(custom debug)", stringify!(#name))
            }
        }
    };

    TokenStream::from(expanded)
}

// Usage:
#[derive(MyDebug)]
struct Person {
    name: String,
    age: u32,
}

let p = Person { name: "Alice".into(), age: 30 };
println!("{:?}", p); // Person(custom debug)
```

#### Derive with Helper Attributes

```rust
// Derive macro with custom attributes
#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let builder_name = syn::Ident::new(
        &format!("{}Builder", name),
        name.span()
    );

    // Extract fields from struct
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => panic!("Builder can only be derived for structs"),
    };

    // Generate builder implementation
    let builder_fields: Vec<_> = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            #name: Option<#ty>
        }
    }).collect();

    let builder_methods = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        quote! {
            pub fn #name(mut self, value: #ty) -> Self {
                self.#name = Some(value);
                self
            }
        }
    });

    let build_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: self.#name.expect(concat!(stringify!(#name), " is required"))
        }
    });

    // Create initial values for builder
    let builder_init: Vec<_> = fields.iter().map(|f| {
        let name = &f.ident;
        quote! {
            #name: None
        }
    }).collect();

    let expanded = quote! {
        pub struct #builder_name {
            #(#builder_fields,)*
        }

        impl #builder_name {
            #(#builder_methods)*

            pub fn build(self) -> #name {
                #name {
                    #(#build_fields,)*
                }
            }
        }

        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#builder_init,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

// Usage:
#[derive(Builder)]
struct Config {
    host: String,
    port: u16,
    #[builder(default = "false")]
    debug: bool,
}

let config = Config::builder()
    .host("localhost".into())
    .port(8080)
    .build();
```

### Attribute Macros

```rust
// Attribute macro for logging function calls
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn logged(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as syn::ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_body = &input_fn.block;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;

    let output = quote! {
        #fn_vis #fn_sig {
            println!("Calling function: {}", stringify!(#fn_name));
            let result = #fn_body;
            println!("Function {} completed", stringify!(#fn_name));
            result
        }
    };

    TokenStream::from(output)
}

// Usage:
#[logged]
fn calculate(x: i32, y: i32) -> i32 {
    x + y
}
```

### Advanced: Parsing Custom Syntax

```rust
use syn::{parse::{Parse, ParseStream}, Result};
use syn::{Token, Ident, Expr};

// Define custom syntax: name = value
struct NameValue {
    name: Ident,
    equals: Token![=],
    value: Expr,
}

impl Parse for NameValue {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(NameValue {
            name: input.parse()?,
            equals: input.parse()?,
            value: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn config(input: TokenStream) -> TokenStream {
    let NameValue { name, value, .. } = parse_macro_input!(input as NameValue);

    let output = quote! {
        const #name: &str = #value;
    };

    TokenStream::from(output)
}

// Usage:
// Usage:
// config!(DATABASE_URL = "postgresql://localhost/mydb");
```

## Publishing and Using Macros

### Publishing a Macro Crate

#### Directory Structure
```
my_macros/
├── Cargo.toml
├── src/
│   └── lib.rs
└── tests/
    └── integration.rs
```

#### Main Crate with Re-exports
```toml
# Cargo.toml for a crate that includes macros
[package]
name = "my_library"
version = "0.1.0"

[dependencies]
my_library_macros = { version = "0.1.0", path = "./macros" }

[features]
default = ["derive"]
derive = ["my_library_macros/derive"]
```

```rust
// src/lib.rs - Re-exporting macros
// Re-export derive macros
#[cfg(feature = "derive")]
pub use my_library_macros::{MyTrait, Builder};

// Re-export proc macros
pub use my_library_macros::{config, logged};

// The trait that the derive macro implements
pub trait MyTrait {
    fn my_method(&self);
}
```

### Using Macros from Other Crates

```toml
# Consumer's Cargo.toml
[dependencies]
# Using derive features
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }

# Custom macro crate
my_library = "0.1"
```

```rust
// Using the macros
use my_library::{MyTrait, Builder};
use serde::{Serialize, Deserialize};

#[derive(MyTrait, Builder, Serialize, Deserialize)]
struct Data {
    id: u32,
    name: String,
}
```

### Optional Dependencies and Features

```toml
# Advanced Cargo.toml with optional macro features
[package]
name = "advanced_lib"
version = "0.1.0"

[dependencies]
syn = { version = "2.0", optional = true }
quote = { version = "1.0", optional = true }
proc-macro2 = { version = "1.0", optional = true }

# Optional macro dependencies
[dependencies.advanced_lib_macros]
version = "0.1.0"
path = "./macros"
optional = true

[features]
default = []
derive = ["advanced_lib_macros", "syn", "quote", "proc-macro2"]
full = ["derive"]

# Documentation metadata
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

## Testing Procedural Macros

### Unit Testing with `proc-macro2`

```rust
// tests/test_macro.rs
use proc_macro2::TokenStream;
use quote::quote;

fn test_expansion(input: TokenStream) -> TokenStream {
    // Your macro logic here, using proc_macro2 types
    quote! {
        fn generated_function() {}
    }
}

#[test]
fn test_macro_expansion() {
    let input = quote! {
        struct TestStruct;
    };

    let output = test_expansion(input);
    let expected = quote! {
        fn generated_function() {}
    };

    assert_eq!(output.to_string(), expected.to_string());
}
```

### Integration Testing

```rust
// tests/integration.rs
// Assuming MyDerive generates a method called generated_method
use my_macros::MyDerive;

#[derive(MyDerive)]
struct TestStruct {
    field: String,
}

#[test]
fn test_derive_works() {
    let instance = TestStruct {
        field: "test".into(),
    };
    // Test the generated code
    instance.generated_method();
}
```

### Compile-time Testing with `trybuild`

```toml
# Cargo.toml
[dev-dependencies]
trybuild = "1.0"
```

```rust
// tests/compile_tests.rs
#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-parse-struct.rs");
    t.compile_fail("tests/02-invalid-syntax.rs");
}
```

## Debugging Macros

### Using `cargo expand`

```bash
# Install cargo-expand
cargo install cargo-expand

# Expand all macros
cargo expand

# Expand specific module
cargo expand module::submodule

# Expand specific test
cargo expand --test test_name

# Pretty print with syntax highlighting
cargo expand --theme=GitHub
```

### Debug Printing in Proc Macros

```rust
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn debug_tokens(input: TokenStream) -> TokenStream {
    // Print tokens during compilation
    eprintln!("Input tokens: {:#?}", input);

    let output = quote! {
        // Generated code
    };

    eprintln!("Output tokens: {}", output);
    TokenStream::from(output)
}
```

### Using `syn`'s Debug Features

```rust
// Enable extra-traits for better debug output
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MyMacro)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Debug print the parsed syntax tree
    eprintln!("Parsed input: {:#?}", input);

    // Your macro logic
    TokenStream::from(quote! {})
}
```

## Advanced Metaprogramming Features

### Const Generics in Macros

```rust
macro_rules! fixed_array {
    ($t:ty; $size:expr) => {
        [$t; $size]
    };
}

type Array32 = fixed_array!(i32; 32);
```

### Type-level Programming

```rust
// Type state pattern with macros
macro_rules! state_machine {
    (
        $name:ident {
            states: [$($state:ident),+],
            transitions: [
                $($from:ident -> $to:ident),+
            ]
        }
    ) => {
        $(struct $state;)+

        struct $name<State> {
            _state: std::marker::PhantomData<State>,
        }

        $(impl $name<$from> {
            fn transition(self) -> $name<$to> {
                $name { _state: std::marker::PhantomData }
            }
        })+
    };
}

state_machine! {
    Door {
        states: [Open, Closed, Locked],
        transitions: [
            Open -> Closed,
            Closed -> Open,
            Closed -> Locked,
            Locked -> Closed
        ]
    }
}
```

### Compile-time Validation

```rust
// Macro that validates at compile time
macro_rules! const_assert {
    ($cond:expr) => {
        const _: () = assert!($cond);
    };
}

const_assert!(std::mem::size_of::<u32>() == 4);
```

### Generated Documentation

```rust
// Proc macro that generates documentation
#[proc_macro_derive(Documented, attributes(doc_string))]
pub fn derive_documented(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract doc string from attributes (simplified)
    let doc_string = "Auto-generated documentation";

    let output = quote! {
        impl #name {
            #[doc = #doc_string]
            pub fn description(&self) -> &'static str {
                #doc_string
            }
        }
    };

    TokenStream::from(output)
}
```

## Common Patterns

### Creating DSLs (Domain Specific Languages)

```rust
macro_rules! html {
    ($tag:ident { $($content:tt)* }) => {
        format!("<{0}>{1}</{0}>", stringify!($tag), html!($($content)*))
    };
    ($text:literal) => {
        $text.to_string()
    };
    ($($content:tt)*) => {
        format!("{}", stringify!($($content)*))
    };
}

let page = html! {
    div {
        h1 { "Welcome" }
        p { "Hello World" }
    }
};
```

### Configuration Macros

```rust
macro_rules! config {
    ($($key:ident: $value:expr),*) => {
        struct Config {
            $(pub $key: String,)*
        }

        impl Default for Config {
            fn default() -> Self {
                Config {
                    $($key: $value.to_string(),)*
                }
            }
        }
    };
}

config! {
    host: "localhost",
    port: "8080",
    debug: "true"
}
```

## Try It Yourself

### Exercise 1: Math Operations
Create a macro that handles different math operations:
```rust
// Should work like:
let result = math!(5 + 3);
let result = math!(10 - 2);
let result = math!(4 * 6);
```

### Exercise 2: Struct Builder
Create a macro that builds structs with optional fields:
```rust
// Should generate:
build_struct!(Person {
    name: String,
    age?: u32,
    email?: String
});
```

### Exercise 3: Test Generator
Create a macro that generates multiple similar tests:
```rust
// Should generate test functions
generate_tests!(
    test_add: add(2, 3) == 5,
    test_sub: sub(5, 2) == 3,
    test_mul: mul(3, 4) == 12
);
```

## Best Practices

### Macro Design
- **Use declarative macros** for simple pattern-based code generation
- **Use proc macros** for complex AST manipulation
- **Prefer functions** when runtime computation suffices
- **Make macros composable** - design them to work well together
- **Provide good error messages** using `syn::Error` and spans

### Error Handling in Proc Macros

```rust
use syn::{Error, Result};
use quote::quote;

struct Config {
    // Config fields
}

fn parse_attributes(attrs: &[syn::Attribute]) -> Result<Config> {
    // Return informative errors with correct spans
    let _attr = attrs.iter()
        .find(|attr| attr.path().is_ident("my_attr"))
        .ok_or_else(|| {
            Error::new(
                proc_macro2::Span::call_site(),
                "missing required attribute #[my_attr]"
            )
        })?;
    // ... parse and return Config
    Ok(Config {})
}
```

### Performance Considerations
- **Cache parsed data** in proc macros when possible
- **Use `once_cell` or `lazy_static`** for expensive computations
- **Minimize dependencies** in proc macro crates
- **Consider compile-time impact** of complex macros

### Testing Strategy
1. **Unit test** macro logic with `proc-macro2`
2. **Integration test** actual macro usage
3. **Compile-fail tests** with `trybuild`
4. **Document examples** that are tested by `cargo test`
5. **Benchmark** compile-time impact for complex macros

## When to Use Macros

✅ **Good for:**
- Reducing boilerplate code (derive macros)
- Creating DSLs (declarative macros)
- Compile-time code generation
- Working with exact tokens/identifiers
- Implementing traits for multiple types
- Build-time configuration
- Zero-cost abstractions

❌ **Avoid for:**
- Simple calculations (use const functions)
- Type conversions (use `From`/`Into` traits)
- Runtime logic (use regular functions)
- Complex business logic (harder to debug)
- When generics would suffice

## Macro Ecosystem

### Popular Macro Crates
- **`serde`** - Serialization framework with derive macros
- **`tokio`** - Async runtime with attribute macros
- **`thiserror`** - Error type derivation
- **`derive_more`** - Common trait derivations
- **`paste`** - Token pasting in macros
- **`quote`** - Quasi-quoting for code generation

### Resources for Learning
- [The Little Book of Rust Macros](https://danielkeep.github.io/tlborm/book/)
- [Procedural Macro Workshop](https://github.com/dtolnay/proc-macro-workshop)
- [syn documentation](https://docs.rs/syn)
- [quote documentation](https://docs.rs/quote)
- [Cargo Book - Features](https://doc.rust-lang.org/cargo/reference/features.html)

---

Next: [Chapter 22: Unsafe Rust & FFI with Bindgen](./22_unsafe_ffi.md)