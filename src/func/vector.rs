use super::*;

/// The ID of a vector record.
#[cfg_attr(feature = "py", pyclass(module = "oasysdb.vector"))]
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[derive(Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct VectorID(pub u32);

#[cfg_attr(feature = "py", pymethods)]
impl VectorID {
    #[cfg(feature = "py")]
    #[new]
    fn py_new(id: u32) -> Self {
        id.into()
    }

    /// True if this vector ID is valid.
    pub fn is_valid(&self) -> bool {
        self.0 != u32::MAX
    }

    /// Returns the vector ID as u32 type.
    pub fn to_u32(&self) -> u32 {
        self.0
    }

    /// Returns the vector ID as usize type.
    pub fn to_usize(&self) -> usize {
        self.0 as usize
    }
}

impl From<u32> for VectorID {
    fn from(id: u32) -> Self {
        VectorID(id)
    }
}

impl From<usize> for VectorID {
    fn from(id: usize) -> Self {
        VectorID(id as u32)
    }
}

impl From<VectorID> for u32 {
    fn from(v: VectorID) -> Self {
        v.0
    }
}

impl From<VectorID> for usize {
    fn from(v: VectorID) -> Self {
        v.0 as usize
    }
}

/// The vector embedding of float numbers.
#[cfg_attr(feature = "py", pyclass(module = "oasysdb.vector"))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[derive(PartialEq, PartialOrd)]
pub struct Vector(pub Vec<f32>);

// Methods available to Python.
// If this implementation is modified, make sure to modify:
// - py/tests/test_vector.py
// - py/oasysdb/vector.pyi
#[cfg_attr(feature = "py", pymethods)]
impl Vector {
    #[cfg(feature = "py")]
    #[new]
    fn py_new(vector: Vec<f32>) -> Self {
        vector.into()
    }

    #[cfg(feature = "py")]
    fn to_list(&self) -> Vec<f32> {
        self.0.clone()
    }

    #[cfg(feature = "py")]
    #[staticmethod]
    #[pyo3(name = "random")]
    fn py_random(dimension: usize) -> Self {
        Vector::random(dimension)
    }

    #[cfg(feature = "py")]
    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }

    #[cfg(feature = "py")]
    fn __len__(&self) -> usize {
        self.len()
    }

    /// Returns the dimension of the vector.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Vector {
    /// Generates a random vector for testing.
    /// * `dimension`: Vector dimension.
    pub fn random(dimension: usize) -> Self {
        let mut vec = vec![0.0; dimension];

        for float in vec.iter_mut() {
            *float = random::<f32>();
        }

        vec.into()
    }
}

impl Index<&VectorID> for [Vector] {
    type Output = Vector;
    fn index(&self, index: &VectorID) -> &Self::Output {
        &self[index.0 as usize]
    }
}

impl From<Vec<f32>> for Vector {
    fn from(vec: Vec<f32>) -> Self {
        Vector(vec)
    }
}

impl From<&Vec<f32>> for Vector {
    fn from(vec: &Vec<f32>) -> Self {
        Vector(vec.clone())
    }
}

impl From<Vector> for Vec<f32> {
    fn from(vector: Vector) -> Self {
        vector.0
    }
}
