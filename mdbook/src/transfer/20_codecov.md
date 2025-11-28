# Chapter 20: Code Coverage with cargo llvm-cov

Code coverage is a critical metric for understanding test effectiveness and identifying untested code paths in your Rust projects. The `cargo llvm-cov` tool provides source-based code coverage using LLVM's instrumentation capabilities, offering precise and efficient coverage analysis that integrates seamlessly with Rust's toolchain.

## Chapter Overview

This chapter covers comprehensive code coverage implementation:

- **Installation and setup** of cargo llvm-cov
- **Basic usage** and common commands
- **Report formats** (HTML, JSON, LCOV, Cobertura)
- **Filtering and exclusions** for focused coverage
- **Workspace and multi-crate** coverage strategies
- **CI/CD integration** with GitHub Actions and GitLab
- **Advanced configuration** and customization
- **Best practices** for meaningful coverage metrics

---

## 1. Installation and Setup

### Installing cargo llvm-cov

```bash
# Install from crates.io
cargo install cargo-llvm-cov

# Ensure you have the required LLVM tools
rustup component add llvm-tools-preview
```

### Verify Installation

```bash
# Check version
cargo llvm-cov --version

# View available options
cargo llvm-cov --help
```

### System Requirements

- Rust 1.60.0 or newer (Rust 1.82+ uses LLVM 19)
- LLVM tools preview component
- Supported platforms: Linux, macOS, Windows
- Note: Different Rust versions use different LLVM versions:
  - Rust 1.60-1.77: LLVM 14-17
  - Rust 1.78-1.81: LLVM 18
  - Rust 1.82+: LLVM 19+

---

## 2. Basic Usage

### Generate Coverage for All Tests

```bash
# Run tests and generate coverage
cargo llvm-cov

# Clean previous coverage data and run
cargo llvm-cov clean
cargo llvm-cov
```

### View Coverage in Browser

```bash
# Generate HTML report and open in browser
cargo llvm-cov --open

# Generate HTML report without opening
cargo llvm-cov --html
```

### Example Output

```
Finished test [unoptimized + debuginfo] target(s) in 2.14s

Filename                      Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover
----------------------------------------------------------------------------------------------------------------------------------------------------------------
src/calculator.rs                  12                 2    83.33%           4                 0   100.00%          45                 3    93.33%           8                 2    75.00%
src/parser.rs                      25                 5    80.00%           8                 1    87.50%         120                15    87.50%          20                 4    80.00%
src/lib.rs                          8                 0   100.00%           3                 0   100.00%          30                 0   100.00%           4                 0   100.00%
----------------------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL                              45                 7    84.44%          15                 1    93.33%         195                18    90.77%          32                 6    81.25%
```

---

## 3. Report Formats

### HTML Reports

```bash
# Generate HTML report
cargo llvm-cov --html

# HTML output location: target/llvm-cov/html/index.html
```

### JSON Format

```bash
# Generate JSON report for programmatic processing
cargo llvm-cov --json --output-path coverage.json

# Pretty-printed JSON
cargo llvm-cov --json --output-path coverage.json --json-pretty
```

### LCOV Format

```bash
# Generate LCOV format (for integration with coverage services)
cargo llvm-cov --lcov --output-path lcov.info
```

### Cobertura XML

```bash
# Generate Cobertura format (for CI/CD tools)
cargo llvm-cov --cobertura --output-path cobertura.xml
```

### Text Summary

```bash
# Display only summary without detailed output
cargo llvm-cov --summary-only
```

---

## 4. Practical Example: Calculator Library

Let's create a calculator library to demonstrate coverage analysis:

### Project Setup

```bash
cargo new calculator --lib
cd calculator
```

### Implementation (src/lib.rs)

