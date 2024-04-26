use self::distance::Distance;

use super::*;

pub const INVALID: VectorID = VectorID(u32::MAX);

/// The M value for the HNSW algorithm.
pub const M: usize = 32;

pub trait Layer {
    type Slice: Deref<Target = [VectorID]>;
    fn nearest_iter(&self, vector_id: &VectorID) -> NearestIter<Self::Slice>;
}

pub struct NearestIter<T> {
    node: T,
    current: usize,
}

impl<T: Deref<Target = [VectorID]>> NearestIter<T> {
    pub fn new(node: T) -> Self {
        Self { node, current: 0 }
    }
}

impl<T: Deref<Target = [VectorID]>> Iterator for NearestIter<T> {
    type Item = VectorID;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.node.len() {
            return None;
        }

        let item = self.node[self.current];
        if !item.is_valid() {
            self.current = self.node.len();
            return None;
        }

        self.current += 1;
        Some(item)
    }
}

struct DescendingLayerIter {
    next: Option<usize>,
}

impl Iterator for DescendingLayerIter {
    type Item = LayerID;
    fn next(&mut self) -> Option<Self::Item> {
        let current_next = self.next?;

        let next = if current_next == 0 {
            self.next = None;
            0
        } else {
            self.next = Some(current_next - 1);
            current_next
        };

        Some(LayerID(next))
    }
}

#[derive(Clone, Copy, Debug)]
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LayerID(pub usize);

