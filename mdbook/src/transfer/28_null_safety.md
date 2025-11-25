# Chapter 21: Null Safety - Option<T> vs Null Pointers
## Eliminating Null Reference Exceptions Forever

### The Billion Dollar Mistake

Tony Hoare called null references his "billion-dollar mistake." Here's how each language handles nullability:

| Language | Null Representation | Compile-time Safety | Runtime Safety |
|----------|-------------------|-------------------|----------------|
| **C++** | `nullptr`, raw pointers | No | Segmentation faults |
| **C#/.NET** | `null`, `Nullable<T>` | Partial (C# 8+) | NullReferenceException |
| **Rust** | `Option<T>` | Complete | Impossible to dereference null |

---

## From Null Pointers to Option<T>

### C++ Null Handling
```cpp
std::string* find_user(int id) {
    if (id == 1) {
        return new std::string("Alice");  // Caller must delete!
    }
    return nullptr;  // Potential crash site
}

void use_user() {
    auto user = find_user(42);
    if (user != nullptr) {  // Must remember to check!
        std::cout << *user << std::endl;
        delete user;  // Must remember to delete!
    }
}
```

### C# Null Handling
```csharp
string FindUser(int id) {
    if (id == 1) {
        return "Alice";
    }
    return null;  // Runtime bomb waiting to explode
}

void UseUser() {
    var user = FindUser(42);
    if (user != null) {  // Must remember to check!
        Console.WriteLine(user);
    }
    // Or use nullable reference types (C# 8+)
    string? nullableUser = FindUser(42);
    Console.WriteLine(nullableUser?.Length ?? 0);
}
```

### Rust Option<T> Handling
```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some("Alice".to_string())
    } else {
        None
    }
}

fn use_user() {
    let user = find_user(42);
    match user {
        Some(name) => println!("Found user: {}", name),
        None => println!("User not found"),
    }
    
    // Or use if let
    if let Some(name) = find_user(1) {
        println!("User: {}", name);
    }
    
    // Impossible to forget null check - won't compile otherwise!
}
```

---

## Option<T> Patterns

### Safe Unwrapping
```rust
fn safe_option_handling() {
    let maybe_number: Option<i32> = Some(42);
    
    // Pattern matching (preferred)
    match maybe_number {
        Some(n) => println!("Number: {}", n),
        None => println!("No number"),
    }
    
    // Provide default value
    let number = maybe_number.unwrap_or(0);
    
    // Lazy default computation
    let number = maybe_number.unwrap_or_else(|| {
        println!("Computing default...");
        100
    });
    
    // Transform if Some
    let doubled = maybe_number.map(|n| n * 2);
    
    // Chain operations
    let result = maybe_number
        .map(|n| n * 2)
        .filter(|n| *n > 50)
        .unwrap_or(0);
}
```

### Combining Options
```rust
fn combine_options() {
    let opt1 = Some(10);
    let opt2 = Some(20);
    let opt3: Option<i32> = None;
    
    // Combine two Options
    let sum = match (opt1, opt2) {
        (Some(a), Some(b)) => Some(a + b),
        _ => None,
    };
    
    // Or use and_then for chaining
    let result = opt1.and_then(|a| {
        opt2.map(|b| a + b)
    });
    
    println!("Sum: {:?}", sum);  // Some(30)
}
```

---

## Migration Strategies

### From C++ Pointers to Option<T>
```cpp
// C++ - prone to crashes
class UserService {
    User* currentUser = nullptr;
    
public:
    User* getCurrentUser() { return currentUser; }
    
    void setCurrentUser(User* user) {
        delete currentUser;  // Potential double-delete
        currentUser = user;
    }
};
```

```rust
// Rust - impossible to crash
struct UserService {
    current_user: Option<User>,
}

impl UserService {
    fn new() -> Self {
        UserService {
            current_user: None,
        }
    }
    
    fn get_current_user(&self) -> Option<&User> {
        self.current_user.as_ref()
    }
    
    fn set_current_user(&mut self, user: User) {
        self.current_user = Some(user);  // Old user automatically dropped
    }
    
    fn clear_current_user(&mut self) {
        self.current_user = None;  // User automatically dropped
    }
}
```

