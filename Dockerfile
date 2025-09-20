FROM rust:1.89.0-alpine3.22 AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev 
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo fetch
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc AS runner
WORKDIR /root
COPY --from=builder /app/target/release/rodan-sse .
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
CMD ["/root/rodan-sse"]
