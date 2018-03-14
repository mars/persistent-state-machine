Persistent State Machine
========================

A reference application written in Rust language providing a pattern for a database-backed finite state machine.

Goals
-----

Create a reference application that is usable as the programmatic model for an on-line service, supporting concurrent execution
and stateless, horizontally scaling.

* implement a finite state machine in pure Rust
  * no libs, just the type system; [inspiration](https://hoverbear.org/2016/10/12/rust-state-machine-pattern/)
* use a database connection pool to support many simultaneous state machines

Design
------

A simple life model to illustrate a state machine: a life that is either gestating, alive, or dead.

* enforce possible states and allow pattern matching of states with a Rust **fat enum** at the top-level
* enum variants each contain an **eponymous type** (Gestating, Alive, Dead) that embodies that phase of life
* state machine transitions performed `into` by implementing **From** on variants' types
* Each variant's child type has a state field containing:
  * a database record of its persistent attributes including
    * `state_type`: the textual name of the parent variant's type; used to construct model from database result
    * other use-case specific attributes: `created_at`, `updated_at`, `born_at`, `died_at`


```
    +---------------------------------+              +-----------------------------+             +----------------------------+
    |                                 |              |                             |             |                            |
    | enum variant Phase::Gestating   |              | enum variant Phase::Alive   |             | enum variant Phase::Dead   |
    |                                 |              |                             |             |                            |
    | +-----------------------------+ |              | +-------------------------+ |             | +------------------------+ |
    | |                             | |              | |                         | |             | |                        | |
+-----+ Gestating                   +------------------->Alive                   | +---------------->Dead                  <---------+
|   | |                             | |         into | |                         | |        into | |                        | |      |
|   | | state: Life (database record) |              | | state: Life (database record)           | | state: Life (database record)   |
|   | |        state_type: "Gestating"|              | |        state_type: "Alive"|             | |        state_type: "Dead"|      |
|   | |        born_at: none        | |              | |        born_at: some time |             | |        born_at: some time|      |
|   | |        ...                  | |              | |        ...              | |             | |        ...             | |      |
|   | |                             | |              | |                         | |             | |                        | |      |
|   | |                             | |              | |                         | |             | |                        | |      |
|   | +-----------------------------+ |              | +-------------------------+ |             | +------------------------+ |      |
|   +---------------------------------+              +-----------------------------+             +----------------------------+      |
|                                                                                                                                    |
|                                                                                                                                    |
+------------------------------------------------------------------------------------------------------------------------------------+
                                                into
```


Requirements
------------

* [Rust + Cargo](https://www.rust-lang.org)
* [Postgres](https://www.postgresql.org/download/)
* `cargo install diesel_cli --no-default-features --features postgres`

Usage
-----

```bash
cp .env.sample .env
diesel setup
```

Now, build & test the program:

```
cargo test
```
