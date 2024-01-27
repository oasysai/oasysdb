use super::*;

#[test]
fn build_large() {
    let len = 10000;
    let records = gen_records::<128>(len);
    let collection = create_collection::<128>(&records);
    assert_eq!(collection.len(), len);
}

#[test]
fn insert() {
    let len = 100;
    let records = gen_records::<128>(len);
    let mut collection = create_collection::<128>(&records);

    let vector = gen_vector::<128>();
    let data = random::<usize>();

    let id = VectorID(len as u32);
    collection.insert(&Record { vector, data });

    assert_eq!(collection.len(), len + 1);
    assert_eq!(collection.get(&id).data, data);
}

#[test]
fn delete() {
    let len = 100;
    let records = gen_records::<128>(len);
    let mut collection = create_collection::<128>(&records);

    let id = VectorID(1);
    collection.delete(&id);

    assert_eq!(collection.len(), len - 1);
}

#[test]
fn update() {
    let len = 100;
    let records = gen_records::<128>(len);
    let mut collection = create_collection::<128>(&records);

    let id = VectorID(5);
    let data = random::<usize>();
    let record = Record { vector: gen_vector::<128>(), data };
    collection.update(&id, &record);

    assert_eq!(collection.len(), len);
    assert_eq!(collection.get(&id).data, data);
}

#[test]
fn search() {
    let len = 1000;
    let records = gen_records::<128>(len);
    let collection = create_collection::<128>(&records);
    let query = gen_vector::<128>();

    let result = collection.search(&query, 5);
    let truth = brute_force_search(&records, &query, 10);

    // Collect the distances from the true nearest neighbors.
    let truth_distances: Vec<f32> = truth.iter().map(|i| i.0).collect();

    assert_eq!(result.len(), 5);

    // The search is not always exact, so we check if
    // the distance is within the true distances.
    assert_eq!(truth_distances.contains(&result[0].distance), true);
}

#[test]
fn get() {
    let len = 100;
    let records = gen_records::<128>(len);
    let collection = create_collection::<128>(&records);

    let id = VectorID(5);
    let record = collection.get(&id);

    assert_eq!(record.data, records[5].data);
    assert_eq!(record.vector, records[5].vector);
}
