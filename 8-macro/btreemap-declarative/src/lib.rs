//! Declarative macro for creating BTreeMap.
//!
//! Provides the `btreemap!` macro for ergonomic BTreeMap creation,
//! similar to how `vec!` works for vectors.
//!
//! # Examples
//!
//! ```
//! use btreemap_macro::btreemap;
//! use std::collections::BTreeMap;
//!
//! // Empty map
//! let map: BTreeMap<i32, &str> = btreemap!();
//! assert!(map.is_empty());
//!
//! // With key-value pairs
//! let map = btreemap! {
//!     1 => "one",
//!     2 => "two",
//!     3 => "three",
//! };
//! assert_eq!(map.len(), 3);
//! assert_eq!(map.get(&2), Some(&"two"));
//! ```

#![warn(missing_docs)]
#![warn(broken_intra_doc_links)]
#![warn(missing_crate_level_docs)]

/// Create a `BTreeMap` from key-value pairs.
///
/// This macro provides a convenient way to create and initialize a `BTreeMap`
/// with a syntax similar to map literals in other languages.
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
/// ## Empty Map
///
/// ```
/// use btreemap_macro::btreemap;
/// use std::collections::BTreeMap;
///
/// let map: BTreeMap<i32, &str> = btreemap!();
/// assert!(map.is_empty());
/// ```
///
/// ## Single Pair
///
/// ```
/// use btreemap_macro::btreemap;
///
/// let map = btreemap! {
///     "key" => "value"
/// };
/// assert_eq!(map.get("key"), Some(&"value"));
/// ```
///
/// ## Multiple Pairs
///
/// ```
/// use btreemap_macro::btreemap;
///
/// let map = btreemap! {
///     1 => "one",
///     2 => "two",
///     3 => "three",
/// };
/// assert_eq!(map.len(), 3);
/// assert_eq!(map.get(&1), Some(&"one"));
/// assert_eq!(map.get(&2), Some(&"two"));
/// assert_eq!(map.get(&3), Some(&"three"));
/// ```
///
/// ## With Expressions
///
/// ```
/// use btreemap_macro::btreemap;
///
/// let x = 10;
/// let map = btreemap! {
///     x + 1 => x * 2,
///     x + 2 => x * 3,
/// };
/// assert_eq!(map.get(&11), Some(&20));
/// assert_eq!(map.get(&12), Some(&30));
/// ```
///
/// ## Sorted Order
///
/// BTreeMap maintains sorted order by keys:
///
/// ```
/// use btreemap_macro::btreemap;
///
/// let map = btreemap! {
///     5 => "five",
///     2 => "two",
///     8 => "eight",
///     1 => "one",
/// };
///
/// let keys: Vec<_> = map.keys().copied().collect();
/// assert_eq!(keys, vec![1, 2, 5, 8]);
/// ```
#[macro_export]
macro_rules! btreemap {
    // Empty map: btreemap!()
    () => {
        ::std::collections::BTreeMap::new()
    };

    // Single pair without trailing comma: btreemap! { key => value }
    ($key:expr => $value:expr) => {
        {
            let mut map = ::std::collections::BTreeMap::new();
            map.insert($key, $value);
            map
        }
    };

    // Single pair with trailing comma: btreemap! { key => value, }
    ($key:expr => $value:expr,) => {
        btreemap!($key => $value)
    };

    // Multiple pairs: btreemap! { k1 => v1, k2 => v2, ... }
    // The $(,)? at the end allows optional trailing comma
    ($($key:expr => $value:expr),+ $(,)?) => {
        {
            let mut map = ::std::collections::BTreeMap::new();
            $(
                map.insert($key, $value);
            )+
            map
        }
    };
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    #[test]
    fn test_empty_map() {
        let map: BTreeMap<i32, &str> = btreemap!();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_single_pair() {
        let map = btreemap! {
            1 => "one"
        };
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&1), Some(&"one"));
    }

    #[test]
    fn test_single_pair_trailing_comma() {
        let map = btreemap! {
            1 => "one",
        };
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&1), Some(&"one"));
    }

    #[test]
    fn test_multiple_pairs() {
        let map = btreemap! {
            1 => "one",
            2 => "two",
            3 => "three"
        };
        assert_eq!(map.len(), 3);
        assert_eq!(map.get(&1), Some(&"one"));
        assert_eq!(map.get(&2), Some(&"two"));
        assert_eq!(map.get(&3), Some(&"three"));
    }

    #[test]
    fn test_multiple_pairs_trailing_comma() {
        let map = btreemap! {
            1 => "one",
            2 => "two",
            3 => "three",
        };
        assert_eq!(map.len(), 3);
        assert_eq!(map.get(&3), Some(&"three"));
    }

    #[test]
    fn test_string_keys() {
        let map = btreemap! {
            "a" => 1,
            "b" => 2,
            "c" => 3
        };
        assert_eq!(map.len(), 3);
        assert_eq!(map.get("a"), Some(&1));
        assert_eq!(map.get("b"), Some(&2));
        assert_eq!(map.get("c"), Some(&3));
    }

    #[test]
    fn test_string_values() {
        let map = btreemap! {
            1 => "one".to_string(),
            2 => "two".to_string(),
            3 => "three".to_string()
        };
        assert_eq!(map.len(), 3);
        assert_eq!(map.get(&1), Some(&"one".to_string()));
    }

    #[test]
    fn test_owned_string_keys() {
        let map = btreemap! {
            "apple".to_string() => 1,
            "banana".to_string() => 2,
            "cherry".to_string() => 3
        };
        assert_eq!(map.len(), 3);
        assert_eq!(map.get(&"apple".to_string()), Some(&1));
    }

    #[test]
    fn test_expressions_as_keys_and_values() {
        let x = 10;
        let y = 20;
        let map = btreemap! {
            x + 1 => y * 2,
            x + 2 => y * 3,
            x + 3 => y * 4
        };
        assert_eq!(map.get(&11), Some(&40));
        assert_eq!(map.get(&12), Some(&60));
        assert_eq!(map.get(&13), Some(&80));
    }

    #[test]
    fn test_sorted_order() {
        let map = btreemap! {
            5 => "five",
            2 => "two",
            8 => "eight",
            1 => "one",
            9 => "nine"
        };

        let keys: Vec<_> = map.keys().copied().collect();
        assert_eq!(keys, vec![1, 2, 5, 8, 9]);
    }

    #[test]
    fn test_duplicate_keys_last_wins() {
        let map = btreemap! {
            1 => "first",
            1 => "second",
            1 => "third"
        };
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&1), Some(&"third"));
    }

    #[test]
    fn test_complex_types() {
        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
        struct Point {
            x: i32,
            y: i32,
        }

        let map = btreemap! {
            Point { x: 0, y: 0 } => "origin",
            Point { x: 1, y: 0 } => "right",
            Point { x: 0, y: 1 } => "up"
        };

        assert_eq!(map.get(&Point { x: 0, y: 0 }), Some(&"origin"));
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_nested_maps() {
        let inner1 = btreemap! {
            "a" => 1,
            "b" => 2
        };

        let inner2 = btreemap! {
            "c" => 3,
            "d" => 4
        };

        let outer = btreemap! {
            "first" => inner1,
            "second" => inner2
        };

        assert_eq!(outer.len(), 2);
        assert_eq!(outer.get("first").unwrap().get("a"), Some(&1));
        assert_eq!(outer.get("second").unwrap().get("c"), Some(&3));
    }

    #[test]
    fn test_variable_interpolation() {
        let key1 = "key1";
        let value1 = 100;
        let key2 = "key2";
        let value2 = 200;

        let map = btreemap! {
            key1 => value1,
            key2 => value2
        };

        assert_eq!(map.get(&"key1"), Some(&100));
        assert_eq!(map.get(&"key2"), Some(&200));
    }

    #[test]
    fn test_function_calls() {
        fn get_key() -> i32 {
            42
        }
        fn get_value() -> &'static str {
            "answer"
        }

        let map = btreemap! {
            get_key() => get_value(),
            100 => "century"
        };

        assert_eq!(map.get(&42), Some(&"answer"));
        assert_eq!(map.get(&100), Some(&"century"));
    }

    #[test]
    fn test_mixed_types() {
        let map = btreemap! {
            String::from("owned") => 1,
            String::from("string") => 2,
            String::from("keys") => 3
        };

        assert_eq!(map.get(&String::from("owned")), Some(&1));
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_capacity() {
        let map = btreemap! {
            1 => "a",
            2 => "b",
            3 => "c",
            4 => "d",
            5 => "e"
        };

        assert!(map.len() == 5);
        assert!(map.get(&3).is_some());
    }
}