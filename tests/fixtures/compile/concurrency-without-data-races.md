---
title: "Concurrency Without Data Races"
url: https://doc.rust-lang.org/book/ch16-00-concurrency.html
collected_at: 2025-06-01T10:03:00Z
updated_at: 2025-06-01T10:03:00Z
---

# Concurrency Without Data Races

Rust's ownership model extends naturally to concurrency. The borrow checker prevents data races at compile time: if multiple threads could access the same data simultaneously, the code simply won't compile unless the access is properly synchronised.

The `Send` and `Sync` marker traits define which types are safe to transfer between threads and which are safe to share. The standard library's `Mutex`, `RwLock`, and `Arc` types build safe concurrent abstractions on top of these guarantees.

Fearless concurrency is one of Rust's core value propositions: you can write concurrent code with the confidence that entire classes of bugs (data races, deadlocks from shared mutable state) are statically eliminated. This stands in contrast to languages like C++ where concurrency bugs often only appear under specific timing conditions.

Async Rust extends these guarantees to cooperative multitasking, with futures and the async/await syntax enabling high-performance I/O-bound code.
