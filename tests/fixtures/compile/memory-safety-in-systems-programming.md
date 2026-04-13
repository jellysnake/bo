---
title: "Memory Safety in Systems Programming"
url: https://research.mozilla.org/2019/02/01/memory-safety-in-firefox/
collected_at: 2025-06-01T10:01:00Z
updated_at: 2025-06-01T10:01:00Z
---

# Memory Safety in Systems Programming

Memory safety bugs — use-after-free, buffer overflows, null pointer dereferences — account for the majority of serious security vulnerabilities in systems software. Languages like C and C++ leave memory management entirely to the programmer, making these bugs easy to introduce and hard to find.

Rust addresses this through its ownership and borrow checker, which statically rules out an entire class of memory errors at compile time. Unlike garbage-collected languages, Rust achieves memory safety without a runtime performance penalty.

Other approaches to memory safety include AddressSanitizer (runtime detection), software fault isolation, and formal verification. Each trades off between performance, developer ergonomics, and the completeness of guarantees provided.

The industry shift toward memory-safe languages is accelerating, with major organisations recommending against C and C++ for new systems software.
