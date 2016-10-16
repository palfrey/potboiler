FROM jimmycuadra/rust:1.12.0
WORKDIR /code/kv
ADD kv/Cargo.* /code/kv/
ADD potboiler_common/Cargo.* /code/potboiler_common/
RUN cargo fetch
ADD . /code
RUN cargo build
ENV RUST_BACKTRACE=1
CMD ../wait-for-postgres.sh postgres-kv && cargo run
