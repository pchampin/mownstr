MownStr: Maybe Owned String
===========================

[![Latest Version](https://img.shields.io/crates/v/mownstr.svg)](https://crates.io/crates/mownstr)
[![Documentation](https://docs.rs/mownstr/badge.svg)](https://docs.rs/mownstr/)
[![Actions Status](https://github.com/pchampin/mownstr/actions/workflows/lint_and_test.yml/badge.svg)](https://github.com/pchampin/mownstr/actions)

This crate provides `MownStr`,
a type for storing and exchanging read-only strings that may be either borrowed or owned.

Compared to [`Cow<str>`]:
* `MownStr` is 2/3 smaller,
* `MownStr` is slightly slower to construct and deference to a `str`.

So using `MownStr` makes sense if you need to store a lot of them.
Otherwise, [`Cow<str>`] may be a better option.

Note also that `MownStr` can not represent strings longer than `usize::MAX`/2.
This theoretical limitation is not an issue in practice, because
* it still allows for huge string, e.g. 8EiB on a 64-bit architectures;
* it is not even clear that Rust supports strings bigger than that,
  as [`from_raw_parts`] limits the size of a slice to `isize::MAX`,
  not `usize::MAX` (despite sizes being typed as `usize`).

[`Cow<str>`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html
[`from_raw_parts`]: https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html

### Benchmark

To run the benchmark, you need the feature `criterion`:

```
cargo bench --features criterion
```

### Supported Rust version

This crate is tested with Rust 1.67 and with the latest stable release.
