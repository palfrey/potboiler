#!/bin/bash
set -eux -o pipefail

if [ "$PROJECT" = "docker" ]; then
	pip install PyYAML
	python generate-compose.py 1 > docker-compose.yml
	docker-compose build
	docker login -u="$DOCKER_USERNAME" -p="$DOCKER_PASSWORD"
	docker push potboiler/core
	docker push potboiler/kv
	docker push potboiler/pigtail
	exit 0
fi

cargo install rustfmt || true

export PATH=$PATH:~/.cargo/bin &&
cd $PROJECT
cargo fmt -- --write-mode=diff
cargo build
cargo test
