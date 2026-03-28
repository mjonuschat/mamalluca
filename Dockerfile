FROM rust:1-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --locked -p mamalluca

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/mamalluca /
ENTRYPOINT ["/mamalluca"]
