default: lint

# Formatting
fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

# Linting
clippy:
  cargo clippy --workspace --all-features --locked -- -D warnings

# Includes tests/examples/benches; requires nightly for this repo.
clippy-all:
  cargo +nightly clippy --workspace --all-targets --all-features --locked -- -D warnings

clippy-fix:
  cargo clippy --fix --allow-dirty --allow-staged --workspace --all-features --locked -- -D warnings

# Includes tests/examples/benches; requires nightly for this repo.
clippy-fix-all:
  cargo +nightly clippy --fix --allow-dirty --allow-staged --workspace --all-targets --all-features --locked -- -D warnings

# Common workflows
lint: fmt clippy

fix: fmt clippy-fix
