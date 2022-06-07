FROM rust:1.52-slim
RUN cargo install cargo-watch --version 7.8.0
RUN apt-get update && apt-get install -y libssl-dev pkg-config
WORKDIR /code/core
ADD . /code
RUN cargo fetch
RUN cargo build
ENV RUST_BACKTRACE=1
CMD cargo build && ../wait-for-it.sh --timeout=0 postgres:5432 -s -- cargo watch -x run
