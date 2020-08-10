# Let's write a parser combinator in C!

[Parsers combinators](https://en.wikipedia.org/wiki/Parser_combinator) are generally used
in high-level languages such as functional ones. There are a few implementations for C,
such as [mpc](https://github.com/orangeduck/mpc) or
[Cesium3](https://github.com/wbhart/Cesium3/tree/combinators), but these offer way more
than what I'm trying to accomplish here.

I've recently been using [nom](https://github.com/Geal/nom/) for
[broccoli](https://github.com/cohenarthur/broccoli) and
[svz](https://github.com/cohenarthur/svz), and it has changed my vision of parsing (which
was pitiful, let's be honest). It's much easier to use and intuitive than writing your
own lexer-parser combo or using a parser generator, especially for languages such as C.

The idea behind parser combinators is that, instead of tokenizing your input using a lexer
and then transforming it into an AST (or any other data structure), you directly combine
_functions_ to recognize your grammar. It's much more natural, and allows your code to
read in a human-like way

> I want to recognize the `struct` keyword, then its name, then
> a set of curly brackets with maybe some fields inside

With a lexer-parser combo, it would probably read something like that:

> I have created a `struct` token
> I have created an `identifier` token
> I have created a `left curly bracket` token
> I have created all the tokens necessary for the fields inside the struct
> I have created a `right curly bracket` token
> I give all those tokens to my parser, which makes sure that they're in a syntactically
> correct order

Overall, parsers are annoying to write, annoying to test, annoying to read and annoying to
use. Which is why many parser generators exist, that is, binaries that directly generate
code so you don't have to write it yourself. Says a lot about how fun it is.

The libraries I've mentionned are much more complex than what we're trying to accomplish
here. Ideally, the API I'd like to expose would be the following:

|function name|Description|
|---|---|
|`char`|Recognizes a character|
|`tok`|Recognizes a token|
|`opt`|Takes a parser as argument, and treats it as optional|
|`alt`|Takes two parsers, execute one OR the other if the first one doesn't match|
