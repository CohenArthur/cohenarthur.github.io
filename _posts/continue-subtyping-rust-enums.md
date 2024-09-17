_NOTE: Remove talks about order of variants in source enum since we are gonna work around that_

Friend, anything past an SSA form is waaaaaay above my pay-grade. If you want stuff to run you should talk to LLVM or something.

But our reviewer is right. What happens after typechecking? We cannot simply convert from one type to another willy-nilly. I mean, we *can* ~~core::mem::transmute~~, sometimes, but really we shouldn't, so we won't. To understand why, we need to look The first issue is that our source and destination enum might not have the same size, despite one being a subset of the other.

_NOTE: Look at layout of Rust enums_
_NOTE: Add examples from playground + LLVM IR_
