FROM jimmycuadra/rust:1.12.0
ADD . /code
WORKDIR /code/kv
RUN cargo build
ENV RUST_BACKTRACE=1
CMD cargo run