```rust
#[derive(Debug, PartialEq)]
pub enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

pub struct Calculator {
    precision: usize,
}

impl Calculator {
    pub fn new() -> Self {
        Self { precision: 2 }
    }

    pub fn with_precision(precision: usize) -> Self {
        Self { precision }
    }

    pub fn calculate(&self, op: Operation, a: f64, b: f64) -> Result<f64, String> {
        let result = match op {
            Operation::Add => a + b,
            Operation::Subtract => a - b,
            Operation::Multiply => a * b,
            Operation::Divide => {
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                a / b
            }
        };

        Ok(self.round_to_precision(result))
    }

    fn round_to_precision(&self, value: f64) -> f64 {
        let multiplier = 10_f64.powi(self.precision as i32);
        (value * multiplier).round() / multiplier
    }

    pub fn chain_operations(&self, initial: f64, operations: Vec<(Operation, f64)>) -> Result<f64, String> {
        operations.iter().try_fold(initial, |acc, (op, value)| {
            self.calculate(op.clone(), acc, *value)
        })
    }
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Operation {
    fn clone(&self) -> Self {
        match self {
            Self::Add => Self::Add,
            Self::Subtract => Self::Subtract,
            Self::Multiply => Self::Multiply,
            Self::Divide => Self::Divide,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        let calc = Calculator::new();
        assert_eq!(calc.calculate(Operation::Add, 5.0, 3.0), Ok(8.0));
    }

    #[test]
    fn test_subtraction() {
        let calc = Calculator::new();
        assert_eq!(calc.calculate(Operation::Subtract, 10.0, 4.0), Ok(6.0));
    }

    #[test]
    fn test_multiplication() {
        let calc = Calculator::new();
        assert_eq!(calc.calculate(Operation::Multiply, 3.0, 7.0), Ok(21.0));
    }

    #[test]
    fn test_division() {
        let calc = Calculator::new();
        assert_eq!(calc.calculate(Operation::Divide, 20.0, 4.0), Ok(5.0));
    }

    #[test]
    fn test_division_by_zero() {
        let calc = Calculator::new();
        assert_eq!(
            calc.calculate(Operation::Divide, 10.0, 0.0),
            Err("Division by zero".to_string())
        );
    }

    // Note: with_precision, chain_operations, and round_to_precision are not tested
}
```

### Analyze Coverage

```bash
# Generate and view coverage
cargo llvm-cov --open
```

The report will show:
- `with_precision()` is not covered
- `chain_operations()` is not covered
- `Default` implementation is not tested

### Improve Coverage

```rust
#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_custom_precision() {
        let calc = Calculator::with_precision(4);
        assert_eq!(calc.calculate(Operation::Divide, 10.0, 3.0), Ok(3.3333));
    }

    #[test]
    fn test_chain_operations() {
        let calc = Calculator::new();
        let operations = vec![
            (Operation::Add, 5.0),      // 10 + 5 = 15
            (Operation::Multiply, 2.0), // 15 * 2 = 30
            (Operation::Subtract, 10.0), // 30 - 10 = 20
        ];
        assert_eq!(calc.chain_operations(10.0, operations), Ok(20.0));
    }

    #[test]
    fn test_default_impl() {
        let calc = Calculator::default();
        assert_eq!(calc.precision, 2);
    }
}
```

---

## 5. Filtering and Exclusions

### Exclude Test Code

```bash
# Measure only library code
cargo llvm-cov --lib

# Measure library and binaries
cargo llvm-cov --lib --bins
```

### File Pattern Exclusion

```bash
# Exclude files matching regex patterns
cargo llvm-cov --ignore-filename-regex '(tests?/|benches/|examples/)'

# Exclude test modules
cargo llvm-cov --ignore-filename-regex '_test\.rs$'
```

### Include Specific Files

```bash
# Include only specific patterns
cargo llvm-cov --include-ffi --ignore-filename-regex '^(?!src/)'
```

### Workspace Coverage

```bash
# Coverage for entire workspace
cargo llvm-cov --workspace

# Specific packages only
cargo llvm-cov --package core-lib --package api-server

# Exclude specific packages
cargo llvm-cov --workspace --exclude integration-tests
```

---

## 6. Configuration File

### Create .cargo/config.toml

