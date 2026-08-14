# iOS Platform

This directory contains iOS build configurations, plist definitions, and guidelines for DuoSnake.

## Prerequisites

1. **macOS & Xcode**:
   - Xcode 14+ installed with iOS SDK and command-line tools.
2. **Rust iOS Targets**:
   ```bash
   rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
   ```
3. **Build Tooling (`cargo-mobile2` or `cargo-dinghy` / Xcode project)**:
   ```bash
   cargo install cargo-mobile2
   # or
   cargo install cargo-dinghy
   ```

## Building for iOS

```bash
# Build static library for iOS device
cargo build --target aarch64-apple-ios --release --lib

# Or build for iOS simulator
cargo build --target aarch64-apple-ios-sim --release --lib
```
