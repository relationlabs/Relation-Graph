wrk.method = "POST"
wrk.body   = '{"id":1, "jsonrpc":"2.0", "method": "sparql_query", "params": ["SELECT ?friends1 WHERE { :P1 :friends ?friends1 . }"]}'
wrk.headers["Content-Type"] = "application/json"
