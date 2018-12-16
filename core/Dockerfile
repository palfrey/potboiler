FROM palfrey/rustup-watchexec:1.31.0
WORKDIR /code/core
RUN apt-get update && apt-get install -y libssl-dev pkg-config
ADD potboiler_common/Cargo.* /code/potboiler_common/
ADD core/Cargo.* /code/core/
RUN cargo fetch
ADD . /code
RUN cargo build
ENV RUST_BACKTRACE=1
CMD cargo build && ../wait-for-it.sh --timeout=0 postgres:5432 -s -- watchexec -e rs -i target/ -r cargo run
