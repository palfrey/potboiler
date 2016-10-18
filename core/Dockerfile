FROM jimmycuadra/rust:1.12.0
RUN cargo install --git https://github.com/mattgreen/watchexec.git
WORKDIR /code/core
ADD core/Cargo.* /code/core/
ADD potboiler_common/Cargo.* /code/potboiler_common/
RUN cargo fetch
ADD . /code
RUN cargo build
ENV RUST_BACKTRACE=1
CMD ../wait-for-postgres.sh postgres-core && ~/.cargo/bin/watchexec --run-initially -e rs -r cargo run
