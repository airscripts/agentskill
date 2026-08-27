CARGO ?= cargo
ACTIONLINT ?= actionlint
SHELLCHECK ?= shellcheck

.PHONY: all build check coverage fmt lint security test verify workflows

all: verify

build:
	$(CARGO) build --release --workspace --locked

check:
	$(CARGO) check --workspace --locked

coverage:
	$(CARGO) llvm-cov --workspace --locked --summary-only --fail-under-lines 80

fmt:
	$(CARGO) fmt --all

lint:
	$(CARGO) clippy --workspace --all-targets --locked -- -D warnings

security:
	$(CARGO) deny check

test:
	$(CARGO) test --workspace --locked

verify: lint check test workflows

workflows:
	$(ACTIONLINT) -color
	$(SHELLCHECK) --shell=bash agentskill-scripts/*.sh
