# NuttX Rust Bindings

Safe Rust bindings for the NuttX RTOS API, providing a Rust-idiomatic interface to interact with NuttX's native C API. These bindings support both `std` and `no_std` environments, making them suitable for resource-constrained embedded systems running NuttX.

## Project Goals

- Provide safe, idiomatic Rust APIs for NuttX system interfaces
- Support both `std` and `no_std` environments
- Maintain minimal runtime overhead compared to direct C API usage
- Follow Rust best practices for error handling and resource management

## Environment Setup

### Requirements

- Configured NuttX build directory

### Environment Variables

This crate requires the following environment variable to be set:

- `NUTTX_INCLUDE_DIR` - Path to your NuttX build directory containing the compiled NuttX code and headers

Example setup:

```bash
# Set the environment variable to your NuttX build directory
export NUTTX_INCLUDE_DIR=nuttx/include:nuttx/include/arch

# Build your Rust project
cargo build
```

## Features

**Input Devices**
  - Touchscreen

**Video**
  - Framebuffer access
  - Display information queries

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
```

## Safety

This crate uses `unsafe` blocks to interface with NuttX's C API. All public APIs are designed to be safe to use, with proper error handling and resource management through RAII patterns (like implementing `Drop` for resource cleanup).
