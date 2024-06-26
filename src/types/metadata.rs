use arrow_schema::{DataType, Field};

/// Data types supported in OasysDB Arrow fields.
pub enum MetadataType {
    Integer,
    Float,
    String,
    Boolean,
    Vector,
}

// Available OasysDB data types in string form.
// This constant prevents typos in the code.
const INTEGER: &str = "integer";
const FLOAT: &str = "float";
const STRING: &str = "string";
const BOOLEAN: &str = "boolean";
const VECTOR: &str = "vector";

// Implement interoperability FROM and INTO other data types.

impl From<&str> for MetadataType {
    fn from(value: &str) -> Self {
        match value {
            INTEGER => MetadataType::Integer,
            FLOAT => MetadataType::Float,
            STRING => MetadataType::String,
            BOOLEAN => MetadataType::Boolean,
            VECTOR => MetadataType::Vector,
            _ => panic!("Unsupported metadata type: {value}"),
        }
    }
}

impl From<String> for MetadataType {
    fn from(value: String) -> Self {
        MetadataType::from(value.as_str())
    }
}

impl From<MetadataType> for DataType {
    fn from(value: MetadataType) -> Self {
        let field_float = Field::new("element", DataType::Float32, false);
        match value {
            MetadataType::Integer => DataType::Int32,
            MetadataType::Float => DataType::Float32,
            MetadataType::String => DataType::Utf8,
            MetadataType::Boolean => DataType::Boolean,
            MetadataType::Vector => DataType::List(field_float.into()),
        }
    }
}
