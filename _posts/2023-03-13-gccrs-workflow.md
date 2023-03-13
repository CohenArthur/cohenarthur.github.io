---
layout: post
title: "A gccrs workflow"
author: Arthur Cohen
tags:
    - gccrs
    - rust
---

I recently had a discussion with [Guillaume Gomez](https://github.com/guillaumegomez)
regarding contributing to GCC, and the process of submitting patches to various mailing lists.

As someone *extremely* involved in the development of `gccrs`, and also extremely unfamiliar
with patch-based workflows up until a few months ago, Guillaume
pointed out that it might be interesting for me to lay out some tips and tricks for working
on GCC. This led to the heap of characters you're about to read.

## Building GCC

Building GCC is quite an involved process, with a lot of gotchas and documentation-reading required. I strongly recommend reading through [the GCC wiki's section on building](https://gcc.gnu.org/wiki/InstallingGCC) before attempting this.

One of the most important note is the one about not building GCC directly within the root of the project. This will cause you pain. I'll put together what my usual workflow is for building a clean `gccrs` from a fresh `git clone`, and we can have a more detailed look at some of the steps.
I have annotated the most important lines with a number, to which I'll refer in the next section.

```shell-session
$ pwd
/home/arthur/Git/gccrs
$ mkdir build
$ cd build
$ ../configure # ...    # 1
$ cd ..
$ make -C build -j14    # 2
```

1. `../configure`

My entire `./configure` invocation looks like this:

> `../configure CC='ccache clang' CXX='ccache clang++' CXXFLAGS='-O0 -g -fno-pie' CFLAGS='-O0 -g -fno-pie' LDFLAGS='-no-pie -fuse-ld=mold' --enable-multilib --enable-languages=rust --disable-bootstrap`

It is the product of many, many hours building and re-building `gccrs`, in the hopes that my distribution's security measures will allow me to compile it and in the search of the shortest possible incremental build times.

As it stands, I was able to achieve the following timings on my machine (a 2022 Lenovo Laptop with an AMD Ryzen 7 PRO 5850U and 32GB of RAM). I have included the timing for compiling `gccrs` without changing the compiler or linker for reference, as this is what I was using when getting started on the project.
I have timed an incremental build by simply running `touch` on `gcc/rust/checks/errors/rust-unsafe-checker.cc` or on `gcc/rust/ast/rust-ast.h`, a very often included header.

|                      |Clean build|Incremental (.cc touch)|Incremental (.h touch)|
|----------------------|-----------|-----------------------|----------------------|
|basic ./configure line|     7m00s |                   13s |                  74s |
|cool ./configure line |     5m30s |                    2s |                   3s |

The `./configure` invocation can be broken down into a few parts.

1. `CC='ccache clang' CXX='ccache clang++'`

In order to achieve fast iteration on the project and short build times, I use the `clang`
compiler rather than `gcc` to build the project. `clang` is much, much faster than `gcc`,
and I don't have any particular opinion on C++ compilers, so it does just fine. As it stands,
`clang` also has a lot more warnings enabled by default - so while our `gcc`-based build
of `gccrs` is warning free, the `clang`-based one [isn't (or maybe it is, if you're from the
future)](https://gcc.gnu.org/bugzilla/show_bug.cgi?id=108111).

2. The various `-fno-pie` and `-no-pie` flags

These flags are simply to allow me to build the project on ArchLinux. There is also an
`--enable-static` option to `./configure`, which I believe is supposed to achieve the same
outcome, but I could not get it to work. I haven't looked into it more, and if it ain't broke
don't fix it. But this could probably be cleaned up and improved.

3. `LDFLAGS='-no-pie -fuse-ld=mold'`

You can see once again a flag related to Position Independant Executables (`pie`), for the same reason stated above. The more interesting bit is the use of `mold` to link GCC. I won't benchmark the various linkers I have tried, but it does make a nice noticeable difference, as well as adds color to linker errors :D

4. `--enable-languages=rust` and `--enable-multilib`

This flag only enables the compilation of the Rust frontend compiler (`gccrs` and `rust1`/`crab1`) as well as the default C frontend. This way, you won't end up compiling all default frontends for GCC, which include the C++ and Fortran compilers. Enabling multilib is useful for running tests in 32-bit mode.

5. `--disable-bootstrap`

For developing GCC, you do not need to make a full bootstrap build of the compiler each time. However, it is enabled by default, as most people who build GCC are simply interested in using the compiler, not modifying it. Make sure to disable bootstrapping unless you want to check you did not introduce any new unaccepted warnings or error. But for most intents in purposes, you do not *need* to do bootstrap builds to work on GCC.

However, you need to build the compiler often! To help check I did not break anything during various stages of my contributions, I will often keep multiple build directories with different `./configure` lines. My current list is as follows:

* `build` with the above-mentioned `./configure` line

This is my most-used build directory. I will run `make -C build -j14` every chance I get.
Having such a fast build time is very useful for checking other contributors' pull-requests as
well, or making sure I don't give someone a bogus C++ indication. As I mentioned above, this
compiles `gccrs` using `clang`, which notices a lot of warnings we haven't fixed yet, so I'll
usually ignore them. While this sounds extremely bad, we do have a CI workflow for checking new
warnings, so it's almost impossible to merge new code in the project which would introduce warnings.

* `build-gcc`, which builds using `ccache gcc` instead of `ccache clang`

Here, nothing changes apart from the compilers (`gcc` instead of `clang`). I still use `ccache` and `mold` to speed things up as much as possible.
 
> `../configure CC='ccache gcc' CXX='ccache g++' CXXFLAGS='-O0 -g -fno-pie' CFLAGS='-O0 -g -fno-pie' LDFLAGS='-no-pie -fuse-ld=mold' --enable-multilib --enable-languages=rust --disable-bootstrap`

Building with `gcc` from time to time allows me to make sure I haven't committed too many crimes, and is the prefered building method for most of our contributors.

* `build-bootstrap`, which performs a full bootstrap build of the compiler

I will disable `ccache` and `mold` for bootstrapping the compiler as it is going to be a lengthy process anyway. For bootstrap builds, our favorite Frenchman [Marc Poulhiès](https://github.com/dkm/) recommends against doing incremental builds, as it has caused him issues in the past. So make sure to remove the build directory before compilation.

> `../configure --enable-languages=rust`

Using `ccache` and `clang` does not help a lot with bootstrapping, as we will be building `g++` from scratch and using it to compile `gccrs`. This is a very, very long process, and I don't run bootstrap builds often, as they take around one hour to complete on my machine.

2. `make -C build -j14`

Finally, compiling! It's easier for me to compile `gccrs` from the root of the project,
as I'll often do multiple commands at once and am not very good at noticing when I change
directories. Since my machine has 16 threads, I use 14 of them when compiling anything, which
leaves me enough for other programs running on my computer without running out of cores or memory.

I often see first-time contributors running `make` in single-threaded mode (a default `make`
invocation with no argument). Since GCC contains *a lot* of files, using multiple cores helps
a lot. However, using `-j` without any number of jobs specified will allow `make` to create as
many jobs as it wants. And since GCC contains *a lot* of files, this will cause your computer to suffer.

## Editor support

Since GCC is hard, and programming overall is hard, and C++ in particular is ~~impossible
to write~~hard, I find it extremely helpful to rely on IDE-like features when working on
`gccrs`. I use [helix](github.com/helix-editor/helix) as my text editor, which ships with support for the Language Server Protocol.

However, since this is C++ and we can't have nice things, you first need to create a compilation database to allow your linter/analyzer/lsp-thingamajig to do its thing (in my case, `clangd`).

To do this, you can usually choose between `compiledb` and [`bear`](https://github.com/rizsotto/Bear).
I typically use `bear` for no particular reason. `bear` works by wrapping your
`make` invocation and performing dark-magic to extract the various flags and options you give your compiler.

However, because it's written in C++ and we can't have nice things, you will have
to run `bear` using only a single job in your make invocation. No worries, you only have to this once. Whenever I need to do this (very rarely, probably only when I get a new computer or accidentally deletes the compilation database), I usually start the command and go to lunch.

My bear command usually looks like this:

```shell-session
$ bear -- make -C build -j1
```

This will use the same options as specified in your `./configure` line and will allow you to get diagnostics and warnings according to your chosen compiler. But because this is C++ and you can't have nice things, the LSP will still not work with our parser implementation. And to be fair I don't really blame it since it's 17 000 lines of templated C++ code. But still!

## `git`

I used to have `neovim` as my editor, which allowed me to use the [fugitive](https://github.com/tpope/vim-fugitive) plugin to do git operations directly within my editor. However, since `helix` does not yet have support for plugins, I have switched to using `git` directly on the command-line.

One very ~~annoying~~important GCC step is writing Changelogs for your commits. They look like this and have a very specific format you need to respect:

```
commit 7394a6893dd9fc2bc34822e002b53eb200ff51d5
Author: Arthur Cohen <arthur.cohen@embecosm.com>
Date:   Wed Mar 1 11:03:24 2023 +0100

hir: Refactor ASTLoweringStmt to source file.

gcc/rust/ChangeLog:

        * Make-lang.in: Add rust-ast-lower-stmt.o
        * hir/rust-ast-lower-stmt.h: Move definitions to...
        * hir/rust-ast-lower-stmt.cc: ...here.
```

Because they are difficult to write, there are multiple scripts to help GCC hackers with them. The most basic one (but not the easiest one to use) is `contrib/mklog.py`. It takes the contents of a `patch` as input and will spew out the appropriate Changelog skeleton in your terminal.

```
gcc/rust/ChangeLog:

        * Make-lang.in:
        * hir/rust-ast-lower-stmt.h:
        * hir/rust-ast-lower-stmt.cc:
```

You can then copy this skeleton and fill it out when commiting your changes. Aaaaaaand because it's 2023 and no one can agree on whether one should use spaces or tabs, you're getting a CI failure on our "check-changelogs" step.

Thankfully there is an easier way to generate Changelog skeletons directly when making your commits. Enter `contrib/gcc-git-customization.sh`. This nifty little script will help you setup some `git` config variables such as your name, as well as setup branch names for specific GCC workflows, but most importantly for us it will add the "gcc-commit-mklog" `git` subcommand.

So you'll now be able to replace `git commit` with `git gcc-commit-mklog` and have Changelog skeletons directly embedded in your commits! Since this is just a wrapper around `git commit`, you can use all of the other commit options such as `--amend`, `-s`, `-m`... Often, I'll create commits without a Changelog by accident and run `git gcc-commit-mklog --amend` to fix them.

In case you're still unsure about whether or not your Changelogs are correct (as I often am), you can use `contrib/gcc-changelog/git_check_commit.py`. This script takes as input a commit or rev-list of multiple commits and will check each message's format. I'll often run the script before pushing my changes, as even with `git gcc-commit-mklog` you might create lines that are too long or forget some files that were changed.
I usually run one of the following two commands

```shell-session
$ contrib/gcc-changelog/git_check_commit.py $(git log -1 --format=%h)
$ # or
$ contrib/gcc-changelog/git_check_commit.py upstream/master..HEAD
```

However, if you are contributing to `gccrs`, we have GitHub actions in place which will check the format of your commits. So you do not need to worry about pushing commits with invalid Changelog entries! We also want to ensure that each commit builds and passes the testsuite, and are working towards adding more CI for this. In the meantime, if your PR contains multiple commits, or if you're unsure about the state of each one since you've spent a lot of time splitting commits and rebasing them and squashing and fixup'ing and... well you can use the following command, which will build each commit and run our testsuite.

```  
$ git rebase master -x 'make -C build -j4' \
    -x 'make -C build check-rust' \
    -x 'if grep \'unexpected\' build/gcc/testsuite/rust/rust.sum; then false; else true; fi'
```

Since the various `check-*` recipes will not return a non-zero exit-code on unexpected failures, you need to add the extra `grep` at the end. I think this can be simplified into `! grep '\unexpected\' build/gcc/testsuite/rust/rust.sum` but I wouldn't trust anything I say regarding shell scripting.

There are probably tips that I missed, so feel free to get in touch to let me know about all of the good tricks :)
If you want even more compilation tips, or explanation about various options, you should have a look at [`gccrs`' README](https://github.com/rust-gcc/gccrs), which contains a lot of extra info :) If you have any questions, you can also come chat with us on our Zulip!

<br>
<br>
<br>
<br>
<p style="font-family:'Source Code Pro'">
<span style="color:#d784f3">type</span> <a href="https://github.com/cohenarthur">GitHub = <span style="color:#69c908">"/CohenArthur"</span></a>;<br>
<span style="color:#d784f3">type</span> <a href="https://twitter.com/cohenarthurdev">Twitter = <span style="color:#69c908">"/CohenArthurDev"</span></a>;
</p>
