# Writing a language parser with [nom](https://github.com/geal/nom)

When designing [jinko](https://github.com/cohenarthur/jinko), I chose to use a classic
Lexer-Parser-AST architecture. But writing parsers yourself is a hassle, and I'm not a
big fan of parser generators (I haven't tried any of the ones available for Rust, mind
you). Since Rust is a kinda-functional language, there are parser combinators available,
which I had less experience with and wanted to try.

_Explain what a parser combinator is or link to Wikipedia
Link to F# parser combinator tutorial
Talk about nom's limitations (doc, examples)..._

I set off to use [nom](https://github.com/geal/nom). I have only made one project with
nom, which involved parsing C structures, and it did not go very well for a number of
reasons:
* nom's examples are a bit weird. The Hex parser example on the README is very simple,
which works well for most of nom's uses but not for a language parser.
* The language projects using nom, such as _$PHP_INTERP_ and _$HTML_PARSER_, are big
rust projects, which are therefore harder to read. On top of that, parsers are annoying
to understand anyway.
* The "Getting Started" guides still use the nom macro system, which is deprecated in
favor of new, easier to use functions.
* C structs' grammar is... interesting. The typedef keyword can be used for structs, as
well as types, such as function pointers. Types in C deserve their own parser, as they
are very complex on their own. And struct declarations are full of them! You can have
a `char array[]`, or a `char* array`, or a `char*** * * ** *array[]`, and it's all valid
types.

Therefore, I had very little experience with [nom](https://github.com/geal/nom) when getting
into [jinko](https://github.com/cohenarthur/jinko). I had trouble finding all available
combinators, and deciding which ones to use. Examples online were either too complex or
too simple for my needs.



For anyone getting into nom, I would suggest getting used to it before getting into
big projects. For example, a simple binary format, or a small interpreter for an esoteric
language with a limited set of instructions _$LINK TO FUCKFUCK IN RUST_
