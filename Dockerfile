FROM rustlang/rust:nightly-slim as builder

WORKDIR /usr/src/ultralight-worker

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/ultralight-worker/target \
    cargo install --bins --path .

FROM debian:buster-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/ultralight-worker /usr/local/bin/ultralight-worker