```toml
[alias]
# Local development coverage with browser
cov = "llvm-cov --open"

# Quick coverage check
cov-check = "llvm-cov --summary-only"

# CI coverage with LCOV output
cov-ci = "llvm-cov --lcov --output-path lcov.info"

# Detailed coverage with all formats
cov-full = """llvm-cov --html --json --output-path coverage.json
              --lcov --output-path lcov.info"""
```

### Project-specific Configuration

```toml
# Cargo.toml
[package.metadata.llvm_cov]
# Workspace members to include
workspace-members = ["core", "api", "cli"]

# Files to exclude
exclude-files = [
    "src/generated/*",
    "src/vendor/*",
]

# Use .gitignore patterns
ignore-gitignore = true
```

---

## 7. CI/CD Integration

### GitHub Actions

```yaml
name: Coverage

on:
  push:
    branches: [main]
  pull_request:

jobs:
  coverage:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v4

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: llvm-tools-preview

    - name: Install cargo-llvm-cov
      uses: taiki-e/install-action@cargo-llvm-cov

    - name: Generate coverage
      run: |
        cargo llvm-cov clean --workspace
        cargo llvm-cov --workspace --lcov --output-path lcov.info

    - name: Upload to Codecov
      uses: codecov/codecov-action@v5
      with:
        files: lcov.info
        fail_ci_if_error: true
        verbose: true
        token: ${{ secrets.CODECOV_TOKEN }}  # Required for v5

    - name: Archive coverage report
      uses: actions/upload-artifact@v4
      with:
        name: coverage-report
        path: target/llvm-cov/html/
```

### GitLab CI

```yaml
coverage:
  stage: test
  image: rust:latest

  before_script:
    - rustup component add llvm-tools-preview
    - cargo install cargo-llvm-cov

  script:
    - cargo llvm-cov clean --workspace
    - cargo llvm-cov --workspace --cobertura --output-path cobertura.xml
    - cargo llvm-cov --workspace --text --output-dir coverage

  coverage: '/TOTAL\s+\d+\s+\d+\s+([\d\.]+)%/'

  artifacts:
    reports:
      coverage_report:
        coverage_format: cobertura
        path: cobertura.xml
    paths:
      - coverage/
```

### Coverage Badge

```markdown
[![Coverage Status](https://codecov.io/gh/username/repo/branch/main/graph/badge.svg)](https://codecov.io/gh/username/repo)
```

---

## 8. Advanced Usage

### Continuous Monitoring

```bash
# Watch mode for development (requires cargo-watch)
cargo watch -x "llvm-cov --summary-only"
```

### MC/DC Coverage (Modified Condition/Decision Coverage)

```bash
# Enable MC/DC coverage (unstable feature as of 2024)
cargo llvm-cov --mcdc

# Note: Requires nightly Rust and may change
```

### Profile-Guided Coverage

```toml
# Cargo.toml
[profile.coverage]
inherits = "test"
opt-level = 0
overflow-checks = false
debug-assertions = true

[profile.coverage.package."*"]
opt-level = 0
```

```bash
# Use coverage profile
cargo llvm-cov --profile coverage
```

### Environment Variables

```bash
# Set output directory
CARGO_LLVM_COV_TARGET_DIR=./coverage cargo llvm-cov

# Disable colored output
NO_COLOR=1 cargo llvm-cov

# Increase verbosity
RUST_LOG=debug cargo llvm-cov
```

### Custom Test Runner

```bash
# Run specific tests
cargo llvm-cov --test integration_tests

# With test arguments
cargo llvm-cov -- --test-threads=1 --nocapture

# Run doctests
cargo llvm-cov --doctests
```

---

## 9. Troubleshooting Common Issues

### Issue: Low Coverage Due to Generic Code

```rust
// Generic functions may show as uncovered
pub fn process<T: Display>(item: T) -> String {
    format!("Processing: {}", item)
}

// Solution: Test with multiple types
#[test]
fn test_process_multiple_types() {
    assert_eq!(process(42), "Processing: 42");
    assert_eq!(process("hello"), "Processing: hello");
    assert_eq!(process(3.14), "Processing: 3.14");
}
```

### Issue: Macro-Generated Code

