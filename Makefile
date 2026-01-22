# Makefile for RustKV

# Variables
CARGO = cargo
DOCKER = docker compose

# Default target
all: build

# 1. Check code quality
check:
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy -- -D warnings

# 2. Build binaries
build:
	$(CARGO) build --release

# 3. Run the cluster (Docker)
up:
	$(DOCKER) up --build

# 4. Stop the cluster
down:
	$(DOCKER) down

# 5. Run the Benchmark
bench:
	$(CARGO) run --release --bin kvs-bench

# 6. Clean up artifacts
clean:
	$(CARGO) clean
	rm -f *.db
