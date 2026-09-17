FROM rust:slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin api --bin migrate

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/api /usr/local/bin/api
COPY --from=builder /app/target/release/migrate /usr/local/bin/migrate
EXPOSE 8080
CMD ["api"]
