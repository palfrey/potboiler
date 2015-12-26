import falcon
import json
import leveldb
import zmq
import time
import threading
import json
from voluptuous import Schema, Required, All, Length, MultipleInvalid, Extra
import uuid

def UUID(value):
    return uuid.UUID(value)

class Client:
    def __init__(self):
        self.zmq = None

class StoreResource:
    def __init__(self, db):
        self.db = db

    schema = Schema({
        Required("table"): All(str, Length(min=1)),
        Required("id"): UUID,
        Required("entry_id"): UUID,
        Required("data"): {Extra: object}
    })

    def on_put(self, req, resp):
        try:
            data = json.loads(req.stream.read().decode("utf-8"))
            if type(data) != dict:
                raise falcon.HTTPBadRequest("Invalid JSON", "Request was not a JSON dictionary")
            StoreResource.schema(data)
            key = data["entry_id"].encode("utf-8")
            try:
                existing = self.db.Get(key)
                raise falcon.HTTPConflict("Duplicate key", "Already have entry for %s" % key)
            except KeyError:
                self.db.Put(key, json.dumps(data["data"]).encode("utf-8"))
        except ValueError:
            raise falcon.HTTPBadRequest("Invalid JSON", "Request was not valid JSON")
        except MultipleInvalid as e:
            raise falcon.HTTPInvalidParam("Invalid keys", e.msg)

def ZMQ(poller):
	while True:
		if len(poller.sockets) == 0:
			time.sleep(5)
			continue
		try:
			socks = dict(poller.poll())
		except KeyboardInterrupt:
			break
		print ("socks", socks)

def make_api(db_path = "./db", port = "5555"):
    api = falcon.API()
    db = leveldb.LevelDB(db_path, paranoid_checks = True)
    api.add_route('/store', StoreResource(db))
    context = zmq.Context.instance()
    clients = context.socket(zmq.ROUTER)
    clients.bind("tcp://*:" + str(port))
    poller = zmq.Poller()
    poller.register(clients, zmq.POLLIN)

    thread = threading.Thread(target=ZMQ, args=(poller, ), daemon = True)
    thread.start()
    return api

if __name__ == "__main__":
    api = make_api()
