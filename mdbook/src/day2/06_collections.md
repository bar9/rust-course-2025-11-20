# Chapter 6: Collections Beyond Vec
## HashMap and HashSet for Real-World Applications

### Learning Objectives
By the end of this chapter, you'll be able to:
- Use HashMap<K, V> efficiently for key-value storage
- Apply HashSet<T> for unique value collections
- Master the Entry API for efficient map operations
- Choose between HashMap, BTreeMap, and other collections
- Work with custom types as keys

---

## Quick Collection Reference

| Collection | Use When You Need | Performance |
|------------|-------------------|-------------|
| `Vec<T>` | Ordered sequence, index access | O(1) index, O(n) search |
| `HashMap<K,V>` | Fast key-value lookups | O(1) average all operations |
| `HashSet<T>` | Unique values, fast membership test | O(1) average all operations |
| `BTreeMap<K,V>` | Sorted keys, range queries | O(log n) all operations |

---

## HashMap<K, V>: The Swiss Army Knife

### Basic Operations

```rust
use std::collections::HashMap;

fn hashmap_basics() {
    // Creation
    let mut scores = HashMap::new();
    scores.insert("Alice", 100);
    scores.insert("Bob", 85);
    
    // From iterator
    let teams = vec!["Blue", "Red"];
    let points = vec![10, 50];
    let team_scores: HashMap<_, _> = teams.into_iter()
        .zip(points.into_iter())
        .collect();
    
    // Accessing values
    if let Some(score) = scores.get("Alice") {
        println!("Alice's score: {}", score);
    }
    
    // Check existence
    if scores.contains_key("Alice") {
        println!("Alice is in the map");
    }
}
```

### The Entry API: Powerful and Efficient

```rust
use std::collections::HashMap;

fn entry_api_examples() {
    let mut word_count = HashMap::new();
    let text = "the quick brown fox jumps over the lazy dog the";
    
    // Count words efficiently
    for word in text.split_whitespace() {
        *word_count.entry(word).or_insert(0) += 1;
    }
    
    // Insert if absent
    let mut cache = HashMap::new();
    cache.entry("key").or_insert_with(|| {
        // Expensive computation only runs if key doesn't exist
        expensive_calculation()
    });
    
    // Modify or insert
    let mut scores = HashMap::new();
    scores.entry("Alice")
        .and_modify(|score| *score += 10)
        .or_insert(100);
}

fn expensive_calculation() -> String {
    "computed_value".to_string()
}
```

### HashMap with Custom Keys

```rust
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Hash)]
struct UserId(u64);

#[derive(Debug, Eq, PartialEq, Hash)]
struct CompositeKey {
    category: String,
    id: u32,
}

fn custom_keys() {
    let mut user_data = HashMap::new();
    user_data.insert(UserId(1001), "Alice");
    user_data.insert(UserId(1002), "Bob");
    
    let mut composite_map = HashMap::new();
    composite_map.insert(
        CompositeKey { category: "user".to_string(), id: 1 },
        "User One"
    );
    
    // Access with custom key
    if let Some(name) = user_data.get(&UserId(1001)) {
        println!("Found user: {}", name);
    }
}
```

---

## HashSet<T>: Unique Value Collections

### Basic Operations and Set Theory

```rust
use std::collections::HashSet;

fn hashset_operations() {
    // Create and populate
    let mut set1: HashSet<i32> = vec![1, 2, 3, 2, 4].into_iter().collect();
    let set2: HashSet<i32> = vec![3, 4, 5, 6].into_iter().collect();
    
    // Set operations
    let union: HashSet<_> = set1.union(&set2).cloned().collect();
    let intersection: HashSet<_> = set1.intersection(&set2).cloned().collect();
    let difference: HashSet<_> = set1.difference(&set2).cloned().collect();
    
    println!("Union: {:?}", union);           // {1, 2, 3, 4, 5, 6}
    println!("Intersection: {:?}", intersection); // {3, 4}
    println!("Difference: {:?}", difference);     // {1, 2}
    
    // Check membership
    if set1.contains(&3) {
        println!("Set contains 3");
    }
    
    // Insert returns bool indicating if value was new
    if set1.insert(10) {
        println!("10 was added (wasn't present before)");
    }
}

fn practical_hashset_use() {
    // Track visited items
    let mut visited = HashSet::new();
    let items = vec!["home", "about", "home", "contact", "about"];
    
    for item in items {
        if visited.insert(item) {
            println!("First visit to: {}", item);
        } else {
            println!("Already visited: {}", item);
        }
    }
}
```

---

## When to Use BTreeMap/BTreeSet

Use **BTreeMap/BTreeSet** when you need:
- Keys/values in sorted order
- Range queries (`map.range("a".."c")`)
- Consistent iteration order
- No hash function available for keys

