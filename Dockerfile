# Build phase.
FROM rust:bookworm AS builder
RUN update-ca-certificates
WORKDIR /oasysdb
COPY . .
RUN cargo build --release

# Finalize image.
FROM debian:bookworm
WORKDIR /oasysdb
COPY --from=builder /oasysdb/target/release/oasysdb .
CMD ["./oasysdb"]
