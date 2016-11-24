FROM palfrey/rustup-watchexec:1.13.0
WORKDIR /code/pigtail
ADD pigtail/Cargo.* /code/pigtail/
ADD potboiler_common/Cargo.* /code/potboiler_common/
RUN cargo fetch
ADD . /code
RUN cargo build
ENV RUST_BACKTRACE=1
CMD ../wait-for-port.sh postgres && ../wait-for-port.sh core 8000 && watchexec --run-initially -e rs -i target/ -r cargo run
