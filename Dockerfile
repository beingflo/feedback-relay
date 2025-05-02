# Builder stage
FROM rust:bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim AS runtime
RUN apt update && apt install -y ca-certificates && apt install -y openssl

WORKDIR /app
# Copy the compiled binary from the builder environment # to our runtime environment
COPY --from=builder /app/target/release/feedback-relay feedback-relay
ENTRYPOINT ["./feedback-relay"]