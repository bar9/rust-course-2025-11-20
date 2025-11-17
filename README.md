# Course overview
This course is targeted at developers experienced in other procedural or object-oriented programming languages.

* Day 1: Rust foundations and the concept of ownership
* Day 2: Type system and error handling
* Day 3: Systems programming & concurrency
* Transfer day: other languages to Rust

Each day is a mix of theory and exercises. day 1 and 2 feature exercises in a std environment (building cli applications on desktop). day 3 and transfer day feature no_std and building embedded applications on an ESP32C3 microcontroller.

# This repository
Contains the course slides/script as an mdbook and solutions to the exercises on the `solutions` branch. Will be updated before and during the course.

# Installation Instructions Day 1 and 2
Please ensure the following software is installed on the device you bring to the course.

If there are any questions or difficulties during the installation please don't hesitate to contact the instructor (rolandbrand11@gmail.com).

## Rust
Install Rust using rustup (Rust's official installer)
- Visit [rust-lang.org](https://rust-lang.org/tools/install) and follow the installation instructions for your operating system.
- Verify installation with: `rustc --version` and `cargo --version`

## Git
Git for version control: [git-scm.com](https://git-scm.com/)
- Make sure you can access it through the command line: `git --version`

## Zed Editor
Download from [zed.dev](https://zed.dev/)

During the course the trainer will use Zed - participants are recommended to use the same editor, but are free to choose any other editor or IDE. The trainer will not be able to provide setup or configuration support for other editors or IDEs during the course.

## Create a Test Project
Create a new Rust project and build it:

```bash
cargo new hello-rust
cd hello-rust
cargo build
```

## Run the Project
Execute the project to verify your Rust installation:

```bash
cargo run
```

You should see "Hello, world!" printed to your terminal.

## Troubleshooting
If you encounter any issues:

### Rust Installation Issues
- On Unix-like systems, you might need to install build essentials: `sudo apt install build-essential` (Ubuntu/Debian)
- On Windows, you might need to install Visual Studio C++ Build Tools

### Cargo Issues
- Try clearing the cargo cache: `cargo clean`
- Update rust: `rustup update`


## Cleanup
To remove the test project:

```bash
cd
rm -rf hello-rust
```

If you can complete all these steps successfully, your environment is ready for the first two days of the Rust course!

**→ Regularly pull updates to the repo. There will also be additional setup instructions for days 3 and 4.**

