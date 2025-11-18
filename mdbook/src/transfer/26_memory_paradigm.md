# Chapter 20: Memory Management Paradigm Shift
## From C++ RAII/.NET GC to Rust Ownership

### Memory Management Comparison

| Aspect | C++ (Manual/RAII) | .NET (GC) | Rust (Ownership) |
|--------|-------------------|-----------|------------------|
| **Memory Safety** | Manual vigilance | Automatic at runtime | Compile-time guaranteed |
| **Resource Management** | RAII/Manual | GC-based | Ownership-based |
| **Performance** | High (when correct) | Variable (GC overhead) | Predictably high |
| **Determinism** | Manual control | GC pauses | Deterministic cleanup |
| **Zero-Cost Abstractions** | Limited | No | Yes |

---


## RAII to Ownership Migration

### C++ RAII Pattern
```cpp
class FileHandler {
    std::unique_ptr<FILE, decltype(&fclose)> file;
public:
    FileHandler(const char* name) : file(fopen(name, "r"), fclose) {}
    ~FileHandler() { /* automatic cleanup via unique_ptr */ }
};
```

### Rust Ownership Pattern
```rust
use std::fs::File;

struct FileHandler {
    file: File,  // Direct ownership, no pointers needed
}

impl FileHandler {
    fn new(name: &str) -> Result<Self, std::io::Error> {
        let file = File::open(name)?;
        Ok(FileHandler { file })
    }
    // Drop automatically implemented - file closed when dropped
}
```

---

## GC to Ownership Migration

### .NET Pattern
```csharp
public class DataProcessor {
    private List<Item> items = new List<Item>();
    
    public void Process() {
        // GC manages memory automatically
        var results = items.Select(item => item.Transform()).ToList();
        // Old collections eventually GC'd
    }
}
```

### Rust Pattern
```rust
struct DataProcessor {
    items: Vec<Item>,
}

impl DataProcessor {
    fn process(self) -> Vec<TransformedItem> {
        self.items
            .into_iter()  // Take ownership
            .map(|item| item.transform())
            .collect()    // Memory managed explicitly, zero overhead
    }
}
```

---

## Key Mental Model Shifts

1. **From "Who deletes?" to "Who owns?"** - Focus on ownership rather than cleanup
2. **From runtime safety to compile-time safety** - Bugs caught before deployment
3. **From unpredictable to predictable** - Know exactly when cleanup happens
4. **From complex to simple** - Less cognitive overhead once learned

### Performance Benefits
- **Zero-cost abstractions**: Safety without runtime overhead
- **No GC pauses**: Predictable latency for real-time systems
- **Cache-friendly**: Better memory layout control
- **Optimal resource usage**: Resources freed immediately when not needed

---

Next: [Chapter 27: Null Safety & Error Handling](./27_null_safety.md)
