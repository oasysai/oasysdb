use common::Dataset;
use futures::executor;
use oasysdb::prelude::*;
use std::error::Error;

mod common;

fn main() -> Result<(), Box<dyn Error>> {
    let dataset = Dataset::SIFTSMALL;
    let db_url = dataset.database_url();
    let config = SourceConfig::new(dataset.name(), "id", "vector");

    executor::block_on(dataset.populate_database())?;

    let db = Database::open("odb_example", Some(db_url))?;
    create_index_flat(&db, &config)?;
    create_index_ivfpq(&db, &config)?;

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
    println!("Recall@{k}: {recall}");

    Ok(())
}

fn create_index_ivfpq(
    db: &Database,
    config: &SourceConfig,
) -> Result<(), Box<dyn Error>> {
    let index_name = "ivfpq";
    if db.get_index_ref(index_name).is_some() {
        return Ok(());
    }

    let params = ParamsIVFPQ {
        sub_centroids: 8,
        sub_dimension: 16,
        sampling: 0.1,
        ..Default::default()
    };

    let algorithm = IndexAlgorithm::IVFPQ(params);
    db.create_index(index_name, algorithm, config.clone())?;
    Ok(())
}

fn create_index_flat(
    db: &Database,
    config: &SourceConfig,
) -> Result<(), Box<dyn Error>> {
    let index_name = "flat";
    if db.get_index_ref(index_name).is_some() {
        return Ok(());
    }

    let params = ParamsFlat::default();
    let algorithm = IndexAlgorithm::Flat(params);
    db.create_index(index_name, algorithm, config.clone())?;
    Ok(())
}
