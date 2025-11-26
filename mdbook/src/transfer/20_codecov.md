# Chapter 20: Code Coverage with cargo llvm-cov

Code coverage is essential for understanding test quality and identifying untested code paths. The `cargo llvm-cov` tool provides precise, LLVM-based coverage analysis for Rust projects. However, achieving meaningful coverage metrics requires careful configuration to exclude third-party code, handle test coverage properly, and manage mock code appropriately.

## Chapter Overview

This chapter addresses practical code coverage challenges:

- **Excluding third-party dependencies** from coverage metrics
- **Handling test code coverage** and assertion failures
- **Managing mock code** in coverage reports
- **Configuring precise project-only coverage**
- **Best practices** for enterprise coverage policies
- **Integration** with CI/CD and reporting tools

---

## 1. The Third-Party Code Problem

### Issue: Serde and Dependencies Affecting Coverage

**The Problem:** When using `cargo llvm-cov --open`, you notice function coverage appears lower because third-party crate code (like serde, tokio, etc.) is included in coverage calculations.

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub age: u32,
}

impl User {
    pub fn new(name: String, age: u32) -> Self {
        Self { name, age }  // Your code: covered
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()  // Calls serde internals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new("Alice".to_string(), 25);
        let json = user.to_json();
        assert!(json.contains("Alice"));
    }
}
```

**What happens:** Coverage report includes serde's serialization code, making your project coverage appear lower than actual.

### Solution: Exclude Dependencies

**Method 1: Source-based exclusion (Recommended)**
```bash
# Only include your project source
cargo llvm-cov --open --ignore-filename-regex '\.cargo|target'

# More precise: only include src/ directory
cargo llvm-cov --open --include-ffi --ignore-filename-regex '^(?!src/)'
```

**Method 2: Using lcov.excludes configuration**
```toml
# Cargo.toml
[package.metadata.coverage.excludes]
paths = [
    "target/*",
    ".cargo/*",
    "*/src/lib.rs",  # If you want to exclude serde's lib.rs
]
```

**Method 3: Workspace-aware coverage**
```bash
# For workspace projects - only measure workspace members
cargo llvm-cov --workspace --open --ignore-filename-regex '\.cargo'

# Or specific packages only
cargo llvm-cov --package my-app --package my-lib --open
```

### Project Structure Example

```
my-project/
├── src/
│   ├── lib.rs           ✅ Include in coverage
│   ├── main.rs          ✅ Include in coverage
│   └── models/
│       └── user.rs      ✅ Include in coverage
├── tests/
│   └── integration.rs   ⚠️  Configurable
├── target/              ❌ Exclude from coverage
└── .cargo/              ❌ Exclude from coverage
```

**Recommended command:**
```bash
cargo llvm-cov --open --ignore-filename-regex '^(?!src/)'
```

---

## 2. Test Code Coverage Dilemma

### Issue: Test Code Appears in Coverage Reports

**The Problem:** Test code itself appears in coverage reports, especially assertion failures that never execute in successful tests.

```rust
pub fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        return Err("Division by zero".to_string());
    }
    Ok(a / b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divide_success() {
        assert_eq!(divide(10.0, 2.0), Ok(5.0));  // ✅ This line is "covered"
    }

    #[test]
    fn test_divide_by_zero() {
        assert_eq!(divide(10.0, 0.0), Err("Division by zero".to_string()));
        // The assertion success path is covered, but what about assert failure?
    }

    #[test]
    #[should_panic]
    fn test_divide_panic_case() {
        // This test might show missing coverage on panic paths
        assert_eq!(divide(10.0, 0.0), Ok(5.0));  // ❌ Assertion fails - uncovered path
    }
}
```

### Solution 1: Exclude Test Files

```bash
# Exclude test files and directories
cargo llvm-cov --open --ignore-filename-regex 'tests?/|test_'

# More comprehensive exclusion
cargo llvm-cov --open --ignore-filename-regex '(tests?/|test_|_test\.rs$|mock)'
```

### Solution 2: Library-Only Coverage

```bash
# Only measure library code, not tests or binaries
cargo llvm-cov --lib --open

