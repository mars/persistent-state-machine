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
