//! Procedural macro for creating BTreeMap.
//!
//! Provides the `btreemap!` macro as a procedural macro alternative
//! to the declarative macro implementation.

#![warn(missing_docs)]
#![warn(broken_intra_doc_links)]
#![warn(missing_crate_level_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Result, Token,
};

/// A key-value pair in the btreemap macro.
///
/// Represents a single `key => value` pair.
struct KeyValuePair {
    key: Expr,
    _arrow: Token![=>],
    value: Expr,
}

impl Parse for KeyValuePair {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(KeyValuePair {
            key: input.parse()?,
            _arrow: input.parse()?,
            value: input.parse()?,
        })
    }
}

/// The input to the btreemap macro.
///
/// Contains a comma-separated list of key-value pairs.
struct BTreeMapInput {
    pairs: Punctuated<KeyValuePair, Token![,]>,
}

impl Parse for BTreeMapInput {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(BTreeMapInput {
            pairs: Punctuated::parse_terminated(input)?,
        })
    }
}

/// Create a `BTreeMap` from key-value pairs using procedural macro.
///
/// This is a procedural macro implementation of the `btreemap!` macro.
/// It provides the same functionality as the declarative version but
/// uses Rust's procedural macro system.
///
/// # Syntax
///
/// ```text
/// btreemap!()                           // Empty map
/// btreemap! { key => value }            // Single pair
/// btreemap! { key => value, }           // Single pair with trailing comma
/// btreemap! { k1 => v1, k2 => v2 }     // Multiple pairs
/// btreemap! { k1 => v1, k2 => v2, }    // Multiple pairs with trailing comma
/// ```
///
/// # Examples
///
/// ```
/// use btreemap_proc_macro::btreemap;
/// use std::collections::BTreeMap;
///
/// // Empty map
/// let map: BTreeMap<i32, &str> = btreemap!();
/// assert!(map.is_empty());
///
/// // With key-value pairs
/// let map = btreemap! {
///     1 => "one",
///     2 => "two",
///     3 => "three",
/// };
/// assert_eq!(map.len(), 3);
/// assert_eq!(map.get(&2), Some(&"two"));
/// ```
///
/// # Implementation Details
///
/// The macro expands to code that:
/// 1. Creates a new `BTreeMap`
/// 2. Inserts each key-value pair
/// 3. Returns the map
///
/// For example:
/// ```ignore
/// btreemap! { 1 => "a", 2 => "b" }
/// ```
///
/// Expands to:
/// ```ignore
/// {
///     let mut map = ::std::collections::BTreeMap::new();
///     map.insert(1, "a");
///     map.insert(2, "b");
///     map
/// }
/// ```
#[proc_macro]
pub fn btreemap(input: TokenStream) -> TokenStream {
    // Parse the input
    let input = parse_macro_input!(input as BTreeMapInput);

    // Extract keys and values
    let keys = input.pairs.iter().map(|pair| &pair.key);
    let values = input.pairs.iter().map(|pair| &pair.value);

    // Generate the code
    let expanded = quote! {
        {
            let mut map = ::std::collections::BTreeMap::new();
            #(
                map.insert(#keys, #values);
            )*
            map
        }
    };

    TokenStream::from(expanded)
}