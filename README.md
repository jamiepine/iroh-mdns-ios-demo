# iroh mDNS Discovery iOS Test

Minimal test project to investigate mDNS discovery behavior between desktop and iOS platforms using iroh.

## Project Goal

Test mDNS-based peer discovery between desktop and iOS using iroh's `swarm-discovery` implementation.

Peers advertise themselves with `user_data` to enable definitive identification.

## Project Structure

```
iroh-mdns-test/
├── mdns-peer/      # Shared implementation (CLI binary + iOS library)
├── MdnsTest/       # iOS app (Xcode project)
├── xtask/          # Build automation tasks (Rust-native)
└── Cargo.toml      # Workspace configuration
```

The `mdns-peer` crate provides both:

- A CLI binary that accepts a peer identifier as argument
- An iOS static library that can be embedded in Swift apps

## Requirements

- Rust toolchain with iOS targets:
  ```bash
  rustup target add aarch64-apple-ios aarch64-apple-ios-sim
  ```
- Xcode with iOS SDK

## Building

This project uses the **xtask** pattern for building - a Rust-native approach to build tasks.

### Desktop Binary

```bash
cargo build --bin mdns-peer
```

### iOS Framework

Build for both iOS device and simulator in one command:

```bash
cargo xtask build-ios
# Or use the convenient alias:
cargo ios
```

Or both together for a fresh test

```bash
cargo build --bin mdns-peer && cargo ios
```

This will:

1. Build `mdns-peer` for `aarch64-apple-ios` (device)
2. Build `mdns-peer` for `aarch64-apple-ios-sim` (simulator)
3. Create the `mdns_peer.xcframework` directory structure
4. Copy the static libraries to the appropriate locations
5. Generate all required Info.plist files (XCFramework and per-architecture)

The XCFramework will be available at `mdns-peer/mdns_peer.xcframework/` for use in Xcode.

**About xtask:** The `xtask` crate is a workspace member that provides build tasks as a Rust binary. This is the idiomatic Rust way to handle build automation - no bash scripts, no external tools like `make`, just pure Rust. See `xtask/README.md` for more details on the xtask pattern.

## Running the Test

### Desktop Peers

```bash
# Terminal 1 - Run as "alice"
cargo run --bin mdns-peer alice

# Terminal 2 - Run as "bob"
cargo run --bin mdns-peer bob
```

Each peer will advertise itself with its identifier and report discoveries.

### iOS Peer

Open `MdnsTest/MdnsTest.xcodeproj` in Xcode and run on simulator or device.

The iOS app calls `bob_start()` which internally uses identifier "bob".

## Logging Configuration

Control logging verbosity using the `RUST_LOG` environment variable. The default configuration is:

```bash
RUST_LOG="mdns_peer=info,swarm_discovery=debug,iroh=info"
```

### Logging Presets

**Verbose (see all mDNS activity):**

```bash
RUST_LOG="mdns_peer=debug,swarm_discovery=debug,iroh=debug" cargo run --bin mdns-peer alice
```

**Quiet (errors and warnings only):**

```bash
RUST_LOG="mdns_peer=warn,swarm_discovery=warn,iroh=warn" cargo run --bin mdns-peer alice
```

**Minimal (just peer discoveries):**

```bash
RUST_LOG="mdns_peer=info,swarm_discovery=warn,iroh=warn" cargo run --bin mdns-peer alice
```

**Everything (maximum verbosity):**

```bash
RUST_LOG="trace" cargo run --bin mdns-peer alice
```

### What to Watch For

- `swarm_discovery=debug` - Shows mDNS queries, responses, and socket operations
- `mdns_peer=info` - Shows peer discoveries and routing table updates
- `iroh=debug` - Shows endpoint and network operations

**Note:** iOS logging is configured at compile time and always uses the default filter. To change iOS logging, modify `mdns-peer/src/lib.rs` and rebuild with `cargo ios`.

## Expected Behavior

### Success Case

When peers discover each other, you'll see:

```
Peer discovered:
  Node ID: a8a2977385602aa4732868253fd0383678b37841d1bced7596456c60c768bab5
  User data: Some(UserData("alice"))
  Source: mdns
[[[ SUCCESS ]]]: Discovered peer 'alice'!
```

The `user_data` field provides definitive peer identification.

## Known Issues

### iOS Cannot Send Multicast (errno 65)

**Symptom:** iOS peer receives mDNS queries but cannot send responses:

```
WARN swarm_discovery::socket: error sending mDNS: No route to host (os error 65)
```

**Result:** One-way discovery only. iOS (Bob) successfully receives and discovers desktop (Alice), but fails to send mDNS responses (errno 65). Desktop (Alice) never discovers iOS peer, even with correct iOS entitlements and granted permissions

**Potential Cause:** The `swarm-discovery` crate does not properly configure multicast sockets for iOS. iOS requires:

- Explicit network interface binding
- `SO_REUSEPORT` socket option
- `IP_MULTICAST_IF` set correctly

**Status:** This is potentially a limitation of `swarm-discovery` on iOS, not a protocol compatibility issue.

## Testing Notes

- **Simulator:** Works both ways, nodes discover each other.
- **Physical Device:** One-way discovery only. iOS (Bob) successfully receives and discovers desktop (Alice), but fails to send mDNS responses (errno 65). Desktop (Alice) never discovers iOS peer, even with correct iOS entitlements and granted permissions
- **Info.plist:** Requires `NSLocalNetworkUsageDescription` and `NSBonjourServices` (already configured)

## Dependencies

This test uses a local iroh repository:

- Path: `../iroh/iroh`
- Feature: `discovery-local-network`

## License

Same as iroh project (MIT/Apache-2.0)
