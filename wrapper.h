/**
 * NuttX Rust Bindings - Wrapper Header
 *
 * This header includes all the necessary NuttX C headers that should be
 * exposed to Rust through bindgen. The build script uses this file as the
 * entry point for generating Rust bindings to the NuttX C API.
 *
 * When adding new NuttX APIs to the Rust bindings, include the relevant
 * headers here.
 */

/* Touchscreen interface */
#include <nuttx/input/touchscreen.h>

/* Framebuffer interface */
#include <nuttx/video/fb.h>
