# Chapter 23: Traits vs OOP - From Inheritance to Composition
## Mastering Zero-Cost Serialization with Trait-Based Design

### Serialization Paradigm Comparison

| Concept | C++ (Boost.Serialization) | C#/.NET (System.Text.Json) | Rust (Serde) |
|---------|---------------------------|----------------------------|---------------|
| **Code Generation** | Template-based | Runtime reflection | Compile-time macros |
| **Performance** | Good (but complex) | Variable (reflection) | Excellent (zero-cost) |
| **Schema Evolution** | Manual versioning | Attribute-driven | Type-safe traits |
| **Format Support** | Limited | JSON focus | Format agnostic |
| **Memory Usage** | Manual control | GC pressure | Zero-copy capable |
| **Error Handling** | Exceptions | Exception/nullable | Result types |

---

## From Inheritance to Composition

### C++ Inheritance Hierarchy
```cpp
class Animal {
protected:
    std::string name;
public:
    Animal(std::string n) : name(n) {}
    virtual void make_sound() = 0;
    virtual ~Animal() = default;
};

class Mammal : public Animal {
protected:
    bool warm_blooded = true;
public:
    Mammal(std::string n) : Animal(n) {}
};

class Dog : public Mammal {
public:
    Dog(std::string n) : Mammal(n) {}
    void make_sound() override {
        std::cout << name << " says Woof!" << std::endl;
    }
};
```

### Rust Trait Composition
```rust
// Define behaviors as traits
trait Animal {
    fn name(&self) -> &str;
    fn make_sound(&self);
}

trait Mammal {
    fn is_warm_blooded(&self) -> bool { true }
    fn give_birth(&self) {
        println!("Giving birth to live young");
    }
}

// Data structure - just data
struct Dog {
    name: String,
}

// Implement behaviors separately
impl Animal for Dog {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn make_sound(&self) {
        println!("{} says Woof!", self.name);
    }
}

impl Mammal for Dog {}  // Use default implementations

// Can implement multiple traits easily
struct Cat {
    name: String,
}

impl Animal for Cat {
    fn name(&self) -> &str { &self.name }
    fn make_sound(&self) { println!("{} says Meow!", self.name); }
}

impl Mammal for Cat {}
```

---

## Interface vs Trait Differences

### C# Interface Pattern
```csharp
public interface IDrawable {
    void Draw();
    double Area { get; }
}

public interface IColorable {
    string Color { get; set; }
}

// Must implement all interface methods
public class Circle : IDrawable, IColorable {
    public double Radius { get; set; }
    public string Color { get; set; }
    
    public void Draw() {
        Console.WriteLine($"Drawing {Color} circle");
    }
    
    public double Area => Math.PI * Radius * Radius;
}
```

### Rust Trait Pattern
```rust
trait Drawable {
    fn draw(&self);
    fn area(&self) -> f64;
    
    // Default implementation
    fn description(&self) -> String {
        format!("A shape with area {:.2}", self.area())
    }
}

trait Colorable {
    fn color(&self) -> &str;
    fn set_color(&mut self, color: String);
    
    // Default behavior
    fn is_colored(&self) -> bool {
        !self.color().is_empty()
    }
}

struct Circle {
    radius: f64,
    color: String,
}

impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing {} circle", self.color);
    }
    
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
    
    // Can override default
    fn description(&self) -> String {
        format!("A {} circle with radius {}", self.color, self.radius)
    }
}

impl Colorable for Circle {
    fn color(&self) -> &str { &self.color }
    fn set_color(&mut self, color: String) { self.color = color; }
}
```

---

## Polymorphism Patterns

### C++ Virtual Functions
```cpp
class Shape {
public:
    virtual double area() const = 0;
    virtual void draw() const = 0;
    virtual ~Shape() = default;
};

class Rectangle : public Shape {
    double width, height;
public:
    Rectangle(double w, double h) : width(w), height(h) {}
    double area() const override { return width * height; }
    void draw() const override { std::cout << "Rectangle"; }
};

void process_shapes(const std::vector<std::unique_ptr<Shape>>& shapes) {
    for (const auto& shape : shapes) {
        shape->draw();  // Virtual dispatch
        std::cout << " Area: " << shape->area() << std::endl;
    }
}
```

### Rust Trait Objects
```rust
trait Shape {
    fn area(&self) -> f64;
    fn draw(&self);
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
    
    fn draw(&self) {
        print!("Rectangle");
    }
}

struct Circle {
    radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
    
    fn draw(&self) {
        print!("Circle");
    }
}

fn process_shapes(shapes: &[Box<dyn Shape>]) {
    for shape in shapes {
        shape.draw();  // Dynamic dispatch
        println!(" Area: {:.2}", shape.area());
    }
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Rectangle { width: 10.0, height: 5.0 }),
        Box::new(Circle { radius: 3.0 }),
    ];
    
    process_shapes(&shapes);
}
```

