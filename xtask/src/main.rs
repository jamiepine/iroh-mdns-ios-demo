//! Build automation tasks for iroh-mdns-test
//!
//! This crate provides build tasks using the xtask pattern - a Rust-native
//! approach to build automation. No bash scripts or external tools needed.
//!
//! ## Usage
//!
//! ```bash
//! cargo xtask build-ios    # Build iOS framework
//! ```
//!
//! ## About xtask
//!
//! The xtask pattern is the idiomatic Rust way to handle build automation.
//! It's just a regular Rust binary in your workspace that you invoke via
//! `cargo xtask <command>`. This approach is used by major projects like
//! rust-analyzer, tokio, and many others.
//!
//! Benefits:
//! - Pure Rust - no shell scripts to maintain
//! - Type-safe and easy to debug
//! - Cross-platform by default
//! - No external tools required

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo xtask <command>");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  build-ios    Build mdns-peer for iOS devices and simulator");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  cargo xtask build-ios");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "build-ios" => build_ios()?,
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Run 'cargo xtask' for usage information.");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Build mdns-peer for iOS devices and simulator, creating an XCFramework
///
/// This task:
/// 1. Builds for aarch64-apple-ios (physical devices)
/// 2. Builds for aarch64-apple-ios-sim (simulator)
/// 3. Creates the XCFramework directory structure
/// 4. Copies the static libraries to the correct locations
///
/// The resulting XCFramework can be imported into Xcode projects.
fn build_ios() -> Result<()> {
    println!("🔨 Building mdns-peer for iOS...");
    println!();

    // Target triple and corresponding XCFramework architecture directory
    let targets = [
        ("aarch64-apple-ios", "ios-arm64"),
        ("aarch64-apple-ios-sim", "ios-arm64-simulator"),
    ];

    // Build for each target
    for (target, arch) in &targets {
        println!("📦 Building for {} ({})...", arch, target);

        let status = Command::new("cargo")
            .args(&["build", "--release", "--target", target, "-p", "mdns-peer"])
            .env("IPHONEOS_DEPLOYMENT_TARGET", "14.0")
            .status()
            .context(format!("Failed to build for {}", target))?;

        if !status.success() {
            anyhow::bail!("Build failed for target: {}", target);
        }
        println!("   ✓ Built successfully");
    }

    // Create XCFramework directory structure
    println!();
    println!("📁 Creating XCFramework structure...");
    let xcframework_path = Path::new("mdns-peer/mdns_peer.xcframework");
    let framework_name = "libmdns_peer";

    // Platform mapping for Info.plist
    let platform_map = [
        ("ios-arm64", "iPhoneOS"),
        ("ios-arm64-simulator", "iPhoneSimulator"),
    ];

    for ((target, arch), (_, platform)) in targets.iter().zip(platform_map.iter()) {
        let arch_dir = xcframework_path.join(arch);
        std::fs::create_dir_all(&arch_dir)
            .context(format!("Failed to create directory for {}", arch))?;

        // Copy static library
        let src = format!("target/{}/release/libmdns_peer.a", target);
        let dst = arch_dir.join("libmdns_peer.a");
        std::fs::copy(&src, &dst).context(format!("Failed to copy library for {}", arch))?;

        // Create Info.plist for this architecture
        let info_plist = create_architecture_info_plist(framework_name, platform);
        let plist_path = arch_dir.join("Info.plist");
        std::fs::write(&plist_path, info_plist)
            .context(format!("Failed to write Info.plist for {}", arch))?;

        println!("   ✓ Created {} with library and Info.plist", arch);
    }

    // Create top-level XCFramework Info.plist
    let xcframework_info_plist = create_xcframework_info_plist(framework_name);
    let xcframework_plist_path = xcframework_path.join("Info.plist");
    std::fs::write(&xcframework_plist_path, xcframework_info_plist)
        .context("Failed to write XCFramework Info.plist")?;
    println!("   ✓ Created XCFramework Info.plist");

    // Success message with next steps
    println!();
    println!("✅ iOS framework built successfully!");
    println!();
    println!("📍 XCFramework location:");
    println!("   {}", xcframework_path.display());
    println!();
    println!("📝 Next steps:");
    println!("   1. Open MdnsTest/MdnsTest.xcodeproj in Xcode");
    println!("   2. The framework reference should already be configured");
    println!("   3. Build and run on simulator or device");
    println!();

    Ok(())
}

/// Generate an Info.plist file for each architecture in the XCFramework
///
/// Each architecture directory needs its own Info.plist that describes
/// the framework metadata including bundle identifier, version, and platform.
fn create_architecture_info_plist(framework_name: &str, platform: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>{}</string>
    <key>CFBundleIdentifier</key>
    <string>com.spacedrive.mdns-peer</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundlePackageType</key>
    <string>FMWK</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>CFBundleSupportedPlatforms</key>
    <array>
        <string>{}</string>
    </array>
    <key>MinimumOSVersion</key>
    <string>14.0</string>
</dict>
</plist>
"#,
        framework_name, framework_name, platform
    )
}

/// Generate the top-level Info.plist for the XCFramework
///
/// This describes the XCFramework structure and lists all available libraries
/// for different platforms and architectures.
fn create_xcframework_info_plist(framework_name: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>AvailableLibraries</key>
    <array>
        <dict>
            <key>LibraryIdentifier</key>
            <string>ios-arm64</string>
            <key>LibraryPath</key>
            <string>{}.a</string>
            <key>SupportedArchitectures</key>
            <array>
                <string>arm64</string>
            </array>
            <key>SupportedPlatform</key>
            <string>ios</string>
        </dict>
        <dict>
            <key>LibraryIdentifier</key>
            <string>ios-arm64-simulator</string>
            <key>LibraryPath</key>
            <string>{}.a</string>
            <key>SupportedArchitectures</key>
            <array>
                <string>arm64</string>
            </array>
            <key>SupportedPlatform</key>
            <string>ios</string>
            <key>SupportedPlatformVariant</key>
            <string>simulator</string>
        </dict>
    </array>
    <key>CFBundlePackageType</key>
    <string>XFWK</string>
    <key>XCFrameworkFormatVersion</key>
    <string>1.0</string>
</dict>
</plist>
"#,
        framework_name, framework_name
    )
}
