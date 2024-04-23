# Use the official Rust Docker image as the base image
FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the entire project into the container
COPY . .

# Build the application
RUN cargo build --release

# Install OpenSSL and libpq for Diesel to work
RUN apt-get update && \
    apt-get install -y libssl-dev libpq-dev && \
    rm -rf /var/lib/apt/lists/*

# Expose the port that your Actix-web application listens on
EXPOSE 8000

# Command to run the application
CMD ["cargo", "run", "--release", "--bin", "api"]
