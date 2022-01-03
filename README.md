MownStr: Maybe Owned String
===========================

This crate provides `MownStr`,
a type for exchanging read-only strings that may be either borrowed or owned.

Contrarily to other types (such as for example [`Cow<str>`]),
`MownStr` does not aim to be mutable nor generic,
which allows it to be fast and lean.

Actually, a `MownStr` takes no more memory than a regular `&str` or `Box<str>`,
and has a minimal runtime overhead.
The drawback is that the maximum size of a `MownStr`
is half the size of a regular `str`
(which is still 8EiB on a 64-bit architectures...).

[`Cow<str>`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html

Supported Rust version
----------------------

This crate targets the stable channel of `rustc` and `cargo`. I do not aim for compatibility with older versions, so new releases of `mownstr` may require you to upgrade your Rust toolchain.
