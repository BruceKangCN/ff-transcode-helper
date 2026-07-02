FROM rust:1-slim-trixie AS builder

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y \
        pkgconf \
        libclang-dev \
        libavcodec-dev \
        libavformat-dev \
        libavfilter-dev \
        libavdevice-dev \
        libavutil-dev \
        libswscale-dev \
        libswresample-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

RUN cargo build --release --bins

FROM debian:trixie-slim

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y ffmpeg \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder [ \
        "/app/target/release/ff-x265-opus", \
        "/app/target/release/ff-av1-opus-hdr", \
        "/usr/local/bin/" \
    ]
