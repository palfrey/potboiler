import falcon.testing as testing
import falcon
from hypothesis import given, note, assume
import hypothesis.strategies as st
import server
import tempfile
import json
import uuid

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
			if port in self.ports:
				continue
			self.ports.append(port)
			break
		self.api = server.make_api(tempfile.mkdtemp(), port)
		self.existing_stores = []

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
		note(self.srmock.__dict__)
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
