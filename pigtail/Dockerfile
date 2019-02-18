FROM rust:1.32-slim
RUN cargo install cargo-watch
RUN apt-get update && apt-get install -y libssl-dev pkg-config
WORKDIR /code/pigtail
ADD pigtail/Cargo.* /code/pigtail/
ADD potboiler_common/Cargo.* /code/potboiler_common/
RUN cargo fetch
ADD . /code
RUN cargo build
ENV RUST_BACKTRACE=1
CMD cargo build && ../wait-for-it.sh --timeout=0 postgres:5432 -s -- ../wait-for-it.sh --timeout=0 core:8000 -s -- watchexec -e rs -i target/ -r cargo run
