FROM palfrey/rustup-watchexec:1.31.0
WORKDIR /code/kv
ADD kv/Cargo.* /code/kv/
ADD potboiler_common/Cargo.* /code/potboiler_common/
RUN cargo fetch
ADD . /code
RUN cargo build
ENV RUST_BACKTRACE=1
CMD ../wait-for-it.sh postgres:5432 -s -- ../wait-for-it.sh core:8000 -s -- watchexec -e rs -i target/ -r cargo run
