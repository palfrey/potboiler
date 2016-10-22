FROM jimmycuadra/rust:1.12.0
RUN cargo install --git https://github.com/mattgreen/watchexec.git --rev f0650274a0ba42d0eb1dbe54e6dd6967d252bd5f
WORKDIR /code/core
ADD core/Cargo.* /code/core/
ADD potboiler_common/Cargo.* /code/potboiler_common/
RUN cargo fetch
ADD . /code
RUN cargo build
ENV RUST_BACKTRACE=1
CMD ../wait-for-postgres.sh postgres && ~/.cargo/bin/watchexec --run-initially -e rs -r cargo run
