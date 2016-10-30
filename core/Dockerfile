FROM palfrey/rustup-watchexec:1.12.1
WORKDIR /code/core
ADD core/Cargo.* /code/core/
ADD potboiler_common/Cargo.* /code/potboiler_common/
RUN cargo fetch
ADD . /code
RUN cargo build
ENV RUST_BACKTRACE=1
CMD ../wait-for-port.sh postgres && watchexec --run-initially -e rs -i target/ -r cargo run
