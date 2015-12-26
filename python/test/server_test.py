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

JSONEncoder_olddefault = json.JSONEncoder.default
def JSONEncoder_newdefault(self, o):
	if isinstance(o, uuid.UUID): return str(o)
	return JSONEncoder_olddefault(self, o)
json.JSONEncoder.default = JSONEncoder_newdefault

json_st = st.recursive(st.floats() | st.booleans() | st.text() | st.none(), lambda children: st.lists(children) | st.dictionaries(st.text(), children))

valid_msg = st.fixed_dictionaries({
	'table': st.text(min_size=1),
	'id': st.uuids(),
	'entry_id': st.uuids(),
	'data': st.dictionaries(st.text(), st.floats() | st.booleans() | st.text() | st.none())
})

class ServerTest(testing.TestBase):
	def before(self):
		self.ports = []
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
		self.existing_stores = []

	def after(self):
		self.info["event"].set()
		self.info["clients"].clear()

	@given(st.text(min_size = 1))
	def test_clients_dislikes_args(self, s):
		res = self.simulate_request("/clients", body = s)
		self.assertEqual(self.srmock.status, falcon.HTTP_400)

	def get_client_list(self):
		res = self.simulate_request("/clients")
		self.assertEqual(self.srmock.status, falcon.HTTP_200)
		data = json.loads(res[0].decode("utf-8"))
		return data

	def test_clients_works(self):
		data = self.get_client_list()
		self.assertEqual(type(data), dict)
		self.assertEqual(self.srmock.status, falcon.HTTP_200)

	@given(st.fixed_dictionaries({
		'host': st.text(alphabet=string.ascii_letters + string.digits + "-", min_size = 3, average_size = 5),
		'port': st.integers(min_value = 1)
	}))
	def test_clients_can_be_added(self, s):
		assume(not s["host"].startswith("-"))
		res = self.simulate_request("/clients", body = json.dumps(s), method = 'PUT')
		self.assertEqual(self.srmock.status, falcon.HTTP_200)
		clients_now = self.get_client_list()
		note("now: %r" % clients_now)
		self.assertTrue("{host}:{port}".format(**s) in clients_now.keys())

	@given(st.text())
	def test_store_cant_decode_random_text(self, s):
		res = self.simulate_request("/store", body = s, method = 'PUT')
		self.assertEqual(self.srmock.status, falcon.HTTP_400)

	@given(json_st)
	def test_store_doesnt_like_random_json(self, s):
		res = self.simulate_request("/store", body = json.dumps(s), method = 'PUT')
		self.assertEqual(self.srmock.status, falcon.HTTP_400)

	@given(valid_msg)
	def test_store_likes_proper_stores(self, s):
		assume(s["entry_id"] not in self.existing_stores)
		res = self.simulate_request("/store", body = json.dumps(s), method = 'PUT')
		note("res: %r" % res)
		self.assertEqual(self.srmock.status, falcon.HTTP_200)
		self.existing_stores.append(s["entry_id"])

	@given(valid_msg)
	def test_store_forbids_double_store(self, s):
		assume(s["entry_id"] not in self.existing_stores)
		res = self.simulate_request("/store", body = json.dumps(s), method = 'PUT')
		note(self.srmock.__dict__)
		self.assertEqual(self.srmock.status, falcon.HTTP_200)
		res = self.simulate_request("/store", body = json.dumps(s), method = 'PUT')
		self.assertEqual(self.srmock.status, falcon.HTTP_409)
		self.existing_stores.append(s["entry_id"])
