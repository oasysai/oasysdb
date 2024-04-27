use super::*;

#[test]
fn new_with_distance() {
    let mut config = Config::default();
    config.distance = Distance::Cosine;
    let mut collection = Collection::new(&config);
    collection.insert(&Record::random(DIMENSION)).unwrap();
}

#[test]
fn build_large() {
    let len = 10000;
    let records = Record::many_random(DIMENSION, len);
    let config = Config::default();
    let collection = Collection::build(&config, &records).unwrap();
    assert_eq!(collection.len(), len);
}

#[test]
fn insert() {
    let mut collection = create_collection();

    // Create a new record to insert.
    let new_record = Record::random(DIMENSION);
    collection.insert(&new_record).unwrap();

    // Assert the new record is in the collection.
    let id = VectorID::from(LEN);
    assert_eq!(collection.len(), LEN + 1);
    assert_eq!(collection.get(&id).unwrap().data, new_record.data);
}

#[test]
fn insert_invalid_dimension() {
    let mut collection = create_collection();

    // Create a new record with an invalid dimension.
    let new_record = Record::random(DIMENSION + 1);

    // Assert the new record is not inserted.
    assert_eq!(collection.dimension(), DIMENSION);
    assert_eq!(collection.insert(&new_record).is_err(), true);
}

#[test]
fn insert_data_type_object() {
    let mut collection = create_collection();

    // Create a new record with a data of type HashMap.
    let vector = Vector::random(DIMENSION);
    let data = HashMap::from([("key", "value")]);
    let new_record = Record::new(&vector, &data.clone().into());

    collection.insert(&new_record).unwrap();

    // Assert the new data is in the collection.
    let id = VectorID::from(LEN);
    assert_eq!(collection.len(), LEN + 1);
    assert_eq!(collection.get(&id).unwrap().data, data.into());
}

#[test]
fn insert_many() {
    let mut collection = create_collection();

    // Create records to insert.
    let new_records = Record::many_random(DIMENSION, LEN);
    let ids = collection.insert_many(&new_records).unwrap();

    // Assert the new records are in the collection.
    assert_eq!(collection.len(), 2 * LEN);
    assert_eq!(ids.len(), LEN);
    assert_eq!(ids[0], VectorID(LEN as u32));
}

#[test]
fn delete() {
    let mut collection = create_collection();

    // Delete a record from the collection.
    let id = VectorID(0);
    collection.delete(&id).unwrap();
    assert_eq!(collection.len(), LEN - 1);
}

#[test]
fn update() {
    let mut collection = create_collection();

    // New record to update.
    let id = VectorID(5);
    let record = Record::random(DIMENSION);
    collection.update(&id, &record).unwrap();

    assert_eq!(collection.len(), LEN);
    assert_eq!(collection.get(&id).unwrap().data, record.data);
}

#[test]
fn search() {
    let len = 1000;
    let config = Config::default();
    let records = Record::many_random(DIMENSION, len);

    // Build the collection with a minimum relevancy.
    let mut collection = Collection::build(&config, &records).unwrap();
    collection.relevancy = 4.5;

    // Generate a random query vector.
    let query = Vector::random(DIMENSION);

    // Get the approximate and true nearest neighbors.
    let result = collection.search(&query, 5).unwrap();
    let truth = collection.true_search(&query, 10).unwrap();

    assert_eq!(result.len(), 5);

    // The search is not always exact, so we check if
    // the distance is within the true distances.
    let distances: Vec<f32> = truth.par_iter().map(|i| i.distance).collect();
    assert_eq!(distances.contains(&result[0].distance), true);

    // Search results should be within the relevancy.
    let last_result = result.last().unwrap();
    let last_truth = truth.last().unwrap();
    assert!(last_result.distance <= collection.relevancy);
    assert!(last_truth.distance <= collection.relevancy);
}

#[test]
fn get() {
    let records = Record::many_random(DIMENSION, LEN);
    let config = Config::default();
    let collection = Collection::build(&config, &records).unwrap();

    // Get a record from the collection.
    let index: usize = 5;
    let id = VectorID::from(index);
    let record = collection.get(&id).unwrap();

    assert_eq!(record.data, records[index].data);
    assert_eq!(record.vector, records[index].vector);
}

#[test]
fn list() {
    let collection = create_collection();
    let list = collection.list().unwrap();
    assert_eq!(list.len(), LEN);
    assert_eq!(list.len(), collection.len());
}

#[test]
fn config_with_distance() {
    let ef = 10;
    let ml = 1.0;
    for dist in vec!["cosine", "dot", "euclidean"] {
        Config::new(ef, ef, ml, dist).unwrap();
    }
}

#[test]
#[should_panic(expected = "Distance function not supported.")]
fn config_with_distance_panic() {
    let ef = 10;
    let ml = 1.0;
    for dist in vec!["l2", "test"] {
        Config::new(ef, ef, ml, dist).unwrap();
    }
}
