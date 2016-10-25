#!/bin/bash

host="${1:-postgres}"
port="${2:-5432}"

while true; do
  # bash specific!
  echo > "/dev/tcp/$host/$port"
  if [ $? -eq 0 ]; then
    break
  fi
  >&2 echo "Port $port is unavailable - sleeping"
  sleep 1
done

>&2 echo "Port $port is up"
