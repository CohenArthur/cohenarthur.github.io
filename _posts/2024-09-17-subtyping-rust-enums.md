---
layout: post
title: "Adding subtyping to Rust enums"
author: Arthur Cohen
tags:
    - rust
    - jinko
---

Alternative title: _Making Rust enums a little bit more powerful and a lot more stupid_.

Imagine the following. You are sending values to an API, which expects more variants than what you need to handle for your project. Let's say this is about creating a feed schedule for any kind of farm animal, when you only have ducks and bees. We could represent this using the following enumerations in Rust:

```rust
enum MyAnimal {
    Duck,
    Bee,
}

enum ApiAnimal {
    Duck,
    Bee,
    Goose,
    Horse,
    Cow,
    Donkey,
}
```

We would then convert from our animals to the API's expected list of animals using a function that looks something like this:

```rust
fn convert(animal: MyAnimal) -> ApiAnimal {
    match animal {
        MyAnimal::Bee => ApiAnimal::Bee,
        MyAnimal::Duck => ApiAnimal::Duck,
    }
}
```

Utterly uninteresting. However, as many of you will have noticed, this is a non-problem and probably something no one ever runs into. You'll probably need to do more complex conversions, handle more cases, handle errors, or something - but still, I like making solutions for non-issues so I will keep going. _NOTE: Reword?_

We can see that the variants of our `MyAnimal` enum are exactly the same as the corresponding variants in the `ApiAnimal` enum. They have the same fields (none), the same order (`Duck` in first position, then `Bee`...), and frankly, it's quite easy to see that all of the variants from our source enum are contained in our destination enum. So why can't we simply... transform an instance of `MyAnimal` into one of `ApiAnimal`? Without writing any boring boilerplate code? Plus, if we decide to start adopting geese, we'll need to add *one extra match arm* to our function, and frankly that won't do. So let's make the compiler do it for us automatically. Our aim is to be able to write the `convert` function like so:

```rust
fn convert(animal: MyAnimal) -> ApiAnimal {
    animal
}
```

Let's look at what `rustc` 1.81 has to say about our code before we irreversibly worsen the compiler:

```rust
error[E0308]: mismatched types
  --> src/lib.rs:23:5
   |
22 | fn convert(animal: MyAnimal) -> ApiAnimal {
   |                                 --------- expected `ApiAnimal` because of return type
23 |     animal
   |     ^^^^^^ expected `ApiAnimal`, found `MyAnimal`

For more information about this error, try `rustc --explain E0308`.
error: could not compile `playground` (lib) due to 1 previous error
```

The types do not match, since we are trying to return an instance of `MyAnimal` when `rustc` expects an `ApiAnimal`. That means we need to let our typechecker know that in this particular case, that code is a-okay because we made sure all of the variants of `MyAnimal` are all members of the variants of `ApiAnimal`. We can model this using sets for each of the enums' variants: if the variants of the source enum are all contained within the variants of the destination enum, then the conversion is okay. The code basically boils down to this:

```rust
fn is_a_okay_enum_conversion(src_enum: Enum, dst_enum: Enum) -> bool {
    return dst_enum.variants().contains(src_enum.variants())
}
```

The compiler no longer complains about our function. Success. We create our branch, commit our code, and send a pull-request to `rustc`. All happy and blissful, satisfied, we now turn our attention to other matters in compiler-fantasy-land. Then the first review comes in:

"What about at runtime?"

What? What runtime? Who cares??? 

_NOTE: Remove talks about order of variants in source enum since we are gonna work around that_

Friend, anything past an SSA form is waaaaaay above my pay-grade. If you want stuff to run you should talk to LLVM or something.

But our reviewer is right. What happens after typechecking? We cannot simply convert from one type to another willy-nilly. I mean, we *can* ~~core::mem::transmute~~, sometimes, but really we shouldn't, so we won't. To understand why, we need to look at the layout of Rust enums:

https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=b0c0b827c7ee1f8e3892b2024a3a9644

The first issue is that our source and destination enum might not have the same size, despite one being a subset of the other.

_NOTE: Look at layout of Rust enums_
_NOTE: Add examples from playground + LLVM IR_

<br>
<br>
<br>
<br>
<p style="font-family:'Source Code Pro'">
<span style="color:#d784f3">type</span> <a href="https://github.com/cohenarthur">GitHub = <span style="color:#69c908">"/CohenArthur"</span></a>;<br>
<span style="color:#d784f3">type</span> <a href="https://twitter.com/cohenarthurdev">Twitter = <span style="color:#69c908">"/CohenArthurDev"</span></a>;<br>
<span style="color:#d784f3">type</span> <a href="https://hachyderm.io/@cohenarthur">Mastodon = HachydermIO<span style="color:#666666">[</span><span style="color:#69c908">"@cohenarthur"</span><span style="color:#666666">]</span></a>;<br>
</p>

