# Contributing

## Setup Your Development Environment

If you want to contribute code to `xrpl-rust`, the following sections describe
how to set up your developer environment.

### Setup the Rust/Cargo Environment

Getting started with Rust and `xrpl-rust` is easy. To install `rust` and
`cargo` follow these steps:

- Install [`rust`](https://doc.rust-lang.org/cargo/getting-started/installation.html):

        curl https://sh.rustup.rs -sSf | sh

- Update rust using `rustup` and install a few development dependencies:

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
> [`cargo-husky`](https://github.com/rhysd/cargo-husky)

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

For integration tests, we use a `rippled` node in standalone mode to test xrpl-rust code against. To set this up, you can either configure and run `rippled` locally, or set up the Docker container `rippleci/rippled` by [following these instructions](#integration-tests). The latter will require you to [install Docker](https://docs.docker.com/get-docker/).

#### Unit Tests

```bash
# Test with default features
cargo test --release
# Test for no_std
cargo test --release --no-default-features --features embassy-rt,core,utils,wallet,models,helpers,websocket,json-rpc
```

> Note that the tests will automatically run via pre-commit hook

#### Integration Tests

From the `xrpl-rust` folder, run the following commands:

```bash
# Sets up the rippled standalone Docker container — skip if you already have it running
docker run -p 5005:5005 -p 6006:6006 --rm -it --name rippled_standalone \
  --entrypoint bash rippleci/rippled:develop \
  -c 'mkdir -p /var/lib/rippled/db/ && rippled -a'
cargo test --release --features integration,std,json-rpc,helpers
```

To run a specific group of tests (e.g. escrow):

```bash
cargo test --release --features integration,std,json-rpc,helpers escrow
```

Breaking down the `docker run` command:

- `-p 5005:5005 -p 6006:6006` exposes the HTTP JSON-RPC and WebSocket admin ports.
- `--rm` closes the container automatically when it exits.
- `-it` keeps stdin open so you can stop the node with Ctrl-C.
- `--name rippled_standalone` is an instance name for clarity.
- `--volume $PWD/.ci-config:/etc/opt/ripple/` mounts `rippled.cfg` so the node binds on `0.0.0.0` and is reachable from the host. It must be an absolute path, so we use `$PWD` instead of `./`.
- `rippleci/rippled` is an image that is regularly updated with the latest `rippled` releases.
- `--entrypoint bash rippleci/rippled:develop` manually overrides the entrypoint (for the latest version of rippled on the `develop` branch).
- `-c 'mkdir -p /var/lib/rippled/db/ && rippled -a'` starts `rippled` in standalone mode, where ledgers only close on demand.

**Notes**

- Integration tests are serialized via a global mutex — they do not run in
  parallel, so it is safe to run the whole suite at once.

### Coverage

Coverage is measured with [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov).

Install the tool and run a coverage report locally:

```bash
cargo install cargo-llvm-cov --locked
cargo llvm-cov --summary-only
```

The CI enforces the following minimum thresholds (current baseline is ~78% lines / ~68% regions / ~75% functions, measured with default features only — integration tests are excluded from coverage):

| Metric    | Minimum |
|-----------|---------|
| Lines     | 75%     |
| Regions   | 65%     |
| Functions | 72%     |

To generate an HTML report and open it in a browser:

```bash
cargo llvm-cov --open
```

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
template. Fill in _all_ information including a minimal reproducible
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

1. Create a processing branch `process/[VERSION]`
2. Brach management:

- If this is a new version, increment the version in the `Cargo.toml` and target `main`.
- If this a patch release, chery-pick commits being released and target `versions/v[major]`.

3. Collect required merge approvals.
4. Merge release PR.
5. Tag release.
6. [TODO automate] Run `cargo publish`.

### Editing the Code

- Your changes should have unit and/or integration tests.
- New functionality should include a minimal reproducible sample.
- Your changes should pass the linter.
- Your code should pass all the actions on GitHub.
- Open a PR against `main` and ensure that all CI passes.
- Get a full code review from one of the maintainers.
- Merge your changes.
