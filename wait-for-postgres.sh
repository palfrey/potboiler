#!/bin/bash

host="${1:-postgres}"

while true; do
  # bash specific!
  echo > "/dev/tcp/$host/5432"
  if [ $? -eq 0 ]; then
    break
  fi
  >&2 echo "Postgres is unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up"
