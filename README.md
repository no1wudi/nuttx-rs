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

## Binding Generation

This project uses `bindgen` to generate Rust FFI bindings to the NuttX C API. The bindings are generated from the C header files listed in the `wrapper.h` file:

```c
// wrapper.h - Entry point for bindgen
#include <nuttx/input/touchscreen.h>
#include <nuttx/video/fb.h>
// Additional headers are included here as needed
```

The build script uses this `wrapper.h` file as the entry point for generating the Rust bindings.

## Safety

This crate uses `unsafe` blocks to interface with NuttX's C API. All public APIs are designed to be safe to use, with proper error handling and resource management through RAII patterns (like implementing `Drop` for resource cleanup).

## Contributing

### Adding New Bindings

To add bindings for new NuttX APIs:

1. **Update the wrapper.h file**
   - Add the appropriate `#include` statements for the NuttX headers you want to expose
   - The wrapper.h file serves as the central point for controlling which C APIs get bindings

2. **Rebuild the project**
   - The build script will automatically regenerate the bindings

### Adding New Wrappers

To add a new wrapper for NuttX API functionality:

1. **Identify the NuttX API to wrap**
   - Find relevant headers in the NuttX codebase
   - Ensure these headers are included in `wrapper.h` to generate the raw bindings
   - Understand the API's purpose, behavior, and usage patterns

2. **Create module structure**
   - Add a new module file in the appropriate directory
   - Update the corresponding `mod.rs` to expose your new module

3. **Implement the wrapper**
   - Start with raw bindings from the `bindings` module
   - Create safe Rust APIs that encapsulate the unsafe C functions
   - Use Rust patterns for resource management (RAII, ownership)
   - Add proper error handling using `Result` types

4. **Documentation**
   - Add thorough documentation with examples
   - Include safety notes for any `unsafe` code
   - Reference the original NuttX headers/documentation

5. **Testing**
   - Add unit tests if possible
   - Ensure the wrapper works in both `std` and `no_std` environments

#### Example Wrapper Pattern

```rust
pub struct MyNuttxResource {
    handle: *mut bindings::nuttx_resource_t,
}

impl MyNuttxResource {
    pub fn new(config: &Config) -> Result<Self, i32> {
        // Convert Rust config to C config
        let c_config = convert_config(config);

        // Call NuttX API, handle errors
        let handle = unsafe { bindings::nuttx_resource_initialize(&c_config) };
        if handle.is_null() {
            return Err(-1); // Or get actual errno
        }

        Ok(Self { handle })
    }

    pub fn operation(&mut self, param: u32) -> Result<u32, i32> {
        let result = unsafe { bindings::nuttx_resource_operation(self.handle, param) };
        if result < 0 {
            return Err(result);
        }
        Ok(result as u32)
    }
}

impl Drop for MyNuttxResource {
    fn drop(&mut self) {
        unsafe { bindings::nuttx_resource_uninitialize(self.handle) };
    }
}
```
