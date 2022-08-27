//! An OGM (Object Graph Mapper) maps nodes and relationships in the graph to objects
//! and references in a domain model.

use serde::de::DeserializeOwned;
use serde_json::{json, Map, Value};

use crate::model::{Literal, Term, vocab::xsd};
use crate::sparql::QuerySolution;

impl QuerySolution {
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
}

impl Literal {
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
}
