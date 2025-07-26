# Use the official Rust nightly image as build environment (needed for edition 2024)
FROM rustlang/rust:nightly AS builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached unless Cargo files change)
RUN cargo build --release
RUN rm src/main.rs

# Copy the source code
COPY src ./src

# Build the application
RUN cargo build --release

# Start a new stage for the runtime image
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1001 appuser

# Set the working directory
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/app/target/release/Morphius-tt /app/

# Copy static files
COPY static ./static

# Change ownership to the app user
RUN chown -R appuser:appuser /app
USER appuser

# Expose the port the app runs on
EXPOSE 3000

# Run the application
CMD ["./Morphius-tt"]