# Measure specific targets
cargo llvm-cov --lib --bins --open --ignore-filename-regex 'tests?/'
```

### Solution 3: Configuration File

Create `.cargo/config.toml`:
```toml
[alias]
cov = "llvm-cov --lib --open --ignore-filename-regex '(tests?/|test_|_test\\.rs$|bench|mock)'"
cov-ci = "llvm-cov --lib --lcov --output-path lcov.info --ignore-filename-regex '(tests?/|test_|_test\\.rs$|bench|mock)'"
```

Usage:
```bash
cargo cov        # Local development
cargo cov-ci     # CI/CD pipeline
```

---

## 3. Mock Code Coverage Management

### Issue: Mock Code Inflating Coverage

**The Problem:** Mock implementations appear in coverage reports and can create misleading metrics.

```rust
// src/database.rs
pub trait DatabaseTrait {
    fn get_user(&self, id: u64) -> Result<User, DatabaseError>;
    fn save_user(&self, user: &User) -> Result<(), DatabaseError>;
}

pub struct PostgresDatabase {
    // Real implementation
}

impl DatabaseTrait for PostgresDatabase {
    fn get_user(&self, id: u64) -> Result<User, DatabaseError> {
        // Real database logic - ✅ Should be covered
        todo!()
    }

    fn save_user(&self, user: &User) -> Result<(), DatabaseError> {
        // Real database logic - ✅ Should be covered
        todo!()
    }
}

// src/mocks.rs or tests/mocks.rs
pub struct MockDatabase {
    pub users: std::collections::HashMap<u64, User>,
}

impl DatabaseTrait for MockDatabase {
    fn get_user(&self, id: u64) -> Result<User, DatabaseError> {
        // Mock logic - ❓ Should this count toward coverage?
        self.users.get(&id).cloned()
            .ok_or(DatabaseError::NotFound)
    }

    fn save_user(&self, user: &User) -> Result<(), DatabaseError> {
        // Mock logic - ❓ Should this count toward coverage?
        Ok(())
    }
}
```

### Best Practices for Mock Code

**Option 1: Separate Mock Module (Recommended)**
```
src/
├── lib.rs
├── database.rs          ✅ Include in coverage
├── models.rs            ✅ Include in coverage
└── mocks/               ❌ Exclude from coverage
    ├── mod.rs
    └── database.rs

tests/
├── integration.rs       ❌ Exclude from coverage
└── mocks/               ❌ Exclude from coverage
    └── external_api.rs
```

**Configuration:**
```bash
cargo llvm-cov --open --ignore-filename-regex '(tests?/|test_|_test\.rs$|mock|benches?/)'
```

**Option 2: Conditional Compilation**
```rust
#[cfg(test)]
pub mod mocks {
    use super::*;

    pub struct MockDatabase {
        // Mock implementation - excluded via #[cfg(test)]
    }
}
```

**Option 3: Feature-Based Exclusion**
```toml
# Cargo.toml
[features]
default = []
testing = []  # Enable mocks only for testing

[dependencies]
# Mock dependencies only when testing feature is enabled
mockall = { version = "0.11", optional = true }
```

```rust
#[cfg(feature = "testing")]
pub mod mocks {
    // Mock implementations
}
```

Run coverage without testing feature:
```bash
cargo llvm-cov --no-default-features --open
```

---

## 4. Comprehensive Coverage Configuration

### Complete Coverage Script

Create `scripts/coverage.sh`:
```bash
#!/bin/bash
set -e

# Coverage configuration
IGNORE_PATTERNS="(tests?/|test_|_test\.rs$|bench|mock|examples?/)"
SRC_ONLY="^src/"

case "${1:-local}" in
    "local")
        echo "🔍 Running local coverage analysis..."
        cargo llvm-cov clean
        cargo llvm-cov --lib --open \
            --ignore-filename-regex "$IGNORE_PATTERNS" \
            --include-ffi
        ;;

    "ci")
        echo "🤖 Running CI coverage analysis..."
        cargo llvm-cov clean
        cargo llvm-cov --lib \
            --lcov --output-path lcov.info \
            --ignore-filename-regex "$IGNORE_PATTERNS"

        # Extract coverage percentage
        coverage=$(cargo llvm-cov --lib --summary-only \
            --ignore-filename-regex "$IGNORE_PATTERNS" \
            | grep "TOTAL" | awk '{print $4}' | tr -d '%')

        echo "📊 Coverage: ${coverage}%"

        # Check threshold
        if (( $(echo "$coverage < 80" | bc -l) )); then
            echo "❌ Coverage ${coverage}% below threshold"
            exit 1
        fi
        ;;

    "detailed")
        echo "📋 Running detailed coverage analysis..."
        cargo llvm-cov clean

        # Generate multiple reports
        cargo llvm-cov --lib --html \
            --ignore-filename-regex "$IGNORE_PATTERNS"

        cargo llvm-cov --lib --json \
            --ignore-filename-regex "$IGNORE_PATTERNS" \
            --output-path coverage.json

        echo "📂 Reports generated:"
        echo "  - HTML: target/llvm-cov/html/index.html"
        echo "  - JSON: coverage.json"
        ;;

    *)
        echo "Usage: $0 [local|ci|detailed]"
        exit 1
        ;;
