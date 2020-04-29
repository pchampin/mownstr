MownStr: Maybe Owned String
===========================

This crate provides `MownStr`,
a type for exchanging read-only strings that may be either borrowed or owned.

Contrarily to other types (such as for example [`Cow<str>`]),
`MownStr` does not aim to be mutable nor generic,
which allows it to be fast and lean.

Actually, a `MownStr` takes no more memory than a regular `&str` or `Box<str>`,
and has a minimal runtime overhead.
The drawback is that `MownStr` the maximum size of a `MownStr`
is half the size of a regular `str`
(which is still 8EiB on a 64-bit architectures...).


## Speed or safety

As mentioned above, the maximum size for a `MownStr` is `usize::MAX / 2`,
so in theory, not all `&str` or `Box<str>` can be converted to `MownStr`.

In practice though, such huge strings are very unlikely.
By default, when creating a `MownStr`,
this crate will therefore only check the size of the data in `debug` mode.
This will save a few CPU cycles in production code,
in order to make `MownStr` as fast as possible.

If you are not ok with this,
you can enable the `paranoid` feature,
which will enable these tests in `release` mode as well.

[`Cow<str>`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html
