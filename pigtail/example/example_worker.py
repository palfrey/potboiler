import requests
import time
import logging
import uuid
import json

logging.basicConfig(format='%(asctime)s: (%(levelname)s) %(name)s - %(message)s', level=logging.INFO)
logging.getLogger("requests").setLevel(logging.WARNING)
worker_id = str(uuid.uuid4())
logging.info("Worker id is %s", worker_id)

while True:
    todo = requests.get("http://localhost:8003/queue/example")
    if todo.status_code == 404:
        logging.info("Still waiting for provider to come up, waiting 5 seconds...")
        time.sleep(5)
        continue
    todo = todo.json()
    logging.info("todo: %s",todo)
    for k in todo.keys():
        v = todo[k]
        if v["state"] == "pending" and v["task_name"] == "hello_world":
            logging.info("Grabbing %s", k)
            grab = requests.put("http://localhost:8003/queue/example/%s" % k, data=json.dumps({"worker_id": worker_id}))
            if grab.status_code != 200:
                raise Exception, (grab.status_code, grab.text)
            v = grab.json()
            logging.info("Hello World with data %s", v["info"])
            done = requests.delete("http://localhost:8003/queue/example/%s" % k, data=json.dumps({"worker_id": worker_id}))
            if done.status_code != 200:
                raise Exception, (grab.status_code, grab.text)
            logging.info("Marked %s as done", k)
    time.sleep(5)
