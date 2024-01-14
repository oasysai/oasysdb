use super::*;

pub const INVALID: VectorID = VectorID(u32::MAX);

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
pub struct BaseNode<const M: usize>(
    #[serde(with = "BigArray")] pub [VectorID; M],
);

impl<const M: usize> Default for BaseNode<M> {
    fn default() -> Self {
        Self([INVALID; M])
    }
}

impl<const M: usize> BaseNode<M> {
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
            let end = M - 1;
            self.0.copy_within(index..end, index + 1);
        }

        self.set(index, vector_id)
    }

    /// Sets the vector ID at the index.
    pub fn set(&mut self, index: usize, vector_id: &VectorID) {
        self.0[index] = *vector_id;
    }
}

impl<const M: usize> Index<&VectorID> for [RwLock<BaseNode<M>>] {
    type Output = RwLock<BaseNode<M>>;
    fn index(&self, index: &VectorID) -> &Self::Output {
        &self[index.0 as usize]
    }
}

impl<const M: usize> Deref for BaseNode<M> {
    type Target = [VectorID];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, const M: usize> Layer for &'a [BaseNode<M>] {
    type Slice = &'a [VectorID];
    fn nearest_iter(&self, vector_id: &VectorID) -> NearestIter<Self::Slice> {
        NearestIter::new(&self[vector_id.0 as usize])
    }
}

impl<'a, const M: usize> Layer for &'a [RwLock<BaseNode<M>>] {
    type Slice = MappedRwLockReadGuard<'a, [VectorID]>;
    fn nearest_iter(&self, vector_id: &VectorID) -> NearestIter<Self::Slice> {
        NearestIter::new(RwLockReadGuard::map(
            self[vector_id.0 as usize].read(),
            Deref::deref,
        ))
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct UpperNode<const M: usize>(
    #[serde(with = "BigArray")] pub [VectorID; M],
);

impl<const M: usize> UpperNode<M> {
    pub fn from_zero(node: &BaseNode<M>) -> Self {
        let mut nearest = [INVALID; M];
        nearest.copy_from_slice(&node.0[..M]);
        Self(nearest)
    }
}

impl<'a, const M: usize> Layer for &'a [UpperNode<M>] {
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
pub struct Search<const M: usize, const N: usize> {
    pub ef: usize,
    pub visited: Visited,
    candidates: BinaryHeap<Reverse<Candidate>>,
    nearest: Vec<Candidate>,
    working: Vec<Candidate>,
    discarded: Vec<Candidate>,
}

impl<const M: usize, const N: usize> Search<M, N> {
    pub fn new(capacity: usize) -> Self {
        let visited = Visited::with_capacity(capacity);
        Self { visited, ..Default::default() }
    }

    /// Searches the nearest neighbors in the graph layer.
    pub fn search<L: Layer>(
        &mut self,
        layer: L,
        vector: &Vector<N>,
        vectors: &HashMap<VectorID, Vector<N>>,
        links: usize,
    ) {
        while let Some(Reverse(candidate)) = self.candidates.pop() {
            // Skip candidates that are too far.
            if let Some(furthest) = self.nearest.last() {
                if candidate.distance > furthest.distance {
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
        vector: &Vector<N>,
        vectors: &HashMap<VectorID, Vector<N>>,
    ) {
        if !self.visited.insert(vector_id) {
            return;
        }

        // Create a new candidate.
        let other = &vectors[vector_id];
        let distance = OrderedFloat::from(vector.distance(other));
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

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = Candidate> + ExactSizeIterator + '_ {
        self.nearest.iter().copied()
    }
}

impl<const M: usize, const N: usize> Default for Search<M, N> {
    fn default() -> Self {
        Self {
            visited: Visited::with_capacity(0),
            candidates: BinaryHeap::new(),
            nearest: Vec::new(),
            working: Vec::new(),
            discarded: Vec::new(),
            ef: 5,
        }
    }
}

pub struct SearchPool<const M: usize, const N: usize> {
    pool: Mutex<Vec<(Search<M, N>, Search<M, N>)>>,
    len: usize,
}

impl<const M: usize, const N: usize> SearchPool<M, N> {
    pub fn new(len: usize) -> Self {
        let pool = Mutex::new(Vec::new());
        Self { pool, len }
    }

    /// Returns the last searches from the pool.
    pub fn pop(&self) -> (Search<M, N>, Search<M, N>) {
        match self.pool.lock().pop() {
            Some(result) => result,
            None => (Search::new(self.len), Search::new(self.len)),
        }
    }

    /// Pushes the searches to the pool.
    pub fn push(&self, item: &(Search<M, N>, Search<M, N>)) {
        self.pool.lock().push(item.clone());
    }
}
