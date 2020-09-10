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
a `char array[]`, or a `char* array`, or a `struct some_struct *** * * ** *array[]`,
and it's all valid types.

Therefore, I had very little experience with [nom](https://github.com/geal/nom) when getting
into [jinko](https://github.com/cohenarthur/jinko). I had trouble finding all available
combinators, and deciding which ones to use. Examples online were either too complex or
too simple for my needs. I came up with a simple way to use nom, which fits my needs but
might not be very idiomatic, and is definitely a little ugly.

## `nom` usage in `jinko`

The `jinko` parser is in its own module, `parser`, which exposes only one function:

```rust
fn parse(input: &str) -> Result<Interpreter, JinkoError>;
```

Given an input string (the source code), the parser should return a correct `Interpreter`
(which is the data structure used to execute `jinko` code) or an error. In this module,
two submodules are present: `Construct` and `Token`. `Token` exposes functions useful
for token specification (recognizing an interpreter, a reserved keyword, a string, a
floating point number, and so on) while `Construct` combines tokens to recognize complex
lexical compounds, such as a function declaration, an if-else block, etc.


Both `Token` and `Construct` use `nom`. Separating the parser like this creates a sort
of lexer-parser composite, similar to what you're used to when using a parser generator
or writing a homemade one. `Token` uses what I would qualify as "low level" combinators,
in the sense that they act mostly on characters (`is_a`, `tag`, `char`) while `Consruct`
uses "combinator combinators" (`alt` to denotes a parser or another, `opt` to mark
the optional usage of a parser...).

Let's look at `Token::loop_tok`. The function is as follow:

```rust
// A bit simplified, but equivalent
fn loop_tok(input: &str) -> IResult<&str, &str> {
    tag("loop ")(input)
}
```

If you look at the [documentation for `tag`](_$LINK_TO_DOC), you'll see that it's used to
recognize a sequence of characters. The `loop_tok` function is pretty simple, it just
needs to recognize the `loop` keyword.

You'll notice a few things: First of all, what is an `IResult<A, B>`? It's equivalent to
a `Result<(A, B), E>`, with `E` being nom's error type, `nom::Error`. This is what every
nom combinator returns. The first value is the input left after execution of the
combinator, while the second value is the recongized pattern. You can thus expect the
following:

```rust
// Using loop as a keyword
assert_eq!(Token::loop_tok("loop {}", Ok(("{}", "loop ")));

// Using `loop` in a variable name for example
assert_eq!(Token::loop_tok("loopyaplop", Err(/* some nom error */));
```

Secondly, why recognize "loop "  and not "loop"? As shown in the second assertion, the
combinator would return `Ok` even if "loop" was used as something else than the keyword.
Since nom is not specifically made for language parsers.

If we were in a classic lexer, this would probably

_Speak about <keyword><space>, how it's ugly, speak about identifiers
For construct, speak about `mut`, speak about separation of funcs_

For anyone getting into nom, I would suggest getting used to it before getting into
big projects. For example, a simple binary format, or a small interpreter for an esoteric
language with a limited set of instructions _$LINK TO FUCKFUCK IN RUST_
