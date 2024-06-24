use super::*;

#[test]
fn test_create_collection() -> Result<(), Error> {
    let db = create_new_test_database()?;
    db._create_collection("test_collection")?;

    let state = db.state()?;
    assert!(state.collection_refs.contains_key("test_collection"));

    Ok(())
}
