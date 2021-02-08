FROM ubuntu:20.04

COPY . /app

ENV DEBIAN_FRONTEND noninteractive

RUN apt update && \
    apt install -y redis-server vim curl libssl-dev gcc make pkg-config && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    chsh -s /bin/bash

CMD . $HOME/.cargo/env && \
    cd /app && \
    redis-server --daemonize yes && \
    cargo run 2>&1 | tee -a telates_$(date +%s)
