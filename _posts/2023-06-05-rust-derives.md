---
layout: post
title: "Looking at Rust builtin derives"
author: Arthur Cohen
tags:
    - gccrs
    - rust
---

We are currently working on supporting [builtin procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html) in `gccrs`, an area of Rust macro expansion we had left unexplored. Specifically, we are currently focusing on builtin derive macros: That work is going nicely, and at the time of writing this blogpost, we have support for some of the basic ones, as well as a framework for new contributors who'd like to implement the remaining macros.

Figuring out how to implement these macros in our compiler required us to spend a lot of time understanding their official implementation.
So since I had to spend some time looking at `rustc`'s innards, as well as the eerie output produced by the compiler when invoking it with the proper `-Z` incantation, I thought that it would be fun for me to share some of the interesting bits I encountered.

## What are builtin derive macros?

__FIXME__:
Procedural macros are special macros which, like *regular* macros, receive a list of tokens
and return a list of tokens to be integrated to the current AST node, but unlike regular
macros, are invoked and expanded in a completely different manner: They usually involve performing foreign function calls to a user provided shared
library (a `.so` file on Linux), serializing a stream of tokens and deserializing the one
received. Furthermore, the tokens handled by a procedural macros are different from the compiler's tokens,
and are defined in the `proc_macro` crate, which we are currently reimplementing as part of `gccrs` (- we have to!).

In short, there is a lot of work that goes into performing procedural macro expansion: serialization, bridging, dynamic procedure calls, deserialization, insertion...

The procedural macros I'd like to take a look at in this blogpost are much simpler: they are *builtin* procedural macros, meaning that they are being handled directly by the compiler itself (`rustc`, or in our case, `gccrs`).

Specifically, this blogpost focuses on builtin *derive* macros: You've probably encountered these macros when writing Rust code, as they are literally present everywhere:

```rust
#[derive(Clone, Copy)] // here!
enum Fruit {
  Apple { ripe: bool, },
  Pear,
  Orange,
  Banana,
}

#[derive(Clone)] // here too!
struct FruitBowl {
  content: Vec<Fruit>,
  needs_cleaning: bool,
}
```

These special incantations will cause our two types, `Fruit` and `FruitBowl`, to gain special powers: We are now able to make a simply "copy" of an instance of `Fruit`, directly from this instance or from a reference to this instance:

```rust
let apple = Fruit::Apple { ripe: false };
let mut transmuted_apple = apple;

// this does not modify `apple` - we made a copy
transmuted_apple = Fruit::Banana;
```

And we are also able to duplicate a fruit bowl, even if that operation might be a little more costly and needs to be explicit:

```rust
let bowl = FruitBowl {
    content: vec![apple, transmuted_apple],
    needs_cleaning: false,
};
let your_bowl = bowl.clone();
```

