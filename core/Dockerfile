FROM palfrey/rustup-watchexec:1.31.0
WORKDIR /code/core
ADD potboiler_common/Cargo.* /code/potboiler_common/
ADD core/Cargo.* /code/core/
RUN cargo fetch
ADD . /code
RUN cargo build
ENV RUST_BACKTRACE=1
CMD ../wait-for-it.sh postgres:5432 -s -- watchexec -e rs -i target/ -r cargo run
