use super::*;

/// The collection HNSW index configuration.
#[pyclass(module = "oasysdb.collection")]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Config {
    /// Nodes to consider during construction.
    #[pyo3(get, set)]
    pub ef_construction: usize,
    /// Nodes to consider during search.
    #[pyo3(get, set)]
    pub ef_search: usize,
    /// Layer multiplier. The optimal value is `1/ln(M)`.
    #[pyo3(get, set)]
    pub ml: f32,
}

// Any modifications to this methods should be reflected in:
// - py/tests/test_collection.py
// - py/oasysdb/collection.pyi
#[pymethods]
impl Config {
    /// Creates a new collection config with the given parameters.
    #[new]
    pub fn new(ef_construction: usize, ef_search: usize, ml: f32) -> Self {
        Self { ef_construction, ef_search, ml }
    }

    #[staticmethod]
    fn create_default() -> Self {
        Self::default()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl Default for Config {
    /// Default configuration for the collection index.
    /// * `ef_construction`: 40
    /// * `ef_search`: 15
    /// * `ml`: 0.3
    fn default() -> Self {
        Self { ef_construction: 40, ef_search: 15, ml: 0.3 }
    }
}

struct IndexConstruction<'a> {
    search_pool: SearchPool,
    top_layer: LayerID,
    base_layer: &'a [RwLock<BaseNode>],
    vectors: &'a HashMap<VectorID, Vector>,
    config: &'a Config,
}

impl<'a> IndexConstruction<'a> {
    /// Inserts a vector ID into a layer.
    /// * `vector_id`: Vector ID to insert.
    /// * `layer`: Layer to insert into.
    /// * `layers`: Upper layers.
    fn insert(
        &self,
        vector_id: &VectorID,
        layer: &LayerID,
        layers: &[Vec<UpperNode>],
    ) {
        let vector = &self.vectors[vector_id];

        let (mut search, mut insertion) = self.search_pool.pop();
        insertion.ef = self.config.ef_construction;

        // Find the first valid vector ID to push.
        let validator = |i: usize| self.vectors.get(&i.into()).is_some();
        let valid_id = (0..self.vectors.len())
            .into_par_iter()
            .find_first(|i| validator(*i))
            .unwrap();

        search.reset();
        search.push(&valid_id.into(), vector, self.vectors);

        for current_layer in self.top_layer.descend() {
            if current_layer <= *layer {
                search.ef = self.config.ef_construction;
            }

            // Find the nearest neighbor candidates.
            if current_layer > *layer {
                let layer = layers[current_layer.0 - 1].as_slice();
                search.search(layer, vector, self.vectors, M);
                search.cull();
            } else {
                search.search(self.base_layer, vector, self.vectors, M * 2);
                break;
            }
        }

        // Select the neighbors.
        let candidates = {
            let candidates = search.select_simple();
            &candidates[..Ord::min(candidates.len(), M)]
        };

        for (i, candidate) in candidates.iter().enumerate() {
            let vid = candidate.vector_id;
            let old = &self.vectors[&vid];
            let distance = candidate.distance;

            // Function to sort the vectors by distance.
            let ordering = |id: &VectorID| {
                if !id.is_valid() {
                    Ordering::Greater
                } else {
                    let other = &self.vectors[id];
                    distance.cmp(&old.distance(other).into())
                }
            };

            // Find the correct index to insert at to keep the order.
            let index = self.base_layer[&vid]
                .read()
                .binary_search_by(ordering)
                .unwrap_or_else(|error| error);

            self.base_layer[&vid].write().insert(index, vector_id);
            self.base_layer[vector_id].write().set(i, vector_id);
        }

        self.search_pool.push(&(search, insertion));
    }
}

/// The collection of vector records with HNSW indexing.
#[pyclass(module = "oasysdb.collection")]
#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    /// The collection configuration object.
    #[pyo3(get)]
    pub config: Config,
    // Private fields below.
    data: HashMap<VectorID, Metadata>,
    vectors: HashMap<VectorID, Vector>,
    slots: Vec<VectorID>,
    base_layer: Vec<BaseNode>,
    upper_layers: Vec<Vec<UpperNode>>,
    // Utility fields.
    count: usize,
    dimension: usize,
}

impl Index<&VectorID> for Collection {
    type Output = Vector;
    fn index(&self, index: &VectorID) -> &Self::Output {
        &self.vectors[index]
    }
}

/// Converts standard Error to PyErr for Python methods.
fn to_pyerr(e: Box<dyn Error>) -> PyErr {
    let message = format!("{}", e);
    PyErr::new::<PyAny, String>(message)
}