impl LayerID {
    pub fn descend(&self) -> impl Iterator<Item = LayerID> {
        DescendingLayerIter { next: Some(self.0) }
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct BaseNode(#[serde(with = "BigArray")] pub [VectorID; M * 2]);

impl Default for BaseNode {
    fn default() -> Self {
        Self([INVALID; M * 2])
    }
}

impl BaseNode {
    pub fn allocate(&mut self, mut iter: impl Iterator<Item = VectorID>) {
        for slot in self.0.iter_mut() {
            if let Some(vector_id) = iter.next() {
                *slot = vector_id;
            } else if *slot != INVALID {
                *slot = INVALID;
            } else {
                break;
            }
        }
    }

    /// Inserts a vector ID to the base node at the index.
    pub fn insert(&mut self, index: usize, vector_id: &VectorID) {
        if index >= self.0.len() {
            return;
        }

        // Shift the vector IDs.
        if self.0[index].is_valid() {
            let end = M * 2 - 1;
            self.0.copy_within(index..end, index + 1);
        }

        self.set(index, vector_id)
    }

    /// Sets the vector ID at the index.
    pub fn set(&mut self, index: usize, vector_id: &VectorID) {
        self.0[index] = *vector_id;
    }
}

impl Index<&VectorID> for [RwLock<BaseNode>] {
    type Output = RwLock<BaseNode>;
    fn index(&self, index: &VectorID) -> &Self::Output {
        &self[index.0 as usize]
    }
}

impl Deref for BaseNode {
    type Target = [VectorID];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Layer for &'a [BaseNode] {
    type Slice = &'a [VectorID];
    fn nearest_iter(&self, vector_id: &VectorID) -> NearestIter<Self::Slice> {
        NearestIter::new(&self[vector_id.0 as usize])
    }
}

impl<'a> Layer for &'a [RwLock<BaseNode>] {
    type Slice = MappedRwLockReadGuard<'a, [VectorID]>;
    fn nearest_iter(&self, vector_id: &VectorID) -> NearestIter<Self::Slice> {
        NearestIter::new(RwLockReadGuard::map(
            self[vector_id.0 as usize].read(),
            Deref::deref,
        ))
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct UpperNode(#[serde(with = "BigArray")] pub [VectorID; M]);

impl UpperNode {
    pub fn from_zero(node: &BaseNode) -> Self {
        let mut nearest = [INVALID; M];
        nearest.copy_from_slice(&node.0[..M]);
        Self(nearest)
    }

    pub fn set(&mut self, index: usize, vector_id: &VectorID) {
        self.0[index] = *vector_id;
    }
}

impl<'a> Layer for &'a [UpperNode] {
    type Slice = &'a [VectorID];
    fn nearest_iter(&self, vector_id: &VectorID) -> NearestIter<Self::Slice> {
        NearestIter::new(&self[vector_id.0 as usize].0)
    }
}

#[derive(Clone)]
pub struct Visited {
    store: Vec<u8>,
    generation: u8,
}

impl Visited {
    /// Creates a new visited object with the capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { store: vec![0; capacity], generation: 1 }
    }

    pub fn resize_capacity(&mut self, capacity: usize) {
        if self.store.len() != capacity {
            self.store.resize(capacity, self.generation - 1);
        }
    }

    /// Inserts a vector ID into the visited object.
    pub fn insert(&mut self, vector_id: &VectorID) -> bool {
        let slot = match self.store.get_mut(vector_id.0 as usize) {
            Some(slot) => slot,
            None => return false,
        };

        if *slot != self.generation {
            *slot = self.generation;
            return true;
        }

        false
    }

    /// Inserts multiple vector IDs into the visited object.
    pub fn extend(&mut self, iter: impl Iterator<Item = VectorID>) {
        for vector_id in iter {
            self.insert(&vector_id);
        }
    }

    pub fn clear(&mut self) {
        if self.generation < 249 {
            self.generation += 1;
            return;
        }

        self.store.clear();
        self.store.resize(self.store.len(), 0);
        self.generation = 1;
    }
}

/// Candidate for the nearest neighbors.
#[derive(Clone, Copy, Debug)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct Candidate {
    pub distance: OrderedFloat<f32>,
    pub vector_id: VectorID,
}

#[derive(Clone)]
pub struct Search {
    pub ef: usize,
    pub visited: Visited,
    candidates: BinaryHeap<Reverse<Candidate>>,
    nearest: Vec<Candidate>,
    working: Vec<Candidate>,
    discarded: Vec<Candidate>,
    distance: Distance,
}

impl Search {
    pub fn new(capacity: usize, distance: Distance) -> Self {
        let visited = Visited::with_capacity(capacity);
        Self { visited, distance, ..Default::default() }
    }

    /// Searches the nearest neighbors in the graph layer.
    pub fn search<L: Layer>(
        &mut self,
        layer: L,
        vector: &Vector,
        vectors: &HashMap<VectorID, Vector>,
        links: usize,
    ) {
        while let Some(Reverse(candidate)) = self.candidates.pop() {
            // Skip candidates conditionally.
            // For Euclidean metrics, skip candidate with larger distances
            // because 0.0 is the smallest and best distance.
            // For other metrics, the bigger the distance, the better.
            if let Some(furthest) = self.nearest.last() {
                if let Distance::Euclidean = self.distance {
                    if candidate.distance > furthest.distance {
                        break;
                    }
                } else if candidate.distance < furthest.distance {
                    break;
                }
            }

            let layer_iter = layer.nearest_iter(&candidate.vector_id);
            for vector_id in layer_iter.take(links) {
                self.push(&vector_id, vector, vectors);
            }

            self.nearest.truncate(self.ef);
        }
    }

    /// Pushes a new neighbor candidate to the search object.
    pub fn push(
        &mut self,
        vector_id: &VectorID,
        vector: &Vector,
        vectors: &HashMap<VectorID, Vector>,
    ) {
        if !self.visited.insert(vector_id) {
            return;
        }

        // Create a new candidate.
        let other = &vectors[vector_id];
        let distance = self.distance.calculate(vector, other);
        let distance = OrderedFloat(distance);
        let new = Candidate { distance, vector_id: *vector_id };

        // Make sure the index to insert to is within the EF scope.
        let index = match self.nearest.binary_search(&new) {
            Err(index) if index < self.ef => index,
            Err(_) => return,
            Ok(_) => unreachable!(),
        };

        self.nearest.insert(index, new);
        self.candidates.push(Reverse(new));
    }

    /// Lowers the search to the next lower layer.
    pub fn cull(&mut self) {
        self.candidates.clear();
        self.visited.clear();

        for &candidate in self.nearest.iter() {
            self.candidates.push(Reverse(candidate));
        }

        let candidates = self.nearest.iter().map(|c| c.vector_id);
        self.visited.extend(candidates);
    }

    /// Resets the search object data.
    pub fn reset(&mut self) {
        self.visited.clear();
        self.candidates.clear();
        self.nearest.clear();
        self.working.clear();
        self.discarded.clear();
    }

    /// Selects the nearest neighbors.
    pub fn select_simple(&mut self) -> &[Candidate] {
        &self.nearest
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = Candidate> + '_ {
        self.nearest.iter().copied()
    }
}

impl Default for Search {
    fn default() -> Self {
        Self {
            visited: Visited::with_capacity(0),
            candidates: BinaryHeap::new(),
            nearest: Vec::new(),
            working: Vec::new(),
            discarded: Vec::new(),
            ef: 5,
            distance: Distance::Euclidean,
        }
    }
}

pub struct SearchPool {
    pool: Mutex<Vec<(Search, Search)>>,
    distance: Distance,
    len: usize,
}

impl SearchPool {
    pub fn new(len: usize, distance: Distance) -> Self {
        let pool = Mutex::new(Vec::new());
        Self { pool, len, distance }
    }

    /// Returns the last searches from the pool.
    pub fn pop(&self) -> (Search, Search) {
        let search = Search::new(self.len, self.distance);
        match self.pool.lock().pop() {
            Some(result) => result,
            None => (search.clone(), search),
        }
    }

    /// Pushes the searches to the pool.
    pub fn push(&self, item: &(Search, Search)) {
        self.pool.lock().push(item.clone());
    }
}

pub struct IndexConstruction<'a> {
    pub search_pool: SearchPool,
    pub top_layer: LayerID,
    pub base_layer: &'a [RwLock<BaseNode>],
    pub vectors: &'a HashMap<VectorID, Vector>,
    pub config: &'a Config,
}

impl<'a> IndexConstruction<'a> {
    /// Inserts a vector ID into a layer.
    /// * `vector_id`: Vector ID to insert.
    /// * `layer`: Layer to insert into.
    /// * `layers`: Upper layers.
    pub fn insert(
        &self,
        vector_id: &VectorID,
        layer: &LayerID,
        layers: &[Vec<UpperNode>],
    ) {
        let vector = &self.vectors[vector_id];
        let dist = self.config.distance;

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
                    distance.cmp(&dist.calculate(old, other).into())
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
