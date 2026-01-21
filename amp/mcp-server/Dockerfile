FROM rust:latest as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml ./
COPY mcp-server/Cargo.toml ./mcp-server/

# Copy source
COPY mcp-server/src ./mcp-server/src

# Build release
RUN cd mcp-server && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/mcp-server/target/release/amp-mcp-server /usr/local/bin/

ENTRYPOINT ["amp-mcp-server"]
