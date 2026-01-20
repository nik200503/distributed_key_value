# STAGE 1: Build the binary
# We use the official Rust image
FROM rust:latest as builder

# Create a new empty shell project
WORKDIR /usr/src/app
COPY . .

# Build for release (optimized for speed)
RUN cargo build --release

# STAGE 2: Create the runtime image
# We use a lightweight Linux (Debian Slim)
FROM debian:bookworm-slim

# Install OpenSSL (needed for some Rust networking libs, though we mostly use std)
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the binaries from Stage 1
COPY --from=builder /usr/src/app/target/release/kvs-server /usr/local/bin/kvs-server
COPY --from=builder /usr/src/app/target/release/kvs-client /usr/local/bin/kvs-client

# Default command (can be overridden)
CMD ["kvs-server"]
