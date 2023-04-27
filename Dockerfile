# We use the latest Rust stable release as base image
# Builder stage
FROM rust:1.69.0 AS builder
# Let's switch our working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created for us by Docker in case it does not
# exist already.
WORKDIR /app
# Install the required system dependencies for our linking configuration
RUN apt update && apt install lld clang -y
# Copy all files from our working environment to our Docker image
COPY . .
# Force sqlx to look at the saved metadata(sqlx-data.json) instead of trying to query a live database
ENV SQLX_OFFLINE true
# Let's build our binary!
# We'll use the release profile to make it faaaast
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim AS runtime
WORKDIR /app
# Install OpenSSL - it is dynamically linked by some of our dependencies
# Install ca-certificates - it is needed to verify TLS certificates
#  when establishing HTTPS connections
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
# Copy the compiled binary from the builder environment to our runtime environment
COPY --from=builder /app/target/release/zero-2-prod zero-2-prod
# We need the configuration file at runtime!
COPY configuration configuration
# Instruct the binary in our Docker image to use the production configuration
ENV APP_ENV prod
# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./zero-2-prod"]