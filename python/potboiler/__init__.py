import falcon
import json
import leveldb
import zmq
import threading
from voluptuous import Schema, Required, All, Length, MultipleInvalid, Extra, Range
import uuid

kind_key = b"_kinds"
self_key = b"_self"
stores_key = b"_stores"

JSONEncoder_olddefault = json.JSONEncoder.default
def JSONEncoder_newdefault(self, o):
	if isinstance(o, uuid.UUID): return str(o)
	return JSONEncoder_olddefault(self, o)
json.JSONEncoder.default = JSONEncoder_newdefault

def UUID(value):
	return uuid.UUID(value)

class Client:
	def __init__(self, zmq, host, port):
		self.zmq = zmq
		self.host = host
		self.port = port
		self.kinds = {}

class JSONResource:
	def get_key(self, key):
		return self.db.Get(key).decode("utf-8")

	def get_json_key(self, key):
		return json.loads(self.get_key(key))

	def put_key(self, key, value):
		self.db.Put(key, json.dumps(value).encode("utf-8"))

	def check_req(self, req):
		try:
			data = json.loads(req.stream.read().decode("utf-8"))
			if not isinstance(data, dict):
				raise falcon.HTTPBadRequest("Invalid JSON", "Request was not a JSON dictionary")
			self.schema(data)
			return data
		except ValueError:
			raise falcon.HTTPBadRequest("Invalid JSON", "Request was not valid JSON")
		except MultipleInvalid as e:
			raise falcon.HTTPBadRequest("Invalid keys", str(e))

	def check_noargs(self, req):
		if len(req.stream.read()) > 0:
			raise falcon.HTTPBadRequest("accepts no arguments", "Gave a body to a method that doesn't accept one")


class ClientResource(JSONResource):
	def __init__(self, context, poller, clients):
		self.clients = clients
		self.context = context
		self.poller = poller

	def on_get(self, req, resp):
		self.check_noargs(req)
		resp.body = json.dumps(dict([("%s:%s"%k, self.clients[k].kinds) for k in self.clients.keys()]))

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
		resp.status = falcon.HTTP_CREATED

class StoreResource(JSONResource):
	def __init__(self, db):
		self.db = db
		self.self_key = self.get_key(self_key)

	schema = Schema({
		Required("kind"): All(str, Length(min=1)),
		Required("id"): UUID,
		Required("entry_id"): UUID,
		Required("data"): {Extra: object}
	})

	def on_get(self, req, resp, key = None):
		self.check_noargs(req)
		if key is None:
			resp.body = self.get_key(stores_key)
		else:
			resp.body = self.get_key(key.encode("utf-8"))

	def on_put(self, req, resp):
		data = self.check_req(req)
		key = data["entry_id"].encode("utf-8")
		try:
			existing = self.get_key(key)
			raise falcon.HTTPConflict("Duplicate key", "Already have entry for %s: %s" % (data["entry_id"], existing))
		except KeyError:
			self.put_key(key, data)
			kinds = self.get_json_key(kind_key)
			if data["kind"] not in kinds:
				kinds[data["kind"]] = {"key": data["entry_id"], "previous": None}
			else:
				kinds[data["kind"]] = {"key": data["entry_id"], "previous": kinds[data["kind"]]["key"]}
			self.put_key(kind_key, kinds)

			stores = self.get_json_key(stores_key)
			if self.self_key not in stores:
				stores[self.self_key] = {"key": data["entry_id"], "previous": None}
			else:
				stores[self.self_key] = {"key": data["entry_id"], "previous": stores[self.self_key]["key"]}
			self.put_key(stores_key, stores)

			resp.status = falcon.HTTP_CREATED


class KindResource(JSONResource):
	def __init__(self, db):
		self.db = db

	def on_get(self, req, resp):
		self.check_noargs(req)
		resp.body = self.db.Get(kind_key)

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

def setup_key(db, key, value):
	try:
		db.Get(key)
	except KeyError:
		db.Put(key, json.dumps(value).encode("utf-8"))

def make_api(db_path = "./db", port = "5555"):
	api = falcon.API()
	db = leveldb.LevelDB(db_path, paranoid_checks = True)

	setup_key(db, kind_key, {})
	setup_key(db, self_key, uuid.uuid4())
	setup_key(db, stores_key, {})

	context = zmq.Context.instance()
	poller = zmq.Poller()
	client_list = {}
	api.add_route('/clients', ClientResource(context, poller, client_list))
	api.add_route('/store', StoreResource(db))
	api.add_route('/store/{key}', StoreResource(db))
	api.add_route('/kinds', KindResource(db))
	clients = context.socket(zmq.ROUTER)
	clients.bind("tcp://*:" + str(port))
	poller.register(clients, zmq.POLLIN)
	event = threading.Event()
	thread = threading.Thread(target=ZMQ, args=(poller, event), daemon = True)
	thread.start()
	return {"api": api, "context": context, "thread": thread, "event": event, "db": db, "clients": client_list}

if __name__ == "__main__":
	api = make_api()["api"]
