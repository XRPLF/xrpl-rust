# Contributing

## Setup Your Development Environment

If you want to contribute code to `xrpl-rust`, the following sections describe 
how to set up your developer environment.

### Setup the Rust/Cargo Environment

Getting started with Rust and `xrpl-rust` is easy. To install `rust` and 
`cargo` follow these steps:

* Install [`rust`](https://doc.rust-lang.org/cargo/getting-started/installation.html):

        curl https://sh.rustup.rs -sSf | sh

* Update rust using `rustup` and install a few development dependencies:

        // Rustup
        rustup update
        rustup component add rustfmt
        rustup component add clippy-preview

        // Cargo
        cargo install cargo-audit

### Git `pre-commit` Hooks

To run linting and other checks, `xrpl-rust` uses 
[`pre-commit`](https://pre-commit.com/).

> This should already be setup thanks to 
[`cargo-husky`](https://github.com/rhysd/cargo-husky)

### Run the Formatter

To run the linter:

```bash
cargo fmt
```

> Note that the formatter will automatically run via pre-commit hook

### Run the Linter

To run the linter:

```bash
cargo clippy
```

> Note that the linter will automatically run via pre-commit hook

### Running Tests

To run tests:

```bash
# Test the core feature for no_std
cargo test --no-default-features --features core
# Test all features enabled
cargo test --all-features
```

> Note that the tests will automatically run via pre-commit hook

### Generate Documentation

You can see the complete reference documentation at 
[`xrpl-rust` docs](https://docs.rs/xrpl). 

You can also generate them locally using `cargo`:

```bash
cargo doc
```

### Audit Crates

To test dependencies for known security advisories, run:

```bash
cargo audit
```

### Submitting Bugs

Bug reports are welcome. Please create an issue using the default issue
template. Fill in *all* information including a minimal reproducible 
code example. Every function in the library comes with such an example
and can adapted to look like the following for an issue report:

```rust
// Required Dependencies
use xrpl::core::keypairs::derive_keypair;
use xrpl::core::keypairs::exceptions::XRPLKeypairsException;

// Provided Variables
let seed: &str = "sn259rEFXrQrWyx3Q7XneWcwV6dfL";
let validator: bool = false;

// Expected Result
let tuple: (String, String) = (
    "ED60292139838CB86E719134F848F055057CA5BDA61F5A529729F1697502D53E1C".into(),
    "ED009F66528611A0D400946A01FA01F8AF4FF4C1D0C744AE3F193317DCA77598F1".into(),
);

// Operation
match derive_keypair(seed, validator) {
    Ok(seed) => assert_eq!(tuple, seed),
    Err(e) => match e {
        XRPLKeypairsException::InvalidSignature => panic!("Fails unexpectedly"),
        _ => (),
    },
};
```
> This format makes it easy for maintainers to replicate and test against.

## Release Process

### Editing the Code

* Your changes should have unit and/or integration tests.
* New functionality should include a minimal reproducible sample.
* Your changes should pass the linter.
* Your code should pass all the actions on GitHub.
* Open a PR against `main` and ensure that all CI passes.
* Get a full code review from one of the maintainers.
* Merge your changes.

### Release

TODO
