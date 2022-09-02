# ACL Introdction

#### [Test Case] Insert Data.
From now on, we finished the access control list (ACL) funciton which allow user to use Reltion Graph with the different levels of access the data. 

In the source code, we can see, when start the Relation Graph, the first thing is initialize configuration of Database, which contains Loading the ACL Data file.

```
pub fn init_data() {
        let graph_store = GraphStore::<T>::new();

        let acl_data = include_bytes!("../../../data/relation_acl.ttl");
        let acl_graph = GraphName::from(NamedNode::new(GRAPH_NAME_ACL).unwrap());
        graph_store.load_graph(BufReader::new(&acl_data[..]),
                               GraphFormat::Turtle,
                               &acl_graph,
                               None).unwrap();

        // default graph
        let data = include_bytes!("../../../data/relation_samples.ttl");
        graph_store.load_graph(BufReader::new(&data[..]),
                               GraphFormat::Turtle,
                               &GraphName::DefaultGraph,
                               None).unwrap();

}
```

```
###################
# Schema
###################

acl:User a rdfs:Class ;
    rdfs:label "User" ;
    rdfs:comment "A db user." .

acl:id a rdf:Property ;
    rdfs:label "id" ;
    rdfs:comment "The id of a user." ;
    rdfs:range xsd:string .

acl:role a rdf:Property ;
    rdfs:label "role" ;
    rdfs:comment "The role of a user." ;
    rdfs:range xsd:string .
```
In this file, there is a db user schema which has different roles. For example, you can set "admin" for the administrator of db, and "user" for the normal user of db. Through this setting, you can give different person with different access level and privileges.


