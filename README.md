![crates.io](https://img.shields.io/crates/v/c3p0.svg)
![Build Status](https://github.com/ufoscout/c3p0/actions/workflows/build_and_test.yml/badge.svg)
[![codecov](https://codecov.io/gh/ufoscout/c3p0/branch/master/graph/badge.svg)](https://codecov.io/gh/ufoscout/c3p0)

# "A pleasure to meet you. I am C-3p0, JSON-DB Relations."


__C3p0__: "Hello, I don't believe we have been introduced.
     A pleasure to meet you. I am C-3p0, JSON-DB Relations."

Are you playing with Postgres and you like it? 

Do you think JSON is excellent, but it could be better handled in your DB code?

whether you prefer [rust-postgres](https://github.com/sfackler/rust-postgres) or 
[Diesel](https://github.com/diesel-rs/diesel), 
C3p0 brings you a set of tools to simplify JSON integration in your database workflow.

So, if you would like to:

- use any `serde_json::Serializable` struct as a valid field in your _Diesel_ models
- seamlessly integrate any `serde_json::Serializable` struct in your _rust-postgres_ code 
- automatically upgrade your schema in _rust-postgres_ as in _Diesel migration_  

then keep reading!

## What C3p0 is not

Even when it offers a high-level interface to perform basic CRUD operations,
it is not an ORM nor an alternative to Diesel or similar products.

__C3p0__: "I see, Sir Luke".

Great!


## How it works 

_C3p0_ is composed of a set of independent small Rust libraries for:
 - simplifying JSON-Postgres interactions
 - facilitating general schema management

_C3p0_ components:
- [c3p0_diesel_macro](c3p0_diesel_macro/README.md)
- [c3p0_pg](c3p0_pg/README.md)
- [c3p0_pg_migrate](c3p0_pg_migrate/README.md)

_C3p0_ components not ready yet:
- [c3p0_diesel](c3p0_diesel/README.md) This will be the
equivalent *c3p0_pg* build on _Diesel_. 


## Prerequisites

You must have Rust version 1.33 or later installed.


## History
The first _C3p0_ version was written in Java...

__C3p0__: "If I told you half the things I’ve heard about this Jabba the Hutt, you’d probably short circuit."

I said "Java", "Ja"-"va". Stay focused, please!

Anyway, Java is slowly showing its age, and we got a bit bored about it.

__C3p0__: "they're using a very primitive dialect".

Indeed.

On the contrary, our interest in the Rust programming language has kept growing over time;
so, experimented with it and, finally, migrated some critical portions of our code to Rust.

Just said, we love it.

We believe that Rust is a better overall language.

__C3p0__: "The city's central computer told you?"
 
Yes! It allows us
to achieve better resource usage,
to avoid the garbage collector and the virtual machine,
and, at the same time, to get a better and safer concurrency level.


## Can I use it in production?
__Han__ "Don't worry. Everything's gonna be fine. Trust me."

__C3p0__: "Every time he employs that phrase, my circuitry becomes erratic!"

__Han__: ???

__C3p0__: "Artoo says that the chances of survival are 725 to 1".
