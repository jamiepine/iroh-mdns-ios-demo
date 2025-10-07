# xtask

Build automation tasks for iroh-mdns-test using the xtask pattern.

## What is xtask?

The **xtask pattern** is the idiomatic Rust way to handle project-specific build automation. Instead of using shell scripts, Makefiles, or external build tools, you write your build tasks as Rust code in a workspace member called `xtask`.

This approach is used by major Rust projects including:

- **rust-analyzer** - IDE support
- **tokio** - async runtime
- **cargo** itself - Rust's package manager

## Benefits

- ✅ **Pure Rust** - No shell scripts to maintain
- ✅ **Type-safe** - Catch errors at compile time
- ✅ **Debuggable** - Use standard Rust debugging tools
- ✅ **Cross-platform** - Works on Windows, macOS, Linux
- ✅ **No external tools** - Just `cargo` and Rust

## Usage

Run tasks using `cargo xtask`:

```bash
# Build iOS framework (device + simulator)
cargo xtask build-ios
```

## Available Commands

### `build-ios`

Builds the `mdns-peer` library for iOS:

1. Compiles for `aarch64-apple-ios` (physical devices)
2. Compiles for `aarch64-apple-ios-sim` (M1/M2 simulator)
3. Creates XCFramework directory structure
4. Copies static libraries to framework locations
5. Generates Info.plist files:
   - Top-level XCFramework Info.plist
   - Per-architecture Info.plist files

**Output:** `mdns-peer/mdns_peer.xcframework/`

The XCFramework structure looks like:

```
mdns_peer.xcframework/
├── Info.plist                           # Top-level XCFramework metadata
├── ios-arm64/
│   ├── libmdns_peer.a                   # Device library
│   └── Info.plist                       # Device metadata
└── ios-arm64-simulator/
    ├── libmdns_peer.a                   # Simulator library
    └── Info.plist                       # Simulator metadata
```

## How It Works

The xtask pattern works through Cargo's `--bin` feature. When you run:

```bash
cargo xtask build-ios
```

Cargo:

1. Looks for a workspace member named `xtask`
2. Builds and runs its binary
3. Passes `build-ios` as an argument

The `xtask` binary is just a regular Rust program that uses `std::process::Command` to invoke cargo builds and file operations.

## Adding New Tasks

To add a new task:

1. Add a new function in `src/main.rs`
2. Add a match arm in `main()` to call your function
3. Document it in the help message

Example:

```rust
fn clean_ios() -> Result<()> {
    // Clean iOS builds
    todo!()
}

// In main():
match args[1].as_str() {
    "build-ios" => build_ios()?,
    "clean-ios" => clean_ios()?, // Add this
    _ => { /* ... */ }
}
```

## Further Reading

- [matklad's blog post on xtask](https://matklad.github.io/2018/01/03/make-your-own-make.html) (creator of rust-analyzer)
- [cargo-xtask documentation](https://github.com/matklad/cargo-xtask)
