MownStr: Maybe Owned String
===========================

This crate provides `MownStr`,
a type for exchanging read-only strings that may be either borrowed or owned.

Contrarily to other types (such as for example [`Cow<str>`]),
`MownStr` does not aim to be mutable nor generic.

[`Cow<str>`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html
