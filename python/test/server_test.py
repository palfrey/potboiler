import potboiler

import falcon.testing as testing
import falcon
from hypothesis import given, note, assume
import hypothesis.strategies as st
import tempfile
import json
import zmq
import string
import contextlib
import testtools.matchers as matchers
import math

json_st = st.dictionaries(st.text(), st.floats() | st.booleans() | st.text() | st.none())

def msg_gen(kind):
	return st.fixed_dictionaries({
		'kind': kind,
		'id': st.uuids().map(lambda x: str(x)),  # pylint: disable=unnecessary-lambda
		'entry_id': st.uuids().map(lambda x: str(x)),  # pylint: disable=unnecessary-lambda
		'data': st.dictionaries(st.text(), st.floats().filter(lambda x: not math.isnan(x)) | st.booleans() | st.text() | st.none())
	}) # pylint: disable=unnecessary-lambda

valid_msg = msg_gen(st.text(min_size=1))
def same_kind_msg(count):
	return st.text(min_size=1).flatmap(lambda t: st.lists(msg_gen(st.just(t)), min_size=count, max_size=count))

class ServerTest(testing.TestBase):
	def before(self):
		self.info = potboiler.make_api(tempfile.mkdtemp())
		self.api = self.info["api"]
		self.existing_stores = list(self.info["db"].RangeIter(include_value=False))
		self.db = self.info["db"]
		self.context = self.info["context"]

	@contextlib.contextmanager
	def withDB(self):
		try:
			# Clear out any existing keys
			keys = list(self.db.RangeIter(include_value=False))
			for k in keys:
				if k in [potboiler.kind_key, potboiler.stores_key]:
					value = json.loads(self.db.Get(k).decode("utf-8"))
					if value != {}:
						self.db.Put(k, "{}".encode("utf-8"))
					continue
				elif k != potboiler.self_key:
					self.db.Delete(k)
			self.info["clients"].clear()
			yield
		finally:
			if hasattr(self, "info"):
				self.info["event"].set()
				self.info["clients"].clear()

	@given(st.text(min_size=1))
	def test_clients_dislikes_args(self, s):
		with self.withDB():
			self.simulate_request("/clients", body=s)
			self.assertEqual(self.srmock.status, falcon.HTTP_400)

	def get_client_list(self):
		res = self.simulate_request("/clients")
		self.assertEqual(self.srmock.status, falcon.HTTP_200)
		data = json.loads(res[0].decode("utf-8"))
		return data

	def test_clients_works(self):
		with self.withDB():
			data = self.get_client_list()
			self.assertEqual(type(data), dict)
			self.assertEqual(self.srmock.status, falcon.HTTP_200)

	@given(st.fixed_dictionaries({
		'host': st.text(alphabet=string.ascii_letters + string.digits + "-", min_size=3, average_size=5, max_size=63),
		'port': st.integers(min_value=1)
	}))
	def test_bad_hosts_fail(self, s):
		try:
			assume(not int(s["host"], 16)) # matches as IP
		except ValueError:
			pass
		note("host: %r" % s["host"])
		with self.withDB():
			res = self.simulate_request("/clients", body=json.dumps(s), method='PUT')
			note(res)
			self.assertEqual(self.srmock.status, falcon.HTTP_400)
			self.assertIn("DNS can\'t find host", res[0].decode("utf-8"))

	@given(st.fixed_dictionaries({
		'host': st.just('localhost'),
		'port': st.integers(min_value=1)
	}))
	def test_clients_can_be_added(self, s):
		with self.withDB():
			self.simulate_request("/clients", body=json.dumps(s), method='PUT')
			note(self.srmock.status)
			self.assertEqual(self.srmock.status, falcon.HTTP_201)
			clients_now = self.get_client_list()
			note("now: %r" % clients_now)
			self.assertIn("{host}:{port}".format(**s), clients_now.keys())

	@given(st.fixed_dictionaries({
		'host': st.text(alphabet=string.ascii_letters + string.digits + "-", min_size=64),
		'port': st.integers(min_value=1)
	}))
	def test_idna_host_failure(self, s):
		with self.withDB():
			assume(not s["host"].startswith("-"))
			res = self.simulate_request("/clients", body=json.dumps(s), method='PUT')
			note(res)
			self.assertEqual(self.srmock.status, falcon.HTTP_400)
			self.assertIn("Too long hostname", res[0].decode("utf-8"))

	@given(st.text())
	def test_store_cant_decode_random_text(self, s):
		with self.withDB():
			self.simulate_request("/store", body=s, method='PUT')
			self.assertEqual(self.srmock.status, falcon.HTTP_400)

	@given(json_st)
	def test_store_doesnt_like_random_json(self, s):
		with self.withDB():
			self.simulate_request("/store", body=json.dumps(s), method='PUT')
			self.assertEqual(self.srmock.status, falcon.HTTP_400)

	def store_item(self, msg):
		res = self.simulate_request("/store", body=json.dumps(msg), method='PUT')
		note("res: %r" % res)
		note("db: %r" % list(self.info["db"].RangeIter(include_value=False)))
		self.assertEqual(self.srmock.status, falcon.HTTP_201)
		self.existing_stores.append(msg["entry_id"])

	@given(valid_msg)
	def test_store_likes_proper_stores(self, s):
		with self.withDB():
			self.store_item(s)

	@given(valid_msg)
	def test_store_forbids_double_store(self, s):
		with self.withDB():
			self.store_item(s)
			self.simulate_request("/store", body=json.dumps(s), method='PUT')
			self.assertEqual(self.srmock.status, falcon.HTTP_409)

	def test_can_get_kinds(self):
		res = self.simulate_request("/kinds")
		self.assertEqual(self.srmock.status, falcon.HTTP_200)
		data = json.loads(res[0].decode("utf-8"))
		self.assertEqual(dict, type(data))

	@given(valid_msg)
	def test_can_get_kinds_with_msg(self, msg):
		with self.withDB():
			self.store_item(msg)
			res = self.simulate_request("/kinds")
			self.assertEqual(self.srmock.status, falcon.HTTP_200)
			data = json.loads(res[0].decode("utf-8"))
			self.assertEqual(dict, type(data))
			note("kind data: %r" % data)
			self.assertIn(msg["kind"], data.keys())
			self.assertIn("key", data[msg["kind"]].keys())
			self.assertEqual(str(msg["entry_id"]), data[msg["kind"]]["key"])
			self.assertIn("previous", data[msg["kind"]].keys())
			self.assertEqual(None, data[msg["kind"]]["previous"])

	@given(same_kind_msg(2))
	def test_multiple_kind_insert(self, items):
		first, second = items
		with self.withDB():
			kind = first["kind"] # always the same as second["kind"]
			self.store_item(first)
			self.store_item(second)
			res = self.simulate_request("/kinds")
			self.assertEqual(self.srmock.status, falcon.HTTP_200)
			data = json.loads(res[0].decode("utf-8"))
			self.assertEqual(dict, type(data))
			note("Data: %r" % data)
			self.assertTrue(first["kind"] in data.keys())
			self.assertEqual(str(second["entry_id"]), data[kind]["key"])
			self.assertEqual(str(first["entry_id"]), data[kind]["previous"])

	def test_get_store_list(self):
		res = self.simulate_request("/store")
		self.assertEqual(self.srmock.status, falcon.HTTP_200)
		data = json.loads(res[0].decode("utf-8"))
		self.assertEqual(dict, type(data), res[0].decode("utf-8"))

	@given(valid_msg)
	def test_msgs_get_stored(self, msg):
		with self.withDB():
			self.store_item(msg)
			res = self.simulate_request("/store")
			self.assertEqual(self.srmock.status, falcon.HTTP_200)
			data = json.loads(res[0].decode("utf-8"))
			self.assertEqual(1, len(data.keys()))
			client_key = list(data.keys())[0]
			note("Client key: %r" % client_key)
			entry = data[client_key]
			key = entry['key']
			self.assertIsNone(entry["previous"])
			self.assertEqual(key, msg["entry_id"])
			stored_msg = self.simulate_request("/store/{0}".format(key))
			self.assertEqual(self.srmock.status, falcon.HTTP_200)
			self.assertThat(json.loads(stored_msg[0].decode("utf-8")), matchers.MatchesStructure.fromExample(msg))

	@given(valid_msg, valid_msg)
	def test_msgs_get_stored_twice(self, first, second):
		assume(first["kind"] != second["kind"]) # should also work with this, but need to check it works without
		with self.withDB():
			self.store_item(first)
			self.store_item(second)
			res = self.simulate_request("/store")
			self.assertEqual(self.srmock.status, falcon.HTTP_200)
			data = json.loads(res[0].decode("utf-8"))
			self.assertEqual(1, len(data.keys()))
			client_key = list(data.keys())[0]
			note("Client key: %r" % client_key)
			entry = data[client_key]
			key = entry['key']
			self.assertIsNotNone(entry["previous"])
			stored_msg = self.simulate_request("/store/{0}".format(key))
			self.assertEqual(self.srmock.status, falcon.HTTP_200)
			self.assertDictEqual(json.loads(stored_msg[0].decode("utf-8")), second)

	def add_client(self, host="127.0.0.1", port=1234):
		self.simulate_request("/clients", body=json.dumps({"host": host, "port": port}), method='PUT')
		self.assertEqual(self.srmock.status, falcon.HTTP_201)

	def make_client(self):
		socket = self.context.socket(zmq.PAIR)
		socket.linger = 0
		port = socket.bind_to_random_port("tcp://*", min_port=2000)
		return (socket, port)

	def get_message(self, socket):
		socks = socket.poll(timeout=1000)
		if socks == 0:
			raise Exception("Nothing ready!")
		return socket.recv_json(flags=zmq.NOBLOCK)

	def check_get_message(self, socket, message):
		from_msg = self.get_message(socket)
		self.assertIsNotNone(from_msg)
		self.assertDictEqual(from_msg["data"], message)

	@given(valid_msg)
	def test_clients_get_messages(self, msg):
		(socket, port) = self.make_client()
		try:
			with self.withDB():
				self.add_client(port=port)
				self.store_item(msg)
				self.check_get_message(socket, msg)

				clients = self.get_client_list()
				client = clients["127.0.0.1:%d" % port]
				note("Client: %r" % client)
				self.assertEqual(1, len(client.keys()))
				client_key = list(client.keys())[0]
				self.assertDictEqual({client_key: msg["entry_id"]}, client)
		finally:
			socket.close()

	def check_empty_client_list(self, port=1234):
		clients = self.get_client_list()
		client = clients["127.0.0.1:%d" % port]
		self.assertDictEqual({}, client)

	def test_clients_have_state(self):
		with self.withDB():
			self.add_client()
			self.check_empty_client_list()

	@given(valid_msg, valid_msg)
	def test_clients_dont_update_if_older(self, first, second):
		with self.withDB():
			self.store_item(first)
			self.add_client()
			self.check_empty_client_list()

			self.store_item(second)
			self.check_empty_client_list() # because out of date

	@given(valid_msg)
	def test_clients_can_be_updated(self, msg):
		with self.withDB():
			self.store_item(msg)
			self.add_client()
			self.check_empty_client_list()

			self.simulate_request("/update/127.0.0.1:1234", method='POST')
			self.assertEqual(self.srmock.status, falcon.HTTP_200)

			clients = self.get_client_list()
			client = clients["127.0.0.1:1234"]
			note("Client: %r" % client)
			self.assertEqual(1, len(client.keys()))
			client_key = list(client.keys())[0]
			self.assertDictEqual({client_key: msg["entry_id"]}, client)

	@given(st.text())
	def test_update_fails_on_non_existent_client(self, client):
		self.simulate_request("/update/%s" % client, method='POST')
		self.assertEqual(self.srmock.status, falcon.HTTP_404)

	@given(valid_msg, valid_msg)
	def test_clients_can_be_updated_from_older(self, first, second):
		(socket, port) = self.make_client()
		with self.withDB():
			self.store_item(first)
			self.add_client(port=port)
			self.check_empty_client_list(port)

			self.store_item(second)
			self.check_empty_client_list(port)

			self.simulate_request("/update/127.0.0.1:%d" % port, method='POST')
			self.assertEqual(self.srmock.status, falcon.HTTP_200)

			self.check_get_message(socket, first)
			self.check_get_message(socket, second)

			clients = self.get_client_list()
			client = clients["127.0.0.1:%d" % port]
			note("Client: %r" % client)
			self.assertEqual(1, len(client.keys()))
			client_key = list(client.keys())[0]
			self.assertDictEqual({client_key: second["entry_id"]}, client)
