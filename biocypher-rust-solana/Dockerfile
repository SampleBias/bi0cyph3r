# Bi0cyph3r terminal workbench
FROM rust:bookworm AS build
WORKDIR /build

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend

# Build release binary
RUN cargo build --locked --release --bin bi0cyph3r

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /build/target/release/bi0cyph3r /usr/local/bin/
ENV TERM=xterm-256color
WORKDIR /data
ENTRYPOINT ["bi0cyph3r"]