```rust
// Macros can generate uncovered code
macro_rules! impl_ops {
    ($($t:ty),*) => {
        $(
            impl MyTrait for $t {
                fn process(&self) -> String {
                    format!("{:?}", self)
                }
            }
        )*
    };
}

impl_ops!(i32, i64, f32, f64);

// Solution: Test each generated implementation
#[test]
fn test_macro_implementations() {
    assert!((42i32).process().contains("42"));
    assert!((42i64).process().contains("42"));
    assert!((3.14f32).process().contains("3.14"));
    assert!((3.14f64).process().contains("3.14"));
}
```

### Issue: Platform-Specific Code

```rust
#[cfg(unix)]
fn unix_specific() -> &'static str {
    "Unix implementation"
}

#[cfg(windows)]
fn windows_specific() -> &'static str {
    "Windows implementation"
}

// Solution: Use cfg_attr for tests
#[test]
#[cfg_attr(not(unix), ignore = "Unix only test")]
fn test_unix_specific() {
    #[cfg(unix)]
    assert_eq!(unix_specific(), "Unix implementation");
}

#[test]
#[cfg_attr(not(windows), ignore = "Windows only test")]
fn test_windows_specific() {
    #[cfg(windows)]
    assert_eq!(windows_specific(), "Windows implementation");
}
```

---

## 10. Best Practices

### Coverage Thresholds

```bash
#!/bin/bash
# coverage-check.sh

THRESHOLD=80
COVERAGE=$(cargo llvm-cov --summary-only --json | jq '.data[0].totals.lines.percent')

if (( $(echo "$COVERAGE < $THRESHOLD" | bc -l) )); then
    echo "❌ Coverage $COVERAGE% is below threshold $THRESHOLD%"
    exit 1
fi

echo "✅ Coverage $COVERAGE% meets threshold"
```

### Meaningful Coverage Metrics

**DO Focus On:**
- Business logic coverage (aim for 85-95%)
- Error handling paths (aim for 90%+)
- Public API surface (aim for 95%+)
- Critical safety code (aim for 100%)

**DON'T Focus On:**
- Simple getters/setters
- Debug/Display implementations
- Boilerplate code
- Generated code

### Integration with Development Workflow

```bash
# Pre-push hook (.git/hooks/pre-push)
#!/bin/sh

echo "Running coverage check..."
cargo llvm-cov --summary-only

read -p "Push with current coverage? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
fi
```

### Documentation

```rust
/// Calculate the factorial of a number
///
/// # Examples
///
/// ```
/// use calculator::factorial;
///
/// assert_eq!(factorial(5), 120);
/// assert_eq!(factorial(0), 1);
/// ```
pub fn factorial(n: u32) -> u32 {
    match n {
        0 => 1,
        _ => n * factorial(n - 1),
    }
}

// Doctests are included in coverage!
```

---

## Summary

`cargo llvm-cov` provides comprehensive, source-based code coverage for Rust projects with minimal setup and excellent integration capabilities.

**Key Takeaways:**

- **Easy Installation**: Single cargo install command with LLVM tools
- **Multiple Formats**: HTML, JSON, LCOV, Cobertura for different needs
- **Flexible Filtering**: Exclude tests, dependencies, and irrelevant code
- **CI/CD Ready**: Integrates with GitHub Actions, GitLab CI, and other platforms
- **Accurate Metrics**: Source-based coverage provides precise line and branch coverage
- **Development Friendly**: Fast execution and helpful visualization
- **Active Development**: Regular updates with new features like MC/DC coverage

**Quick Start Commands:**

```bash
# Install
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview

# Basic usage
cargo llvm-cov --open           # View in browser
cargo llvm-cov --summary-only   # Quick check
cargo llvm-cov --lcov --output-path lcov.info  # CI/CD

# Workspace
cargo llvm-cov --workspace --lib --bins

# With exclusions
cargo llvm-cov --ignore-filename-regex '(tests|benches|examples)/'
```

Code coverage is not about achieving 100% coverage, but about ensuring critical paths are tested and maintaining confidence in code changes. Use `cargo llvm-cov` as a tool to identify gaps in testing and guide test development efforts.