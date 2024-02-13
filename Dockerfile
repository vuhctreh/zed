# syntax = docker/dockerfile:1.2

FROM rust:1.76-bullseye as builder
FROM debian:bullseye-slim as runtime
RUN apt-get update; \
    apt-get install -y --no-install-recommends libcurl4-openssl-dev ca-certificates
COPY ./target/release/collab /app/collab
COPY ./crates/collab/migrations /app/migrations

WORKDIR app
ENV MIGRATIONS_PATH=/app/migrations
ENTRYPOINT ["/app/collab"]
