#!/bin/bash
set -eux -o pipefail

if [ "$PROJECT" = "docker" ]; then
	pip install PyYAML
	python generate-compose.py 1 > docker-compose.yml
	docker-compose build
	docker-compose up &
	pip install requests
	python3 wait-for-http.py http://localhost:8001/kv/_config
	docker-compose stop
	docker login -u="$DOCKER_USERNAME" -p="$DOCKER_PASSWORD"
	docker push potboiler/core
	docker push potboiler/kv
	docker push potboiler/pigtail
	exit 0
elif [ "$PROJECT" = "check" ]; then
	rustup component add rustfmt-preview
	rustup component add clippy-preview
	cargo fmt -- --check
	cargo clippy
fi

export PATH=$PATH:~/.cargo/bin &&
cd "$PROJECT"
cargo build
cargo test