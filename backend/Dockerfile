# Backend Dockerfile
FROM rust:1.93-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /app/target/release/mana-panel-backend /app/mana-panel

# Create data directory
RUN mkdir -p /app/data

# Environment variables
ENV HOST=0.0.0.0
ENV PORT=3000
ENV DATABASE_URL=sqlite:/app/data/mana.db?mode=rwc
ENV JWT_SECRET=change-this-in-production
ENV JWT_EXPIRY_HOURS=24
ENV RUST_LOG=mana_panel_backend=info

EXPOSE 3000

# To enable Docker management, mount the host Docker socket at runtime:
#   docker run -v /var/run/docker.sock:/var/run/docker.sock ...
# The panel gracefully degrades if the socket is not available.
VOLUME ["/var/run/docker.sock"]

CMD ["/app/mana-panel"]
