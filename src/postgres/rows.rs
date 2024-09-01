#![allow(missing_docs)]

use super::*;
use sqlx::FromRow;

/// Node's index parameters.
///
/// Fields:
/// - metric: Formula used to calculate distance.
/// - dimension: Vector dimension.
/// - density: Number of records in each cluster.
#[derive(Debug, FromRow)]
pub struct NodeParameters {
    #[sqlx(try_from = "String")]
    pub metric: Metric,
    #[sqlx(try_from = "i32")]
    pub dimension: usize,
    #[sqlx(try_from = "i32")]
    pub density: usize,
}
