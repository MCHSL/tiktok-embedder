FROM rust:1.63-slim-buster AS builder
WORKDIR /usr/src/tiktok-embedder
COPY . .
RUN cargo build --release


FROM debian:buster-slim
RUN  apt-get update \
	&& apt-get install -y wget python3 \
	&& rm -rf /var/lib/apt/lists/*
RUN wget https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -O /usr/bin/youtube-dl && chmod a+rx /usr/bin/youtube-dl
COPY --from=builder /usr/src/tiktok-embedder/target/release/tiktok-embedder /usr/bin/tiktok-embedder
CMD tiktok-embedder
