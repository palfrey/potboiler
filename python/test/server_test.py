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

JSONEncoder_olddefault = json.JSONEncoder.default
def JSONEncoder_newdefault(self, o):
	if isinstance(o, uuid.UUID): return str(o)
	return JSONEncoder_olddefault(self, o)
json.JSONEncoder.default = JSONEncoder_newdefault

json_st = st.recursive(st.floats() | st.booleans() | st.text() | st.none(), lambda children: st.lists(children) | st.dictionaries(st.text(), children))

def msg_gen(table):
	return st.fixed_dictionaries({
		'table': table,
		'id': st.uuids(),
		'entry_id': st.uuids(),
		'data': st.dictionaries(st.text(), st.floats() | st.booleans() | st.text() | st.none())
	})

valid_msg = msg_gen(st.text(min_size=1))
same_table_msg = st.text(min_size=1).flatmap(lambda t: msg_gen(st.just(t)))

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
				if k == potboiler.table_key:
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
			self.assertEqual(self.srmock.status, falcon.HTTP_200)
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
		self.assertEqual(self.srmock.status, falcon.HTTP_200)
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

	def test_can_get_tables(self):
		res = self.simulate_request("/tables")
		self.assertEqual(self.srmock.status, falcon.HTTP_200)
		data = json.loads(res[0].decode("utf-8"))
		self.assertEqual(dict, type(data))

	@given(valid_msg)
	def test_can_get_tables(self, msg):
		with self.withDB():
			res = self.simulate_request("/store", body = json.dumps(msg), method = 'PUT')
			note("res: %r" % res)
			note("db: %r" % list(self.info["db"].RangeIter(include_value = False)))
			self.assertEqual(self.srmock.status, falcon.HTTP_200)
			self.existing_stores.append(msg["entry_id"])
			res = self.simulate_request("/tables")
			self.assertEqual(self.srmock.status, falcon.HTTP_200)
			data = json.loads(res[0].decode("utf-8"))
			self.assertEqual(dict, type(data))
			note("Data: %r" % data)
			self.assertIn(msg["table"], data.keys())
			self.assertIn("key", data[msg["table"]].keys())

	@given(same_table_msg, same_table_msg)
	def test_multiple_table_insert(self, first, second):
		with self.withDB():
			self.store_item(first)
			self.store_item(second)
			res = self.simulate_request("/tables")
			self.assertEqual(self.srmock.status, falcon.HTTP_200)
			data = json.loads(res[0].decode("utf-8"))
			self.assertEqual(dict, type(data))
			note("Data: %r" % data)
			self.assertTrue(first["table"] in data.keys())
		#raise Exception("%r - %r"%(first, second))
