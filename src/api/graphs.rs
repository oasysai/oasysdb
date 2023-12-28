use super::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;

/// A struct for the body of the create graph endpoint.
/// To improve the UX, this data is optional. That's why we use
/// `Option` for each field and implement `Default` for the struct.
#[derive(Serialize, Deserialize, Default)]
pub struct CreateGraphBody {
    pub name: Option<String>,
    pub ef_construction: Option<usize>,
    pub ef_search: Option<usize>,
    pub filter: Option<Data>,
}

impl CreateGraphBody {
    fn default() -> Self {
        CreateGraphBody {
            name: None,
            ef_construction: None,
            ef_search: None,
            filter: None,
        }
    }
}

#[post("/", data = "<data>")]
pub fn create_graph(
    db: &State<Database>,
    data: Option<Json<CreateGraphBody>>,
    _auth: Auth,
) -> (Status, Response) {
    let data = match data {
        Some(data) => data.into_inner(),
        None => CreateGraphBody::default(),
    };

    let config = {
        let name = data.name.unwrap_or("default".into());
        let ef_construction = data.ef_construction.unwrap_or(25);
        let ef_search = data.ef_search.unwrap_or(15);
        let filter = data.filter;
        GraphConfig { name, ef_construction, ef_search, filter }
    };

    match db.create_graph(config) {
        Ok(_) => (Status::Created, Response::empty()),
        Err(message) => (Status::BadRequest, Response::error(message)),
    }
}

#[delete("/<name>")]
pub fn delete_graph(
    db: &State<Database>,
    name: &str,
    _auth: Auth,
) -> (Status, Response) {
    match db.delete_graph(name) {
        Ok(_) => (Status::Ok, Response::empty()),
        Err(message) => (Status::BadRequest, Response::error(message)),
    }
}

/// A struct for the body to query the graph. The embedding is
/// required and its dimension must match the dimension of the
/// graph that is set by the `OASYSDB_DIMENSION` environment variable.
#[derive(Serialize, Deserialize)]
pub struct QueryGraphBody {
    pub embedding: Embedding,
    pub k: Option<usize>,
}

#[post("/<name>/query", data = "<data>")]
pub fn query_graph(
    db: &State<Database>,
    name: &str,
    data: Json<QueryGraphBody>,
    _auth: Auth,
) -> (Status, Response) {
    let data = data.into_inner();

    // Default value for k is 10.
    let k = data.k.unwrap_or(10);

    match db.query_graph(name, data.embedding, k) {
        Ok(data) => (Status::Ok, Response::from(data)),
        Err(message) => (Status::BadRequest, Response::error(message)),
    }
}
