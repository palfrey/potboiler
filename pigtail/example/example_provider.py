import requests
import logging
import json
import time

logging.basicConfig(format='%(asctime)s: (%(levelname)s) %(name)s - %(message)s', level=logging.INFO)
logging.getLogger("requests").setLevel(logging.WARNING)

queue = requests.post("http://localhost:8003/create", data=json.dumps({"name":"example", "timeout_ms":1000}))
logging.info("queue: %s", queue)

while True:
    task = requests.post("http://localhost:8003/queue/example", data=json.dumps({"task_name":"hello_world", "info":{"foo":"bar"}}))
    logging.info(task.headers["location"])
    time.sleep(5)