// This exposes Collection methods to Python.
// Any modifications to these methods should be reflected in:
// - py/tests/test_collection.py
// - py/oasysdb/collection.pyi
#[pymethods]
impl Collection {
    /// Creates an empty collection with the given configuration.
    #[new]
    pub fn new(config: &Config) -> Self {
        Self {
            config: *config,
            count: 0,
            dimension: 0,
            data: HashMap::new(),
            vectors: HashMap::new(),
            slots: vec![],
            base_layer: vec![],
            upper_layers: vec![],
        }
    }

    #[staticmethod]
    fn from_records(config: &Config, records: Vec<Record>) -> PyResult<Self> {
        let collection =
            Collection::build(config, &records).map_err(to_pyerr)?;
        Ok(collection)
    }

    /// Returns the number of vector records in the collection.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Checks if the collection contains a vector ID.
    /// * `id`: Vector ID to check.
    pub fn contains(&self, id: &VectorID) -> bool {
        self.vectors.contains_key(id)
    }

    fn __len__(&self) -> usize {
        self.len()
    }
}

impl Collection {
    /// Builds the collection index from vector records.
    /// * `config`: Collection configuration.
    /// * `records`: List of vectors to build the index from.
    pub fn build(
        config: &Config,
        records: &[Record],
    ) -> Result<Self, Box<dyn Error>> {
        if records.is_empty() {
            return Ok(Self::new(config));
        }

        // Ensure the number of records is within the limit.
        if records.len() >= u32::MAX as usize {
            let message = format!(
                "The collection record limit is {}. Given: {}",
                u32::MAX,
                records.len()
            );

            return Err(message.into());
        }

        // Ensure that the vector dimension is consistent.
        let dimension = records[0].vector.len();
        if records.par_iter().any(|i| i.vector.len() != dimension) {
            let message = format!(
                "The vector dimension is inconsistent. Expected: {}.",
                dimension
            );

            return Err(message.into());
        }

        // Find the number of layers.

        let mut len = records.len();
        let mut layers = Vec::new();

        loop {
            let next = (len as f32 * config.ml) as usize;

            if next < M {
                break;
            }

            layers.push((len - next, len));
            len = next;
        }

        layers.push((len, len));
        layers.reverse();

        let num_layers = layers.len();
        let top_layer = LayerID(num_layers - 1);

        // Give all vectors a random layer and sort the list of nodes
        // by descending order for construction.

        // This allows us to copy higher layers to lower layers as
        // construction progresses, while preserving randomness in
        // each point's layer and insertion order.

        let vectors = records
            .par_iter()
            .enumerate()
            .map(|(i, item)| (i.into(), item.vector.clone()))
            .collect::<HashMap<VectorID, Vector>>();

        // Figure out how many nodes will go on each layer.
        // This helps us allocate memory capacity for each
        // layer in advance, and also helps enable batch
        // insertion of points.

        let mut ranges = Vec::with_capacity(top_layer.0);
        for (i, (size, cumulative)) in layers.into_iter().enumerate() {
            let start = cumulative - size;
            let layer_id = LayerID(num_layers - i - 1);
            let value = max(start, 1)..cumulative;
            ranges.push((layer_id, value));
        }

        // Create index constructor.

        let search_pool = SearchPool::new(vectors.len());
        let mut upper_layers = vec![vec![]; top_layer.0];
        let base_layer = vectors
            .par_iter()
            .map(|_| RwLock::new(BaseNode::default()))
            .collect::<Vec<_>>();

        let state = IndexConstruction {
            base_layer: &base_layer,
            search_pool,
            top_layer,
            vectors: &vectors,
            config,
        };

        // Initialize data for layers.

        for (layer, range) in ranges {
            let end = range.end;

            range.into_par_iter().for_each(|i: usize| {
                state.insert(&i.into(), &layer, &upper_layers)
            });

            // Copy the base layer state to the upper layer.
            if !layer.is_zero() {
                (&state.base_layer[..end])
                    .into_par_iter()
                    .map(|zero| UpperNode::from_zero(&zero.read()))
                    .collect_into_vec(&mut upper_layers[layer.0 - 1]);
            }
        }

        let data = records
            .iter()
            .enumerate()
            .map(|(i, item)| (i.into(), item.data.clone()))
            .collect();

        // Unwrap the base nodes for the base layer.
        let base_iter = base_layer.into_par_iter();
        let base_layer = base_iter.map(|node| node.into_inner()).collect();

        // Add IDs to the slots.
        let slots = (0..vectors.len()).map(|i| i.into()).collect();

        Ok(Self {
            data,
            vectors,
            base_layer,
            upper_layers,
            slots,
            dimension,
            config: *config,
            count: records.len(),
        })
    }

