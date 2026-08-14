# Android Platform

This directory contains Android build configurations, manifest, and guidelines for DuoSnake.

## Prerequisites

1. **Android SDK & NDK**:
   - Set environment variables `ANDROID_SDK_ROOT` (or `ANDROID_HOME`) and `NDK_HOME`.
2. **Rust Android Targets**:
   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
   ```
3. **Build Tooling (`cargo-apk` or `xbuild`)**:
   ```bash
   cargo install cargo-apk
   # or
   cargo install xbuild
   ```

## Building APK

```bash
# Build debug APK using cargo-apk
cargo apk build --manifest-path platforms/android/Cargo.toml (or via root)

# Or run directly on connected Android device / emulator
cargo apk run
```
