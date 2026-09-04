.PHONY: check test lint fmt validate simulate rust-test lean-test contracts-test test-all run-api manifest archive

check:
	cargo check --locked --workspace --all-targets

test: rust-test

rust-test:
	cargo test --locked --workspace

lean-test:
	cd lean && ./self-test.sh

contracts-test:
	cd contracts && forge test -vvv

test-all: validate simulate fmt lint rust-test lean-test contracts-test

lint:
	cargo clippy --locked --workspace --all-targets -- -D warnings

fmt:
	cargo fmt --all -- --check

validate:
	python3 scripts/validate_repo.py

simulate:
	python3 scripts/simulate_economics.py

run-api:
	cargo run -p xlemma-api

manifest:
	python3 scripts/generate_manifest.py

archive:
	python3 scripts/archive_repo.py
