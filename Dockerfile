FROM rust:latest as builder
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y sqlite3 libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Copy binaries from builder
COPY --from=builder /build/target/release/ironcage-kernel /app/
COPY --from=builder /build/target/release/ironcage-api /app/

# Create data directory
RUN mkdir -p /data

EXPOSE 3000 3001

# Default to kernel; can override with CMD
CMD ["/app/ironcage-kernel"]
