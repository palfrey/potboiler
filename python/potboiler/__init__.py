import falcon
import json
import leveldb
import zmq
import time
import threading
import json
from voluptuous import Schema, Required, All, Length, MultipleInvalid, Extra, Range
import uuid

table_key = b"_tables"

def UUID(value):
	return uuid.UUID(value)

class Client:
	def __init__(self, zmq, host, port):
		self.zmq = zmq
		self.host = host
		self.port = port
		self.tables = {}

class JSONResource:
	def check_req(self, req):
		try:
			data = json.loads(req.stream.read().decode("utf-8"))
			if type(data) != dict:
				raise falcon.HTTPBadRequest("Invalid JSON", "Request was not a JSON dictionary")
			self.schema(data)
			return data
		except ValueError:
			raise falcon.HTTPBadRequest("Invalid JSON", "Request was not valid JSON")
		except MultipleInvalid as e:
			raise falcon.HTTPBadRequest("Invalid keys", str(e))

class ClientResource(JSONResource):
	def __init__(self, context, poller, clients):
		self.clients = clients
		self.context = context
		self.poller = poller

	def on_get(self, req, resp):
		if len(req.stream.read()) > 0:
			raise falcon.HTTPBadRequest("clients accepts no arguments", "Gave a body to a method that doesn't accept one")
		resp.body = json.dumps(dict([("%s:%s"%k, self.clients[k].tables) for k in self.clients.keys()]))

	schema = Schema({
		Required("host"): All(str, Length(min=1)),
		Required("port"): All(int, Range(min=1))
	})

	def on_put(self, req, resp):
		data = self.check_req(req)
		conn = self.context.socket(zmq.SUB)
		host, port = data["host"], data["port"]
		conn.connect("tcp://{host}:{port}".format(**data))
		self.poller.register(conn)
		self.clients[(host,port)] = Client(conn, host, port)

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
			existing = self.db.Get(key).decode("utf-8")
			raise falcon.HTTPConflict("Duplicate key", "Already have entry for %s: %s" % (data["entry_id"], existing))
		except KeyError:
			self.db.Put(key, json.dumps(data["data"]).encode("utf-8"))
			tables = json.loads(self.db.Get(table_key).decode("utf-8"))
			if data["table"] not in tables:
				tables[data["table"]] = {"key": data["entry_id"], "previous": None}
			else:
				tables[data["table"]] = {"key": data["entry_id"], "previous": tables[data["table"]]["key"]}
			self.db.Put(table_key, json.dumps(tables).encode("utf-8"))


class TableResource(JSONResource):
	def __init__(self, db):
		self.db = db

	def on_get(self, req, resp):
		resp.body = self.db.Get(table_key)

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
		raise Exception("socks", socks)

def make_api(db_path = "./db", port = "5555"):
	api = falcon.API()
	db = leveldb.LevelDB(db_path, paranoid_checks = True)
	try:
		db.Get(table_key)
		raise Exception
	except KeyError:
		db.Put(table_key, json.dumps({}).encode("utf-8"))
	context = zmq.Context.instance()
	poller = zmq.Poller()
	client_list = {}
	api.add_route('/clients', ClientResource(context, poller, client_list))
	api.add_route('/store', StoreResource(db))
	api.add_route('/tables', TableResource(db))
	clients = context.socket(zmq.ROUTER)
	clients.bind("tcp://*:" + str(port))
	poller.register(clients, zmq.POLLIN)
	event = threading.Event()
	thread = threading.Thread(target=ZMQ, args=(poller, event), daemon = True)
	thread.start()
	return {"api": api, "context": context, "thread": thread, "event": event, "db": db, "clients": client_list}

if __name__ == "__main__":
	api = make_api()["api"]
