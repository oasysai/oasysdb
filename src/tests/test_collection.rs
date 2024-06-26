use super::*;

#[test]
fn test_collection_new() -> Result<(), Error> {
    let collection = get_test_collection()?;
    assert_eq!(collection.state()?.count, 0);
    Ok(())
}

#[test]
fn test_collection_add_field() -> Result<(), Error> {
    let collection = get_test_collection()?;
    let field = Field::new("id", DataType::Utf8, false);
    collection.add_fields(vec![field])?;

    // OasysDB has 2 default fields: internal_id and vector.
    let schema = collection.state()?.schema;
    assert_eq!(schema.fields().len(), 3);
    assert_eq!(schema.field(2).name(), "id");

    Ok(())
}
