import requests
import sys
import time

last_error = None

while True:
    error = None
    try:
        req = requests.get(sys.argv[1])
        req.raise_for_status()
        break
    except requests.exceptions.ConnectionError as e:
        error = "Connection error, sleeping"
    except requests.exceptions.RequestException as e:
        print(e)
        raise
    if last_error != error:
        last_error = error
        print(error)
    time.sleep(2)

print("%s responded ok" % sys.argv[1])