    /// Inserts a vector record into the collection.
    /// * `record`: Vector record to insert.
    pub fn insert(&mut self, record: &Record) -> Result<(), Box<dyn Error>> {
        // Ensure the number of records is within the limit.
        if self.slots.len() == u32::MAX as usize {
            return Err(err::COLLECTION_LIMIT.into());
        }

        // Ensure the vector dimension matches the collection config.
        // If it's the first record, set the dimension.
        if self.vectors.is_empty() && self.dimension == 0 {
            self.dimension = record.vector.len();
        } else if record.vector.len() != self.dimension {
            let message = format!(
                "Invalid vector dimension. Expected dimension of {}.",
                self.dimension
            );

            return Err(message.into());
        }

        // Create a new vector ID using the next available slot.
        let id: VectorID = self.slots.len().into();

        // Insert the new vector and data.
        self.vectors.insert(id, record.vector.clone());
        self.data.insert(id, record.data.clone());

        // Add new vector id to the slots.
        self.slots.push(id);

        // Update the collection count.
        self.count += 1;

        // This operation is last because it depends on
        // the updated vectors data.
        self.insert_to_layers(&id);

        Ok(())
    }

    /// Deletes a vector record from the collection.
    /// * `id`: Vector ID to delete.
    pub fn delete(&mut self, id: &VectorID) -> Result<(), Box<dyn Error>> {
        // Ensure the vector ID exists in the collection.
        if !self.contains(id) {
            return Err(err::RECORD_NOT_FOUND.into());
        }

        self.delete_from_layers(id);

        // Update the collection data.
        self.vectors.remove(id);
        self.data.remove(id);
        self.slots[id.0 as usize] = INVALID;

        // Update the collection count.
        self.count -= 1;

        Ok(())
    }

    /// Updates a vector record in the collection.
    /// * `id`: Vector ID to update.
    /// * `record`: New vector record.
    pub fn update(
        &mut self,
        id: &VectorID,
        record: &Record,
    ) -> Result<(), Box<dyn Error>> {
        if !self.contains(id) {
            return Err(err::RECORD_NOT_FOUND.into());
        }

        // Remove the old vector from the index layers.
        self.delete_from_layers(id);

        // Insert the updated vector and data.
        self.vectors.insert(*id, record.vector.clone());
        self.data.insert(*id, record.data.clone());
        self.insert_to_layers(id);

        Ok(())
    }

    /// Returns the vector record associated with the ID.
    /// * `id`: Vector ID to retrieve.
    pub fn get(&self, id: &VectorID) -> Result<Record, Box<dyn Error>> {
        if !self.contains(id) {
            return Err(err::RECORD_NOT_FOUND.into());
        }

        let vector = self.vectors[id].clone();
        let data = self.data[id].clone();
        Ok(Record::new(&vector, &data))
    }

    /// Searches the collection for the nearest neighbors.
    /// * `vector`: Vector to search.
    /// * `n`: Number of neighbors to return.
    pub fn search(
        &self,
        vector: &Vector,
        n: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        let mut search = Search::default();

        if self.vectors.is_empty() {
            return Ok(vec![]);
        }

        // Find the first valid vector ID from the slots.
        let slots_iter = self.slots.as_slice().into_par_iter();
        let vector_id = match slots_iter.find_first(|id| id.is_valid()) {
            Some(id) => id,
            None => return Err("Unable to initiate search.".into()),
        };

        search.visited.resize_capacity(self.vectors.len());
        search.push(vector_id, vector, &self.vectors);

        for layer in LayerID(self.upper_layers.len()).descend() {
            search.ef = if layer.is_zero() { self.config.ef_search } else { 5 };

            if layer.0 == 0 {
                let layer = self.base_layer.as_slice();
                search.search(layer, vector, &self.vectors, M * 2);
            } else {
                let layer = self.upper_layers[layer.0 - 1].as_slice();
                search.search(layer, vector, &self.vectors, M);
            }

            if !layer.is_zero() {
                search.cull();
            }
        }

        let map_result = |candidate: Candidate| {
            let id = candidate.vector_id.0;
            let distance = candidate.distance.0;
            let data = self.data[&candidate.vector_id].clone();
            SearchResult { id, distance, data }
        };

        Ok(search.iter().map(map_result).take(n).collect())
    }

