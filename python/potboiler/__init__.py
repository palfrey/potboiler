import falcon
import json
import leveldb
import zmq
import threading
from voluptuous import Schema, Required, All, Length, MultipleInvalid, Extra, Range
import uuid
import logging
import asyncio
import aiodns
import python_hosts

kind_key = b"_kinds"
self_key = b"_self"
stores_key = b"_stores"

log = logging.getLogger(__name__)

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
		self.stores = {}

	def __del__(self):
		self.zmq.close()

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
		resp.body = json.dumps(dict([(k, self.clients[k].stores) for k in self.clients.keys()]))

	schema = Schema({
		Required("host"): All(str, Length(min=1)),
		Required("port"): All(int, Range(min=1))
	})

	def on_put(self, req, resp):
		data = self.check_req(req)
		conn = self.context.socket(zmq.PAIR)
		conn.linger = 0
		host, port = data["host"], data["port"]
		ip = None
		if python_hosts.utils.is_ipv4(host):
			ip = host
		else:
			hosts = python_hosts.hosts.Hosts()
			for x in hosts.entries:
				if x.entry_type == "comment":
					continue
				if host in x.names:
					ip = x.address
					break
		if ip is None:
			loop = asyncio.get_event_loop()
			resolver = aiodns.DNSResolver(loop=loop)
			try:
				f = resolver.query(host, 'A')
				result = loop.run_until_complete(f)
				ip = result[0].host
			except UnicodeError: # hostname too long for IDNA
				raise falcon.HTTPInvalidParam("Too long hostname", "host")
			except aiodns.error.DNSError:
				raise falcon.HTTPInvalidParam("DNS can't find host", "host")
		conn.connect("tcp://{ip}:{port}".format(ip=ip, port=port))
		self.clients["%s:%d" %(host,port)] = Client(conn, host, port)
		resp.status = falcon.HTTP_CREATED

class StoreResource(JSONResource):
	def __init__(self, db, clients):
		self.db = db
		self.self_key = self.get_key(self_key)
		self.clients = clients

	schema = Schema({
		Required("kind"): All(str, Length(min=1)),
		Required("id"): UUID,
		Required("entry_id"): UUID,
		Required("data"): {Extra: object}
	})

	def on_get(self, req, resp, key=None):
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
			pass
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
			old_store = str(uuid.uuid4())
			self.put_key(old_store.encode("utf-8"), stores[self.self_key])
			stores[self.self_key] = {"key": data["entry_id"], "previous": old_store}
		self.put_key(stores_key, stores)

		if len(self.clients.keys()) > 0:
			tosend = stores[self.self_key]
			tosend["data"] = data
			for key in self.clients.keys():
				client = self.clients[key]
				previous = None
				if self.self_key in client.stores:
					previous = client.stores[self.self_key]
				if previous == stores[self.self_key]["previous"]:
					client.zmq.send_json(tosend)
					client.stores[self.self_key] = data["entry_id"]
				else:
					log.info("Not sending to %s because previous is %s rather than %s", key, stores[self.self_key]["previous"], previous)

		resp.status = falcon.HTTP_CREATED


class KindResource(JSONResource):
	def __init__(self, db):
		self.db = db

	def on_get(self, req, resp):
		self.check_noargs(req)
		resp.body = self.db.Get(kind_key)

class UpdateResource(JSONResource):
	def __init__(self, db, client_list):
		self.db = db
		self.client_list = client_list

	def on_post(self, req, resp, client_key):
		self.check_noargs(req)
		print(self.client_list.keys())
		if client_key not in self.client_list:
			raise falcon.HTTPNotFound()
		client = self.client_list[client_key]
		stores = self.get_json_key(stores_key)
		for s in stores:
			if s not in client.stores:
				tosend = [stores[s]]
				while tosend[0]["previous"] != None:
					prev = self.get_json_key(tosend[0]["previous"].encode("utf-8"))
					tosend.insert(0, prev)
				for ts in tosend:
					ts["data"] = self.get_json_key(ts["key"].encode("utf-8"))
					client.zmq.send_json(ts)
				client.stores[s] = tosend[-1]["key"]

		resp.status = falcon.HTTP_OK

def ZMQ(poller, event):
	while True:
		if event.is_set():
			break
		if len(poller.sockets) == 0:
			event.wait(timeout=5)
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

def make_api(db_path="./db"):
	api = falcon.API()
	db = leveldb.LevelDB(db_path, paranoid_checks=True)

	setup_key(db, kind_key, {})
	setup_key(db, self_key, uuid.uuid4())
	setup_key(db, stores_key, {})

	context = zmq.Context.instance()
	poller = zmq.Poller()
	client_list = {}
	api.add_route('/clients', ClientResource(context, poller, client_list))
	api.add_route('/store', StoreResource(db, client_list))
	api.add_route('/store/{key}', StoreResource(db, client_list))
	api.add_route('/kinds', KindResource(db))
	api.add_route('/update/{client_key}', UpdateResource(db, client_list))
	event = threading.Event()
	thread = threading.Thread(target=ZMQ, args=(poller, event), daemon=True)
	thread.start()
	return {"api": api, "context": context, "thread": thread, "event": event, "db": db, "clients": client_list}

if __name__ == "__main__":
	api = make_api()["api"]