```rust
use std::collections::BTreeMap;

// Example: Leaderboard that needs sorted scores
let mut leaderboard = BTreeMap::new();
leaderboard.insert(95, "Alice");
leaderboard.insert(87, "Bob");
leaderboard.insert(92, "Charlie");

// Iterate in score order (ascending)
for (score, name) in &leaderboard {
    println!("{}: {}", name, score);
}

// Get top 3 scores
let top_scores: Vec<_> = leaderboard
    .iter()
    .rev()  // Reverse for descending order
    .take(3)
    .collect();
```

---

## Common Pitfalls

### HashMap Key Requirements

```rust
use std::collections::HashMap;

// ❌ f64 doesn't implement Eq (NaN issues)
// let mut map: HashMap<f64, String> = HashMap::new();

// ✅ Use ordered wrapper or integer representation
#[derive(Debug, PartialEq, Eq, Hash)]
struct OrderedFloat(i64); // Store as integer representation

impl From<f64> for OrderedFloat {
    fn from(f: f64) -> Self {
        OrderedFloat(f.to_bits() as i64)
    }
}
```

### Borrowing During Iteration

```rust
// ❌ Can't modify while iterating
// for (key, value) in &map {
//     map.insert(new_key, new_value); // Error!
// }

// ✅ Collect changes first, apply after
let changes: Vec<_> = map.iter()
    .filter(|(_, &v)| v > threshold)
    .map(|(k, v)| (format!("new_{}", k), v * 2))
    .collect();

for (key, value) in changes {
    map.insert(key, value);
}
```

---

## Exercise: Build a Cache System

Create an LRU (Least Recently Used) cache with expiration:

```rust
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

struct CacheEntry<V> {
    value: V,
    last_accessed: Instant,
    expires_at: Option<Instant>,
}

struct LRUCache<K: Clone + Eq + std::hash::Hash, V> {
    capacity: usize,
    cache: HashMap<K, CacheEntry<V>>,
    access_order: VecDeque<K>,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> LRUCache<K, V> {
    fn new(capacity: usize) -> Self {
        LRUCache {
            capacity,
            cache: HashMap::new(),
            access_order: VecDeque::new(),
        }
    }
    
    fn get(&mut self, key: &K) -> Option<&V> {
        // TODO: Check if key exists
        // TODO: Check if entry is expired (remove if expired)
        // TODO: Update last_accessed time
        // TODO: Move key to end of access_order (most recently used)
        // TODO: Return the value
        todo!()
    }
    
    fn insert(&mut self, key: K, value: V, ttl: Option<Duration>) {
        // TODO: If at capacity, remove least recently used item
        // TODO: Create cache entry with expiration if ttl provided
        // TODO: Add to cache and access_order
        todo!()
    }
    
    fn remove(&mut self, key: &K) -> Option<V> {
        // TODO: Remove from cache and access_order
        // TODO: Return the value if it existed
        todo!()
    }
    
    fn clear_expired(&mut self) {
        // TODO: Remove all expired entries
        todo!()
    }
    
    fn stats(&self) -> (usize, usize) {
        // Return (current_size, capacity)
        (self.cache.len(), self.capacity)
    }
}

fn main() {
    let mut cache = LRUCache::new(3);
    
    // Test basic operations
    cache.insert("user:1", "Alice", Some(Duration::from_secs(60)));
    cache.insert("user:2", "Bob", None);  // No expiration
    cache.insert("user:3", "Charlie", Some(Duration::from_secs(5)));
    
    // Access user:1 to make it most recently used
    println!("Got: {:?}", cache.get(&"user:1"));
    
    // Add one more - should evict user:2 (least recently used)
    cache.insert("user:4", "David", None);
    
    // Try to get user:2 - should be None (evicted)
    println!("User 2 (should be evicted): {:?}", cache.get(&"user:2"));
    
    let (size, capacity) = cache.stats();
    println!("Cache stats - Size: {}/{}", size, capacity);
}
```

**Hints:**
- Use `VecDeque::retain()` to keep only non-expired keys
- HashMap's `entry()` API is useful for get-or-insert patterns
- Remember to update both the HashMap and VecDeque when modifying

---

## Key Takeaways

1. **HashMap<K,V>** for fast key-value lookups with the Entry API for efficiency
2. **HashSet<T>** for unique values and set operations
3. **BTreeMap/BTreeSet** when you need sorted data or range queries
4. **Custom keys** must implement Hash + Eq (or Ord for BTree*)
5. **Can't modify while iterating** - collect changes first
6. **Entry API** prevents redundant lookups and improves performance

**Next Up:** In Chapter 7, we'll explore traits - Rust's powerful system for defining shared behavior and enabling polymorphism without inheritance.