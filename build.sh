#!/bin/bash
set -eux -o pipefail

if [ "$PROJECT" = "docker" ]; then
	pip install PyYAML
	python generate-compose.py 1 > docker-compose.yml
	docker-compose build
	docker-compose up &
	./wait-for-it.sh --timeout=0 localhost:8001
	sleep 5 # to wait for fully booted	
	curl --retry 100 --retry-connrefused -v http://localhost:8001/kv/_config
	docker-compose stop
	docker login -u="$DOCKER_USERNAME" -p="$DOCKER_PASSWORD"
	docker push potboiler/core
	docker push potboiler/kv
	docker push potboiler/pigtail
	exit 0
fi

export PATH=$PATH:~/.cargo/bin &&
cd $PROJECT
cargo fmt -- --check
cargo build
cargo test
