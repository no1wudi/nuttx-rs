//! # Kconfig - Conditional compilation for NuttX Rust code
//!
//! This crate provides procedural macros that enable conditional compilation based on NuttX
//! Kconfig options. It works by examining Kconfig bindings at compile time to determine if
//! specified conditions are met.
//!
//! ## Usage
//!
//! Use the `#[kconfig]` attribute macro to conditionally include or exclude Rust items:
//!
//! ```rust
//! use kconfig::kconfig;
//!
//! #[kconfig(CONFIG_FEATURE_X = "y")]
//! fn feature_x_implementation() {
//!     // This function will only be compiled when CONFIG_FEATURE_X is enabled
//! }
//!
//! #[kconfig(CONFIG_FEATURE_X = "y", CONFIG_DEBUG = "n")]
//! struct FeatureImplementation {
//!     // This struct will only be included when CONFIG_FEATURE_X is enabled
//!     // and CONFIG_DEBUG is disabled
//! }
//! ```
//!
//! ## How it works
//!
//! The macro processes Kconfig bindings that are generated during the NuttX build process.
//! These bindings contain Rust constants that represent the values of Kconfig options.
//! The macro checks these constants to determine if the specified conditions are met.

use proc_macro::TokenStream;
use quote::quote;
use std::fs;
use syn::{
    Expr, File, Ident, Item, ItemConst, Lit, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_file, parse_macro_input,
    punctuated::Punctuated,
};

/// Represents a single Kconfig option in the attribute macro.
///
/// Each option consists of a name (identifier) and a value (string literal).
/// For example, in `#[kconfig(CONFIG_FEATURE_X = "y")]`, `CONFIG_FEATURE_X` is the name
/// and `"y"` is the value.
struct KconfigOption {
    /// The name of the Kconfig option (e.g., `CONFIG_FEATURE_X`)
    name: Ident,
    /// The expected value of the option, either `"y"` or `"n"`
    value: LitStr,
}

/// Implementation for parsing a single Kconfig option from a token stream.
///
/// Parses a key-value pair in the form `name = "value"` where:
/// - `name` is a valid Rust identifier
/// - `value` is a string literal
impl Parse for KconfigOption {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let _: Token![=] = input.parse()?; // Parse but don't store equals
        let value = input.parse()?;
        Ok(KconfigOption { name, value })
    }
}

/// Represents all options in a `#[kconfig(...)]` attribute.
///
/// Contains a punctuated sequence of `KconfigOption` items, separated by commas.
/// For example, in `#[kconfig(CONFIG_A = "y", CONFIG_B = "n")]`, the options are
/// `CONFIG_A = "y"` and `CONFIG_B = "n"`.
struct KconfigAttr {
    /// A comma-separated list of Kconfig options
    options: Punctuated<KconfigOption, Token![,]>,
}

/// Implementation for parsing a comma-separated list of Kconfig options.
impl Parse for KconfigAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let options = Punctuated::parse_terminated(input)?;
        Ok(KconfigAttr { options })
    }
}

/// Fetches and parses the Rust bindings file generated from NuttX Kconfig options.
///
/// This function:
/// 1. Retrieves the output directory path from the `OUT_DIR` environment variable
/// 2. Constructs the path to the bindings.rs file
/// 3. Reads the file contents
/// 4. Parses the file into a Rust AST (Abstract Syntax Tree)
///
/// # Returns
///
/// Returns a `syn::Result<File>` containing the parsed AST if successful
///
/// # Errors
///
/// This function will return an error if:
/// - The `OUT_DIR` environment variable is not set
/// - The bindings file cannot be found or read
/// - The bindings file cannot be parsed as valid Rust code
///
/// When an error occurs, it provides descriptive error messages to aid debugging.
fn fetch_bindings_ast() -> syn::Result<File> {
    // Get the output directory from the environment variable
    // This is set by Cargo when building and contains build artifacts
    let output_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");

    // Convert the string path to a PathBuf for easier manipulation
    let output_path = std::path::PathBuf::from(output_dir);

    // Construct the full path to the bindings.rs file
    // This file contains Rust constants generated from NuttX Kconfig options
    let bindings_path = output_path.join("bindings.rs");

    // Read the bindings file into a string
    // Return a descriptive error if the file cannot be read
    let bindings_source = fs::read_to_string(&bindings_path).map_err(|error| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!(
                "Bindings file not found or not readable at {}: {}",
                bindings_path.display(),
                error
            ),
        )
    })?;

    // Parse the file contents into a Rust AST (Abstract Syntax Tree)
    // This allows us to inspect the constants and their values programmatically
    parse_file(&bindings_source).map_err(|error| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("Failed to parse bindings file: {}", error),
        )
    })
}

