---
title: "Rust Ownership and Borrowing"
url: https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html
collected_at: 2025-06-01T10:00:00Z
updated_at: 2025-06-01T10:00:00Z
---

# Rust Ownership and Borrowing

Rust's ownership system is its most distinctive feature. Every value in Rust has a single owner, and when that owner goes out of scope, the value is dropped. This eliminates the need for a garbage collector while preventing memory leaks.

The borrow checker enforces these rules at compile time: you can have either one mutable reference or any number of immutable references to a piece of data, but not both simultaneously. This guarantees memory safety without runtime overhead.

Ownership also enables Rust's zero-cost abstractions: types like `String`, `Vec`, and `Box` are stack-allocated wrappers around heap data, and the compiler knows exactly when to free memory.

The lifetime system extends ownership to references that cross function boundaries. Lifetimes ensure that references never outlive the data they point to, preventing dangling pointers entirely.
