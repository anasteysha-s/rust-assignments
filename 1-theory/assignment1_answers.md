# Assignment 1: Basic Theory - Answers

## 1. Is Rust single-threaded or multi-threaded? Is it synchronous or asynchronous?

Rust supports both single-threaded and multi-threaded programming. It is synchronous by default but has first-class support for asynchronous programming through async/await syntax and the Future trait. The choice between sync/async and single/multi-threaded is up to the developer.

## 2. What runtime Rust has? Does it use a GC (garbage collector)?

Rust has a minimal runtime with no garbage collector. Instead, it uses compile-time ownership and borrowing rules to manage memory safely without runtime overhead. The only runtime components are for stack unwinding (panic handling) and some basic startup code, making Rust suitable for systems programming and embedded environments.

## 3. What static typing means? What are the benefits of using it?

Static typing means that variable types are checked at compile time, not at runtime. Benefits include: catching type errors before the program runs, enabling better IDE support and refactoring tools, and allowing compiler optimizations that improve performance. Rust's type system also prevents many common bugs like null pointer dereferences and data races.

## 4. What is immutability? What is the benefit of using it?

Immutability means that once a value is bound to a variable, it cannot be changed. In Rust, variables are immutable by default (must use `mut` keyword for mutability). Benefits include: easier reasoning about code (values don't change unexpectedly), thread safety (immutable data can be safely shared across threads), and prevention of accidental modifications.

## 5. What are move semantics? What are borrowing rules? What is the benefit of using them?

Move semantics transfer ownership of a value from one variable to another, invalidating the original variable. Borrowing rules allow temporary access to values without transferring ownership, with restrictions: you can have either one mutable reference OR multiple immutable references, but not both simultaneously. These systems prevent data races, use-after-free bugs, and double-free errors at compile time without garbage collection overhead.

## 6. What are traits? How are they used? How do they compare to interfaces?

Traits define shared behavior across types, similar to interfaces in other languages. They specify method signatures that implementing types must provide. Compared to interfaces, traits can provide default implementations, support associated types, and can be implemented for types you didn't create. Traits enable polymorphism through both static dispatch (generics with trait bounds) and dynamic dispatch (trait objects).

## 7. What are lifetimes? Which problems do they solve?

Lifetimes are annotations that tell the compiler how long references are valid, ensuring references never outlive the data they point to. They solve the problem of dangling references (pointers to freed memory) at compile time without runtime overhead. Lifetimes are often inferred by the compiler, but sometimes need explicit annotations when relationships between input and output references are ambiguous.

## 8. What are macros? Which problems do they solve?

Macros are code that writes code (metaprogramming) and are expanded at compile time before type checking. They solve problems like reducing code duplication when generics aren't flexible enough, implementing domain-specific languages, and generating boilerplate code. Rust has declarative macros (`macro_rules!`) and procedural macros (derive macros, attribute macros, function-like macros).

## 9. What is the difference between `&String` and `&str` types (or between `&Vec` and `&[u8]` types)? Difference between fat and thin pointers?

`&String` is a reference to a String object (heap-allocated, owned), while `&str` is a string slice (reference to a sequence of bytes, doesn't own). `&str` is more flexible as it can reference any string data (String, string literals, substrings). Similarly, `&[u8]` is a slice (view into data) while `&Vec<u8>` is a reference to a Vec. Fat pointers (like `&str`, `&[T]`) contain both a data pointer and metadata (length); thin pointers (like `&String`, `&i32`) contain only a data pointer.

## 10. What are static and dynamic dispatches?

Static dispatch resolves which method to call at compile time using generics/monomorphization, resulting in fast, inlined code but larger binaries. Dynamic dispatch resolves method calls at runtime using vtables (trait objects like `Box<dyn Trait>`), allowing runtime polymorphism with slight overhead but smaller binary size. Static dispatch is faster; dynamic dispatch is more flexible when the concrete type isn't known at compile time.

