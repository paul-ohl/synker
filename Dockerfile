# =============================================================================
# Stage 1 – Chef: install cargo-chef for dependency layer caching
# =============================================================================
FROM rust:1.88-slim-bookworm AS chef

RUN cargo install cargo-chef --locked
WORKDIR /app

# =============================================================================
# Stage 2 – Planner: compute the recipe (dependency fingerprint)
# =============================================================================
FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# Stage 3 – Builder: cache deps, then compile the release binary
# =============================================================================
FROM chef AS builder

# musl target for a fully static binary + cmake/perl for vendored libgit2 and openssl
RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-tools \
    pkg-config \
    cmake \
    make \
    perl \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies only – this layer is cached unless Cargo.toml/lock changes
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --features git2/vendored-openssl --recipe-path recipe.json

# Build the full project
COPY . .
RUN cargo build --release --locked --target x86_64-unknown-linux-musl --features git2/vendored-openssl

# =============================================================================
# Stage 4 – Runtime: minimal Alpine image, no shared lib dependencies needed
# =============================================================================
FROM alpine:3.21 AS runtime

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/synker /app/synker
COPY --from=builder /app/static /app/static
COPY --from=builder /app/templates /app/templates

EXPOSE 3000

ENV MODE=server

CMD ["/bin/sh", "-c", "/app/synker ${MODE}"]
