# Multi-stage build for Linux binary
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Build the release binary
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Install wscat for WebSocket testing (use compatible version)
RUN npm install -g wscat@5.1.1 || npm install -g wscat

# Copy the binary from builder stage
COPY --from=builder /app/target/release/trading_bot /usr/local/bin/trading_bot
COPY --from=builder /app/config.env /app/config.env

WORKDIR /app

# Create logs directory
RUN mkdir -p ollama_logs

ENTRYPOINT ["trading_bot"]