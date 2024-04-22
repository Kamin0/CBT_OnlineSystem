# Use the official Rust Docker image as the base image
FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files to leverage Docker layer caching
COPY Cargo.toml Cargo.lock ./

# Create an empty dummy project and build dependencies to cache them
RUN mkdir src && \
    echo "fn main() {println!(\"dummy\")}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy the entire project into the container
COPY . .

# Build the application
RUN cargo build --release

# Start a new build stage to keep the final image size small
FROM debian:buster-slim

# Install OpenSSL and libpq for Diesel to work
RUN apt-get update && \
    apt-get install -y libssl-dev libpq-dev && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the built executable from the previous stage
COPY --from=builder /usr/src/app/target/release/api ./

# Expose the port that your Actix-web application listens on
EXPOSE 8000

# Command to run the application
CMD ["./api"]
