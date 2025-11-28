# Chapter 19: Cargo & Dependency Management

Modern software development relies heavily on dependencies, and managing them correctly is crucial for reproducible builds, security, and maintainability. Coming from C++, .NET, or other ecosystems, Rust's Cargo provides a refreshingly coherent approach to dependency management.

## Chapter Overview

This chapter covers the critical aspects of dependency management in Rust:

- **Rust editions** and their impact on projects
- **Toolchain channels** (stable, beta, nightly) and when to use each
- **Transitive dependency pinning** and version resolution
- **Cargo resolver versions** and migration to v2/v3
- **Cargo.lock** files and when to commit them
- **Reproducible builds** guarantees and limitations
- **Semantic versioning** strategies and best practices
- **Private package repositories** (Kellnr, Alexandrie, Panamax, Artifactory)
- **Workspace management** for multi-crate projects
- **Benchmarking** with Criterion
- **Examples** directory structure
- **Documentation generation** with rustdoc

> 📚 **Official Documentation**: [The Cargo Book](https://doc.rust-lang.org/cargo/)

---

## 1. Rust Editions

### What Are Editions?

Rust editions are opt-in milestones that allow the language to evolve while maintaining its stability guarantees. Every three years, Rust releases a new edition that may include breaking changes to syntax or semantics, but:

- **All editions are fully interoperable** - crates using different editions work together seamlessly
- **Old code continues to compile** - editions are opt-in, not forced updates
- **The compiler supports all editions** - one compiler handles all edition variants

### Available Editions

| Edition | Released | Default Resolver | Key Changes |
|---------|----------|-----------------|-------------|
| **2015** | Rust 1.0 | v1 | Original edition, `extern crate` required |
| **2018** | Rust 1.31 (Dec 2018) | v1 | Module system improvements, `async`/`await`, NLL |
| **2021** | Rust 1.56 (Oct 2021) | v2 | Disjoint captures, `into_iter()` arrays, reserved identifiers |
| **2024** | Rust 1.85 (Feb 2025) | v3 | MSRV-aware resolver, `gen` keyword, unsafe env functions |

### Configuring Edition

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"  # Specify edition here
# edition = "2024" when available (Rust 1.85+)
```

### Key Edition Changes

**Edition 2018:**
- No more `extern crate` declarations (except for macros)
- Uniform path syntax in `use` statements
- `async`/`await` keywords reserved
- Non-lexical lifetimes (NLL)
- Module system simplification

**Edition 2021:**
- Disjoint captures in closures (only capture used fields)
- `array.into_iter()` iterates by value
- New reserved keywords: `try`
- Default to resolver v2 for Cargo
- Panic macros require format strings

**Edition 2024 (Coming Feb 2025):**
- MSRV-aware dependency resolution (resolver v3)
- `gen` keyword for generators/coroutines
- `std::env::set_var` and `remove_var` marked unsafe
- Tail expression temporary lifetime changes
- `unsafe extern` blocks and attributes

### Migration Between Editions

```bash
# WARNING: This command MODIFIES your code!
# Automatically migrates code to be compatible with the next edition
cargo fix --edition

# Also apply idiomatic style changes for the new edition
cargo fix --edition --edition-idioms

# After running cargo fix, manually update Cargo.toml
# edition = "2021" → edition = "2024"
```

### Edition Selection Strategy

| Project Type | Recommended Edition | Rationale |
|--------------|-------------------|-----------|
| **New projects** | Latest stable | Access to all improvements |
| **Libraries** | Conservative (2018/2021) | Wider compatibility |
| **Applications** | Latest stable | Modern features, better ergonomics |
| **Legacy code** | Keep current | Migrate when beneficial |

> 📖 **Learn More**: [The Rust Edition Guide](https://doc.rust-lang.org/edition-guide/)

---

## 2. Rust Toolchain Channels

### The Release Train Model

Rust uses a "train model" with three channels, each serving different needs:

```
Nightly (daily) → Beta (6 weeks) → Stable (6 weeks)
         ↓              ↓                ↓
    Cutting edge    Pre-release    Production ready
```

### Channel Characteristics

| Channel | Release Cycle | Stability | Features | Use Case |
|---------|--------------|-----------|----------|----------|
| **Stable** | Every 6 weeks | Guaranteed stable | Only stabilized | Production |
| **Beta** | Every 6 weeks | Generally stable | Next stable release | Testing |
| **Nightly** | Daily | May break | Unstable features | Experimentation |

### Stable Channel

The default and recommended channel for most users:

```bash
# Install or switch to stable
rustup default stable

# Update stable toolchain
rustup update stable

# Use specific stable version
rustup install 1.82.0
rustup default 1.82.0
```

**When to use:**
- Production applications
- Published libraries
- CI/CD pipelines
- Learning Rust

### Beta Channel

Pre-release versions for testing upcoming features:

```bash
# Switch to beta
rustup default beta

# Test with beta in CI
rustup run beta cargo test
```

**When to use:**
- Testing for upcoming breakage
- CI regression testing
- Preparing for next stable

### Nightly Channel

Bleeding-edge features and unstable APIs:

```bash
# Switch to nightly
rustup default nightly

# Use nightly for specific project
rustup override set nightly

# Install specific nightly
rustup install nightly-2024-11-28
```

**Enabling unstable features:**
```rust
// Only works on nightly
#![feature(generators)]
#![feature(type_alias_impl_trait)]

fn main() {
    // Use unstable features
}
```

### Managing Toolchains

**Project-specific toolchain (`rust-toolchain.toml`):**
```toml
[toolchain]
channel = "1.82.0"  # Or "stable", "beta", "nightly"
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-unknown"]
```

**Toolchain override commands:**
```bash
# Set override for current directory
rustup override set nightly

# Remove override
rustup override unset

# List overrides
rustup override list

# Run command with specific toolchain
cargo +nightly build
cargo +1.82.0 test
```

### Unstable Features and Feature Gates

Nightly Rust allows access to unstable features via feature gates:

```rust
// src/lib.rs or src/main.rs
#![feature(const_trait_impl)]  // Language feature
#![feature(test)]               // Library feature

// Now you can use the unstable features
extern crate test;
use test::Bencher;
```

**Common unstable features:**
- `async_fn_in_trait` - Async functions in traits
- `generators` - Generator/coroutine support
- `const_generics` - Const generics extensions
- `specialization` - Trait specialization

### Choosing the Right Channel

| Scenario | Recommended Channel | Reasoning |
|----------|-------------------|-----------|
| **Web service production** | Stable | Reliability crucial |
| **Library development** | Stable + Beta CI | Ensure compatibility |
| **Compiler plugin** | Nightly | Requires compiler internals |
| **Embedded no_std** | Stable or Nightly | Depends on features needed |
| **Learning/tutorials** | Stable | Consistent experience |
| **Research/experimentation** | Nightly | Access to latest features |

### CI/CD Multi-Channel Testing

```yaml
# .github/workflows/rust.yml
strategy:
  matrix:
    rust: [stable, beta, nightly]
    continue-on-error: ${{ matrix.rust == 'nightly' }}

steps:
  - uses: actions-rs/toolchain@v1
    with:
      toolchain: ${{ matrix.rust }}
      override: true
```

> 📖 **Learn More**:
> - [The rustup book](https://rust-lang.github.io/rustup/)
> - [Appendix G: How Rust is Made and "Nightly Rust"](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html)

---

## 3. How Cargo Resolves Dependencies

### Semantic Versioning and Caret Requirements

Cargo uses semantic versioning (SemVer) by default, with caret requirements:

```toml
[dependencies]
serde = "1.0"        # Actually means "^1.0" - compatible with 1.x.y
tokio = "1.21.2"     # Compatible with 1.21.2 to 1.x.y (but not 2.0)
```

**Key SemVer rules:**
- **Major version** (1.x.y → 2.x.y): Breaking changes
- **Minor version** (1.1.x → 1.2.x): Backward-compatible features
- **Patch version** (1.1.1 → 1.1.2): Bug fixes only

### Version Requirement Operators

```toml
[dependencies]
# Exact version
exact = "=1.0.0"

# Range requirements
range = ">=1.2, <1.5"

# Wildcard
wildcard = "1.0.*"

# Tilde requirements (patch-level compatible)
patch_only = "~1.0.0"  # 1.0.0 to 1.0.x

# Caret requirements (default - backwards compatible)
default = "^1.0.0"     # 1.0.0 to 1.x.y
```

### Transitive Dependency Resolution

Cargo builds a dependency graph and resolves versions using a **maximum version strategy**:

```
Your Project
├── crate-a = "1.0"
│   └── shared = "2.1"    # Transitive dependency
└── crate-b = "2.0"
    └── shared = "2.3"    # Same dependency, different version requirement
```

**Resolution:** Cargo picks `shared = "2.3"` (highest compatible version).

> 📖 **Learn More**: [Dependency Resolution in The Cargo Book](https://doc.rust-lang.org/cargo/reference/resolver.html)

---

## 4. Cargo Resolver Versions

### Understanding Resolver Versions

Cargo has three resolver versions that control how features and dependencies are resolved:

| Resolver | Default For | Key Behavior | Rust Version |
|----------|------------|-------------|-------------|
| **v1** | Edition 2015/2018 | Unifies features for a package across all uses | All |
| **v2** | Edition 2021 | Features resolved independently per target | 1.51+ |
| **v3** | Edition 2024 | MSRV-aware dependency resolution | 1.84+ |

### Resolver Version 2 Changes

The v2 resolver, default since Edition 2021, provides more precise feature resolution:

```toml
# Cargo.toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"  # Implies resolver = "2"

# Or explicitly set for older editions:
[package]
edition = "2018"
resolver = "2"  # Opt-in to v2 resolver
```

**Key differences in v2:**
- **Platform-specific dependencies**: Features for `target.'cfg(windows)'` dependencies aren't enabled when building for Linux
- **Build dependencies**: `build-dependencies` and proc-macros don't share features with normal dependencies
- **Dev dependencies**: `dev-dependencies` only activate features when building tests/examples

### Workspace Resolver Configuration

For virtual workspaces, explicitly set the resolver:

```toml
# Workspace Cargo.toml
[workspace]
members = ["crate-a", "crate-b"]
resolver = "2"  # Must be set explicitly for workspaces
```

### Migration and Debugging

```bash
# WARNING: This command MODIFIES your code for edition compatibility
cargo fix --edition

# Analyze feature resolution
cargo tree -f '{p} {f}'

# Find duplicate dependencies with different features
cargo tree -d

# See why a feature is enabled
cargo tree -e features -i <package>
```

**Migration notes:**
- Most projects require few or no changes
- `cargo fix --edition` automatically modifies code for compatibility AND reports feature resolution differences
- Use `cargo tree` to debug feature activation issues

> 📖 **Official Documentation**:
> - [Cargo Resolver Documentation](https://doc.rust-lang.org/cargo/reference/resolver.html)
> - [Edition 2021 Resolver Changes](https://doc.rust-lang.org/edition-guide/rust-2021/default-cargo-resolver.html)

---

## 5. Minimum Supported Rust Version (MSRV)

### Declaring MSRV

The `rust-version` field specifies your project's minimum supported Rust version:

```toml
[package]
name = "my-project"
version = "0.1.0"
rust-version = "1.74"  # Minimum Rust version required
```

### How MSRV Works

**What it does:**
- Documents the minimum Rust version for users
- Cargo emits warnings when building with older Rust versions
- With resolver v3, influences dependency resolution
- Used by tools like `clippy` for compatibility lints

**What it doesn't do:**
- Doesn't prevent building with older versions (use `--ignore-rust-version` to bypass)
- Doesn't guarantee dependencies maintain compatible MSRV
- Doesn't automatically test against the specified version

### Finding Your Project's MSRV

Use the `cargo-msrv` tool to determine and verify MSRV:

```bash
# Install cargo-msrv
cargo install cargo-msrv

# Find the minimum supported Rust version
cargo msrv find

# Verify your declared MSRV works
cargo msrv verify

# Use specific search strategy
cargo msrv find --bisect  # Binary search (default)
cargo msrv find --linear  # Linear search
```

### MSRV Best Practices

**For Libraries:**
```toml
[package]
rust-version = "1.74"  # Be conservative for wide compatibility

# Document MSRV policy in README
# Consider supporting 6-12 months of Rust releases
```

**For Applications:**
```toml
[package]
rust-version = "1.82"  # Can use newer features

# Pin exact version for reproducible builds
# rust-toolchain.toml
[toolchain]
channel = "1.82.0"
```

### CI/CD MSRV Testing

```yaml
# GitHub Actions example
name: MSRV Check
on: [push, pull_request]

jobs:
  msrv:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Install MSRV toolchain
      run: |
        rustup toolchain install $(grep rust-version Cargo.toml | cut -d'"' -f2) --profile minimal
        rustup default $(grep rust-version Cargo.toml | cut -d'"' -f2)
    - name: Build with MSRV
      run: cargo build --locked
    - name: Test with MSRV
      run: cargo test --locked
```

### MSRV and Dependencies

**Challenge:** Dependencies can increase their MSRV in minor versions:

```toml
# Your Cargo.toml
rust-version = "1.74"

[dependencies]
serde = "1.0"  # Might update to require Rust 1.75+
```

**Solutions:**

1. **Use resolver v3** (when available) for automatic MSRV-compatible resolution
2. **Pin dependencies** if they break MSRV:
   ```toml
   # Pin to last compatible version
   problematic-dep = "=1.2.3"
   ```
3. **Use `cargo update -Z msrv-policy`** (nightly) to respect MSRV when updating

### MSRV Policy Guidelines

| Project Type | Suggested MSRV | Rationale |
|--------------|----------------|-----------|
| **Foundational libraries** | 6-12 months old | Maximum compatibility |
| **Application libraries** | 3-6 months old | Balance features/compatibility |
| **Applications** | Current stable | Use latest features |
| **Internal tools** | Latest stable | No external users |

> 📖 **Documentation**:
> - [Rust Version in Cargo Book](https://doc.rust-lang.org/cargo/reference/rust-version.html)
> - [cargo-msrv Documentation](https://github.com/foresterre/cargo-msrv)
> - [RFC 2495: Min Rust Version](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)

---

## 6. Cargo.lock: The Version Lockfile

### What Cargo.lock Contains

```toml
# This file is automatically generated by Cargo.
# It is not intended for manual editing.
version = 3

[[package]]
name = "proc-macro2"
version = "1.0.47"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5ea3d908b0e36316caf9e9e2c4625cdde190a7e6f440d794667ed17a1855e725"
dependencies = [
 "unicode-ident",
]

[[package]]
name = "quote"
version = "1.0.21"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bbe448f377a7d6961e30f5955f9b8d106c3f5e449d493ee1b125c1d43c2b5179"
dependencies = [
 "proc-macro2",
]
```

### When to Commit Cargo.lock

| Project Type | Commit Cargo.lock? | Rationale |
|--------------|-------------------|-----------|
| **Binary/Application** | ✅ **Yes** | Ensures reproducible builds across environments |
| **Library** | ❌ **No** | Allows downstream crates flexibility in version selection |
| **Workspace root** | ✅ **Yes** | Pins versions for all workspace members |

**Exception for libraries:** Consider committing if you have specific CI/testing requirements.

### Cargo.lock and CI/CD

```yaml
# GitHub Actions example
- name: Cache dependencies
  uses: actions/cache@v3
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target/
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

The `Cargo.lock` hash ensures cache invalidation when dependencies change.

> 📖 **Learn More**: [Cargo.lock Format](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)

---

## 7. Reproducible Builds in Cargo

### What Cargo Guarantees

✅ **Guaranteed reproducible:**
- Same `Cargo.lock` → same dependency versions
- Same Rust compiler version → same compilation behavior
- Same target platform → same binary output

❌ **Not automatically guaranteed:**
- Compiler version differences
- Different target architectures
- System library differences (for `sys` crates)

### Ensuring Reproducibility

```toml
# rust-toolchain.toml - pins compiler version
[toolchain]
channel = "1.70.0"
components = ["rustfmt", "clippy"]
targets = ["x86_64-unknown-linux-gnu"]
```

```dockerfile
# Dockerfile with pinned Rust version
FROM rust:1.70.0-slim as builder
WORKDIR /app
COPY Cargo.lock Cargo.toml ./
# Pre-build dependencies for faster rebuilds
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release
COPY . .
RUN cargo build --release
```

### The `--locked` Flag

```bash
# CI builds should use --locked to prevent dependency updates
cargo build --release --locked

# Fails if Cargo.lock is out of sync with Cargo.toml
cargo check --locked
```

---

## 8. Dependency Update Strategies

### Manual Updates

```bash
# Update all dependencies within semver constraints
cargo update

# Update specific dependency
cargo update -p serde

# Update to specific version
cargo update -p tokio --precise 1.21.0
```

### Automated Dependency Management

**Dependabot configuration (`.github/dependabot.yml`):**
```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 5
    groups:
      aws:
        patterns:
          - "aws-*"
      tokio:
        patterns:
          - "tokio*"
```

**Renovate configuration:**
```json
{
  "extends": ["config:base"],
  "cargo": {
    "rangeStrategy": "pin"
  }
}
```

### Semantic Versioning Strategy

```toml
# Conservative: Pin major versions
serde = "1.0.144"

# Moderate: Allow minor updates
tokio = "~1.21.0"

# Aggressive: Allow patch updates
uuid = "^1.0.0"
```

---

## 9. Private Package Repositories

### Overview of Private Registry Solutions

| Solution | Type | Key Features | Best For |
|----------|------|--------------|----------|
| **Kellnr** | Self-hosted registry | Web UI, docs hosting, crates.io proxy | Small-medium teams |
| **Alexandrie** | Alternative registry | Modular backends, Git/S3 storage | Custom deployments |
| **Panamax** | Mirror tool | Offline development, full mirror | Air-gapped environments |
| **Artifactory** | Enterprise | Sparse index, CI/CD integration | Large enterprises |

### Kellnr: The Private Rust Registry

[Kellnr](https://kellnr.io) is a complete private registry solution:

```toml
# .cargo/config.toml
[registries]
kellnr = {
    index = "git://your-kellnr-host:9418/index",
    token = "your-auth-token"
}
```

**Docker deployment:**
```bash
docker run -p 8000:8000 \
  -e "KELLNR_ORIGIN__HOSTNAME=your-domain" \
  ghcr.io/kellnr/kellnr:latest
```

**Features:**
- Web UI for crate management
- Documentation hosting (like docs.rs)
- Crates.io proxy/cache
- User and group management
- PostgreSQL/SQLite backends
- S3/filesystem storage

> 📖 **Documentation**: [kellnr.io/documentation](https://kellnr.io/documentation)

### Alexandrie: Modular Alternative Registry

[Alexandrie](https://github.com/Hirevo/alexandrie) offers flexible backend options:

```toml
# alexandrie.toml configuration
[database]
url = "postgresql://localhost/alexandrie"

[storage]
type = "s3"
bucket = "my-crates"
region = "us-east-1"
```

**Features:**
- Multiple database backends (PostgreSQL, MySQL, SQLite)
- Flexible storage (filesystem, S3, Git)
- Optional web frontend
- Docker deployment ready

> 📖 **Documentation**: [Alexandrie Docs](https://hirevo.github.io/alexandrie/)

### Panamax: Mirror for Offline Development

[Panamax](https://github.com/panamax-rs/panamax) creates complete mirrors:

```bash
# Initialize mirror
panamax init my-mirror

# Sync all crates
panamax sync my-mirror

# Or sync only project dependencies
cargo vendor
panamax sync my-mirror vendor/

# Serve the mirror
panamax serve my-mirror --port 8080
```

**Client configuration:**
```toml
# .cargo/config.toml
[source.my-mirror]
registry = "http://panamax.internal/crates.io-index"

[source.crates-io]
replace-with = "my-mirror"
```

> 📖 **Documentation**: [Panamax GitHub](https://github.com/panamax-rs/panamax)

### JFrog Artifactory: Enterprise Solution

[Artifactory](https://jfrog.com/artifactory/) provides enterprise-grade registry:

```toml
# .cargo/config.toml
[registries]
artifactory = {
    index = "https://artifactory.company.com/artifactory/api/cargo/rust-local"
}
```

**Publishing with authentication:**
```bash
cargo publish --registry artifactory \
  --token "Bearer <access-token>"
```

**Features:**
- Sparse index support (Cargo 1.60+)
- Virtual repositories
- LDAP/SAML integration
- Vulnerability scanning
- CI/CD integration
- High availability

> 📖 **Documentation**: [JFrog Cargo Registry](https://www.jfrog.com/confluence/display/JFROG/Cargo+Registry)

---

## 10. Alternative Registries and Git Dependencies

### Using Alternative Registries

```toml
# .cargo/config.toml
[registries]
my-company = { index = "https://github.com/my-company/crate-index" }

[source.crates-io]
replace-with = "my-company"
```

```toml
# In Cargo.toml
[dependencies]
serde = "1.0"  # from crates.io
internal-lib = { version = "2.0", registry = "my-company" }
```

### Git Dependencies

```toml
[dependencies]
# Git repository
my-lib = { git = "https://github.com/user/my-lib" }

# Specific branch
my-lib = { git = "https://github.com/user/my-lib", branch = "feature" }

# Specific tag
my-lib = { git = "https://github.com/user/my-lib", tag = "v1.0.0" }

# Specific commit
my-lib = { git = "https://github.com/user/my-lib", rev = "abc123" }
```

### Path Dependencies

```toml
[dependencies]
# Local path dependency
shared-utils = { path = "../shared-utils" }

# Conditional dependencies
[target.'cfg(windows)'.dependencies]
winapi = "0.3"

[target.'cfg(unix)'.dependencies]
libc = "0.2"
```

---

## 11. Workspace Management

### Workspace Configuration

```toml
# Cargo.toml in workspace root
[workspace]
members = [
    "web-server",
    "database",
    "shared-models",
]
exclude = ["legacy-code"]

# Shared workspace dependencies
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.47", features = ["full"] }
uuid = "1.0"

# Shared configuration
[workspace.package]
authors = ["Your Team <team@company.com>"]
edition = "2021"
license = "MIT OR Apache-2.0"
```

```toml
# Member crate Cargo.toml
[package]
name = "web-server"
version = "0.1.0"
# Inherit workspace configuration
authors.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
# Use workspace dependencies
serde.workspace = true
tokio.workspace = true
# Local workspace dependencies
shared-models = { path = "../shared-models" }
```

### Workspace Commands

```bash
# Build entire workspace
cargo build

# Build specific package
cargo build -p web-server

# Run tests across workspace
cargo test

# Update dependencies for workspace
cargo update
```

---

## 12. Security and Audit

### Dependency Auditing

```bash
# Install cargo-audit
cargo install cargo-audit

# Audit dependencies for known vulnerabilities
cargo audit

# Fix vulnerabilities (updates Cargo.lock)
cargo audit fix
```

### Supply Chain Security

```toml
# .cargo/config.toml - require signatures for some registries
[registries.secure-registry]
index = "https://secure-registry.example.com/"
protocol = "sparse"
```

**CI/CD security checks:**
```yaml
- name: Security audit
  run: |
    cargo install cargo-audit
    cargo audit

- name: Check for vulnerabilities
  run: |
    cargo install cargo-deny
    cargo deny check
```

---

## 13. Performance and Optimization

### Dependency Features

```toml
# Minimize dependencies by disabling default features
[dependencies]
tokio = { version = "1.47", default-features = false, features = ["rt"] }
serde = { version = "1.0", features = ["derive"] }
```

### Build Performance

```toml
# .cargo/config.toml - optimize builds
[build]
jobs = 4                    # Parallel build jobs
rustflags = ["-C", "target-cpu=native"]

[profile.dev]
debug = 1                   # Reduce debug info for faster builds
incremental = true

[profile.release]
codegen-units = 1          # Better optimization
lto = true                 # Link-time optimization
```

### Dependency Caching

```dockerfile
# Multi-stage Docker build optimizing dependency caching
FROM rust:1.70 as planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.70 as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.70 as builder
WORKDIR /app
COPY --from=cacher /app/target target
COPY . .
RUN cargo build --release
```

---

## 14. Benchmarking with Criterion

### Setting Up Benchmarks

Criterion.rs is the de facto standard for benchmarking in Rust:

```toml
# Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "my_benchmark"
harness = false  # Disable default benchmark harness
```

### Writing Benchmarks

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| {
        b.iter(|| fibonacci(black_box(20)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

### Advanced Benchmarking Features

```rust
use criterion::{BenchmarkId, Criterion, Throughput};

fn bench_with_input_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("sorting");

    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                let mut vec: Vec<i32> = (0..size).collect();
                b.iter(|| vec.sort());
            },
        );
    }
    group.finish();
}
```

### Running and Analyzing Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench my_benchmark

# Save baseline for comparison
cargo bench -- --save-baseline my_baseline

# Compare against baseline
cargo bench -- --baseline my_baseline

# Generate HTML reports (in target/criterion/)
cargo bench -- --verbose
```

### Built-in Benchmarking (Nightly)

For simpler needs, Rust's unstable benchmark feature:

```rust
#![feature(test)]
extern crate test;

#[bench]
fn bench_simple(b: &mut test::Bencher) {
    b.iter(|| {
        // Code to benchmark
        1 + 1
    });
}
```

```bash
# Run with nightly
cargo +nightly bench
```

### Best Practices

1. **Use `black_box`** to prevent compiler optimizations
2. **Warm up** with multiple iterations
3. **Test different input sizes** for algorithms
4. **Compare baselines** when optimizing
5. **Run in release mode** for accurate results

> 📖 **Documentation**:
> - [Criterion.rs Book](https://bheisler.github.io/criterion.rs/book/)
> - [cargo bench Documentation](https://doc.rust-lang.org/cargo/commands/cargo-bench.html)

---

## 15. Examples Directory Structure

### Standard Examples Layout

```
my-project/
├── Cargo.toml
├── src/
│   └── lib.rs
├── examples/
│   ├── simple.rs              # Single-file example
│   ├── client.rs              # Another single-file example
│   └── complex_example/       # Multi-file example
│       ├── main.rs
│       └── helper.rs
└── tests/
```

### Creating Examples

**Single-file example:**
```rust
// examples/simple.rs
use my_project::MyStruct;

fn main() {
    let instance = MyStruct::new();
    println!("Example output: {:?}", instance);
}
```

**Multi-file example:**
```rust
// examples/complex_example/main.rs
mod helper;

use my_project::MyStruct;
use helper::process_data;

fn main() {
    let data = MyStruct::new();
    let result = process_data(&data);
    println!("Processed: {:?}", result);
}
```

### Running Examples

```bash
# List all examples
cargo run --example

# Run specific example
cargo run --example simple

# Run with arguments
cargo run --example client -- --host localhost --port 8080

# Build all examples
cargo build --examples

# Test that examples compile
cargo test --examples
```

### Example Configuration

```toml
# Cargo.toml
[[example]]
name = "client"
required-features = ["client-feature"]

[[example]]
name = "server"
path = "examples/custom_path/server.rs"
```

### Testing Examples

```rust
// examples/testable.rs
use my_project::calculate;

fn main() {
    let result = calculate(5, 10);
    println!("Result: {}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_logic() {
        assert_eq!(calculate(5, 10), 15);
    }
}
```

```bash
# Run tests in examples
cargo test --example testable
```

### Best Practices

1. **Keep examples focused** on specific use cases
2. **Include comments** explaining the example
3. **Use realistic scenarios** that users might encounter
4. **Test examples** in CI to ensure they stay working
5. **Document in README** which examples demonstrate what

> 📖 **Documentation**: [Package Layout - The Cargo Book](https://doc.rust-lang.org/cargo/guide/project-layout.html)

---

## 16. Documentation Generation with rustdoc

### Documentation Comments

```rust
//! # My Library
//!
//! This crate provides functionality for processing data.
//!
//! ## Quick Start
//!
//! ```
//! use my_lib::process;
//! let result = process("input");
//! ```

/// Processes the given input string.
///
/// # Arguments
///
/// * `input` - A string slice to process
///
/// # Returns
///
/// The processed string
///
/// # Examples
///
/// ```
/// use my_lib::process;
///
/// let result = process("hello");
/// assert_eq!(result, "HELLO");
/// ```
///
/// # Panics
///
/// Panics if the input is empty.
pub fn process(input: &str) -> String {
    if input.is_empty() {
        panic!("Input cannot be empty");
    }
    input.to_uppercase()
}
```

### Generating Documentation

```bash
# Generate docs for your crate
cargo doc

# Generate and open in browser
cargo doc --open

# Include dependencies
cargo doc --no-deps  # Exclude dependencies (default)
cargo doc --document-private-items  # Include private items

# Generate for all workspace members
cargo doc --workspace

# Generate for specific package
cargo doc -p my_package
```

### Advanced rustdoc Features

```rust
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
pub unsafe fn dangerous(ptr: *const u8) -> u8 {
    *ptr
}

/// This function is deprecated.
///
/// Use [`new_function`] instead.
#[deprecated(since = "0.2.0", note = "Use new_function instead")]
pub fn old_function() {}

/// Links to other items: [`Vec`], [`std::io::Error`]
///
/// External links: [Rust Book](https://doc.rust-lang.org/book/)
pub fn documented() {}
```

### Testing Documentation Examples

```rust
/// ```
/// use my_lib::add;
/// assert_eq!(add(2, 2), 4);
/// ```
///
/// ```compile_fail
/// use my_lib::add;
/// add("not", "numbers");  // This should fail
/// ```
///
/// ```no_run
/// use my_lib::expensive_operation;
/// expensive_operation();  // Compiles but doesn't run
/// ```
///
/// ```ignore
/// // This example is ignored during testing
/// unimplemented_feature();
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

```bash
# Run documentation tests
cargo test --doc

# Run all tests including doc tests
cargo test
```

### Customizing Documentation

```toml
# Cargo.toml
[package.metadata.docs.rs]
# Specify features to enable on docs.rs
all-features = true
# Specify targets for docs.rs
targets = ["x86_64-unknown-linux-gnu"]
# Custom rustdoc args
rustdoc-args = ["--cfg", "docsrs"]
```

### rustdoc vs cargo doc

```bash
# Direct rustdoc usage
rustdoc src/lib.rs --crate-name my_lib

# cargo doc (recommended)
cargo doc

# cargo rustdoc with custom flags
cargo rustdoc -- --document-private-items --html-in-header custom.html
```

### Publishing to docs.rs

Documentation is automatically built and hosted on [docs.rs](https://docs.rs) when you publish to crates.io:

```bash
# Publish crate (docs built automatically)
cargo publish

# Your docs will be available at:
# https://docs.rs/your-crate/latest/
```

### Best Practices

1. **Document all public APIs** with examples
2. **Use doc tests** to ensure examples work
3. **Link between items** using `[`backticks`]`
4. **Include module-level documentation** with `//!`
5. **Add diagrams** using ASCII art or links to images
6. **Use semantic sections**: Examples, Panics, Safety, Errors

> 📖 **Documentation**:
> - [The rustdoc Book](https://doc.rust-lang.org/rustdoc/)
> - [cargo doc Documentation](https://doc.rust-lang.org/cargo/commands/cargo-doc.html)
> - [Documentation on docs.rs](https://docs.rs)

---

## 17. Migration from Other Package Managers

### From npm/package.json

| npm | Cargo |
|-----|-------|
| `package.json` | `Cargo.toml` |
| `package-lock.json` | `Cargo.lock` |
| `npm install` | `cargo build` |
| `npm update` | `cargo update` |
| `devDependencies` | `[dev-dependencies]` |

### From NuGet (.NET)

| NuGet | Cargo |
|-------|-------|
| `*.csproj` | `Cargo.toml` |
| `packages.lock.json` | `Cargo.lock` |
| Package Source | Registry |
| `dotnet restore` | `cargo fetch` |

### From Maven (Java)

| Maven | Cargo |
|-------|-------|
| `pom.xml` | `Cargo.toml` |
| Repository | Registry |
| `mvn dependency:tree` | `cargo tree` |

---

## 18. Best Practices Summary

### Version Management
✅ **Do:**
- Use semantic versioning consistently
- Commit `Cargo.lock` for applications
- Pin Rust version with `rust-toolchain.toml`
- Regular dependency audits

❌ **Don't:**
- Use exact versions unless necessary
- Ignore security advisories
- Mix different versioning strategies

### Workspace Organization
✅ **Do:**
- Use workspaces for multi-crate projects
- Share common dependencies
- Keep workspace configuration minimal

❌ **Don't:**
- Create overly complex workspace hierarchies
- Mix library and application concerns

### CI/CD Integration
✅ **Do:**
- Cache dependencies effectively
- Use `--locked` flag in CI
- Run security audits automatically

❌ **Don't:**
- Allow dependency updates in production builds
- Skip vulnerability checks

---

## Practical Exercise: Complete Project Setup

### Full-Featured Cargo Configuration

Create a robust dependency management setup for a web service:

```toml
# Cargo.toml
[package]
name = "web-service"
version = "0.1.0"
edition = "2021"  # Edition 2024 available with Rust 1.85+
authors = ["Your Name <you@example.com>"]
license = "MIT OR Apache-2.0"
description = "A web service with comprehensive Cargo setup"
repository = "https://github.com/yourname/web-service"
documentation = "https://docs.rs/web-service"

# Explicitly use resolver v2 (optional with edition 2021)
resolver = "2"

[dependencies]
# Web framework
axum = "0.8"

# Async runtime
tokio = { version = "1.47", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres"] }

# Error handling
anyhow = "1.0"
thiserror = "2.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
# Testing
tokio-test = "0.4"
pretty_assertions = "1.4"

# Benchmarking
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "api_benchmark"
harness = false

[[example]]
name = "client"
required-features = ["client"]

[features]
default = ["postgres"]
postgres = ["sqlx/postgres"]
mysql = ["sqlx/mysql"]
client = []

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[profile.bench]
inherits = "release"

# Documentation settings for docs.rs
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: 1.70.0
        override: true
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target/
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    - name: Build
      run: cargo build --locked
    - name: Test
      run: cargo test --locked
    - name: Security audit
      run: |
        cargo install cargo-audit
        cargo audit
```

### Complete Project Structure

```
web-service/
├── Cargo.toml
├── Cargo.lock         # Commit this for applications
├── .cargo/
│   └── config.toml    # Local cargo configuration
├── src/
│   ├── main.rs        # Binary entry point
│   └── lib.rs         # Library code
├── examples/
│   ├── client.rs      # Example client
│   └── server.rs      # Example server
├── benches/
│   └── api_benchmark.rs  # Criterion benchmarks
├── tests/
│   └── integration.rs # Integration tests
└── .github/
    └── workflows/
        └── ci.yml     # GitHub Actions CI
```

### Cargo Configuration

```toml
# .cargo/config.toml
[build]
jobs = 4
target-dir = "target"

[registries]
# Configure private registry if needed
# private = { index = "https://my-registry.com/index" }

[net]
retry = 3
git-fetch-with-cli = true

[profile.dev]
opt-level = 0
debug = true

[profile.release-with-debug]
inherits = "release"
debug = true
```

This comprehensive setup provides:
- **Resolver v2** for precise feature resolution
- **Benchmarking** support with Criterion
- **Examples** directory for usage demonstrations
- **Documentation** configuration for docs.rs
- **Security auditing** in CI/CD
- **Private registry** support
- **Optimized builds** with LTO and single codegen unit

---

## Summary

Cargo's dependency management system provides powerful tools for maintaining reproducible, secure builds. Key takeaways:

- **Understand SemVer**: Use appropriate version constraints for your needs
- **Commit Cargo.lock** for applications, not libraries
- **Pin environments**: Use `rust-toolchain.toml` and `--locked` in CI
- **Audit regularly**: Use `cargo audit` and automated security scanning
- **Organize wisely**: Leverage workspaces for multi-crate projects

Coming from other ecosystems, Cargo's approach might feel restrictive at first, but this strictness enables the reliability and safety that Rust is known for in production environments.

## Additional Resources

- 📚 [The Cargo Book](https://doc.rust-lang.org/cargo/) - Official Cargo documentation
- 📖 [The rustdoc Book](https://doc.rust-lang.org/rustdoc/) - Documentation generation guide
- 🔧 [Cargo Reference](https://doc.rust-lang.org/cargo/reference/) - Detailed reference
- 📊 [Criterion.rs](https://bheisler.github.io/criterion.rs/book/) - Benchmarking guide
- 🏢 [Kellnr Documentation](https://kellnr.io/documentation) - Private registry
- 🔐 [cargo-audit](https://github.com/RustSec/rustsec/tree/main/cargo-audit) - Security auditing