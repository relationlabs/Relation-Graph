# Relation-Graph user guide
## prepare
- get the Executable program 
```shell
git clone https://github.com/relationlabs/Relation-Graph.git
```
- choose the program for your OS and unzip it, take MacOS as example
```shell
cd Executable Program
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
please make sure the port 9944 is available,and if the program launch successfully, you see the command line as below
![image](https://user-images.githubusercontent.com/91399393/176400350-874d2ebe-c01b-47af-9f3e-8fc7dcd17b7d.png)

## Connection Node
- open the browser (highly recommended chrome, Safari and others browsers may encounter some problems)
- open the link: https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer
- when connection the node, you will see the page display as below
![image](https://user-images.githubusercontent.com/91399393/176409173-c464e241-a6e5-4609-b9a6-21a61b37753f.png)


## Usage
- switch to extrinsics from top navigation bar
```shell 
Developer -> extrinsics
```
- choose a account which have balance and choose [graphdb] extrinsic，then initDb
![image](https://user-images.githubusercontent.com/91399393/176415644-857882ac-5eda-43a5-8082-e985aa518bd9.png)
- click Sign and Submit
![image](https://user-images.githubusercontent.com/91399393/176415961-9814c3f1-52dd-4215-a873-a2cf261a1fbb.png)
- after transaction success, switch to sparqlUpdate to manipulate the database
![image](https://user-images.githubusercontent.com/91399393/176416651-8318b78e-8373-4f70-9cff-7a83ad496c01.png)
- [testcase] insert Data. Sample SPARQL: insert a record for person P1001
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
paste this sparql to browser [update] area, and submit this transaction
![image](https://user-images.githubusercontent.com/91399393/176417734-9100574b-df72-4088-84dc-e1c65c6f937a.png)

- [testcase] Update Data by delete & update existing record. Sample SPARQL: select the record for person P1001
```
curl -H "Content-Type: application/json" \
    -d '{"id":1, "jsonrpc":"2.0", "method": "sparql_query", "params": ["SELECT ?name ?age  WHERE { :P1001 :name ?name; :age ?age .}"]}' \
    http://localhost:9933
```
