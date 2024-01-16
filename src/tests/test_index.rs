use super::*;

#[test]
fn test_index_build_large() {
    let len = 10000;
    let records = gen_records::<128>(len);
    let index = create_test_index::<128>(&records);
    assert_eq!(index.data.len(), len);
}

#[test]
fn test_index_insert() {
    let len = 100;
    let records = gen_records::<128>(len);
    let mut index = create_test_index::<128>(&records);

    let vector = gen_vector::<128>();
    let data = random::<usize>();

    let id = VectorID(len as u32);
    index.insert(&IndexRecord { vector, data });

    assert_eq!(index.data.len(), len + 1);
    assert_eq!(index.data[&id], data);
}

#[test]
fn test_index_delete() {
    let len = 100;
    let records = gen_records::<128>(len);
    let mut index = create_test_index::<128>(&records);

    let id = VectorID(1);
    index.delete(&id);

    assert_eq!(index.data.len(), len - 1);
    // Ensure the data is deleted.
    assert_eq!(index.data.contains_key(&id), false);
}

#[test]
fn test_index_search() {
    let len = 100;
    let records = gen_records::<128>(len);
    let index = create_test_index::<128>(&records);
    let query = gen_vector::<128>();

    let result = index.search(&query, 5);
    let truth = brute_force_search(&records, &query, 10);

    // Collect the distances from the true nearest neighbors.
    let truth_distances: Vec<f32> = truth.iter().map(|i| i.0).collect();

    assert_eq!(result.len(), 5);

    // The search is not always exact, so we check if
    // the distance is within the true distances.
    assert_eq!(truth_distances.contains(&result[0].distance), true);
}
