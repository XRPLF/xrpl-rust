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

To run linting and other checks locally before every commit, `xrpl-rust`
uses [`pre-commit`](https://pre-commit.com/). The hooks mirror the CI
`Build & Lint` job (`cargo fmt --all -- --check` and
`cargo clippy --all-features -- -D warnings`) plus a few baseline
file-hygiene checks.

Install `pre-commit` once (see the [official install
docs](https://pre-commit.com/#install) for other options):

```bash
pipx install pre-commit
# or: brew install pre-commit
# or: pip install --user pre-commit
```

Then register the hooks in your clone (one-time, per clone):

```bash
pre-commit install
```

From that point, every `git commit` runs the hooks against staged files.
To run them across the whole repository ad-hoc:

```bash
pre-commit run --all-files
```

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

For integration tests, we use an `xrpld` node in standalone mode to test xrpl-rust code against. To set this up, you can either configure and run `xrpld` locally, or set up the Docker container `rippleci/xrpld` by [following these instructions](#integration-tests). The latter will require you to [install Docker](https://docs.docker.com/get-docker/).

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
# Sets up the xrpld standalone Docker container — skip if you already have it running
docker run -p 5005:5005 -p 6006:6006 --rm -it --name xrpld_standalone \
  --volume "$PWD/.ci-config/:/etc/xrpld/" \
  rippleci/xrpld:develop --standalone
cargo test --release \
  --features std,json-rpc,helpers,cli,websocket,integration \
  -- --test-threads=1
```

To run a specific group of tests (e.g. escrow):

```bash
cargo test --release \
  --features std,json-rpc,helpers,cli,websocket,integration \
  escrow -- --test-threads=1
```

The feature set matches `.github/workflows/integration_test.yml`; `cli` and
`websocket` are required for `cli_integration.rs` and the websocket tests in
`utils.rs` to compile. `--test-threads=1` matches CI and prevents concurrent
tests from racing on the shared `xrpld` container.

Breaking down the `docker run` command:

- `-p 5005:5005 -p 6006:6006` exposes the HTTP JSON-RPC and WebSocket admin ports.
- `--rm` closes the container automatically when it exits.
- `-it` keeps stdin open so you can stop the node with Ctrl-C.
- `--name xrpld_standalone` is an instance name for clarity.
- `--volume $PWD/.ci-config/:/etc/xrpld/`: bind-mounts the host directory (left side) into the container (right side). `xrpld.cfg` lives in `$PWD/.ci-config/`, and this command is intended to be run from the root of the `xrpl-rust` project. The `xrpld` binary searches for its configuration file inside `/etc/xrpld/`. An absolute path is required, so we use `$PWD` instead of `./`.
- `rippleci/xrpld` is an image that is regularly updated with the latest `xrpld` releases (the binary formerly known as `rippled`; see xrpl.js PR #3270).
- `--standalone` starts `xrpld` in standalone mode, where ledgers only close on demand.

**Private xrpld images in CI**

Maintainers can run the CI integration tests against a private xrpld image by committing a version to `.github/xrpld-image.env`. Set `XRPLD_PRIVATE_VERSION` to the version without the `private-` prefix (for example, `XRPLD_PRIVATE_VERSION=3.3.0-rc2`); every workflow run triggered afterwards — including pull request runs — pulls `registry.gitlab.com/ripple/xrpledger/xrpld_package_deploy/xrpld-private:private-<version>`. Leave the value empty to use the default public image `rippleci/xrpld:develop`.

Private image access requires the repository variable `GITLAB_REGISTRY_USERNAME` and repository secret `GITLAB_REGISTRY_TOKEN`, configured with a GitLab deploy token that has `read_registry` access. These credentials are not exposed to fork pull requests, so a private version must be tested from a branch in this repository; a fork pull request with a private version set will fail fast rather than silently fall back.

**Notes**

- Integration tests are serialized via a global mutex — they do not run in
  parallel, so it is safe to run the whole suite at once.

### Coverage

Coverage is measured with [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov)
and uploaded to codecov under two separate flags:

- **unit** — pure-logic code (models, core, utils, `_serde`, signing). Built
  with a minimal feature set so network-bound modules are not compiled.
- **integration** — network-bound code (CLI, async clients, faucet, helpers
  under `account/`, `ledger/`, `transaction/`). Scoped via
  `--ignore-filename-regex` so unit-territory files do not dilute the
  integration metric.

Install the tool once:

```bash
cargo install cargo-llvm-cov --locked
```

#### Unit coverage

Matches `.github/workflows/unit_test.yml`:

```bash
cargo llvm-cov \
  --no-default-features --features std,core,utils,wallet,models \
  --summary-only \
  --fail-under-lines 83 \
  --fail-under-regions 85 \
  --fail-under-functions 73
```

The `--fail-under-*` flags mirror the thresholds CI enforces:

| Metric    | Threshold |
| --------- | --------- |
| Lines     | 83%       |
| Regions   | 85%       |
| Functions | 73%       |

#### Integration coverage

Requires the standalone `xrpld` container running (see [Integration Tests](#integration-tests)).
Matches `.github/workflows/integration_test.yml`:

```bash
# Collect coverage from the integration suite (writes raw profile data)
cargo llvm-cov --no-report --release \
  --features std,json-rpc,helpers,cli,websocket,integration \
  --test integration_test --test cli_integration --test funding \
  --test utils --test test_utils \
  -- --test-threads=1

# Generate lcov scoped to integration territory
cargo llvm-cov report --release --lcov --output-path lcov.info \
  --ignore-filename-regex '(_serde|core|models|utils)/|constants\.rs$|macros\.rs$|lib\.rs$|wallet/(mod|exceptions)\.rs$|tests/'
```

The codecov integration target is 65%. The `--ignore-filename-regex` value
mirrors `COVERAGE_IGNORE_REGEX` in the workflow; without it the integration
metric would be dominated by unit-territory files the integration suite is
not designed to exercise.

#### HTML report

For local exploration:

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

The workspace publishes three crates to crates.io. They depend on each other in one direction: `mpt-crypto-sys` → `mpt-crypto` → `xrpl-rust` (through its `confidential-mpt` feature). `xrpl-rust-macros` is released separately and only when it changes.

### 1. Choose the versions

Follow [semver](https://semver.org/). A breaking change to `xrpl-rust` bumps its major version. `mpt-crypto` and `mpt-crypto-sys` are below 1.0, so a breaking change to either bumps its minor version (0.1.x → 0.2.0). A crate whose dependency took a breaking bump usually takes one too, since the dependency's types appear in its own API.

### 2. Open a release PR

On a `release-<version>` branch from `main`:

- Bump `version` in the `Cargo.toml` of each crate being released.
- Update the `version` of every dependency on a released crate: `mpt-crypto-sys` in `internal/mpt-crypto/Cargo.toml` (under both `[dependencies]` and `[dev-dependencies]`), and `mpt-crypto` in the root `Cargo.toml`.
- In `CHANGELOG.md`, add a `## [[v<version>]]` heading below `## [[Unreleased]]`, so the unreleased entries become that version's notes.
- Check that every crate packages and builds as it will on crates.io:

  ```bash
  cargo publish --dry-run --allow-dirty -p mpt-crypto-sys -p mpt-crypto -p xrpl-rust
  ```

Get an approval and merge the PR before publishing.

### 3. Publish

Publishing needs crates.io ownership of each crate (ask an existing owner to run `cargo owner --add <github-user>`) and an API token with the `publish-update` scope, set up with `cargo login`.

From the merged `main`, publish bottom-up, because each crate needs the previous one to be on crates.io:

```bash
git checkout main && git pull
cargo publish -p mpt-crypto-sys
cargo publish -p mpt-crypto
cargo publish -p xrpl-rust
```

Skip any crate whose version didn't change. A published version can't be replaced or deleted, only yanked, so publish only from the merged commit.

### 4. Tag and create GitHub releases

On the merged commit, create one tag and GitHub release per published crate: `v<version>` for `xrpl-rust`, and `mpt-crypto-v<version>` / `mpt-crypto-sys-v<version>` for the others. Use the crate's `CHANGELOG.md` section as the release notes.

```bash
gh release create v2.0.0 --target <merge-commit-sha> --title v2.0.0 --notes-file <notes.md>
```

### Editing the Code

- Your changes should have unit and/or integration tests.
- New functionality should include a minimal reproducible sample.
- Your changes should pass the linter.
- Your code should pass all the actions on GitHub.
- Open a PR against `main` and ensure that all CI passes.
- Get a full code review from one of the maintainers.
- Merge your changes.
