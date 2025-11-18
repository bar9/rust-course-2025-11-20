// Day 2 - Complete Solutions Demonstration

use day2_all_solutions::*;

fn main() {
    println!("=== DAY 2 RUST COURSE - ALL SOLUTIONS ===\n");
    
    demonstrate_collections();
    demonstrate_traits_chapter();
    demonstrate_generics();
    demonstrate_pattern_matching();
    demonstrate_error_handling();
    demonstrate_iterators();
    demonstrate_modules();
    
    println!("\n✅ All Day 2 exercises completed successfully!");
}

fn demonstrate_collections() {
    println!("\n--- Chapter 6: COLLECTIONS ---\n");
    collections::demonstrate_lru_cache();
}

fn demonstrate_traits_chapter() {
    println!("\n--- Chapter 7: TRAITS ---\n");
    
    // Just show a simple example from traits module
    use traits::Comparable;
    
    let p1 = traits::Person { name: "Alice".to_string(), age: 30 };
    let p2 = traits::Person { name: "Bob".to_string(), age: 25 };
    
    println!("Comparable trait demonstration:");
    if p1.is_greater(&p2) {
        println!("  {} (age {}) is older than {} (age {})", 
                 p1.name, p1.age, p2.name, p2.age);
    }
}

fn demonstrate_generics() {
    println!("\n--- Chapter 8: GENERICS ---\n");

    // Exercise 1: Queue
    let mut queue = generics::Queue::new();
    queue.enqueue("First");
    queue.enqueue("Second");
    queue.enqueue("Third");

    println!("Queue operations:");
    while let Some(item) = queue.dequeue() {
        println!("  Dequeued: {}", item);
    }

    // Exercise 2: Min function
    println!("\nMin function:");
    println!("  min(5, 3) = {}", generics::min(5, 3));
    println!("  min(2.7, 3.14) = {}", generics::min(2.7, std::f64::consts::PI));

    // Exercise 3: Builder with phantom types
    println!("\nRequest builder:");
    let request = generics::RequestBuilder::new("https://api.example.com".to_string())
        .add_header("Content-Type".to_string(), "application/json".to_string())
        .add_header("Authorization".to_string(), "Bearer token".to_string())
        .send();

    println!("  {}", request.execute());
}

fn demonstrate_pattern_matching() {
    println!("\n--- Chapter 9: PATTERN MATCHING ---\n");

    // Exercise 1: HTTP Status Handler
    println!("HTTP Status Handler:");

    let responses = vec![
        pattern_matching::HttpResponse {
            status: pattern_matching::HttpStatus::Ok,
            body: Some("Hello World".to_string()),
            headers: vec![],
        },
        pattern_matching::HttpResponse {
            status: pattern_matching::HttpStatus::Ok,
            body: None,
            headers: vec![],
        },
        pattern_matching::HttpResponse {
            status: pattern_matching::HttpStatus::NotFound,
            body: None,
            headers: vec![],
        },
        pattern_matching::HttpResponse {
            status: pattern_matching::HttpStatus::Custom(202),
            body: None,
            headers: vec![],
        },
        pattern_matching::HttpResponse {
            status: pattern_matching::HttpStatus::Custom(403),
            body: None,
            headers: vec![],
        },
    ];

    for (i, response) in responses.into_iter().enumerate() {
        println!("  Response {}: {}", i + 1, pattern_matching::handle_response(response));
    }

    // Exercise 2: Configuration Parser
    println!("\nConfiguration Parser:");
    let config_lines = vec![
        "name=John",
        "port:int=8080",
        "debug:bool=true",
        "tags:array=rust,programming,tutorial",
        "timeout=30",
    ];

    for line in config_lines {
        match pattern_matching::parse_config_line(line) {
            Ok((key, value)) => println!("  {} -> {:?}", key, value),
            Err(e) => println!("  Error parsing '{}': {:?}", line, e),
        }
    }

    // Exercise 3: State Machine
    println!("\nState Machine:");
    let mut state = pattern_matching::State::Idle;
    println!("  Initial state: {:?}", state);

    let events = vec![
        pattern_matching::Event::Start,
        pattern_matching::Event::Progress(25),
        pattern_matching::Event::Progress(50),
        pattern_matching::Event::Progress(75),
        pattern_matching::Event::Finish,
        pattern_matching::Event::Reset,
    ];

    for event in events {
        state = pattern_matching::transition_state(state, event);
        println!("  After event: {:?}", state);
    }
}