    /// Searches the collection for the true nearest neighbors.
    /// * `vector`: Vector to search.
    /// * `n`: Number of neighbors to return.
    pub fn true_search(
        &self,
        vector: &Vector,
        n: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        let mut nearest = Vec::with_capacity(self.vectors.len());

        // Calculate the distance between the query and each record.
        // Then, create a search result for each record.
        for (id, vec) in self.vectors.iter() {
            let distance = vector.distance(vec);
            let data = self.data[id].clone();
            let res = SearchResult { id: id.0, distance, data };
            nearest.push(res);
        }

        // Sort the nearest neighbors by distance.
        nearest.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        nearest.truncate(n);
        Ok(nearest)
    }

    /// Returns the configured vector dimension of the collection.
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Sets the vector dimension of the collection.
    /// * `dimension`: New vector dimension.
    pub fn set_dimension(
        &mut self,
        dimension: usize,
    ) -> Result<(), Box<dyn Error>> {
        // This can only be set if the collection is empty.
        if !self.vectors.is_empty() {
            return Err("The collection must be empty.".into());
        }

        self.dimension = dimension;
        Ok(())
    }

    /// Inserts a vector ID into the index layers.
    fn insert_to_layers(&mut self, id: &VectorID) {
        self.base_layer.push(BaseNode::default());

        let base_layer = self
            .base_layer
            .par_iter()
            .map(|node| RwLock::new(*node))
            .collect::<Vec<_>>();

        let top_layer = match self.upper_layers.is_empty() {
            true => LayerID(0),
            false => LayerID(self.upper_layers.len()),
        };

        let state = IndexConstruction {
            base_layer: base_layer.as_slice(),
            search_pool: SearchPool::new(self.vectors.len()),
            top_layer,
            vectors: &self.vectors,
            config: &self.config,
        };

        // Insert new vector into the contructor.
        state.insert(id, &top_layer, &self.upper_layers);

        // Update the base layer with the new state.
        let iter = state.base_layer.into_par_iter();
        self.base_layer = iter.map(|node| *node.read()).collect();
    }

    /// Removes a vector ID from all index layers.
    fn delete_from_layers(&mut self, id: &VectorID) {
        // Remove the vector from the base layer.
        let base_node = &mut self.base_layer[id.0 as usize];
        let index = base_node.par_iter().position_first(|x| *x == *id);
        if let Some(index) = index {
            base_node.set(index, &INVALID);
        }

        // Remove the vector from the upper layers.
        for layer in LayerID(self.upper_layers.len()).descend() {
            let upper_layer = match layer.0 > 0 {
                true => &mut self.upper_layers[layer.0 - 1],
                false => break,
            };

            let node = &mut upper_layer[id.0 as usize];
            let index = node.0.par_iter().position_first(|x| *x == *id);

            if let Some(index) = index {
                node.set(index, &INVALID);
            }
        }
    }
}

/// A record containing a vector and its associated data.
#[pyclass(module = "oasysdb.collection")]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Record {
    /// The vector embedding.
    #[pyo3(get, set)]
    pub vector: Vector,
    /// Data associated with the vector.
    #[pyo3(get)]
    pub data: Metadata,
}

// Any modifications to the Python methods should be reflected in:
// - py/tests/test_collection.py
// - py/oasysdb/collection.pyi
#[pymethods]
impl Record {
    #[new]
    fn py_new(vector: Vec<f32>, data: &PyAny) -> Self {
        let vector = Vector::from(vector);
        let data = Metadata::from(data);
        Self::new(&vector, &data)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl Record {
    /// Creates a new record with a vector and data.
    pub fn new(vector: &Vector, data: &Metadata) -> Self {
        Self { vector: vector.clone(), data: data.clone() }
    }
}

impl Record {
    /// Generates a random record for testing.
    /// * `dimension`: Vector dimension.
    pub fn random(dimension: usize) -> Self {
        let vector = Vector::random(dimension);
        let data = random::<usize>().into();
        Self::new(&vector, &data)
    }

    /// Generates many random records for testing.
    /// * `dimension`: Vector dimension.
    /// * `len`: Number of records to generate.
    pub fn many_random(dimension: usize, len: usize) -> Vec<Self> {
        (0..len).map(|_| Self::random(dimension)).collect()
    }
}

/// The collection nearest neighbor search result.
#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    /// Vector ID.
    pub id: u32,
    /// Distance between the query to the collection vector.
    pub distance: f32,
    /// Data associated with the vector.
    pub data: Metadata,
}
