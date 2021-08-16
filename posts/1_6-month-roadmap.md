I thought it would be nice to put together a 6 month roadmap for me to follow and try to
keep to. The roadmap covers topics on which I'd like to work or improve, which are mainly
centered on programming.

I am planning for my work to be mainly focused around languages, interpreted or compiled.

__jinko__:

I'd like for the language to get to a usable state in that period of time. This means:

* Fixing inclusions, namespaces and "paths"
* Adding more to the standard library (`Options`, `Iters`, `Results`...)
* Upgrade to `nom` 6, as it is a breaking change and a bit hard to work around
* Add type checking, which is one of the main goals of the language
* Add proper errors, displaying the location of the line in question.
    * Also prettifying the errors for the user, as they are a pain right now
* Get generics working? They are a core part of the standard library and really useful
for interpreted languages in general, and especially for writing interpreters.
* Actually handle `test` functions, and add assertions to the standard library
* Add an implementation of vectors in the language, keeping track of the size and the items
* Add a cast function using generics or a jinko instruction, something like `cast<T>(value)`
or `value @cast(T)`
* Update the syntax and documentation of the language

__gccrs__:

I'd like to work on adding more useful stuff to the compiler, in order to bring us closer
to interacting with real rust code and real rust projects. While macros are still a way
off, there are still plenty of projects that do not rely on them or only on simple ones.

* Support for external modules
    * Fix location errors on errors in external files
    * Use the `#[path]` attribute
    * Handle directory ownership correctly when resolving module filenames
* Name mangling
    * Fix the legacy name mangling implementation
    * Add v0 name-mangling implementation
* Get more familiar with code-reviews and other parts of the code, such as the typechecker
* Figure out more stuff to work on :D

__Cycling__:

* Manage to do 160km in one day!
