import falcon
import json
import leveldb
import zmq
import time
import threading

url_client = "tcp://*:5555"

db = leveldb.LevelDB('./db')

context = zmq.Context.instance()
clients = context.socket(zmq.ROUTER)
clients.bind(url_client)
poller = zmq.Poller()
poller.register(clients, zmq.POLLIN)

class Client:
    def __init__(self):
        self.zmq = None

class StoreResource:
    def on_put(self, req, resp):
        resp.body = json.dumps(quote)

api = falcon.API()
api.add_route('/store', StoreResource())

def ZMQ():
	while True:
		if len(poller.sockets) == 0:
			time.sleep(5)
			continue
		try:
			socks = dict(poller.poll())
		except KeyboardInterrupt:
			break
		print ("socks", socks)

thread = threading.Thread(target=ZMQ, args=(), daemon = True)
thread.start()