fn demonstrate_error_handling() {
    println!("\n--- Chapter 10: ERROR HANDLING ---\n");
    
    // Exercise 3: Email builder with error handling
    println!("Email builder with validation:");
    
    let email_result = error_handling::EmailBuilder::new()
        .to("user@example.com")
        .and_then(|b| b.from("sender@example.com"))
        .and_then(|b| b.subject("Hello Rust"))
        .and_then(|b| b.body("This is a test email"))
        .and_then(|b| b.build());
    
    match email_result {
        Ok(email) => {
            println!("  ✅ Email created successfully");
            let _ = email.send();
        },
        Err(e) => println!("  ❌ Error creating email: {}", e),
    }
    
    // Demonstrate error on invalid email
    let invalid_result = error_handling::EmailBuilder::new()
        .to("invalid-email");
    
    if let Err(e) = invalid_result {
        println!("  ❌ Expected error for invalid email: {}", e);
    }
}

fn demonstrate_iterators() {
    println!("\n--- Chapter 11: ITERATORS ---\n");
    
    let log_lines = vec![
        "1000|INFO|Server started".to_string(),
        "1001|DEBUG|Connection received".to_string(),
        "1002|ERROR|Failed to connect to database".to_string(),
        "invalid line".to_string(),
        "1003|WARNING|High memory usage".to_string(),
        "1004|INFO|Request processed".to_string(),
        "1005|ERROR|Timeout error".to_string(),
    ];
    
    let analyzer = iterators::LogAnalyzer::new(&log_lines);
    
    println!("Log analysis results:");
    println!("  Valid entries: {}", analyzer.parse_entries().count());
    
    let error_count = analyzer.errors_only().count();
    println!("  Error entries: {}", error_count);
    
    let counts = analyzer.count_by_level();
    println!("  Count by level: {:?}", counts);
    
    let recent = analyzer.most_recent(3);
    println!("  Most recent 3 entries:");
    for entry in recent {
        println!("    [{}] {:?}: {}", entry.timestamp, entry.level, entry.message);
    }
}

fn demonstrate_modules() {
    println!("\n--- Chapter 12: MODULES & VISIBILITY ---\n");
    
    // Exercise 1: Library System
    demonstrate_library();
    
    // Exercise 2: Configuration
    demonstrate_config();
    
    // Exercise 3: Plugin Architecture  
    demonstrate_plugins();
}

fn demonstrate_library() {
    use library_system::library::*;
    
    println!("Library Management System:");
    
    let mut library = Library::new();
    
    // Add books
    library.add_book(Book::new(
        "The Rust Programming Language".to_string(),
        "Steve Klabnik".to_string(),
        "978-1718500440".to_string(),
    ));
    
    library.add_book(Book::new(
        "Programming Rust".to_string(),
        "Jim Blandy".to_string(),
        "978-1492052593".to_string(),
    ));
    
    // Add members
    library.add_member(Member::new(
        1,
        "Alice".to_string(),
        "alice@example.com".to_string(),
    ));
    
    library.add_member(Member::new(
        2,
        "Bob".to_string(),
        "bob@example.com".to_string(),
    ));
    
    println!("  Books: {}, Members: {}", library.books.len(), library.members.len());
    
    // Checkout a book
    match library.checkout_book_to_member("978-1718500440", 1, "2024-01-15".to_string()) {
        Ok(()) => println!("  ✅ Book checked out to Alice"),
        Err(e) => println!("  ❌ Checkout failed: {}", e),
    }
}

fn demonstrate_config() {
    use config::{Config, Environment};
    
    println!("\nConfiguration System:");
    
    let dev_config = Config::for_environment(Environment::Development);
    let prod_config = Config::for_environment(Environment::Production);
    
    println!("  Development - Port: {}, Debug: {}", dev_config.port, dev_config.debug_mode);
    println!("  Production - Port: {}, Debug: {}", prod_config.port, prod_config.debug_mode);
    
    // Validate configuration
    match dev_config.validate() {
        Ok(()) => println!("  ✅ Development config is valid"),
        Err(errors) => println!("  ❌ Config errors: {:?}", errors),
    }
}

fn demonstrate_plugins() {
    use plugin_architecture::plugins::*;
    use plugin_architecture::registry::PluginRegistry;
    
    println!("\nPlugin System:");
    
    let mut registry = PluginRegistry::new();
    
    // Register plugins
    let logger = logger::LoggerPlugin::new(
        "MainLogger".to_string(),
        logger::LogLevel::Info,
    );
    
    let mut metrics = metrics::MetricsPlugin::new("Metrics".to_string());
    metrics.collect();
    metrics.collect();
    
    let auth = auth::AuthPlugin::new(
        "AuthService".to_string(),
        auth::AuthType::JWT,
    );
    
    let _ = registry.register(Box::new(logger));
    let _ = registry.register(Box::new(metrics));
    let _ = registry.register(Box::new(auth));
    
    println!("\nRegistered plugins: {:?}", registry.list_plugins());
    
    // Execute all plugins
    registry.execute_all();
}