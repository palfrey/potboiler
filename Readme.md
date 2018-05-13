# Potboiler

[![Build Status](https://travis-ci.org/palfrey/potboiler.svg?branch=master)](https://travis-ci.org/palfrey/potboiler)

Potboiler is an AP Event Sourcing system. More specifically, it's an MVP/research prototype of said, with known issues and is not even slightly suitable for production use. However, patches welcomed!

## Getting started
1. `pip install PyYAML`
2. `python generate-compose.py 3 > docker-compose.yml` will generate node configs for 3 core/KV/browser sets
3. `docker-compose up -d`
4. Goto `http://localhost:8002/8102/8202` to see any of the Key/Value browsing

## Ports

The default Docker config gives you the following ports for the first node:
* Potboiler node: 8000
* Key/Value node: 8001
* Key/Value browser: 8002

For subsequent nodes, add 100 to the port numbers (e.g. 8100-2 for the second, 8200-2 for the third, etc)

## API

### Core

- List all current log heads
  - `curl http://localhost:8000/log` => `{"69275a71-ec18-4be6-80a9-ac8e5d1d26b2":"d717f81d-dfc8-4c04-8fb3-1f28d63acf88"}`

- List all log starts
  - `curl http://localhost:8000/log/first` => `{"69275a71-ec18-4be6-80a9-ac8e5d1d26b2":"ad586c31-6cfe-43f2-9a7a-57e6371acff9"}`

- Get log item
  - `curl http://localhost:8000/log/6181ddc4-3c0b-4a40-b94c-f73379da886d` => `{"data":{"dfdsf":"sdfdsfs","foo":"bar"},"id":"6181ddc4-3c0b-4a40-b94c-f73379da886d","next":null,"owner":"69275a71-ec18-4be6-80a9-ac8e5d1d26b2","prev":"d717f81d-dfc8-4c04-8fb3-1f28d63acf88"}`

- Add new log item
   - `curl http://localhost:8000/log -d "{\"foo\":\"bar\", \"dfdsf\":\"sdfdsfs\"}"` => redirect to "get log item"

- Register for log updates
  - `curl http://localhost:8000/log/register -d "{\"url\": \"[URL to send msgs to]\"}"` => 204

- Deregister for log updates
  - `curl http://localhost:8000/log/deregister -d "{\"url\": \"[URL to send msgs to]\"}"` => 204 if existed, otherwise 404

- List other nodes
  - `curl http://localhost:8000/nodes` => `["http://core1:8000","http://core0:8000"]`

- Add new other node
  - `curl http://localhost:8000/nodes -d "{\"url\": \"[Potboiler node root]\"}"` => 204

### KV

- Retrieve key
  - `curl http://localhost:8001/kv/[table]/[key]` => `["foo", "bar"]`
  - 404 if no table, or no key
  - [table] and [key] here are alphanumeric strings

Update operations:
"[item]" is any JSON value. "[key]" is a string.
- LWW
  - "set": "[item]"
- OR-Set:
  - "create": {} // makes an empty OR-Set if it doesn't exist
  - "add": {"item":"[item]", "key":"[key]", "metadata": "[metadata]"}
  - "remove": {"item":"[item]", "key":"[key]"}

- Update key
  - `curl http://localhost:8001/kv/[table]/[key] -d "{\"op\": \"[operation]\", \"change\": \"[data]\"}"` => Always 200 if data format is correct, regardless of whether the table has been seen

- Create table
  - Update key. "table" is "\_config", "key" is table name. It's a LWW table, with "[item]" being {"crdt": "[crdt]"} "[crdt]" being only "LWW" current. Other info for the config table is ignored.
   e.g `curl -vL http://localhost:8001/kv/_config/test -d "{\"op\": \"set\", \"change\": \"{\"crdt\": \"LWW\"}\"}"`

### Pigtail

Pigtail is a task queue implementation. Best docs for it at the moment are the example worker and provider in pigtail/example
