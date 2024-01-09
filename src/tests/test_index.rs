use super::*;

#[test]
fn test_insert() {
    let mut index = create_test_index();
    let len = index.count();

    // Insert a new node.
    let node = generate_node();
    index.insert(&node);
    let new_len = index.count();

    assert_eq!(len + 1, new_len);
}

#[test]
fn test_delete() {
    let mut index = create_test_index();
    let len = index.count();

    // Get and delete a node.
    let preview = index.list(3);
    let key = preview[0].key;
    index.delete(key);
    let new_len = index.count();

    assert_eq!(len - 1, new_len);
}

#[test]
fn test_query() {
    let index = create_test_index();
    let vector = generate_node().vector;
    let neighbors = index.query(&vector, 10);
    assert_eq!(neighbors.len(), 10);
}
