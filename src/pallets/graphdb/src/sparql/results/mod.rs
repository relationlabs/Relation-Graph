//! SPARQL 1.1 Query Results Formats (XML, JSON, CSV)

pub(crate) mod csv_results;

#[cfg(feature = "sparql-results-json")]
pub(crate) mod json_results;

#[cfg(feature = "sparql-results-xml")]
pub(crate) mod xml_results;