---

## Design Pattern Translations

### Strategy Pattern

#### C# Version
```csharp
public interface IPaymentStrategy {
    void Pay(decimal amount);
}

public class CreditCardPayment : IPaymentStrategy {
    public void Pay(decimal amount) {
        Console.WriteLine($"Paid {amount:C} with credit card");
    }
}

public class PayPalPayment : IPaymentStrategy {
    public void Pay(decimal amount) {
        Console.WriteLine($"Paid {amount:C} with PayPal");
    }
}

public class ShoppingCart {
    private IPaymentStrategy paymentStrategy;
    
    public void SetPaymentStrategy(IPaymentStrategy strategy) {
        paymentStrategy = strategy;
    }
    
    public void Checkout(decimal amount) {
        paymentStrategy.Pay(amount);
    }
}
```

#### Rust Version
```rust
trait PaymentStrategy {
    fn pay(&self, amount: f64);
}

struct CreditCard {
    card_number: String,
}

impl PaymentStrategy for CreditCard {
    fn pay(&self, amount: f64) {
        println!("Paid ${:.2} with credit card ending in {}", 
                amount, &self.card_number[self.card_number.len()-4..]);
    }
}

struct PayPal {
    email: String,
}

impl PaymentStrategy for PayPal {
    fn pay(&self, amount: f64) {
        println!("Paid ${:.2} with PayPal account {}", amount, self.email);
    }
}

struct ShoppingCart {
    payment_strategy: Option<Box<dyn PaymentStrategy>>,
}

impl ShoppingCart {
    fn new() -> Self {
        ShoppingCart { payment_strategy: None }
    }
    
    fn set_payment_strategy(&mut self, strategy: Box<dyn PaymentStrategy>) {
        self.payment_strategy = Some(strategy);
    }
    
    fn checkout(&self, amount: f64) -> Result<(), &'static str> {
        match &self.payment_strategy {
            Some(strategy) => {
                strategy.pay(amount);
                Ok(())
            }
            None => Err("No payment strategy set"),
        }
    }
}
```

---

## Advanced Trait Patterns

### Trait Bounds for Generic Functions
```rust
// More flexible than inheritance
fn process_drawable_and_colorable<T>(item: &T) 
where
    T: Drawable + Colorable,
{
    println!("Processing a {} item:", item.color());
    item.draw();
    println!("Area: {:.2}", item.area());
}

// Multiple trait bounds
fn compare_shapes<T, U>(shape1: &T, shape2: &U) -> String 
where
    T: Drawable + std::fmt::Display,
    U: Drawable + std::fmt::Display,
{
    format!(
        "Shape 1: {} (area: {:.2}), Shape 2: {} (area: {:.2})",
        shape1, shape1.area(),
        shape2, shape2.area()
    )
}
```

### Associated Types vs Generics
```rust
// Generic trait - can implement multiple times
trait Add<RHS = Self> {
    type Output;
    fn add(self, rhs: RHS) -> Self::Output;
}

// Associated type trait - one implementation per type
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

struct Point { x: f64, y: f64 }

// Can implement Add for different RHS types
impl Add<Point> for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point { x: self.x + other.x, y: self.y + other.y }
    }
}

impl Add<f64> for Point {
    type Output = Point;
    fn add(self, scalar: f64) -> Point {
        Point { x: self.x + scalar, y: self.y + scalar }
    }
}
```

---

## When to Use Each Approach

### Use Traits When:
- You need flexible composition of behaviors
- Multiple types should share common functionality
- You want zero-cost abstractions
- You need to add functionality to existing types

### Traditional OOP Works Better For:
- Clear is-a relationships (though Rust encourages has-a)
- Complex state inheritance (though composition is often better)
- When you need mutable shared state (though Rust makes this harder)

---

## Key Takeaways

1. **Traits are more flexible** than inheritance hierarchies
2. **Composition over inheritance** leads to more maintainable code
3. **Zero-cost abstractions** - traits compile to efficient code
4. **Orphan rules** prevent conflicts but require wrapper types sometimes
5. **Associated types** provide cleaner APIs than generic parameters
6. **Trait objects** enable dynamic dispatch when needed
7. **Multiple traits** can be composed easily

### Mental Model Shift
- **OOP**: "What is this thing?" (inheritance hierarchy)
- **Rust**: "What can this thing do?" (trait implementation)

This leads to more flexible, composable, and maintainable designs.

---

Next: [Chapter 32: Idiomatic Rust Patterns for Production Systems](./32_idiomatic_patterns.md)
