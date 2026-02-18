FROM rust:1.76 AS builder
WORKDIR /app
COPY rust/ ./rust/
WORKDIR /app/rust
RUN cargo build --release --bin worker

FROM ubuntu:24.04
RUN apt update && apt install -y libssl-dev ca-certificates curl iputils-ping && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/rust/target/release/worker /usr/local/bin/worker

CMD ["worker"]
