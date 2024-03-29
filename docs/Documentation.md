# Documentation  

`Relation Graph` is a substrate pallet that allows anyone to use GraphDB in [Substrate platform](https://substrate.io/).
`Relation Graph` provides organizations with ready-to-use GraphDB service for successfully running Dapps on the Substrate.  using `Relation Graph` Dapps builders can focus on bussiness logic by removing the complexities of Substrate.

## Prepare
There are two ways to start up this project, you can choose either one.
1. Get the executable file and launch it directly.
2. Download the src code, compile and launch it.

### 1. Launch project through executable file
#### Get the executable for your OS(Mac or Linux)
[executable file](https://github.com/relationlabs/Relation-Graph/tree/executable-files/Executable%20Files)

#### Mac

- Choose the program for MacOS and unzip it
```shell
cd /Executable Files
unzip subgraph-macos.zip
```

- Launch the executable file
```shell
./subgraph-macos --dev  --base-path ./test-chain
```

#### Linux

- Choose the program for Linux and unzip it
```shell
cd /Executable Files
unzip subgraph-linux.zip
```

- Launch the executable file
```shell
./subgraph-linux --dev  --base-path ./test-chain
```
------

### 2. Start up by compiling the src code
Before compile the src code, please make sure your OS has installed "cargo",which is the Rust build tool and package manager. 
#### Get and compile src code 
- Get the  src code 
```shell
git clone https://github.com/relationlabs/Relation-Graph.git
```
- Compile it 
```shell
cd /src
SKIP_WASM_BUILD=1 cargo build
```
#### Launch the compiled file
```shell
./target/debug/node-template --dev  --base-path ./test-chain
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
