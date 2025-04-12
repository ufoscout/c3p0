[![crates.io](https://img.shields.io/crates/v/c3p0.svg)](https://crates.io/crates/c3p0)
![Build Status](https://github.com/ufoscout/c3p0/actions/workflows/build_and_test.yml/badge.svg)
[![codecov](https://codecov.io/gh/ufoscout/c3p0/branch/master/graph/badge.svg)](https://codecov.io/gh/ufoscout/c3p0)


# "A pleasure to meet you. I am C-3p0, JSON-DB Relations."

__C3p0__: "Hello, I don't believe we have been introduced.
     A pleasure to meet you. I am C-3p0, JSON-DB Relations."

Are you playing with Postgres and you like it? 

Do you think JSON is excellent, but it could be better handled in your DB code?

whether you like [tokio-postgres](https://crates.io/crates/tokio-postgres) or 
[Sqlx](https://crates.io/crates/sqlx), 
C3p0 brings you a set of tools to simplify JSON integration in your database workflow.

So, if you would like to be able to fetch/delete/insert/update JSON object interactively with your Sql DB like it was a NoSQL DB, then keep reading!


## What C3p0 is not

Although it provides a high-level interface for basic CRUD operations, _C3p0_ is neither an ORM nor a replacement for one—or for any similar tool. In fact, _C3p0_ is not an ORM at all. It allows storing and retrieving JSON objects from the database, but it does not manage cross-table relationships. Each object type is stored in its own dedicated table, using a single column of JSON type.

__C3p0__: "I see, Sir Luke".

Great!


## What C3p0 is

_C3p0_ is a library designed for integrating JSON data with relational databases. It offers the following capabilities:
- Performs basic CRUD operations on JSON objects
- Automatically generates the necessary SQL queries to interact with database tables, without relying on macros
- Supports PostgreSQL (via tokio-postgres or sqlx), as well as MySQL and SQLite (via sqlx)


## Prerequisites

It uses async closures, so it requires at least Rust version 1.85.


## History
The first _C3p0_ version was written in Java...

__C3p0__: "If I told you half the things I’ve heard about this Jabba the Hutt, you’d probably short circuit."

I said "Java", "Ja"-"va". Stay focused, please!

Anyway, Java is slowly showing its age, and we got a bit bored about it.

__C3p0__: "they're using a very primitive dialect".

Indeed.

On the contrary, our interest in the Rust programming language has kept growing over time;
so, we experimented with it more and more and, finally, migrated some critical portions of our code to Rust.

Just said, we love it.

We believe that Rust is a better overall language.

__C3p0__: "The city's central computer told you?"
 
Yes! It allows us to achieve better resource usage, to avoid the garbage collector and the virtual machine,
and, at the same time, to get a better and safer concurrency level.


## Can I use it in production?
__Han__ "Don't worry. Everything's gonna be fine. Trust me."

__C3p0__: "Every time he employs that phrase, my circuitry becomes erratic!"

__Han__: ???

__C3p0__: "Artoo says that the chances of survival are 725 to 1".

By the way, regardless of what Artoo said, we've survived using it in production since 2018.


## Example of usage

```rust
use c3p0::prelude::*;

#[tokio::main]
async fn main() {
    let db = C3p0::new("postgres://postgres:postgres@localhost:5432/postgres").await.unwrap();
    let json = db.get("users", "1").await.unwrap();
    println!("{}", json);
    db.delete("users", "1").await.unwrap();
    db.insert("users", "1", json).await.unwrap();
    db.update("users", "1", json).await.unwrap();
    let json = db.get("users", "1").await.unwrap();
    println!("{}", json);
}

```
