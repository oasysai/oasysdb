use super::*;

#[test]
fn test_database_open() {
    assert!(create_test_database().is_ok());
}