### From C# Nullables to Option<T>
```csharp
// C# - runtime exceptions possible
public class ConfigService {
    public string? DatabaseUrl { get; set; }
    
    public void Connect() {
        if (DatabaseUrl != null) {
            // Connect to DatabaseUrl
            Console.WriteLine($"Connecting to {DatabaseUrl}");
        } else {
            throw new InvalidOperationException("Database URL not configured");
        }
    }
}
```

```rust
// Rust - compile-time safety
struct ConfigService {
    database_url: Option<String>,
}

impl ConfigService {
    fn new() -> Self {
        ConfigService {
            database_url: None,
        }
    }
    
    fn set_database_url(&mut self, url: String) {
        self.database_url = Some(url);
    }
    
    fn connect(&self) -> Result<(), &'static str> {
        match &self.database_url {
            Some(url) => {
                println!("Connecting to {}", url);
                Ok(())
            },
            None => Err("Database URL not configured"),
        }
    }
}
```

---

## Advanced Option Patterns

### Option with Complex Types
```rust
#[derive(Debug)]
struct DatabaseConnection {
    url: String,
    pool_size: u32,
}

struct Application {
    db_connection: Option<DatabaseConnection>,
    cache_connection: Option<String>,
}

impl Application {
    fn new() -> Self {
        Application {
            db_connection: None,
            cache_connection: None,
        }
    }
    
    fn configure_database(&mut self, url: String, pool_size: u32) {
        self.db_connection = Some(DatabaseConnection { url, pool_size });
    }
    
    fn is_fully_configured(&self) -> bool {
        self.db_connection.is_some() && self.cache_connection.is_some()
    }
    
    fn start(&self) -> Result<(), &'static str> {
        let db = self.db_connection.as_ref()
            .ok_or("Database not configured")?;
        let cache = self.cache_connection.as_ref()
            .ok_or("Cache not configured")?;
            
        println!("Starting with DB: {} and Cache: {}", db.url, cache);
        Ok(())
    }
}
```

---

## Performance Comparison

### Memory Layout
```rust
// Option<T> is optimized for common cases
use std::mem::size_of;

fn size_comparison() {
    // These are the same size - null pointer optimization
    assert_eq!(size_of::<Option<&str>>(), size_of::<&str>());
    assert_eq!(size_of::<Option<Box<i32>>>(), size_of::<Box<i32>>());
    
    // Option<T> adds minimal overhead for most types
    println!("i32 size: {}", size_of::<i32>());                // 4 bytes
    println!("Option<i32> size: {}", size_of::<Option<i32>>());  // 8 bytes (includes tag)
    
    // Zero cost for nullable pointers
    println!("&str size: {}", size_of::<&str>());                    // 16 bytes
    println!("Option<&str> size: {}", size_of::<Option<&str>>());    // 16 bytes (same!)
}
```

### Runtime Performance
- **Option<T>** compiles to efficient machine code
- **Pattern matching** becomes jump tables
- **No null pointer dereferencing checks** at runtime
- **Compiler optimizations** eliminate many Option operations

---

## Key Takeaways

1. **Option<T> makes nullability explicit** in the type system
2. **Impossible to forget null checks** - compiler enforces handling
3. **Zero-cost abstraction** - no runtime overhead for safety
4. **Rich API** for working with optional values
5. **Composable** - Options work well with other Rust features
6. **Better than nullable types** - compile-time guarantees vs runtime hopes

### Migration Checklist
- Replace `nullptr`/`null` returns with `Option<T>`
- Use `Option<&T>` instead of nullable references
- Prefer pattern matching over `.unwrap()`
- Use combinators (`map`, `and_then`, `filter`) for transformations
- Remember: if you can avoid `Option<T>`, that's often better

**The Result:** Code that simply cannot have null pointer exceptions. The compiler won't let you forget to handle the None case, making your programs fundamentally more reliable.

---

Next: [Chapter 28: Type System Differences](./28_type_differences.md)
