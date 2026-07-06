# Stage 1: Build
FROM rust:1.85-alpine AS builder

WORKDIR /usr/src/checkup

# Install build dependencies
RUN apk add --no-cache musl-dev gcc sqlite sqlite-dev build-base

# Leverage Docker cache mounts for cargo registries and the target directory.
# We also create a dummy sqlite database from the migrations to satisfy SQLx compile-time query verification.
RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=migrations,target=migrations \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/checkup/target \
    sqlite3 /tmp/db.sqlite "VACUUM;" && \
    cat migrations/*.sql | sqlite3 /tmp/db.sqlite && \
    DATABASE_URL=sqlite:///tmp/db.sqlite cargo build --release && \
    cp target/release/checkup /usr/src/checkup/checkup

# Stage 2: Final minimal image
FROM alpine:3.20

# Install SSL certificates for outbound pings/HTTPS requests
RUN apk add --no-cache ca-certificates

WORKDIR /app

# Copy binary from builder stage (copied out of the cache mount)
COPY --from=builder /usr/src/checkup/checkup /app/checkup

# Expose port
EXPOSE 80

# Set environment variables defaults
ENV PORT=80
ENV DATABASE_URL=sqlite:///app/database.db
ENV JWT_SECRET=super-secret-jwt-key

CMD ["/app/checkup"]
