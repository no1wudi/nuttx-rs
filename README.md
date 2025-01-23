# NuttX Rust Bindings

Safe Rust bindings for the NuttX RTOS API, supporting both `std` and `no_std` environments.

## Features

**Important Note:** The optional features must match the corresponding configuration in your NuttX build. For example, enabling `fb_overlay` requires `CONFIG_FB_OVERLAY=y` in your NuttX configuration.

**Input Devices**
  - Touchscreen

**Video**
  - Framebuffer
  - Display information
  - Optional features:
    - FB overlay support (`fb_overlay`)
    - FB module info support (`fb_moduleinfo`)
    - FB update support (`fb_update`)
    - FB sync support (`fb_sync`)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
nuttx = "0.1"
```

Enable optional features as needed:

```toml
[dependencies.nuttx]
version = "0.1"
features = ["fb_overlay", "fb_update"]
```

## Safety

This crate uses `unsafe` blocks to interface with NuttX's C API. All public APIs are designed to be safe to use, with proper error handling and resource management.
