# downcast &emsp; ![Latest Version]

[Latest Version]: https://img.shields.io/crates/v/downcast.svg

> __NOTE__: It is recommended to use the [downcast-rs](https://crates.io/crates/downcast-rs) crate instead.
> It is more actively maintained and offers more features.
> 
> This crate is considered feature-complete and frozen at the current state.

A trait (& utilities) for downcasting trait objects back to their original types.

## [link to API documentation](https://docs.rs/downcast)

## example usage

Add to your Cargo.toml:

```toml
[dependencies]
downcast = "1.0"
```

Add to your crate root:

```rust
#[macro_use]
extern crate downcast;
```

* [simple](examples/simple.rs) showcases the most simple usage of this library.
* [with_params](examples/with_params.rs)  showcases how to deal with traits who have type parameters. 

## build features

* **std (default)** enables all functionality requiring the standard library (`Downcast::downcast()`).
* **nightly** enables all functionality requiring rust nightly (`Any::type_name()`).