esac
```

### Cargo.toml Configuration

```toml
# Cargo.toml
[package.metadata.coverage]
exclude-files = [
    "tests/*",
    "benches/*",
    "examples/*",
    "src/mocks/*",
    "*/mock*",
]

[profile.coverage]
inherits = "test"
debug = true
```

### Workspace Configuration

For multi-crate workspaces:
```toml
# Workspace Cargo.toml
[workspace.metadata.coverage]
exclude-packages = [
    "integration-tests",
    "benchmarks",
    "examples",
]

# Only measure core packages
include-packages = [
    "my-core",
    "my-api",
    "my-models",
]
```

```bash
# Workspace coverage
cargo llvm-cov --workspace \
    --ignore-filename-regex "(tests?/|test_|_test\.rs$|bench|mock)" \
    --exclude integration-tests \
    --exclude benchmarks \
    --open
```

---

## 5. Advanced Coverage Patterns

### Function-Level Coverage Analysis

```rust
pub struct PaymentProcessor {
    api_key: String,
}

impl PaymentProcessor {
    pub fn new(api_key: String) -> Self {
        Self { api_key }  // ✅ Constructor coverage
    }

    pub fn process_payment(&self, amount: f64) -> Result<PaymentResult, PaymentError> {
        if amount <= 0.0 {  // ✅ Validation coverage
            return Err(PaymentError::InvalidAmount);
        }

        if amount > 10000.0 {  // ⚠️ High amount path - might not be tested
            self.process_high_value_payment(amount)
        } else {
            self.process_standard_payment(amount)  // ✅ Standard path coverage
        }
    }

    fn process_high_value_payment(&self, amount: f64) -> Result<PaymentResult, PaymentError> {
        // ❌ Might be uncovered - requires specific test
        unimplemented!("High value payments require manual approval")
    }

    fn process_standard_payment(&self, amount: f64) -> Result<PaymentResult, PaymentError> {
        // ✅ Covered by standard tests
        Ok(PaymentResult::Success { amount })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_payment() {
        let processor = PaymentProcessor::new("test-key".to_string());
        let result = processor.process_payment(100.0);
        assert!(matches!(result, Ok(PaymentResult::Success { amount: 100.0 })));
    }

    #[test]
    fn test_invalid_amount() {
        let processor = PaymentProcessor::new("test-key".to_string());
        let result = processor.process_payment(-10.0);
        assert!(matches!(result, Err(PaymentError::InvalidAmount)));
    }

    // Missing: test_high_value_payment - this causes low coverage
    #[test]
    #[ignore = "requires manual testing"]
    fn test_high_value_payment() {
        let processor = PaymentProcessor::new("test-key".to_string());
        let result = processor.process_payment(15000.0);
        // This test is ignored, so high value path is uncovered
    }
}
```

**Coverage analysis:**
```bash
# See function-level coverage
cargo llvm-cov --open --ignore-filename-regex "(tests?/|test_|_test\.rs$)"

# Include ignored tests to improve coverage
cargo llvm-cov --open --ignored --ignore-filename-regex "(tests?/|test_|_test\.rs$)"
```

### Platform-Specific Coverage

```rust
pub fn get_temp_dir() -> PathBuf {
    #[cfg(windows)]
    {
        std::env::var("TEMP")  // Windows path - only covered on Windows
            .unwrap_or_else(|_| "C:\\tmp".to_string())
            .into()
    }

    #[cfg(unix)]
    {
        std::env::var("TMPDIR")  // Unix path - only covered on Unix
            .unwrap_or_else(|_| "/tmp".to_string())
            .into()
    }

    #[cfg(not(any(windows, unix)))]
    {
        PathBuf::from("tmp")  // Fallback - might never be covered
    }
}
```

Coverage will only show the platform-specific branch that's tested.

---

## 6. Enterprise Coverage Policies

### Coverage Gate Configuration

**GitHub Actions Example:**
```yaml
# .github/workflows/coverage.yml
name: Coverage Gate

on:
  pull_request:
    branches: [main]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: llvm-tools-preview

    - name: Install cargo-llvm-cov
      run: cargo install cargo-llvm-cov

