# Relation-Graph user guide

## Prepare
- Get the Executable program 
```shell
git clone https://github.com/relationlabs/Relation-Graph.git
```
- Choose the program for your OS and unzip it, take MacOS as example
```shell
cd /Executable Program
unzip subgraph-macos.zip
```
## Launch
### MacOS
```shell
./subgraph-macos --dev  --base-path ./test-chain
```
### Linux
```shell
./subgraph-linux --dev  --base-path ./test-chain
```
Please make sure the port 9944 and 9933 is available,and if the program launch successfully, you see the command line as below

![image](https://user-images.githubusercontent.com/91399393/176400350-874d2ebe-c01b-47af-9f3e-8fc7dcd17b7d.png)

## Connection Node
- Open the browser (highly recommended chrome, Safari and others browsers may encounter some problems)
- Open the link: https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer
- When connection the node, you will see the page display as below

![image](https://user-images.githubusercontent.com/91399393/176409173-c464e241-a6e5-4609-b9a6-21a61b37753f.png)


## Usage
- Switch to extrinsics from top navigation bar
```shell 
Developer -> extrinsics
```
- Choose a account which has balance and choose [graphdb] extrinsic，then initDb

![image](https://user-images.githubusercontent.com/91399393/176415644-857882ac-5eda-43a5-8082-e985aa518bd9.png)

- Click Sign and Submit

![image](https://user-images.githubusercontent.com/91399393/176415961-9814c3f1-52dd-4215-a873-a2cf261a1fbb.png)

- After transaction success, switch to sparqlUpdate to manipulate the database

![image](https://user-images.githubusercontent.com/91399393/176416651-8318b78e-8373-4f70-9cff-7a83ad496c01.png)

- [test case] Insert Data. 
Sample SPARQL: insert a record for person P1001
```
INSERT DATA
{
   :P1001 :name "Luna" ;
         :gender "Female" ;
         :age 35 ;
         :birthdate "1986-10-14"^^xsd:date ;
         :friends :P2, :P3 .
}
```
Paste this sparql to browser [update] area, and submit this transaction

![image](https://user-images.githubusercontent.com/91399393/176417734-9100574b-df72-4088-84dc-e1c65c6f937a.png)

- [test case] Update Data
Changes to existing triples are performed as a delete operation followed by an insert operation in a single update request. The specification refers to this as DELETE/INSERT

Sample SPARQL: update age to 36 for person P001

```
DELETE  { :P1001 :age ?o } INSERT { :P1001 :age 36 } WHERE { :P1001 :age ?o }
```
Paste this sparql to browser [update] area, and submit this transaction

![image](https://user-images.githubusercontent.com/91399393/176489974-5be46194-bd71-4d2a-abed-15bcd7b9ff26.png)
- [test case] Delete Data
Sample SPARQL: delete all properties of person P001
```
DELETE  WHERE { :P1001 ?p ?o. } 
```
Paste this sparql to browser [update] area, and submit this transaction

![image](https://user-images.githubusercontent.com/91399393/176491417-04c759b9-2f6a-4a26-be0d-aa23f51f64d6.png)

Sample SPARQL: delete partial properties of person P001

```
DELETE  WHERE { :P1001 :age ?age; :name ?name . } 
```
Paste this sparql to browser [update] area, and submit this transaction

![image](https://user-images.githubusercontent.com/91399393/176491911-fa8e9089-5c78-4054-929d-aed3f38099dc.png)

- [test case] SPARQL Query
For now, data query can only operate by calling RPC sparql_query with SPARQL.
Sample SPARQL: query the basic personal properties of person P001
```
curl -H "Content-Type: application/json" \
    -d '{"id":1, "jsonrpc":"2.0", "method": "sparql_query", "params": ["SELECT ?name ?age  WHERE { :P1001 :name ?name; :age ?age .}"]}' \
    http://localhost:9933
```
Paste this sparql to command line

![image](https://user-images.githubusercontent.com/91399393/176492690-0246ee9b-fe97-4bf3-a7fb-9cdfda8ee541.png)

Sample SPARQL: query the relationship of person P001
```
curl -H "Content-Type: application/json" \
    -d '{"id":2, "jsonrpc":"2.0", "method": "sparql_query", "params": ["SELECT DISTINCT ?name ?age ?gender ?birthdate WHERE {:P1 :friends ?friend1. ?friends1 :friends  ?friends2. ?friends2 :friends  ?friends3. ?friends3 a :Person ; :name ?name; :age ?age; :gender ?gender; :birthdate ?birthdate.} LIMIT 10"]}' \
    http://localhost:9933
```
Paste this sparql to command line

![image](https://user-images.githubusercontent.com/91399393/176493931-21f3f8e5-fffe-4e6c-83c2-93e4d717e003.png)
