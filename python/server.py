import falcon
import json
import leveldb
import zmq
import time
import threading
import json
from voluptuous import Schema, Required, All, Length, MultipleInvalid, Extra, Range
import uuid

def UUID(value):
	return uuid.UUID(value)

class Client:
	def __init__(self, zmq):
		self.zmq = zmq

class JSONResource:
	def check_req(self, req):
		try:
			data = json.loads(req.stream.read().decode("utf-8"))
			if type(data) != dict:
				raise falcon.HTTPBadRequest("Invalid JSON", "Request was not a JSON dictionary")
			StoreResource.schema(data)
			return data
		except ValueError:
			raise falcon.HTTPBadRequest("Invalid JSON", "Request was not valid JSON")
		except MultipleInvalid as e:
			raise falcon.HTTPInvalidParam("Invalid keys", e.msg)

class ClientResource(JSONResource):
	def __init__(self, context, poller):
		self.clients = []
		self.context = context
		self.poller = poller

	def on_get(self, req, resp):
		if len(req.stream.read()) > 0:
			raise falcon.HTTPBadRequest("clients accepts no arguments", "Gave a body to a method that doesn't accept one")
		return self.clients

	schema = Schema({
		Required("host"): All(str, Length(min=1)),
		Required("port"): All(int, Range(min=1))
	})

	def on_put(self, req, resp):
		data = self.check_req(req)
		zmq = self.context.socket(zmq.SUB)
		zmq.connect("tcp://{host}:{port}".format(**data))
		self.poller.append(zmq)
		self.clients.append(Client(zmq))

class StoreResource(JSONResource):
	def __init__(self, db):
		self.db = db

	schema = Schema({
		Required("table"): All(str, Length(min=1)),
		Required("id"): UUID,
		Required("entry_id"): UUID,
		Required("data"): {Extra: object}
	})

	def on_put(self, req, resp):
		data = self.check_req(req)
		key = data["entry_id"].encode("utf-8")
		try:
			existing = self.db.Get(key)
			raise falcon.HTTPConflict("Duplicate key", "Already have entry for %s" % key)
		except KeyError:
			self.db.Put(key, json.dumps(data["data"]).encode("utf-8"))

def ZMQ(poller, event):
	while True:
		if event.is_set():
			break
		if len(poller.sockets) == 0:
			event.wait(timeout = 5)
			continue
		try:
			socks = dict(poller.poll())
		except KeyboardInterrupt:
			break
		except zmq.error.ContextTerminated:
			break
		print ("socks", socks)

def make_api(db_path = "./db", port = "5555"):
	api = falcon.API()
	db = leveldb.LevelDB(db_path, paranoid_checks = True)
	context = zmq.Context.instance()
	poller = zmq.Poller()
	api.add_route('/clients', ClientResource(context, poller))
	api.add_route('/store', StoreResource(db))
	clients = context.socket(zmq.ROUTER)
	clients.bind("tcp://*:" + str(port))
	poller.register(clients, zmq.POLLIN)
	event = threading.Event()
	thread = threading.Thread(target=ZMQ, args=(poller, event), daemon = True)
	thread.start()
	return {"api": api, "context": context, "thread": thread, "event": event}

if __name__ == "__main__":
	api = make_api()["api"]
