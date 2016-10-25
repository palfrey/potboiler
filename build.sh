#!/bin/bash
set -eux -o pipefail

if [ "$PROJECT" = "docker" ]; then
	pip install PyYAML
	python generate-compose.py > docker-compose.yml
	docker-compose build
	exit 0
fi

cargo install rustfmt || true

export PATH=$PATH:~/.cargo/bin &&
cd $PROJECT
cargo fmt -- --write-mode=diff
cargo build
cargo test
