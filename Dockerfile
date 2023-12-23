# Build phase.
FROM rust:slim-buster AS builder
RUN update-ca-certificates
WORKDIR /oasysdb
COPY . .
RUN cargo build --release

# Finalize image.
FROM debian:buster-slim
WORKDIR /oasysdb
COPY --from=builder /oasysdb/target/release/oasysdb .
COPY --from=builder /oasysdb/Rocket.toml .
CMD ["./oasysdb"]
