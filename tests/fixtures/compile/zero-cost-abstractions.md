---
title: "Zero-Cost Abstractions"
url: https://blog.rust-lang.org/2015/05/11/traits.html
collected_at: 2025-06-01T10:04:00Z
updated_at: 2025-06-01T10:04:00Z
---

# Zero-Cost Abstractions

A zero-cost abstraction is one where using the abstraction costs nothing extra at runtime compared to writing the equivalent low-level code by hand. C++ pioneered this idea: "What you don't use, you don't pay for." Rust adopts the same principle and extends it.

Rust's trait system provides polymorphism without virtual dispatch overhead when the concrete type is known at compile time. Generic functions are monomorphised: the compiler generates specialised code for each concrete type used, just as if the programmer had written separate functions.

Iterators are a canonical example. A chained iterator pipeline like `.map().filter().fold()` compiles to code equivalent to a hand-written loop — no heap allocation, no virtual calls, no overhead.

This makes Rust suitable for embedded and real-time systems where predictable performance is essential, while still allowing high-level expressive code in application-layer software.
