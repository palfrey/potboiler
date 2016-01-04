import potboiler

import falcon.testing as testing
import falcon
from hypothesis import given, note, assume
import hypothesis.strategies as st
import tempfile
import json
import uuid
import string
import zmq
import contextlib

json_st = st.recursive(st.floats() | st.booleans() | st.text() | st.none(), lambda children: st.lists(children) | st.dictionaries(st.text(), children))

def msg_gen(kind):
	return st.fixed_dictionaries({
		'kind': kind,
		'id': st.uuids(),
		'entry_id': st.uuids(),
		'data': st.dictionaries(st.text(), st.floats() | st.booleans() | st.text() | st.none())
	})

valid_msg = msg_gen(st.text(min_size=1))
def same_kind_msg(count):
	return st.text(min_size=1).flatmap(lambda t: st.lists(msg_gen(st.just(t)), min_size = count, max_size = count))

class ServerTest(testing.TestBase):
	def before(self):
		while True:
			port = st.integers(min_value=2000).example()
			try:
				self.info = potboiler.make_api(tempfile.mkdtemp(), port)
				break
			except zmq.error.ZMQError as e:
				if e.strerror == "Address already in use":
					continue
				else:
					raise
		self.api = self.info["api"]
		self.existing_stores = list(self.info["db"].RangeIter(include_value = False))
		self.db = self.info["db"]

	@contextlib.contextmanager
	def withDB(self):
		try:
			# Clear out any existing keys
			keys = list(self.db.RangeIter(include_value = False))
			for k in keys:
				if k == potboiler.kind_key:
					value = json.loads(self.db.Get(k).decode("utf-8"))
					if value != {}:
						self.db.Put(k, "{}".encode("utf-8"))
					continue
				else:
					self.db.Delete(k)
			yield
		finally:
			if not hasattr(self, "info"):
				return
			self.info["event"].set()
			self.info["clients"].clear()

	@given(st.text(min_size = 1))
	def test_clients_dislikes_args(self, s):
		with self.withDB():
			res = self.simulate_request("/clients", body = s)
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
		'host': st.text(alphabet=string.ascii_letters + string.digits + "-", min_size = 3, average_size = 5),
		'port': st.integers(min_value = 1)
	}))
	def test_clients_can_be_added(self, s):
		with self.withDB():
			assume(not s["host"].startswith("-"))
			res = self.simulate_request("/clients", body = json.dumps(s), method = 'PUT')
			self.assertEqual(self.srmock.status, falcon.HTTP_201)
			clients_now = self.get_client_list()
			note("now: %r" % clients_now)
			self.assertTrue("{host}:{port}".format(**s) in clients_now.keys())

	@given(st.text())
	def test_store_cant_decode_random_text(self, s):
		with self.withDB():
			res = self.simulate_request("/store", body = s, method = 'PUT')
			self.assertEqual(self.srmock.status, falcon.HTTP_400)

	@given(json_st)
	def test_store_doesnt_like_random_json(self, s):
		with self.withDB():
			res = self.simulate_request("/store", body = json.dumps(s), method = 'PUT')
			self.assertEqual(self.srmock.status, falcon.HTTP_400)

	def store_item(self, msg):
		res = self.simulate_request("/store", body = json.dumps(msg), method = 'PUT')
		note("res: %r" % res)
		note("db: %r" % list(self.info["db"].RangeIter(include_value = False)))
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
			res = self.simulate_request("/store", body = json.dumps(s), method = 'PUT')
			self.assertEqual(self.srmock.status, falcon.HTTP_409)

	def test_can_get_kinds(self):
		res = self.simulate_request("/kinds")
		self.assertEqual(self.srmock.status, falcon.HTTP_200)
		data = json.loads(res[0].decode("utf-8"))
		self.assertEqual(dict, type(data))

	@given(valid_msg)
	def test_can_get_kinds(self, msg):
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
