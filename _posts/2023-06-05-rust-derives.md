---
layout: post
title: "Looking at Rust builtin derives"
author: Arthur Cohen
tags:
    - gccrs
    - rust
---

We are currently working on supporting [builtin procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html) in `gccrs`, an area of Rust macros we had left unexplored.

Procedural macros are a special kind of macros which, like "regular" macros, receive a list of tokens
and return a new one. There are multiple tools to handle this list of tokens, starting with the types provided by the [`proc_macro`](https://doc.rust-lang.org/stable/proc_macro/) crate and ending with complex crates such as [`syn`](https://docs.rs/syn/latest/syn/) that allow you to parse this token input.

Unlike regular macros, procedural macros are invoked and expanded not directly within the compiler but through foreign function calls to a user provided shared
library (a `.so` file on Linux). The procedural macro's input is serialized to a stream of tokens, which gets sent to the macro via a dynamic procedure call. The returned token stream then needs to be deserialized and integrated to the AST. Furthermore, the tokens handled by a procedural macro are different from the compiler's tokens: they contain specific information, and have different stability guarantees and a different API. They are defined in the `proc_macro` crate, which we need to reimplement for `gccrs` to support procedural macros.

Figuring out how to implement these macros in our compiler requires us to extensively research their official implementation.
Since I had to spend so much time looking at `rustc`'s innards, as well as the eerie output produced by the compiler when invoking it with the proper `-Z` incantation, I thought that it would be fun to share some of the interesting bits I encountered. 🧙‍♀️

## What are builtin derive macros?

More specifically, our work is currently focused on *builtin derive macros*: That work is going nicely, and at the time of writing this blogpost, we have support for some of the basic ones, as well as a framework for new contributors who'd like to implement others.

The procedural macros I'd like to take a look at in this blogpost are much simpler than procedural macros in general: they are *builtin* procedural macros, meaning that they are being handled directly by the compiler itself (`rustc`, or in our case, `gccrs`). There is no serialization, no dynamic library calls and not nearly as much pain - but we must still expand them. The list of builtin procedural macros is much smaller than the list of "builtin regular macros", but it still contains attributes you probably use everyday: `#[test]`, `#[derive(Clone)]`, `#[derive(Hash)]`...

Even more specifically, this blogpost focuses on builtin *derive* macros, which are literally present everywhere in Rust code:

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

These special incantations will cause our two types, `Fruit` and `FruitBowl`, to gain special powers: We are now able to make a simple "copy" of an instance of `Fruit`, directly from this instance or from a reference to this instance:

```rust
let apple = Fruit::Apple { ripe: false };
let mut transmuted_apple = apple;

// this does not modify `apple` - we made a copy
transmuted_apple = Fruit::Banana;
// we can also still use `apple` - the value
// has not been *moved*, but *copied*.
```

And we are also able to duplicate a fruit bowl, even if that operation might be a little more costly and needs to be explicit:

```rust
let my_bowl = FruitBowl {
    content: vec![apple, transmuted_apple],
    needs_cleaning: false,
};
let your_bowl = bowl.clone();
```

What is happening behind the scene is that the compiler implemented the [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html) and [`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html) traits for our types, all by itself, through this `derive` mechanism.

There are more builtin derive macros you might have seen, such as `Ord`, `PartialOrd`, `Hash`, `Default`... or funky ones, like the very ominous `RustcEncodable` and `RustcDecodable`, which are internal to the compiler.

## Interesting bits

We can have a look at the automatic implementation of these traits thanks to nightly flags 🧙‍♀️. Running `rustc -Zunpretty=expanded` on our above types yields the following output:

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

The first thing to note is that traits implemented automatically by the compiler have a special attribute: `#[automatically_derived]`:

```rust
#[automatically_derived]
impl ::core::marker::Copy for Fruit { }
```

This attribute helps the compiler differentiate them from regular user implementations for various purposes such as lints or prettier typechecking errors.

Now, if you look at the implementation of `Clone` for `FruitBowl`, you can see that it is quite simple:

```rust
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

We basically call `.clone()` on each of the fields of the struct. This is similar to how you or I would implement the trait. You could think that this is quite easy to generate, and in fact - it is! For regular structures, the implementation of `Clone` is simply a clone of each of the fields. If a struct does not contain any fields, we can simply return an instance of it. The same thing happens for named tuples/tuple structs:

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

heh. Quite easy.

Now the first *very* interesting bit I would like to bring your attention to is the implementation of `Clone` for `Fruit`. Let's isolate that part of the code.

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
    /* Cats have three nicknames */
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

1. There are no fields for `Animal::Horse`, so we just return the same variant


```rust
Animal::Horse => Animal::Horse,
```

2. The name for `Animal::Dog` gets cloned, like the field of a regular `struct`

```rust
Animal::Dog { name: __self_0 } =>
    Animal::Dog { name: ::core::clone::Clone::clone(__self_0) },
```

3. And three index fields for `Animal::Cat`, as the compiler does for a tuple struct.

```rust
Animal::Cat(__self_0, __self_1, __self_2) =>
    Animal::Cat(::core::clone::Clone::clone(__self_0),
        ::core::clone::Clone::clone(__self_1),
        ::core::clone::Clone::clone(__self_2)),
```

So why is our `Fruit` enum getting special treatment? As a 17th century Rust developer would say, the proof is in the `#[derive(Copy)]`ing, and taking a bite reveals our answer: if the compiler can realize, during macro expansion, that a type for which we derive `Clone` is also `Copy`, then it will simply implement cloning as a copy of that type.

By simply dereferencing a reference to a `Fruit`, we get another, new instance of `Fruit`, which will be a copy of the `self` parameter. Wooh!

As pointed out to me by [Nilstrieb](https://github.com/Nilstrieb/), this is not typechecking: If you were to implement `Copy` by hand for this type, `rustc` could not realize (at that stage) that it can reuse the implementation. Macro expansion happens at the AST level, way before Rust code gets properly typechecked. I still think this is a form of funny typechecking since we're checking for a trait implementation but just not going the whole way, but as one says, it's not typechecking unless it comes from the French region of `hir::Ty<'tcx>`, it's just sparkling name resolution.

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

The implementation of `Clone` for our union looks a lot like the one for `Fruit`: We simply dereference `self`, in order to copy the union instance we are referencing. Similarly to `Fruit`'s `Clone` implementation, we see a *let* statement:

```rust
let _: ::core::clone::AssertParamIsCopy<Self>;
```

This `AssertParamIsCopy` type is defined in the core library, and looks a little bit like this:

```rust
pub struct AssertParamIsCopy<T: Copy + ?Sized> {
    _field: crate::marker::PhantomData<T>,
}
```

This arcane struct is `#[doc(hidden)]`, so you won't be able to see it on [https://doc.rust-lang.org/std/](https://doc.rust-lang.org/std/). Hehe. But it is there! If you look at [the source for the `Clone` trait](https://doc.rust-lang.org/src/core/clone.rs.html#110) and scroll down, you'll be able to peruse the magic.

This struct, through Rust's powerful type system, ensures that
its given generic parameter implements the `Copy` trait. Because there is no need to keep an instance of `T` within the type, it uses a [phantom type](https://doc.rust-lang.org/rust-by-example/generics/phantom.html) to enable the struct's genericity.

The let statement creates a new binding, which does not create a value of type `AssertParamIsCopy<Register>`, but forces the compiler to create a monomorphized version of `AssertParamIsCopy` with `Register` as its type parameter. Then, during typechecking, the compiler ensures that `Register` meets the criteria for being passed to `AssertParamIsCopy`: This criteria is `Copy + ?Sized`. If the type is not `Copy`, we will get an error - so if our union is not `Copy`, we cannot clone it!

For experienced Rust playahs, this is baby shit. For others, including myself, this is super cool type-system witchcraft that everyone should know about! Figuring this out made my day a whole lot better, and reminded me of just how cool type systems are. I think it is *mega-fine* to see it being utilized in this way directly by the compiler.

This technique is also used for our `Fruit` type, with one simple difference: `AssertParamIsClone` does not check that its type parameter is `Copy`, but that it is `Clone`. So we make sure that all types *possibly* contained in an instance of `Fruit` (in our case, just the boolean if we are dealing with a `Fruit::Apple`) implement `Clone`.

```rust
#[automatically_derived]
impl ::core::clone::Clone for Fruit {
    #[inline]
    fn clone(&self) -> Fruit {
        let _: ::core::clone::AssertParamIsClone<bool>;
        *self
    }
}
```

For more complex enums, such as this one:

```rust
#[derive(Clone, Copy)]
enum Stmt {
    Item(Item),
    LetStmt {
        pattern: Pattern,
        ty: Option<Type>,
        init_expr: Option<Expr>,
    },
    ExprStmt(Expr),
    Macro(MacroInvocation),
    Empty,
}
```

We get more type level assertions:

```rust
#[automatically_derived]
impl ::core::clone::Clone for Stmt {
    #[inline]
    fn clone(&self) -> Stmt {
        let _: ::core::clone::AssertParamIsClone<Item>;
        let _: ::core::clone::AssertParamIsClone<Pattern>;
        let _: ::core::clone::AssertParamIsClone<Option<Type>>;
        let _: ::core::clone::AssertParamIsClone<Option<Expr>>;
        let _: ::core::clone::AssertParamIsClone<Expr>;
        let _: ::core::clone::AssertParamIsClone<MacroInvocation>;
        *self
    }
}
```

## gccrs bits

Now that the interesting Rust bits are laid on the table, I thought I would dive a little bit into `gccrs`' implementation of builtin derive macros, so you're allowed to close the blogpost and look away.

Our compiler is "visitor-based", and contains multiple frameworks for visiting our AST nodes. Our deriving framework relies on one such visitor, and provides a base class to allow contributors to implement specific derive macros such as `Clone` or `Copy`.

One of the first thing to note about builtin derive macros is that they can only be applied to Rust types: `struct`s, `enum`s or `union`s. You cannot (at least, yet) use `#[derive(...)]` on other items, such as a function or trait declaration. To ensure that unaware contributors such as myself do not make this mistake in the future, we are utilizing C++'s type system (haha) to restrict our `Derive` implementation to these types of items:

```cpp
/**
 * The goal of this class is to accumulate and create the required items from a
 * builtin `#[derive]` macro applied on a struct, enum or union.
 */
class DeriveVisitor : public AST::ASTVisitor
{
public:
  static std::unique_ptr<Item> derive (Item &item, const Attribute &derive,
				       BuiltinMacro to_derive);

private:
  // the 4 "allowed" visitors, which a derive-visitor can specify and override
  virtual void visit_struct (StructStruct &struct_item) = 0;
  virtual void visit_tuple (TupleStruct &tuple_item) = 0;
  virtual void visit_enum (Enum &enum_item) = 0;
  virtual void visit_union (Union &enum_item) = 0;

  // all visitors are final, so no deriving class can implement `derive` for
  // anything other than structs, tuples, enums and unions

  virtual void visit (StructStruct &struct_item) override final
  {
    visit_struct (struct_item);
  }

  virtual void visit (TupleStruct &tuple_struct) override final
  {
    visit_tuple (tuple_struct);
  }

  virtual void visit (Enum &enum_item) override final
  {
    visit_enum (enum_item);
  }

  virtual void visit (Union &union_item) override final
  {
    visit_union (union_item);
  }

  virtual void visit (IdentifierExpr &ident_expr) override final{};
  virtual void visit (Lifetime &lifetime) override final{};
  virtual void visit (LifetimeParam &lifetime_param) override final{};
  virtual void visit (ConstGenericParam &const_param) override final{};
  // ... and so on
```

All derive implementations, like our `DeriveClone` and `DeriveCopy` class, derive from this
base `DeriveVisitor` class: thanks to these inheritance rules, they cannot implement deriving
for items other than the allowed ones.

Furthermore, you may remember the beginning of the blogpost mentioning that procedural macros
received streams of tokens as an input: but just like `rustc`, `gccrs` can benefit from its
AST when expanding builtin derive macros, and directly know whether it is dealing with a
`struct`, `enum` or `union`, without needing to parse the received token stream. This saves an
extra step in the processing and allows us to be more specific.

The goal of builtin derive macros is to create a new `impl` block and integrate it to an existing AST: Basically, turn one item

```rust
#[derive(Copy)]
struct S;
```

into two items

```rust
// item #1
struct S;


// item #2
impl Copy for S {}
```

To do this, we need to be able to easily create AST nodes from our Derive classes: hence the creation of an `AstBuilder` class, whose role is to generate nodes easily and store them in the proper smart pointer types:

```cpp
/* Builder class with helper methods to create AST nodes. This builder is
 * tailored towards generating multiple AST nodes from a single location, and
 * may not be suitable to other purposes */
class AstBuilder
{
public:
  AstBuilder (Location loc) : loc (loc) {}

  /* Create a reference to an expression (`&of`) */
  std::unique_ptr<Expr> ref (std::unique_ptr<Expr> &&of, bool mut = false);

  /* Create a dereference of an expression (`*of`) */
  std::unique_ptr<Expr> deref (std::unique_ptr<Expr> &&of);

  /* Create a block with an optional tail expression */
  std::unique_ptr<Expr> block (std::vector<std::unique_ptr<Stmt>> &&stmts,
			       std::unique_ptr<Expr> &&tail_expr = nullptr);

  // ... etc
```

This class will also be reused for regular builtin macros such as `assert!`, `env!`, `panic!`... since this system also needs to create AST nodes in a simple way.

The role of a deriving class is then to utilize this builder to create the proper `impl` block. For example, if we look at the implementation for `#[derive(Clone)]` on named tuples, `DeriveClone::visit_tuple`:

```cpp
void
DeriveClone::visit_tuple (TupleStruct &item)
{
  // For each index field in the tuple, we create a new clone expression
  auto cloned_fields = std::vector<std::unique_ptr<Expr>> ();

  for (size_t idx = 0; idx < item.get_fields ().size (); idx++)
    cloned_fields.emplace_back (
      // call to clone...
      clone_call (
        // ... to a reference...
        builder.ref (
          // ... of a tuple index expression (`self.0`)
          builder.tuple_idx ("self", idx))));

  // then, create the constructor: if our tuple struct is named `Tuplo`,
  // this amounts to creating `Tuplo(field_1, field_2, field_3...)`.
  auto path = std::unique_ptr<Expr> (new PathInExpression (
    builder.path_in_expression ({item.get_identifier ()})));
  auto constructor = builder.call (std::move (path), std::move (cloned_fields));

  // finally, we move this constructor expression in a function (`clone_fn`)
  // which expands to `fn clone(&self) -> Self { <expression> }`

  // and move this function into a `Clone` impl block:
  // `impl Clone for Tuplo { <clone_fn> }`
  expanded
    = clone_impl (clone_fn (std::move (constructor)), item.get_identifier ());
}
```

Since this is a Rust blogpost, and I do not mean to cause you heart palpitations by forcing you to read C++ code, here is an equivalent Rust implementation:

```rust
fn visit_tuple(&mut self, item: &TupleStruct) {
    let cloned_fields = item
        .get_fields()
        .enumerate()
        .map(|(idx, _)| {
            self.clone_call(
                self.builder.reference(self.builder.tuple_idx("self", idx)))
        })
        .collect::<Vec<Expr>>();

    let path = self.builder.path_in_expression(item.get_identifier());
    let constructor = self.builder.call(path, cloned_fields);

    self.expanded =
        self.clone_impl(self.clone_fn(constructor), item.get_identifier());
}
```

Using the equivalent `gccrs` witchy option to `-Zunpretty=expanded`, we can have a look at the generated code. Let's reuse a previously defined tuple struct.

```rust
#[derive(Clone)]
struct StringPair(String, String)
```

Passing the above snippet to `build/gcc/crab1 test.rs -frust-incomplete-and-experimental-compiler-do-not-use -frust-dump-all` conjures the following code:

```rust
impl Clone for StringPair {
	fn clone(&self) -> Self {
		StringPair(
			Clone::clone(
				&self.0,
			),
			Clone::clone(
				&self.1,
			),
		)
	}
}

struct StringPair(String, String);
```

The generated code is very similar for regular structs, where instead of creating "tuple index" expressions, we create "field access" expressions.

If we look at our implementation of `DeriveClone::visit_union(Union &item)`, we can see the following:

```cpp
void
DeriveClone::visit_union (Union &item)
{
  // <Self>
  auto arg = GenericArg::create_type (builder.single_type_path ("Self"));

  // AssertParamIsCopy::<Self>
  auto type = std::unique_ptr<TypePathSegment> (
    new TypePathSegmentGeneric (PathIdentSegment ("AssertParamIsCopy", loc),
				false, GenericArgs ({}, {arg}, {}, loc), loc));
  auto type_paths = std::vector<std::unique_ptr<TypePathSegment>> ();
  type_paths.emplace_back (std::move (type));

  // AssertParamIsCopy::<Self> with the right smart pointer type
  auto full_path
    = std::unique_ptr<Type> (new TypePath ({std::move (type_paths)}, loc));

  auto stmts = std::vector<std::unique_ptr<Stmt>> ();

  // let _: AssertParamIsCopy::<Self>
  stmts.emplace_back (
    builder.let (
      /* wildcard pattern `_` */ builder.wildcard (),
      /* type `: <full_path>` */ std::move (full_path),
      /* no init expression */ nullptr));

  // *self
  auto tail_expr = builder.deref (builder.identifier ("self"));

  // {
  //     let _: AssertParamIsCopy::<Self>;
  //     *self
  // }
  auto block = builder.block (std::move (stmts), std::move (tail_expr));

  expanded = clone_impl (clone_fn (std::move (block)), item.get_identifier ());
}
```

Even though it is a bit convoluted, we can kind of follow through the creation of the `AssertParamIsCopy` type and its associated let statement and dereference tail expression.

Aaaaaand that is enough C++ for the day!

But if you find this kind of thing interesting, and would like to try your hand at implementing `#[derive(Eq)]` or `#[derive(Default)]`, please come chat and work with us on the compiler! The existing frameworks should provide a good base for someone getting started, and there are a lot of derive macros left :) You can have a look at [our implementation for `#[derive(Copy)]`](https://github.com/Rust-GCC/gccrs/blob/cc09d0bf04fd87afb9f2b717d485a380a05e0a73/gcc/rust/expand/rust-derive-copy.cc), which is really simple.

And finally,

## Today is my dog's birthday!

![Jinko](/jinko.jpg)

He wishes you all a nice day.

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
