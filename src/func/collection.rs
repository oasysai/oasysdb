use super::*;

/// The collection HNSW index configuration.
#[cfg_attr(feature = "py", pyclass(module = "oasysdb.collection", get_all))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Nodes to consider during construction.
    pub ef_construction: usize,
    /// Nodes to consider during search.
    pub ef_search: usize,
    /// Layer multiplier. The optimal value is `1/ln(M)`.
    pub ml: f32,
    /// Distance calculation function.
    pub distance: Distance,
}

// Any modifications to this methods should be reflected in:
// - py/tests/test_collection.py
// - py/oasysdb/collection.pyi
#[cfg(feature = "py")]
#[pymethods]
impl Config {
    #[new]
    fn py_new(
        ef_construction: usize,
        ef_search: usize,
        ml: f32,
        distance: &str,
    ) -> Result<Self, Error> {
        Self::new(ef_construction, ef_search, ml, distance)
    }

    #[setter(ef_construction)]
    fn py_set_ef_construction(&mut self, ef_construction: usize) {
        self.ef_construction = ef_construction;
    }

    #[setter(ef_search)]
    fn py_set_ef_search(&mut self, ef_search: usize) {
        self.ef_search = ef_search;
    }

    #[setter(ml)]
    fn py_set_ml(&mut self, ml: f32) {
        self.ml = ml;
    }

    #[setter(distance)]
    fn py_set_distance(&mut self, distance: &str) -> Result<(), Error> {
        self.set_distance(distance)
    }

    #[staticmethod]
    fn create_default() -> Self {
        Self::default()
    }

    #[staticmethod]
    #[pyo3(name = "default")]
    fn py_default() -> Self {
        Self::default()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}

impl Config {
    /// Creates a new collection config with the given parameters.
    pub fn new(
        ef_construction: usize,
        ef_search: usize,
        ml: f32,
        distance: &str,
    ) -> Result<Self, Error> {
        let distance = Distance::from(distance)?;
        Ok(Self { ef_construction, ef_search, ml, distance })
    }

    /// Sets the distance calculation function.
    /// * `distance`: Distance function, e.g. euclidean or dot.
    pub fn set_distance(&mut self, distance: &str) -> Result<(), Error> {
        self.distance = Distance::from(distance)?;
        Ok(())
    }
}

impl Default for Config {
    /// Default configuration for the collection index.
    /// * `ef_construction`: 40
    /// * `ef_search`: 15
    /// * `ml`: 0.2885
    /// * `distance`: euclidean
    fn default() -> Self {
        Self {
            ef_construction: 40,
            ef_search: 15,
            ml: 0.2885,
            distance: Distance::Euclidean,
        }
    }
}

/// The collection of vector records with HNSW indexing.
#[cfg_attr(feature = "py", pyclass(module = "oasysdb.collection"))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    /// The collection configuration object.
    pub config: Config,
    /// The min/max distance to consider a neighbor.
    pub relevancy: f32,
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

// This exposes Collection methods to Python.
// Any modifications to these methods should be reflected in:
// - py/tests/test_collection.py
// - py/oasysdb/collection.pyi
#[cfg_attr(feature = "py", pymethods)]
impl Collection {
    #[cfg(feature = "py")]
    #[new]
    fn py_new(config: &Config) -> Self {
        Self::new(config)
    }

    #[cfg(feature = "py")]
    #[staticmethod]
    fn from_records(
        config: &Config,
        records: Vec<Record>,
    ) -> Result<Self, Error> {
        Self::build(config, &records)
    }

    #[cfg(feature = "py")]
    #[staticmethod]
    #[pyo3(name = "build")]
    fn py_build(
        config: &Config,
        records: Vec<Record>,
    ) -> Result<Collection, Error> {
        Self::build(config, &records)
    }

