#!/bin/sh
ulimit -n 65536 65536
export CMD="py.test --ignore=ENV --verbose --timeout=30 --timeout-method=thread --exitfirst --ff && ./check.sh"
#eval $CMD
watchman-make --make="$CMD" -p '**/*.py' -t dummy
