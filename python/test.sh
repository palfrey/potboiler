#!/bin/sh
ulimit -n 65536 65536
py.test --ignore=ENV -f --timeout=3 -v
