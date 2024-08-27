# Use the official Rust image as a build environment
FROM rust:1.70 as builder

# Create app directory
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Create a smaller image for the final application
FROM debian:buster-slim

# Copy the compiled binary from the build stage
COPY --from=builder /usr/src/app/target/release/websocket-server /usr/local/bin/rust-websocket-server

# Expose the port the application runs on
EXPOSE 3030

# Run the application
CMD ["rust-websocket-server"]
