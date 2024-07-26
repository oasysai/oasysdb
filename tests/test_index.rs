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
    let params = ParamsIVFPQ {
        sub_centroids: 8,
        sub_dimension: 16,
        sampling: 0.1,
        ..Default::default()
    };

    let algorithm = IndexAlgorithm::IVFPQ(params);
    db.create_index("ivfpq", algorithm, config.clone())?;

    // Create the Flat index.
    let params = ParamsFlat::default();
    let algorithm = IndexAlgorithm::Flat(params);
    db.create_index("flat", algorithm, config)?;

    // Perform search queries
    let queries = {
        let path = dataset.query_dataset_file();
        dataset.read_vectors(path)?
    };

    let k = 10;
    let iteration = 10;
    let mut correct_count = 0;

    for query in queries.into_iter().take(iteration) {
        let vector = Vector::from(query);

        let correct_ids: Vec<RecordID> = db
            .search_index("flat", vector.clone(), k, "")?
            .iter()
            .map(|result| result.id)
            .collect();

        db.search_index("ivfpq", vector, k, "")?.iter().for_each(|r| {
            if correct_ids.contains(&r.id) {
                correct_count += 1;
            }
        });
    }

    let recall = correct_count as f32 / (k * iteration) as f32;
    assert!(recall > 0.0);

    // println!("Recall@{k}: {recall}");
    // assert!(false);
    Ok(())
}
