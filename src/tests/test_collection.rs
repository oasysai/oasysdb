use super::*;

const DIMENSION: usize = 128;
const LEN: usize = 100;

#[test]
fn build_large() {
    let len = 10000;
    let records = gen_records(DIMENSION, len);
    let collection = create_collection(&records);
    assert_eq!(collection.len(), len);
}

#[test]
fn insert() {
    let records = gen_records(DIMENSION, LEN);

    // Create a new record to insert.
    let vector = gen_vector(DIMENSION);
    let data = random::<usize>();
    let new_record = Record { vector, data };

    let mut collection = create_collection(&records);
    collection.insert(&new_record).unwrap();

    assert_eq!(collection.len(), LEN + 1);

    // Assert the new data is in the collection.
    let id = VectorID(LEN as u32);
    assert_eq!(collection.get(&id).unwrap().data, data);
}

#[test]
fn delete() {
    let records = gen_records(DIMENSION, LEN);
    let mut collection = create_collection(&records);

    // Delete a record from the collection.
    let id = VectorID(1);
    collection.delete(&id).unwrap();

    assert_eq!(collection.len(), LEN - 1);
}

#[test]
fn update() {
    let records = gen_records(DIMENSION, LEN);
    let mut collection = create_collection(&records);

    // Create new record to update.
    let id = VectorID(5);
    let data = random::<usize>();
    let record = Record { vector: gen_vector(DIMENSION), data };

    collection.update(&id, &record).unwrap();

    assert_eq!(collection.len(), LEN);
    assert_eq!(collection.get(&id).unwrap().data, data);
}

#[test]
fn search() {
    let len = 1000;
    let records = gen_records(DIMENSION, len);
    let collection = create_collection(&records);

    // Generate a random query vector.
    let query = gen_vector(DIMENSION);

    // Get the approximate and true nearest neighbors.
    let result = collection.search(&query, 5).unwrap();
    let truth = collection.true_search(&query, 10).unwrap();

    // Collect the distances from the true nearest neighbors.
    let distances: Vec<f32> = truth.iter().map(|i| i.distance).collect();

    assert_eq!(result.len(), 5);

    // The search is not always exact, so we check if
    // the distance is within the true distances.
    assert_eq!(distances.contains(&result[0].distance), true);
}

#[test]
fn get() {
    let records = gen_records(DIMENSION, LEN);
    let collection = create_collection(&records);

    // Get a record from the collection.
    let id = VectorID(5);
    let record = collection.get(&id).unwrap();

    assert_eq!(record.data, records[5].data);
    assert_eq!(record.vector, records[5].vector);
}
