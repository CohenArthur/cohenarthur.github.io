# Introduction

## Who am I

My name is Arthur Cohen, I'm a french computer science student working on Embedded
development and Programming languages. I'm trying to contribute to open source software
while learning as much as I can.

## The aim of this blog

This blogs aims to be a collection of stuff that I am working on, or found interesting. On
top of that, I think that putting some of my thought process and work into words will help
me improve as a Developer. Looking back on the mistakes you make is an excellent way to
progress.

For example, I'm currently working on two "programming languages": A scripted intermediate
representation called [STIR](https://github.com/cohenarthur/stir) and a more classic
interpreted language, [broccoli](https://github.com/cohenarthur/broccoli), which would
ultimately use the STIR representation. I chose to tackle these projects in completely
opposite ways:

- For STIR, I started by creating the internal structures, such as those used to represent
a function call, a function definition, a loop, etc... without worrying about the syntax
at all.
- For broccoli, I'm starting with the parser and syntax, and limiting myself to a very
generic [Instruction](https://github.com/CohenArthur/broccoli/blob/master/src/instruction/mod.rs)
data structure, that should ultimately be used to represent all directives in a program.

I do not know yet which method is best, and I believe that putting some of the experience
into blog posts will make it easier for me to reflect on it, as well as for people to
learn from my mistakes (or successes !).

Finally, I really enjoy teaching and helping people understand programming. If this blog
can help someone in any way when it comes to low-level programming, then I'll be happy
and consider it a success.

## Why you should stay tuned

If you're interested in embedded software and programming languages, I might post something
that you'll find interesting ! If somehow some of my projects take off, you'll find updates
here as well.



-> [github](<a href=https://github.com/cohenarthur) <-

-> [linkedin](https://www.linkedin.com/in/arthur-cohen-2b15b5175/) <-
