//! `nuttx` is a Rust binding to the NuttX RTOS.
//! This crate provides a safe interface to the NuttX C API,
//! support both std and no_std environments.
//!

#![no_std]

// Private module for generated bindings - not exposed in public API
#[allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code
)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod input;
pub mod video;