    /// Inserts a vector record into the collection, and return `VectorID` if success.
    /// * `record`: Vector record to insert.
    pub fn insert(&mut self, record: &Record) -> Result<VectorID, Error> {
        // Ensure the number of records is within the limit.
        if self.slots.len() == u32::MAX as usize {
            return Err(Error::collection_limit());
        }

        // Ensure the vector dimension matches the collection config.
        // If it's the first record, set the dimension.
        if self.vectors.is_empty() && self.dimension == 0 {
            self.dimension = record.vector.len();
        } else if record.vector.len() != self.dimension {
            let len = record.vector.len();
            let err = Error::invalid_dimension(len, self.dimension);
            return Err(err);
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
        self.insert_to_layers(&[id]);

        Ok(id)
    }

    #[cfg(feature = "py")]
    #[pyo3(name = "insert_many")]
    fn py_insert_many(
        &mut self,
        records: Vec<Record>,
    ) -> Result<Vec<VectorID>, Error> {
        let ids = self.insert_many(&records)?;
        Ok(ids)
    }

    /// Deletes a vector record from the collection.
    /// * `id`: Vector ID to delete.
    pub fn delete(&mut self, id: &VectorID) -> Result<(), Error> {
        // Ensure the vector ID exists in the collection.
        if !self.contains(id) {
            return Err(Error::record_not_found());
        }

        self.delete_from_layers(&[*id]);

        // Update the collection data.
        self.vectors.remove(id);
        self.data.remove(id);

        // Make the slot invalid so it won't be used again.
        self.slots[id.0 as usize] = INVALID;

        // Update the collection count.
        self.count -= 1;

        Ok(())
    }

    /// Returns vector records in the collection as a HashMap.
    pub fn list(&self) -> Result<HashMap<VectorID, Record>, Error> {
        // Early return if the collection is empty.
        if self.vectors.is_empty() {
            return Ok(HashMap::new());
        }

        // Map the vectors to a hashmap of records.
        let mapper = |(id, vector): (&VectorID, &Vector)| {
            let data = self.data[id].clone();
            let record = Record::new(vector, &data);
            (*id, record)
        };

        let records = self.vectors.par_iter().map(mapper).collect();
        Ok(records)
    }

    /// Returns the vector record associated with the ID.
    /// * `id`: Vector ID to retrieve.
    pub fn get(&self, id: &VectorID) -> Result<Record, Error> {
        if !self.contains(id) {
            return Err(Error::record_not_found());
        }

        let vector = self.vectors[id].clone();
        let data = self.data[id].clone();
        Ok(Record::new(&vector, &data))
    }

    /// Updates a vector record in the collection.
    /// * `id`: Vector ID to update.
    /// * `record`: New vector record.
    pub fn update(
        &mut self,
        id: &VectorID,
        record: &Record,
    ) -> Result<(), Error> {
        if !self.contains(id) {
            return Err(Error::record_not_found());
        }

        // Validate the new vector dimension.
        self.validate_dimension(&record.vector)?;

        // Remove the old vector from the index layers.
        self.delete_from_layers(&[*id]);

        // Insert the updated vector and data.
        self.vectors.insert(*id, record.vector.clone());
        self.data.insert(*id, record.data.clone());
        self.insert_to_layers(&[*id]);

        Ok(())
    }

    /// Searches the collection for the nearest neighbors.
    /// * `vector`: Vector to search.
    /// * `n`: Number of neighbors to return.
    pub fn search(
        &self,
        vector: &Vector,
        n: usize,
    ) -> Result<Vec<SearchResult>, Error> {
        let mut search = Search::new(0, self.config.distance);

        // Early return if the collection is empty.
        if self.vectors.is_empty() {
            return Ok(vec![]);
        }

        // Ensure the vector dimension matches the collection dimension.
        self.validate_dimension(vector)?;

        // Find the first valid vector ID from the slots.
        let slots_iter = self.slots.as_slice().into_par_iter();
        let vector_id = match slots_iter.find_first(|id| id.is_valid()) {
            Some(id) => id,
            None => {
                let kind = ErrorKind::CollectionError;
                let message = "Unable to initiate search.";
                return Err(Error::new(&kind, message));
            }
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

        // Sort the search results by distance.
        let res = search.iter().map(map_result).collect();
        let sorted = self.sort_by_distance(res);

        // Truncate the list based on the relevancy score.
        let mut relevant = self.truncate_irrelevant_result(sorted);
        relevant.truncate(n);
        Ok(relevant)
    }

    /// Searches the collection for the true nearest neighbors.
    /// * `vector`: Vector to search.
    /// * `n`: Number of neighbors to return.
    pub fn true_search(
        &self,
        vector: &Vector,
        n: usize,
    ) -> Result<Vec<SearchResult>, Error> {
        let mut nearest = Vec::with_capacity(self.vectors.len());

        // Ensure the vector dimension matches the collection dimension.
        self.validate_dimension(vector)?;

        // Calculate the distance between the query and each record.
        // Then, create a search result for each record.
        for (id, vec) in self.vectors.iter() {
            let distance = self.config.distance.calculate(vector, vec);
            let data = self.data[id].clone();
            let res = SearchResult { id: id.0, distance, data };
            nearest.push(res);
        }

        // Sort the results by distance depending on the metric.
        let sorted = self.sort_by_distance(nearest);

        // Remove irrelevant results and truncate the list.
        let mut res = self.truncate_irrelevant_result(sorted);
        res.truncate(n);
        Ok(res)
    }

    #[cfg(feature = "py")]
    #[getter(config)]
    fn py_config(&self) -> Config {
        self.config.clone()
    }

    #[cfg(feature = "py")]
    #[getter(dimension)]
    fn py_dimension(&self) -> usize {
        self.dimension
    }

    #[cfg(feature = "py")]
    #[setter(dimension)]
    fn py_set_dimension(&mut self, dimension: usize) -> Result<(), Error> {
        self.set_dimension(dimension)
    }

    #[cfg(feature = "py")]
    #[getter(relevancy)]
    fn py_relevancy(&self) -> f32 {
        self.relevancy
    }

    #[cfg(feature = "py")]
    #[setter(relevancy)]
    fn py_set_relevancy(&mut self, relevancy: f32) {
        self.relevancy = relevancy;
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
    /// Creates an empty collection with the given configuration.
    pub fn new(config: &Config) -> Self {
        Self {
            count: 0,
            dimension: 0,
            relevancy: -1.0,
            config: config.clone(),
            data: HashMap::new(),
            vectors: HashMap::new(),
            slots: vec![],
            base_layer: vec![],
            upper_layers: vec![],
        }
    }

    /// Builds the collection index from vector records.
    /// * `config`: Collection configuration.
    /// * `records`: List of vectors to build the index from.
    pub fn build(config: &Config, records: &[Record]) -> Result<Self, Error> {
        if records.is_empty() {
            return Ok(Self::new(config));
        }

        // Ensure the number of records is within the limit.
        if records.len() >= u32::MAX as usize {
            let kind = ErrorKind::CollectionError;
            let message = format!(
                "The collection record limit is {}. Given: {}",
                u32::MAX,
                records.len()
            );

            return Err(Error::new(&kind, &message));
        }

        // Ensure that the vector dimension is consistent.
        let dimension = records[0].vector.len();
        if records.par_iter().any(|i| i.vector.len() != dimension) {
            let kind = ErrorKind::CollectionError;
            let message = format!(
                "The vector dimension is inconsistent. Expected: {}.",
                dimension
            );

            return Err(Error::new(&kind, &message));
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

        let search_pool = SearchPool::new(vectors.len(), config.distance);
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
            config: config.clone(),
            count: records.len(),
            relevancy: -1.0,
        })
    }

    /// Inserts multiple vector records into the collection.
    /// * `records`: List of vector records to insert.
    pub fn insert_many(
        &mut self,
        records: &[Record],
    ) -> Result<Vec<VectorID>, Error> {
        // Make sure the collection is not full after inserting.
        if self.slots.len() + records.len() >= u32::MAX as usize {
            return Err(Error::collection_limit());
        }

        // Sets the collection dimension if it's the first record.
        if self.vectors.is_empty() && self.dimension == 0 {
            self.dimension = records[0].vector.len();
        }

        // Validate the vector dimension against the collection.
        if records.par_iter().any(|i| i.vector.len() != self.dimension) {
            let kind = ErrorKind::CollectionError;
            let message = format!(
                "The vector dimension is inconsistent. Expected: {}.",
                self.dimension
            );

            return Err(Error::new(&kind, &message));
        }

        // Create new vector IDs for the records.
        let ids: Vec<VectorID> = {
            let first_id = self.slots.len();
            let final_id = self.slots.len() + records.len();
            (first_id..final_id).map(|i| i.into()).collect()
        };

        // Store the new records vector and data.
        for (id, record) in ids.iter().zip(records.iter()) {
            self.vectors.insert(*id, record.vector.clone());
            self.data.insert(*id, record.data.clone());
        }

        // Add new vector IDs to the slots.
        self.slots.extend(ids.clone());

        // Update the collection count.
        self.count += records.len();

        self.insert_to_layers(&ids);
        Ok(ids)
    }

    /// Returns the configured vector dimension of the collection.
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Sets the vector dimension of the collection.
    /// * `dimension`: New vector dimension.
    pub fn set_dimension(&mut self, dimension: usize) -> Result<(), Error> {
        // This can only be set if the collection is empty.
        if !self.vectors.is_empty() {
            let kind = ErrorKind::CollectionError;
            let message = "Collection must be empty to set dimension.";
            return Err(Error::new(&kind, message));
        }

        self.dimension = dimension;
        Ok(())
    }

    /// Sets the min/max relevancy for the search results.
    /// * `relevancy`: Relevancy score.
    pub fn set_relevancy(&mut self, relevancy: f32) {
        self.relevancy = relevancy;
    }

    /// Validates a vector dimension against the collection's.
    fn validate_dimension(&self, vector: &Vector) -> Result<(), Error> {
        let found = vector.len();
        let expected = self.dimension;

        if found != expected {
            Err(Error::invalid_dimension(found, expected))
        } else {
            Ok(())
        }
    }

    /// Inserts vector IDs into the index layers.
    fn insert_to_layers(&mut self, ids: &[VectorID]) {
        // Add new nodes to the base layer.
        for _ in 0..ids.len() {
            self.base_layer.push(BaseNode::default());
        }

        let base_layer = self
            .base_layer
            .par_iter()
            .map(|node| RwLock::new(*node))
            .collect::<Vec<_>>();

        let top_layer = match self.upper_layers.is_empty() {
            true => LayerID(0),
            false => LayerID(self.upper_layers.len()),
        };

        // Create a new index construction state.
        let state = IndexConstruction {
            top_layer,
            base_layer: base_layer.as_slice(),
            vectors: &self.vectors,
            config: &self.config,
            search_pool: SearchPool::new(
                self.vectors.len(),
                self.config.distance,
            ),
        };

        // Insert all vectors into the state.
        for id in ids {
            state.insert(id, &top_layer, &self.upper_layers);
        }

        // Update base layer using the new state.
        let iter = state.base_layer.into_par_iter();
        self.base_layer = iter.map(|node| *node.read()).collect();
    }

    /// Removes vector IDs from all index layers.
    fn delete_from_layers(&mut self, ids: &[VectorID]) {
        // Remove the vectors from the base layer.
        for id in ids {
            let base_node = &mut self.base_layer[id.0 as usize];
            let index = base_node.par_iter().position_first(|x| *x == *id);
            if let Some(index) = index {
                base_node.set(index, &INVALID);
            }
        }

        // Remove the vector from the upper layers.
        for layer in LayerID(self.upper_layers.len()).descend() {
            let upper_layer = match layer.0 > 0 {
                true => &mut self.upper_layers[layer.0 - 1],
                false => break,
            };

            for id in ids {
                let node = &mut upper_layer[id.0 as usize];
                let index = node.0.par_iter().position_first(|x| *x == *id);
                if let Some(index) = index {
                    node.set(index, &INVALID);
                }
            }
        }
    }

    /// Truncates the search result based on the relevancy score.
    fn truncate_irrelevant_result(
        &self,
        result: Vec<SearchResult>,
    ) -> Vec<SearchResult> {
        // Early return if the relevancy score is not set.
        if self.relevancy == -1.0 {
            return result;
        }

        // For Euclidean distance, relevant results are those
        // smaller than the relevancy score with best distance of 0.0.
        if self.config.distance == Distance::Euclidean {
            return result
                .into_par_iter()
                .filter(|r| r.distance <= self.relevancy)
                .collect();
        }

        // For other distance metrics, like cosine similarity
        // and dot product, the relevant results are above
        // the relevancy score.
        result
            .into_par_iter()
            .filter(|r| r.distance >= self.relevancy)
            .collect()
    }

    /// Sorts the search results by distance and distance metric.
    fn sort_by_distance(&self, result: Vec<SearchResult>) -> Vec<SearchResult> {
        let mut result = result;

        // Sort the results by distance based on the metric.
        // For Euclidean distance, sort in ascending order
        // because the best distance is 0.0.
        match self.config.distance {
            Distance::Euclidean => {
                result.sort_by(|a, b| {
                    a.distance.partial_cmp(&b.distance).unwrap()
                });
            }
            _ => {
                result.sort_by(|a, b| {
                    b.distance.partial_cmp(&a.distance).unwrap()
                });
            }
        };

        result
    }
}

/// A record containing a vector and its associated data.
#[cfg_attr(feature = "py", pyclass(module = "oasysdb.collection", get_all))]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Record {
    /// The vector embedding.
    pub vector: Vector,
    /// Data associated with the vector.
    pub data: Metadata,
}

// Any modifications to the Python methods should be reflected in:
// - py/tests/test_collection.py
// - py/oasysdb/collection.pyi
#[cfg(feature = "py")]
#[pymethods]
impl Record {
    #[new]
    fn py_new(vector: Vec<f32>, data: &PyAny) -> Self {
        let vector = Vector::from(vector);
        let data = Metadata::from(data);
        Self::new(&vector, &data)
    }

    #[setter(vector)]
    fn py_set_vector(&mut self, vector: Vec<f32>) {
        self.vector = Vector::from(vector);
    }

    #[setter(data)]
    fn py_set_data(&mut self, data: &PyAny) -> Result<(), Error> {
        self.data = Metadata::from(data);
        Ok(())
    }

    #[staticmethod]
    #[pyo3(name = "random")]
    fn py_random(dimension: usize) -> Self {
        Record::random(dimension)
    }

    #[staticmethod]
    #[pyo3(name = "many_random")]
    fn py_many_random(dimension: usize, len: usize) -> Vec<Self> {
        Record::many_random(dimension, len)
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
#[cfg_attr(feature = "py", pyclass(module = "oasysdb.collection", get_all))]
#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    /// Vector ID.
    pub id: u32,
    /// Distance between the query to the collection vector.
    pub distance: f32,
    /// Data associated with the vector.
    pub data: Metadata,
}

#[cfg(feature = "py")]
impl SearchResult {
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }
}