/// Helper function to find a specific Kconfig option in the bindings AST.
///
/// # Arguments
///
/// * `bindings_ast` - The parsed AST of the bindings file
/// * `option_name` - The name of the Kconfig option to look for (e.g., "CONFIG_FEATURE_X")
///
/// # Returns
///
/// Returns a reference to the constant item if found, otherwise `None`.
fn find_kconfig_option<'a>(bindings_ast: &'a File, option_name: &str) -> Option<&'a ItemConst> {
    for item in &bindings_ast.items {
        if let Item::Const(const_item) = item {
            if const_item.ident.to_string() == option_name {
                return Some(const_item);
            }
        }
    }
    None
}

/// Conditionally includes or excludes Rust items based on NuttX Kconfig options.
///
/// This attribute macro enables conditional compilation based on the values of NuttX Kconfig
/// options. Items decorated with this attribute will only be included in the final binary if
/// all specified Kconfig conditions are met.
///
/// # Parameters
///
/// The macro accepts a comma-separated list of key-value pairs where:
/// - The key is the name of a Kconfig option
/// - The value can be either:
///   - `"y"`: The option must be enabled (set to 1)
///   - `"n"`: The option must be disabled or undefined
///
/// # Examples
///
/// Include a function only when `CONFIG_FEATURE_X` is enabled:
/// ```rust
/// #[kconfig(CONFIG_FEATURE_X = "y")]
/// fn feature_x_implementation() {
///     // This function will only be compiled when CONFIG_FEATURE_X is enabled
/// }
/// ```
///
/// Include a struct only when multiple conditions are met:
/// ```rust
/// #[kconfig(CONFIG_FEATURE_X = "y", CONFIG_DEBUG = "n")]
/// struct FeatureImplementation {
///     // This struct will only be included when:
///     // - CONFIG_FEATURE_X is enabled AND
///     // - CONFIG_DEBUG is disabled or undefined
/// }
/// ```
///
/// # How it works
///
/// The macro examines the generated Kconfig bindings at compile time to determine
/// if the specified conditions are met. It works by checking if the constants with
/// matching names exist in the bindings and if their values match the expected values.
#[proc_macro_attribute]
pub fn kconfig(attr: TokenStream, items: TokenStream) -> TokenStream {
    let kconfig_attr = parse_macro_input!(attr as KconfigAttr);
    let target_item = parse_macro_input!(items as Item);

    // Fetch the bindings AST
    let bindings_ast = match fetch_bindings_ast() {
        Ok(ast) => ast,
        Err(error) => return error.to_compile_error().into(),
    };

    let mut include_item = true;

    for config_option in &kconfig_attr.options {
        let option_name = config_option.name.to_string();
        let expected_value = config_option.value.value();

        // First, check if the option exists in the bindings
        if let Some(const_item) = find_kconfig_option(&bindings_ast, &option_name) {
            // Option exists, now check if its value matches
            if expected_value == "n" {
                // If option exists but required value is "n", condition fails
                include_item = false;
                break;
            }

            // Check if option value matches the value of the const
            if let Expr::Lit(expr_lit) = const_item.expr.as_ref() {
                if let Lit::Int(lit_int) = &expr_lit.lit {
                    // Parse the integer literal
                    let actual_value = lit_int.base10_parse::<i64>().unwrap();

                    if expected_value == "y" && actual_value == 1 {
                        // Option matched, continue checking other options
                    } else {
                        // Option value doesn't match
                        include_item = false;
                        break;
                    }
                }
            }
        } else {
            // Option doesn't exist in bindings
            if expected_value != "n" {
                // If we expected the option to be set (not "n"), but it doesn't exist, condition fails
                include_item = false;
                break;
            }
        }
    }

    if include_item {
        quote! { #target_item }.into()
    } else {
        quote! {}.into()
    }
}
