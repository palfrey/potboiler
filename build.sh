#!/bin/bash
set -eux -o pipefail

if [ "$PROJECT" = "docker" ]; then
	python generate-compose.py
	docker-compose build
	docker-compose up
fi

export PATH=$PATH:~/.cargo/bin &&
cd $PROJECT
cargo fmt -- --write-mode=diff
cargo build
cargo test