_FIXME_:
What happened is that the compiler implemented the [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html) and [`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html) traits for our types, all by itself, through this `derive` mechanism.

## Interesting bits

We can have a look at the automatic implementation of these traits by the compiler through nightly flags 🧙‍♀️. Running `rustc -Zunpretty=expanded` on our above types yields the following output:

```rust
#![feature(prelude_import)]
#![no_std]
#[prelude_import]
use ::std::prelude::rust_2015::*;
#[macro_use]
extern crate std;
// here!
enum Fruit {
    Apple {
        ripe: bool,
    },
    Pear,
    Orange,
    Banana,
}
#[automatically_derived]
impl ::core::clone::Clone for Fruit {
    #[inline]
    fn clone(&self) -> Fruit {
        let _: ::core::clone::AssertParamIsClone<bool>;
        *self
    }
}
#[automatically_derived]
impl ::core::marker::Copy for Fruit { }

// here too!
struct FruitBowl {
    content: Vec<Fruit>,
    needs_cleaning: bool,
}
#[automatically_derived]
impl ::core::clone::Clone for FruitBowl {
    #[inline]
    fn clone(&self) -> FruitBowl {
        FruitBowl {
            content: ::core::clone::Clone::clone(&self.content),
            needs_cleaning: ::core::clone::Clone::clone(&self.needs_cleaning),
        }
    }
}
```

The traits implemented automatically by the compiler have a special attribute: `#[automatically_derived]`. This helps the compiler differentiate them from regular user implementations for various purposes such as lints or pretty typechecking errors.
If you look at the implementation of `Clone` for `FruitBowl`, you can see that it is quite simple: We basically call `.clone()` on each of the fields of the struct. You could think that this is quite simple, and in fact - it is! For regular structures, the implementation of `Clone` is simply a clone of each of the fields. If a struct does not contain any fields, we can simply return an instance of it. This is exactly the same for named tuples:

```rust
#[derive(Clone)]
struct StringPair(String, String);
```

becomes

```rust
struct StringPair(String, String);

#[automatically_derived]
impl ::core::clone::Clone for StringPair {
    #[inline]
    fn clone(&self) -> StringPair {
        StringPair(::core::clone::Clone::clone(&self.0),
            ::core::clone::Clone::clone(&self.1))
    }
}
```

heh. Quite easy. Now the first interesting bit I would like to bring your attention to is the implementation of `Clone` for `Fruit`. Let's isolate that part of the code.

```rust
enum Fruit {
    Apple {
        ripe: bool,
    },
    Pear,
    Orange,
    Banana,
}

#[automatically_derived]
impl ::core::clone::Clone for Fruit {
    #[inline]
    fn clone(&self) -> Fruit {
        let _: ::core::clone::AssertParamIsClone<bool>;
        *self
    }
}
```

huh. Interesting! This looks nothing like the above implementation. In fact, this looks nothing like implementing `Clone` on another enum:

```rust
enum Animal {
    Dog { name: String, },
    Cat(String, String, String),
    Horse,
}

#[automatically_derived]
impl ::core::clone::Clone for Animal {
    #[inline]
    fn clone(&self) -> Animal {
        match self {
            Animal::Dog { name: __self_0 } =>
                Animal::Dog { name: ::core::clone::Clone::clone(__self_0) },
            Animal::Cat(__self_0, __self_1, __self_2) =>
                Animal::Cat(::core::clone::Clone::clone(__self_0),
                    ::core::clone::Clone::clone(__self_1),
                    ::core::clone::Clone::clone(__self_2)),
            Animal::Horse => Animal::Horse,
        }
    }
}
```

This makes more sense: We can see that the compiler proceeds to pattern match `self`, and
clones the inner fields of each variant:

1. No fields for `Animal::Horse`.


```rust
Animal::Horse => Animal::Horse,
```

2. The name for `Animal::Dog`

```rust
Animal::Dog { name: __self_0 } =>
    Animal::Dog { name: ::core::clone::Clone::clone(__self_0) },
```

3. And three index fields for
`Animal::Cat`

```rust
Animal::Cat(__self_0, __self_1, __self_2) =>
    Animal::Cat(::core::clone::Clone::clone(__self_0),
        ::core::clone::Clone::clone(__self_1),
        ::core::clone::Clone::clone(__self_2)),
```

So why is our `Fruit` enum getting special treatment? As a 17th century Rust developer would say, the proof is in the `#[derive(Copy)]`ing, and taking a bite reveals our answer: if the compiler can realize, during macro expansion, that a type for which we implement `Clone` is also `Copy`, then it will simply implement cloning as a copy: By simply dereferencing a reference to a `Fruit`, we get another, new instance of `Fruit`, which will be copy of `&self`. _FIXME_:

As pointed out to me by [Nilstrieb](https://github.com/Nilstrieb/), this is not typechecking: If you were to implement `Copy` by hand for this type, `rustc` could not realize (at that stage) that it can reuse the implementation. Macro expansion happens at the AST level, way before Rust code becomes another intermediate representation, HIR, which is the one that will get properly typechecked. I still think this is a form of funny typechecking since we're checking for a trait implementation but just not going the whole way. But who am I to disagree.

Another interesting line in the `Clone` implementation for `Fruit` is the following:

```rust
let _: ::core::clone::AssertParamIsClone<bool>;
```

In order to understand it, let's look at `#[derive(Clone)]`ing ~~Starbuck's sworn enemy~~ the scariest Rust type... A union.

```rust
#[derive(Clone)]
union Register {
    bytes: [u8; 4],
    value: u32,
}
```

If we try and compile the above code, we'll hit an error:

```rust
error[E0277]: the trait bound `Register: Copy` is not satisfied
   --> <source>:1:10
    |
1   | #[derive(Clone)]
    |          ^^^^^ the trait `Copy` is not implemented for `Register`
    |
```

with a note:

```rust
note: required by a bound in `AssertParamIsCopy`
   --> /home/arthur/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/clone.rs:163:33
    |
163 | pub struct AssertParamIsCopy<T: Copy + ?Sized> {
    |                                 ^^^^ required by this bound in `AssertParamIsCopy`
    = note: this error originates in the derive macro `Clone` (in Nightly builds, run with -Z macro-backtrace for more info)
help: consider annotating `Register` with `#[derive(Copy)]`
    |
2   + #[derive(Copy)]
3   | union Register {
    |
```

Looking at the expanded source, we learn a little bit more:

```rust
union Register {
    bytes: [u8; 4],
    value: u32,
}

#[automatically_derived]
impl ::core::clone::Clone for Register {
    #[inline]
    fn clone(&self) -> Register {
        let _: ::core::clone::AssertParamIsCopy<Self>;
        *self
    }
}
```

The implementation of `Clone` for our union looks a lot like the one for `Fruit`: We simply dereference `self`, in order to copy the union instance we are referencing. Similarly to `Fruit`'s `Clone` implementation, we see a let statement:

```rust
let _: ::core::clone::AssertParamIsCopy<Self>;
```

This `AssertParamIsCopy` type is defined in the core library, and looks a little bit like this:

```rust
pub struct AssertParamIsCopy<T: Copy + ?Sized> {
    _field: crate::marker::PhantomData<T>,
}
```

This arcane struct is `#[doc(hidden)]`, so you won't be able to see it on https://doc.rust-lang.org/std/.

This struct, through Rust's powerful type system and dark magic, ensures that
its given generic parameter implements the `Copy` trait. Because there is no need to keep an instance of `T` within the type, it uses a [phantom type](https://doc.rust-lang.org/rust-by-example/generics/phantom.html) to enable the struct's genericity.

The let statement creates a new binding, which does not create a value of type `AssertParamIsCopy<Register>`, but forces the compiler to create a monomorphized version of `AssertParamIsCopy` with `Register` as its type parameter. Then, during typechecking, the compiler ensures that `Register` meets the criteria for being passed to `AssertParamIsCopy`: This criteria is `Copy + ?Sized`. If the type is not `Copy`, we will get an error - so if our union is not `Copy`, we cannot clone it!

For experienced Rust playahs, this is baby shit. For others, including myself, this is super cool type-system witchcraft that everyone should know about! Figuring this out made my day a whole lot better, and reminded me of just how powerful Rust's type system is. I think it is *extremely* cool to see it being utilized in this way directly by the compiler.

This technique is also used for our `Fruit` type, with one simple difference: `AssertParamIsClone` does not check that its type parameter is `Copy`, but that it is `Clone`.

## `gccrs` bits

Now that the interesting Rust bits are laid on the table, I thought I would dive a little bit into our implementation of builtin derive macros. Our compiler is "visitor-based", and contains multiple frameworks for visiting our AST nodes. Our deriving framework relies on one such visitor, and provides a base class to allow contributors to implement specific derive macros such as `Clone` or `Copy`.

One of the first thing to note about builtin derive macros is that they can only be applied to Rust types: `struct`s, `enum`s or `union`s. You cannot (at least, yet) use `#[derive(...)]` on other items, such as a function or trait declaration. To ensure that unaware contributors such as myself do not make this mistake in the future, we are utilizing C++'s type system (haha) to restrict our `Derive` implementation to these types of items:

### Preventing mistakes
### AST Builder
### Tuple indexing

Also...

## Today is my dog's birthday

![Jinko](/jinko.jpg)

🏳️‍⚧️ 💜 Happy pride month everyone! 💜 🏳️‍🌈

<br>
<br>
<br>
<br>
<p style="font-family:'Source Code Pro'">
<span style="color:#d784f3">type</span> <a href="https://github.com/cohenarthur">GitHub = <span style="color:#69c908">"/CohenArthur"</span></a>;<br>
<span style="color:#d784f3">type</span> <a href="https://twitter.com/cohenarthurdev">Twitter = <span style="color:#69c908">"/CohenArthurDev"</span></a>;<br>
<span style="color:#d784f3">type</span> <a href="https://hachyderm.io/@cohenarthur">Mastodon = HachydermIO<span style="color:#666666">[</span><span style="color:#69c908">"@cohenarthur"</span><span style="color:#666666">]</span></a>;<br>
</p>
