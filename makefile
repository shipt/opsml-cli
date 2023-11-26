PROJECT=opsml-cli
PYTHON_VERSION=3.11.2
SOURCE_OBJECTS=src


setup.project:
	poetry install --all-extras --with dev
	pip install maturin

test.unit:
	cargo test

test.all:
	cargo test -- --include-ignored

format:
	cargo fmt

lints:
	cargo clippy --workspace --all-targets -- -D warnings

