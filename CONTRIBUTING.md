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

## Release Process

### Editing the Code

* Your changes should have unit and/or integration tests.
* Your changes should pass the linter.
* Your code should pass all the actions on Github.
* Open a PR against `main` and ensure that all CI passes.
* Get a full code review from one of the maintainers.
* Merge your changes.

### Release

TODO
