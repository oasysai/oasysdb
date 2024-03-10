use super::*;

#[test]
fn new() {
    let db = Database::new("data/001").unwrap();
    assert_eq!(db.len(), 0);
}

#[test]
fn create_collection() {
    let mut db = Database::new("data/002").unwrap();

    let records = Some(Record::many_random(DIMENSION, LEN));
    let collection = db.create_collection("test", None, records).unwrap();

    assert_eq!(collection.len(), LEN);
    assert_eq!(db.len(), 1);
}

#[test]
fn get_collection() {
    let db = create_test_database("data/003");
    let collection = db.get_collection(NAME).unwrap();
    assert_eq!(collection.len(), LEN);
}

#[test]
fn save_collection_new() {
    let mut db = Database::new("data/004").unwrap();

    // Create a collection from scratch.
    let config = Config::default();
    let mut collection = Collection::new(&config);
    collection.insert(&Record::random(DIMENSION)).unwrap();

    db.save_collection("new", &collection).unwrap();
    assert_eq!(collection.len(), 1);
    assert_eq!(db.len(), 1);
}

#[test]
fn save_collection_update() {
    let mut db = create_test_database("data/005");

    // Update the collection.
    let mut collection = db.get_collection(NAME).unwrap();
    collection.insert(&Record::random(DIMENSION)).unwrap();

    db.save_collection(NAME, &collection).unwrap();
    assert_eq!(collection.len(), LEN + 1);
    assert_eq!(db.len(), 1);
}

#[test]
fn delete_collection() {
    let mut db = create_test_database("data/006");
    db.delete_collection(NAME).unwrap();
    assert_eq!(db.len(), 0);
}
