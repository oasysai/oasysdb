use super::*;
use crate::db::collection::Collection;
use arrow::datatypes::{DataType, Field};

#[test]
fn test_collection_new() {
    let collection = Collection::new();
    assert_eq!(collection.count(), 0);
}

#[test]
fn test_collection_add_field() -> Result<(), Error> {
    let collection = Collection::new();
    let field = Field::new("id", DataType::Utf8, false);
    collection.add_fields(vec![field])?;

    let schema = collection.schema()?;
    assert_eq!(schema.fields().len(), 1);
    assert_eq!(schema.field(0).name(), "id");

    Ok(())
}
