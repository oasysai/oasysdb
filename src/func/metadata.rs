use super::*;

#[cfg(feature = "json")]
use serde_json::{Map, Number, Value};

/// The metadata associated with a vector record.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Metadata {
    /// A piece of text like article title or description.
    Text(String),
    /// An integer number such as external IDs.
    Integer(usize),
    /// A float number to represent something like a score.
    Float(f32),
    /// An array containing any type of metadata.
    Array(Vec<Metadata>),
    /// A map of string and metadata pairs. The most common type.
    Object(HashMap<String, Metadata>),
}

impl From<usize> for Metadata {
    fn from(value: usize) -> Self {
        Metadata::Integer(value)
    }
}

impl From<f32> for Metadata {
    fn from(value: f32) -> Self {
        Metadata::Float(value)
    }
}

impl From<String> for Metadata {
    fn from(value: String) -> Self {
        Metadata::Text(value)
    }
}

impl From<&str> for Metadata {
    fn from(value: &str) -> Self {
        Metadata::Text(value.to_string())
    }
}

impl<T> From<Vec<T>> for Metadata
where
    Metadata: From<T>,
{
    fn from(value: Vec<T>) -> Self {
        let arr = value.into_iter().map(|v| v.into()).collect();
        Metadata::Array(arr)
    }
}

impl<T> From<HashMap<String, T>> for Metadata
where
    Metadata: From<T>,
{
    fn from(value: HashMap<String, T>) -> Self {
        let iter = value.into_iter();
        let obj = iter.map(|(k, v)| (k, v.into())).collect();
        Metadata::Object(obj)
    }
}

impl<T> From<HashMap<&str, T>> for Metadata
where
    Metadata: From<T>,
{
    fn from(value: HashMap<&str, T>) -> Self {
        let iter = value.into_iter();
        let obj = iter.map(|(k, v)| (k.into(), v.into())).collect();
        Metadata::Object(obj)
    }
}

// This implementation allows conversion from
// JSON Value type to the Metadata enum.
#[cfg(feature = "json")]
impl From<Value> for Metadata {
    fn from(value: Value) -> Self {
        // Cast JSON number to Metadata float or integer.
        let convert_number = |number: Number| {
            // Check if the number is float. If not, it's an integer.
            if number.is_f64() {
                let float = number.as_f64().unwrap();
                Metadata::Float(float as f32)
            } else {
                let int = number.as_i64().unwrap();
                Metadata::Integer(int as usize)
            }
        };

        // Cast JSON array to Metadata array.
        let convert_array = |array: Vec<Value>| {
            let vec = array.into_iter().map(|v| v.into()).collect();
            Metadata::Array(vec)
        };

        // Cast JSON object to Metadata object.
        let convert_object = |object: Map<String, Value>| {
            let map = object
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect::<HashMap<String, Metadata>>();
            Metadata::Object(map)
        };

        match value {
            Value::String(text) => Metadata::Text(text),
            Value::Number(number) => convert_number(number),
            Value::Array(array) => convert_array(array),
            Value::Object(object) => convert_object(object),
            _ => panic!("Unsupported JSON type for the metadata."),
        }
    }
}

// This implementation allows conversion from
// the native Metadata enum to JSON Value.
#[cfg(feature = "json")]
impl From<Metadata> for Value {
    fn from(metadata: Metadata) -> Self {
        // Convert Metadata integer to JSON number.
        let convert_integer = |int: usize| {
            let number = Number::from(int as i64);
            Value::Number(number)
        };

        // Convert Metadata float to JSON number.
        let convert_float = |float: f32| {
            let number = Number::from_f64(float as f64).unwrap();
            Value::Number(number)
        };

        // Convert Metadata array to JSON array.
        let convert_array = |arr: Vec<Metadata>| {
            let vec = arr.into_iter().map(|v| v.into()).collect();
            Value::Array(vec)
        };

        // Convert Metadata object to JSON object.
        let convert_object = |obj: HashMap<String, Metadata>| {
            let map = obj
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect::<Map<String, Value>>();
            Value::Object(map)
        };

        match metadata {
            Metadata::Text(text) => Value::String(text),
            Metadata::Integer(int) => convert_integer(int),
            Metadata::Float(float) => convert_float(float),
            Metadata::Array(array) => convert_array(array),
            Metadata::Object(object) => convert_object(object),
        }
    }
}

// This implementation attempts to convert the
// Python object into the Metadata enum.
#[cfg(feature = "py")]
impl From<&PyAny> for Metadata {
    fn from(value: &PyAny) -> Self {
        // Extract string.
        if let Ok(text) = value.extract::<String>() {
            return Metadata::Text(text);
        }

        // Extract integer.
        if let Ok(int) = value.extract::<usize>() {
            return Metadata::Integer(int);
        }

        // Extract float.
        if let Ok(float) = value.extract::<f32>() {
            return Metadata::Float(float);
        }

        // Extract list.
        if let Ok(list) = value.extract::<Vec<&PyAny>>() {
            let arr = list.into_iter().map(|v| v.into()).collect();
            return Metadata::Array(arr);
        }

        // Extract dictionary.
        if let Ok(dict) = value.extract::<HashMap<String, &PyAny>>() {
            let obj = dict.into_iter().map(|(k, v)| (k, v.into())).collect();
            return Metadata::Object(obj);
        }

        // Throw an error if the type is not supported.
        panic!("Unsupported type for the metadata.");
    }
}

// This implementation converts the Metadata
// enum back to the Python object.
#[cfg(feature = "py")]
impl IntoPy<Py<PyAny>> for Metadata {
    fn into_py(self, py: Python) -> Py<PyAny> {
        // Convert array of Metadata to Python list.
        let list_converter = |vec: Vec<Metadata>| {
            let list = vec
                .into_iter()
                .map(|metadata: Metadata| metadata.into_py(py))
                .collect::<Vec<Py<PyAny>>>();
            list.into_py(py)
        };

        // Convert HashMap of Metadata to Python dictionary.
        let dict_converter = |map: HashMap<String, Metadata>| {
            let dict = map
                .into_iter()
                .map(|(key, value)| (key, value.into_py(py)))
                .collect::<HashMap<String, Py<PyAny>>>();
            dict.into_py(py)
        };

        match self {
            Metadata::Text(text) => text.into_py(py),
            Metadata::Integer(int) => int.into_py(py),
            Metadata::Float(float) => float.into_py(py),
            Metadata::Array(arr) => list_converter(arr),
            Metadata::Object(obj) => dict_converter(obj),
        }
    }
}
