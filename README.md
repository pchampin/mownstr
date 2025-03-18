MownStr: Maybe Owned String
===========================

[![Latest Version](https://img.shields.io/crates/v/mownstr.svg)](https://crates.io/crates/mownstr)
[![Documentation](https://docs.rs/mownstr/badge.svg)](https://docs.rs/mownstr/)
[![Actions Status](https://github.com/pchampin/mownstr/actions/workflows/lint_and_test.yml/badge.svg)](https://github.com/pchampin/mownstr/actions)

This crate provides `MownStr`,
a type for exchanging read-only strings that may be either borrowed or owned.

Contrarily to other types (such as for example [`Cow<str>`]),
`MownStr` does not aim to be mutable nor generic,
which allows it to be fast and lean.

Actually, a `MownStr` takes no more memory than a regular `&str` or `Box<str>`,
and has a little runtime overhead.
The drawback is that the maximum size of a `MownStr`
is half the size of a regular `str`
(which is still 8EiB on a 64-bit architectures...).

[`Cow<str>`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html

### Benchmark

To run the benchmark, you need the feature `bench`:

```
cargo bench --features bench
```

### Supported Rust version

This crate is tested with Rust 1.67 and with the latest stable release.
