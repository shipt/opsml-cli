PROJECT=opsml-cli
PYTHON_VERSION=3.11.2
SOURCE_OBJECTS=src


setup.project:
	poetry install --all-extras --with dev

test.unit:
	cargo test

format:
	cargo fmt

lints:
	cargo clippy --workspace --all-targets -- -D warnings

