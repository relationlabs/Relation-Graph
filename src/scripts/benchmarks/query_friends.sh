#!/bin/bash

# https://github.com/wg/wrk
# brew install wrk
wrk -t2 -c10 -d2m -s query_friends.lua  http://localhost:9933