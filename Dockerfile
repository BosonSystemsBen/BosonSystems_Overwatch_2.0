FROM node:22-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

FROM rust:slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin api --bin migrate

FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/api /usr/local/bin/api
COPY --from=builder /app/target/release/migrate /usr/local/bin/migrate
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist
EXPOSE 8080
CMD ["api"]
