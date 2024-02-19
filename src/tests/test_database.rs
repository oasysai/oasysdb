use super::*;

#[test]
fn new() {
    let db = Database::new("data/new").unwrap();
    assert_eq!(db.len(), 0);
}

#[test]
fn create_collection() {
    let mut db = Database::new("data/create_collection").unwrap();

    let records = gen_records(128, 100);
    let collection: Collection<usize, 32> =
        db.create_collection("test", None, Some(&records)).unwrap();

    assert_eq!(collection.len(), 100);
    assert_eq!(db.len(), 1);
}

#[test]
fn get_collection() {
    let db = create_test_database("data/get_collection");
    let collection: Collection<usize, 32> =
        db.get_collection("vectors").unwrap();
    assert_eq!(collection.len(), 100);
}

#[test]
fn save_collection_new() {
    let mut db = Database::new("data/save_collection_new").unwrap();

    // Create a collection from scratch.
    let config = Config::default();
    let mut collection: Collection<usize, 32> = Collection::new(&config);
    collection.insert(&gen_records(128, 1)[0]).unwrap();

    db.save_collection("new", &collection).unwrap();
    assert_eq!(collection.len(), 1);
    assert_eq!(db.len(), 1);
}

#[test]
fn save_collection_update() {
    let mut db = create_test_database("data/save_collection_update");

    // Update the collection.
    let mut collection: Collection<usize, 32> =
        db.get_collection("vectors").unwrap();
    collection.insert(&gen_records(128, 1)[0]).unwrap();

    db.save_collection("vectors", &collection).unwrap();
    assert_eq!(collection.len(), 101);
    assert_eq!(db.len(), 1);
}

#[test]
fn delete_collection() {
    let mut db = create_test_database("data/delete_collection");
    db.delete_collection("vectors").unwrap();
    assert_eq!(db.len(), 0);
}
