use common::Dataset;
use futures::executor;
use oasysdb::prelude::*;
use std::error::Error;

mod common;

#[test]
fn test_recall_ivfpq() -> Result<(), Box<dyn Error>> {
    let dataset = Dataset::SIFTSMALL;
    let db_url = dataset.database_url();
    let config = SourceConfig::new(dataset.name(), "id", "vector");

    executor::block_on(dataset.populate_database())?;

    let db = Database::open("odb_itest", Some(db_url))?;

    // Create the IVFPQ index.
    if db.get_index("ivfpq").is_none() {
        let params = ParamsIVFPQ::default();
        let algorithm = IndexAlgorithm::IVFPQ(params);
        db.create_index("ivfpq", algorithm, config.clone())?;
    }

    // Create the Flat index.
    if db.get_index("flat").is_none() {
        let params = ParamsFlat::default();
        let algorithm = IndexAlgorithm::Flat(params);
        db.create_index("flat", algorithm, config)?;
    }

    // Perform a search query.
    let k = 10;
    let iteration = 10;
    let query = vec![0.0; 128];

    let correct_ids: Vec<RecordID> = db
        .search_index("flat", query.clone(), k, "")?
        .iter()
        .map(|result| result.id)
        .collect();

    let mut correct_count = 0;
    for _ in 0..iteration {
        db.search_index("ivfpq", query.clone(), k, "")?.iter().for_each(|r| {
            if correct_ids.contains(&r.id) {
                correct_count += 1;
            }
        });
    }

    let recall = correct_count as f32 / (k * iteration) as f32;
    assert!(recall > 0.9);
    Ok(())
}
