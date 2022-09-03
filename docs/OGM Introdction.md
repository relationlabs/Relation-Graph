# OGM Introdction

In order to let user to use Relation Graph conveniently, we provide the OGM tools which is for converting data between type systems using Rust.


There are 6 functions in OGM tools


Next, we will use an example to introduce the process of implementing OGM of Relation Graph.

Example: 
- Query a user info by name base on the OGM tools:

```
let userName = "P1001";
let result: GraphReslut<Option<User>> = find_user(userName).await;
```

- The Implement of function of **find_user** is:
```
pub async fn find_user(id: &str) -> GraphResult<Option<User>> {
    ogm::find_one(
    FindById {
        id: &id.to_string(),
    }
    ).await
}

pub mod templates {
    use askama::Template as SparqlTemplate;

    #[derive(SparqlTemplate, Debug)]
    #[template(path = "user_find_by_name.rq", escape = "none")]
    pub struct FindById<'a> {
        pub id: &'a str,
    }
}

```
- In the query project, we create a file folder named **templates**, and
the sparkQL of query user  **user_find_by_name.rq**

![image](https://user-images.githubusercontent.com/91399393/188111906-455adf98-a72f-43ec-8795-8a346a0e8057.png)

- In the **askama.toml** config file, set the path of **templates** folder

![image](https://user-images.githubusercontent.com/91399393/188111736-1ab61449-6569-4b4c-b5d1-b37b61246d71.png)

- In the **find_one** function, the most important thing is converting data logic which implement in the function of **as_typed_value**

```
pub async fn find_one<T: DeserializeOwned, S: SparqlTemplate>(query: S) -> GraphResult<Option<T>> {
    if let Ok(QueryResults::Solutions(mut solutions)) = execute_query(query).await {
        match solutions.next() {
            Some(first) => {
                match solutions.next() {
                    Some(_) => {
                        Err(GraphError::ResultNotUnique("Expected one record".to_string()))
                    }
                    None => {
                        let record: Option<T> = first?.as_typed_value();
                        Ok(record)
                    }
                }
            }
            None => Ok(None),
        }
    } else {
        Ok(None)
    }
}

pub fn as_typed_value<T: DeserializeOwned>(&self) -> Option<T> {
    let mut values = Map::new();
    for (variable, term) in self.iter() {
        if let Term::Literal(literal) = term {
            let value = literal.as_json_value();
            values.insert(variable.as_str().to_owned(), value);
        }
    }
    if values.is_empty() {
        return None;
    }
    let record: Result<T, _> = serde_json::from_value(Value::Object(values));
    record.map_or(None, |r| Some(r))
}

fn as_json_value(&self) -> Value {
    let value = self.value();
    match self.datatype() {
        xsd::BYTE => json!(value.parse::<i8>().unwrap_or_default()),
        xsd::SHORT => json!(value.parse::<i16>().unwrap_or_default()),
        xsd::INT => json!(value.parse::<i32>().unwrap_or_default()),
        // xs:int is a signed 32-bit integer
        // xs:integer is an integer unbounded value
        // INTEGER: Signed integers of arbitrary length (TODO: BigInteger)
        // LONG: 64 bit signed integers
        xsd::NEGATIVE_INTEGER => {
            //  Strictly negative integers of arbitrary length (<0)
            // if >=0, fallback to -1, TODO: better way to handle
            json!(value.parse::<i64>().map_or(-1, |v| if v >= 0 { -1 } else { v }))
        }
        xsd::NON_POSITIVE_INTEGER => {
            //  Strictly negative or equal to zero (<=0)
            json!(value.parse::<i64>().map_or(0, |v| if v > 0 { 0 } else { v }))
        }
        xsd::POSITIVE_INTEGER => {
            //  Strictly positive number (>0)
            // if <0, fallback to 1, TODO: better way to handle
            json!(value.parse::<i64>().map_or(1, |v| if v <= 0 { 1 } else { v }))
        }
        xsd::INTEGER | xsd::NON_NEGATIVE_INTEGER | xsd::LONG => json!(value.parse::<i64>().unwrap_or_default()),
        xsd::FLOAT => json!(value.parse::<f32>().unwrap_or_default()),
        xsd::DOUBLE => json!(value.parse::<f64>().unwrap_or_default()),
        xsd::UNSIGNED_BYTE => json!(value.parse::<u8>().unwrap_or_default()),
        xsd::UNSIGNED_SHORT => json!(value.parse::<u16>().unwrap_or_default()),
        xsd::UNSIGNED_INT => json!(value.parse::<u32>().unwrap_or_default()),
        xsd::UNSIGNED_LONG => json!(value.parse::<u64>().unwrap_or_default()),
        // Arbitrary-precision decimal numbers, rust_decimal::Decimal
        xsd::DECIMAL => json!(value), // TODO
        // NORMALIZED_STRING: Whitespace-normalized strings
        xsd::STRING | xsd::NORMALIZED_STRING => json!(value),
        _ => json!(value),
    }
}
```
