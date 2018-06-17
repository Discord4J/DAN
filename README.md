# DAN (Discord Audio Native)
**Discord Audio Native (DAN)** is an implementation of the Discord UDP Socket for
[Voice Connections](https://discordapp.com/developers/docs/topics/voice-connections) written in
[Rust](https://www.rust-lang.org/en-US/) and exposed using [C](https://en.wikipedia.org/wiki/C_(programming_language)).

**This library is currently extremely experimental!**

## Features
* [IP Discovery](https://discordapp.com/developers/docs/topics/voice-connections#ip-discovery)
* Supports Multiple Platforms*
* Guaranteed Runtime Safety**
* No Garbage Collector
* Flexible Implementation
* Language Agnostic
* Library Agnostic
---
\* Limited by platforms supported by Rust.  
\** Only guaranteed if using `net` module.

## TODO
* Documentation
* More Bindings
* Testing
