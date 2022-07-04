## Documentation  

## Overview
`Relation Graph` is a substrate pallet that allows anyone to use GraphDB in [Substrate platform](https://substrate.io/).
`Relation Graph` provides organizations with ready-to-use GraphDB service for successfully running Dapps on the Substrate.  using `Relation Graph` Dapps builders can focus on bussiness logic by removing the complexities of Substrate.

## Prepare
### Get the Executable files 
```shell
git clone https://github.com/relationlabs/Relation-Graph.git
```
### Choose the program for your OS and unzip it, take MacOS as example
```shell
cd /Executable Files
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
- When connect the node successfully, you will see the page display as below

![image](https://user-images.githubusercontent.com/91399393/176409173-c464e241-a6e5-4609-b9a6-21a61b37753f.png)


## Usage
### Switch to extrinsics from top navigation bar
```shell 
Developer -> extrinsics
```
### Choose a account which has balance and choose [graphdb] extrinsic，then initDb

![image](https://user-images.githubusercontent.com/91399393/176415644-857882ac-5eda-43a5-8082-e985aa518bd9.png)

### Click Sign and Submit

![image](https://user-images.githubusercontent.com/91399393/176415961-9814c3f1-52dd-4215-a873-a2cf261a1fbb.png)

### After transaction success, switch to sparqlUpdate to manipulate the database

![image](https://user-images.githubusercontent.com/91399393/176416651-8318b78e-8373-4f70-9cff-7a83ad496c01.png)