    - name: Generate coverage
      run: |
        cargo llvm-cov clean
        cargo llvm-cov --lib \
          --lcov --output-path lcov.info \
          --ignore-filename-regex "(tests?/|test_|_test\.rs$|bench|mock)"

    - name: Check coverage threshold
      run: |
        coverage=$(cargo llvm-cov --lib --summary-only \
          --ignore-filename-regex "(tests?/|test_|_test\.rs$|bench|mock)" \
          | grep "TOTAL" | awk '{print $4}' | tr -d '%')

        echo "Current coverage: ${coverage}%"

        if (( $(echo "$coverage < 80" | bc -l) )); then
          echo "❌ Coverage ${coverage}% is below minimum 80%"
          echo "Run 'cargo llvm-cov --lib --open' to see detailed report"
          exit 1
        fi

        echo "✅ Coverage requirement met: ${coverage}%"

    - name: Upload to Codecov
      uses: codecov/codecov-action@v3
      with:
        files: lcov.info
        fail_ci_if_error: false
```

### Pre-commit Hook

```bash
#!/bin/sh
# .git/hooks/pre-commit
set -e

echo "🔍 Checking code coverage..."

# Generate coverage report
cargo llvm-cov clean >/dev/null 2>&1
coverage=$(cargo llvm-cov --lib --summary-only \
  --ignore-filename-regex "(tests?/|test_|_test\.rs$|bench|mock)" \
  | grep "TOTAL" | awk '{print $4}' | tr -d '%')

echo "Current coverage: ${coverage}%"

# Check minimum threshold
MIN_COVERAGE=75
if (( $(echo "$coverage < $MIN_COVERAGE" | bc -l) )); then
    echo "❌ Coverage ${coverage}% is below minimum ${MIN_COVERAGE}%"
    echo ""
    echo "To see detailed coverage report, run:"
    echo "  cargo llvm-cov --lib --open"
    echo ""
    echo "To bypass this check, commit with --no-verify"
    exit 1
fi

echo "✅ Coverage check passed"
```

---

## 7. Best Practices Summary

### What to Include in Coverage

✅ **Include:**
- Application source code (`src/lib.rs`, `src/main.rs`, `src/**/*.rs`)
- Business logic implementations
- Error handling paths
- Public API functions
- Critical utility functions

### What to Exclude from Coverage

❌ **Exclude:**
- Third-party dependencies (`.cargo/*`, `target/*`)
- Test code (`tests/*`, `*_test.rs`, `test_*.rs`)
- Mock implementations (`mocks/*`, `mock_*.rs`)
- Benchmark code (`benches/*`)
- Examples (`examples/*`)
- Generated code (protobuf, bindgen output)
- Platform-specific code not relevant to deployment

### Recommended Commands

```bash
# Local development
cargo llvm-cov --lib --open \
  --ignore-filename-regex "(tests?/|test_|_test\.rs$|bench|mock|examples?/)"

# CI/CD pipeline
cargo llvm-cov --lib --lcov --output-path lcov.info \
  --ignore-filename-regex "(tests?/|test_|_test\.rs$|bench|mock|examples?/)"

# Detailed analysis
cargo llvm-cov --lib --html \
  --ignore-filename-regex "(tests?/|test_|_test\.rs$|bench|mock|examples?/)"
```

### Coverage Targets by Code Type

| Code Type | Target Coverage | Rationale |
|-----------|----------------|-----------|
| Business Logic | 85-95% | Critical functionality |
| API Endpoints | 80-90% | User-facing interfaces |
| Utility Functions | 70-80% | Supporting code |
| Error Handling | 90-95% | Failure scenarios |
| Integration Code | 60-70% | External dependencies |

---

## Summary

Effective code coverage in Rust requires careful configuration of `cargo llvm-cov` to focus on project-specific code:

**Key Takeaways:**
- **Exclude third-party code** using `--ignore-filename-regex` patterns
- **Exclude test and mock code** to focus on production logic
- **Use `--lib` flag** to avoid coverage pollution from test binaries
- **Set appropriate thresholds** based on code criticality
- **Automate coverage checks** in CI/CD pipelines

The goal is meaningful coverage metrics that help identify untested business logic, not artificially high numbers that include irrelevant code paths.

**Recommended alias for daily use:**
```bash
# Add to .cargo/config.toml
[alias]
cov = "llvm-cov --lib --open --ignore-filename-regex '(tests?/|test_|_test\\.rs$|bench|mock|examples?/)'"
```

This provides clean, project-focused coverage analysis that developers can trust and act upon.