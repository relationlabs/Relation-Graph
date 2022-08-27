# Subgraph

`Subgraph` is a substrate pallet that allows anyone to use GraphDB in [Substrate platform](https://substrate.io/).
It supports the following specifications:
* [SPARQL 1.1 Query](https://www.w3.org/TR/sparql11-query/), [SPARQL 1.1 Update](https://www.w3.org/TR/sparql11-update/), and [SPARQL 1.1 Federated Query](https://www.w3.org/TR/sparql11-federated-query/).
* [Turtle](https://www.w3.org/TR/turtle/), [TriG](https://www.w3.org/TR/trig/), [N-Triples](https://www.w3.org/TR/n-triples/), and [N-Quads](https://www.w3.org/TR/n-quads/).
* [SPARQL 1.1 Query Results JSON Format](https://www.w3.org/TR/sparql11-results-json/) and [SPARQL 1.1 Query Results CSV and TSV Formats](https://www.w3.org/TR/sparql11-results-csv-tsv/).

![Subgraph architecture](docs/images/architecture.png)

## Usage

### SPARQL Update

Call extrinsic `sparql_update` with SPARQL for `insert, update, delete` operations.

Try SPARQL update in [Pallet Interactor](http://localhost:8000/substrate-front-end-template) as follows.

#### Insert Data

Sample SPARQL: insert a record for person `P001`

```sparql
INSERT DATA
{
   :P001 :name "Luna" ;
         :gender "Female" ;
         :age 35 ;
         :birthdate "1986-10-14"^^xsd:date ;
         :friends :P2, :P3 .
}
```

#### Update Data

Changes to existing triples are performed as a delete operation followed by an insert operation in a single update request. 
The specification refers to this as `DELETE/INSERT`

Sample SPARQL: update age to `36` for person `P001`

```sparql
DELETE
{ :P001 :age ?o }
INSERT
{ :P001 :age 36 }
WHERE
{ :P001 :age ?o }
```

#### Delete Data

Sample SPARQL: delete all properties of person `P001`

```sparql
DELETE WHERE
{
:P001 ?p ?o .
}
```
Sample SPARQL: delete partial properties of person `P001`

```sparql
DELETE WHERE
{
:P001 :age ?age;
      :name ?name .
}
```

### SPARQL Query

Call RPC `sparql_query` with SPARQL for `query` operations.

```bash
curl -H "Content-Type: application/json" \
    -d '{"id":1, "jsonrpc":"2.0", "method": "sparql_query", "params": ["SELECT ?name ?age  WHERE { :P1 :name ?name; :age ?age .}"]}' \
    http://localhost:9933    
```

## Build

```bash
# build native node for subgraph
SKIP_WASM_BUILD=1 cargo build
```

## Run

```bash
./target/debug/node-template --dev  --base-path ./test-chain
```

### Sparql Query

```sh
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "sparql_query", "params": ["SELECT ?friends1 WHERE { :P1 :friends ?friends1 . }"]}' \
     http://localhost:9933 
     
```

### Sparql Update

* Update

Update data in [Pallet Interactor](http://localhost:8000/substrate-front-end-template)

```SPARQL
DELETE
{ :P1 :age ?o }
INSERT
{ :P1 :age 26 }
WHERE
{ :P1 :age ?o }
```

* Query

Query to check update result.

```sh
curl -H "Content-Type: application/json" \
    -d '{"id":1, "jsonrpc":"2.0", "method": "sparql_query", "params": ["SELECT ?name ?age  WHERE { :P1 :name ?name; :age ?age .}"]}' \
    http://localhost:9933    
     
```

```sh
curl -H "Content-Type: application/json" \
    -d '{"id":2, "jsonrpc":"2.0", "method": "sparql_query", "params": ["SELECT DISTINCT ?name ?age ?gender ?birthdate WHERE {:P1 :friends ?friend1. ?friends1 :friends  ?friends2. ?friends2 :friends  ?friends3. ?friends3 a :Person ; :name ?name; :age ?age; :gender ?gender; :birthdate ?birthdate.} LIMIT 10"]}' \
    http://localhost:9933
```

## Benchmarks

Run scripts in `scripts/benchmarks`
