Potboiler
=========

Core
----

- List all current log heads
  - `curl -vL http://localhost:8000/log` => `{"69275a71-ec18-4be6-80a9-ac8e5d1d26b2":"d717f81d-dfc8-4c04-8fb3-1f28d63acf88"}`

- List all log starts
  - `curl -vL http://localhost:8000/log/first` => `{"69275a71-ec18-4be6-80a9-ac8e5d1d26b2":"ad586c31-6cfe-43f2-9a7a-57e6371acff9"}`

- Get log item
  - `curl -vL http://localhost:8000/log/6181ddc4-3c0b-4a40-b94c-f73379da886d` => `{"data":{"dfdsf":"sdfdsfs","foo":"bar"},"id":"6181ddc4-3c0b-4a40-b94c-f73379da886d","next":null,"owner":"69275a71-ec18-4be6-80a9-ac8e5d1d26b2","prev":"d717f81d-dfc8-4c04-8fb3-1f28d63acf88"}`
- Add new log item
   - `curl http://localhost:8000/log -d "{\"foo\":\"bar\", \"dfdsf\":\"sdfdsfs\"}"` => redirect to "get log item"

- Register for log updates
  - `curl http://localhost:8000/log/register -d "{\"url\": \"<URL to send msgs to>\"}"` => 204

- Deregister for log updates
  - `curl http://localhost:8000/log/deregister -d "{\"url\": \"<URL to send msgs to>\"}"` => 204 if existed, otherwise 404
