use super::*;

const AND: &str = " AND ";
const OR: &str = " OR ";

/// The filters to apply to the collection metadata.
#[derive(Debug, PartialEq)]
pub enum Filters {
    /// Results must match all filters.
    AND(Vec<Filter>),
    /// Results must match at least one filter.
    OR(Vec<Filter>),
}

impl Filters {
    /// Matches the collection metadata against the filters.
    pub fn match_metadata(&self, metadata: &Metadata) -> bool {
        match self {
            Filters::AND(filters) => {
                filters.iter().all(|f| f.match_metadata(metadata))
            }
            Filters::OR(filters) => {
                filters.iter().any(|f| f.match_metadata(metadata))
            }
        }
    }
}

impl From<&str> for Filters {
    fn from(filters: &str) -> Self {
        if filters.is_empty() {
            return Filters::AND(vec![]);
        }

        // Check which join operator is used.
        let or_count = filters.matches(OR).count();
        let and_count = filters.matches(AND).count();

        let join = if or_count > 0 && and_count > 0 {
            panic!("Mixing AND and OR join operators is not supported.");
        } else if or_count > 0 {
            OR
        } else {
            // If no join operator is found, use AND since it doesn't matter.
            AND
        };

        // Split the filters.
        let filters = filters.split(join).map(Into::into).collect();
        match join {
            OR => Filters::OR(filters),
            _ => Filters::AND(filters),
        }
    }
}

impl From<String> for Filters {
    fn from(filters: String) -> Self {
        Filters::from(filters.as_str())
    }
}

/// The basic filter operator to use to compare with metadata.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum FilterOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Contains,
}

// String type representing the filter key type.
// This helps us prevent typos and make the code more readable.
const TEXT: &str = "text";
const INTEGER: &str = "integer";
const FLOAT: &str = "float";
const BOOLEAN: &str = "boolean";
const ARRAY: &str = "array";
const OBJECT: &str = "object";

/// The filter to match against the collection metadata.
#[derive(Debug, PartialEq)]
pub struct Filter {
    /// Metadata key to filter.
    pub key: String,
    /// The filter value to match against.
    pub value: Metadata,
    /// Filter operator to use for matching.
    pub operator: FilterOperator,
}

impl Filter {
    /// Creates a new filter instance.
    /// * `key`: Key to filter.
    /// * `value`: Value to use for filtering.
    /// * `operator`: Filter operator.
    pub fn new(key: &str, value: &Metadata, operator: &FilterOperator) -> Self {
        Self::validate_filter(key, value, operator);
        Self {
            key: key.to_string(),
            value: value.clone(),
            operator: operator.clone(),
        }
    }

    /// Matches the collection metadata against the filter.
    pub fn match_metadata(&self, metadata: &Metadata) -> bool {
        let key_parts: Vec<&str> = self.key.split('.').collect();
        let key_type = key_parts[0];

        match key_type {
            TEXT => return self.match_text(metadata),
            INTEGER => return self.match_integer(metadata),
            FLOAT => return self.match_float(metadata),
            BOOLEAN => return self.match_boolean(metadata),
            ARRAY => return self.match_array(metadata),
            OBJECT => return self.match_object(metadata),
            // This should never happen because we validate the key type.
            _ => panic!("Unsupported filter key type: {key_type}"),
        }
    }

    fn match_text(&self, metadata: &Metadata) -> bool {
        let text = match metadata {
            Metadata::Text(text) => text,
            _ => return false,
        };

        let filter_text = match &self.value {
            Metadata::Text(text) => text,
            _ => return false,
        };

        match &self.operator {
            FilterOperator::Equal => text == filter_text,
            FilterOperator::NotEqual => text != filter_text,
            FilterOperator::Contains => text.contains(filter_text),
            _ => false,
        }
    }

    fn match_integer(&self, metadata: &Metadata) -> bool {
        let int = match metadata {
            Metadata::Integer(int) => int,
            _ => return false,
        };

        let filter_int = match &self.value {
            Metadata::Integer(int) => int,
            _ => return false,
        };

        match &self.operator {
            FilterOperator::Equal => int == filter_int,
            FilterOperator::NotEqual => int != filter_int,
            FilterOperator::GreaterThan => int > filter_int,
            FilterOperator::GreaterThanOrEqual => int >= filter_int,
            FilterOperator::LessThan => int < filter_int,
            FilterOperator::LessThanOrEqual => int <= filter_int,
            _ => false,
        }
    }

    fn match_float(&self, metadata: &Metadata) -> bool {
        let float = match metadata {
            Metadata::Float(float) => float,
            _ => return false,
        };

        let filter_float = match &self.value {
            Metadata::Float(float) => float,
            _ => return false,
        };

        match &self.operator {
            FilterOperator::Equal => float == filter_float,
            FilterOperator::NotEqual => float != filter_float,
            FilterOperator::GreaterThan => float > filter_float,
            FilterOperator::GreaterThanOrEqual => float >= filter_float,
            FilterOperator::LessThan => float < filter_float,
            FilterOperator::LessThanOrEqual => float <= filter_float,
            _ => false,
        }
    }

    fn match_boolean(&self, metadata: &Metadata) -> bool {
        let bool = match metadata {
            Metadata::Boolean(bool) => bool,
            _ => return false,
        };

        let filter_bool = match &self.value {
            Metadata::Boolean(bool) => bool,
            _ => return false,
        };

        match &self.operator {
            FilterOperator::Equal => bool == filter_bool,
            FilterOperator::NotEqual => bool != filter_bool,
            _ => false,
        }
    }

    fn match_array(&self, metadata: &Metadata) -> bool {
        let array = match metadata {
            Metadata::Array(arr) => arr,
            _ => return false,
        };

        match &self.operator {
            FilterOperator::Contains => array.contains(&self.value),
            _ => self.match_array_value(array),
        }
    }

    fn match_array_value(&self, array: &Vec<Metadata>) -> bool {
        let key_parts: Vec<&str> = self.key.split('.').collect();

        // This has been validated in the `validate_filter` method.
        // So, we can safely unwrap it here.
        let index = key_parts[1].parse::<usize>().unwrap();
        let value = match array.get(index) {
            Some(value) => value,
            None => return false,
        };

        self.match_subvalue(value)
    }

    fn match_object(&self, metadata: &Metadata) -> bool {
        let object = match metadata {
            Metadata::Object(obj) => obj,
            _ => return false,
        };

        let key_parts: Vec<&str> = self.key.split('.').collect();
        let value = match object.get(key_parts[1]) {
            Some(value) => value,
            None => return false,
        };

        self.match_subvalue(value)
    }

    /// Creates a sub-filter to match primitive value of an array or object.
    fn match_subvalue(&self, value: &Metadata) -> bool {
        // We expect the value we retrieve from the object to be a primitive type.
        // So we create a sub-filter to match the value.
        let subfilter_key = match value {
            Metadata::Text(_) => TEXT,
            Metadata::Integer(_) => INTEGER,
            Metadata::Float(_) => FLOAT,
            Metadata::Boolean(_) => BOOLEAN,
            _ => panic!("Unsupported 2nd level array or object to filter."),
        };

        let subfilter = Filter::new(subfilter_key, &self.value, &self.operator);
        subfilter.match_metadata(value)
    }

    /// Validates the key with the supported value and filter operator.
    /// * `key`: Filter key.
    /// * `value`: Filter metadata value.
    /// * `operator`: Filter operator.
    fn validate_filter(key: &str, value: &Metadata, operator: &FilterOperator) {
        // Check if the key is valid.
        if key.is_empty() {
            panic!("Filter key cannot be empty.");
        }

        let key_parts: Vec<&str> = key.split('.').collect();
        let key_type = key_parts[0];

        // Check if the key is valid.
        let valid_types = vec![TEXT, INTEGER, FLOAT, BOOLEAN, ARRAY, OBJECT];
        if !valid_types.contains(&key_type) {
            panic!("Invalid filter key type: {key_type}");
        }

        // Check if the key has a sub-key for object type.
        if key_type == OBJECT {
            if key_parts.len() != 2 {
                panic!("Object key must have exactly one sub-key.");
            }

            if key_parts[1].is_empty() {
                panic!("Object sub-key must be a non-empty string.");
            }
        }

        // Validate key for array type.
        if key_type == ARRAY {
            if operator != &FilterOperator::Contains && key_parts.len() != 2 {
                panic!("Array filter must provide an index.");
            }

            if key_parts.len() == 2 && key_parts[1].parse::<usize>().is_err() {
                panic!("Array filter index must be a valid integer.");
            }
        }

        Self::validate_value(key_type, value);
        Self::validate_operator(key_type, operator);
    }

    // Validates the filter value based on the key type.
    fn validate_value(key_type: &str, value: &Metadata) {
        // Prevent array and object types for value.
        // Because, we should handle it like this: object.key = value
        match value {
            Metadata::Array(_) | Metadata::Object(_) => {
                panic!("Unsupported array or object type as value.")
            }
            // We handle the primitive types validation below.
            _ => {}
        }

        // Array and object keys are always valid because we will validate
        // the value type when performing the filter.
        let always_valid_key_types = vec![ARRAY, OBJECT];
        if always_valid_key_types.contains(&key_type) {
            return;
        }

        // Error message for invalid filter value type.
        let panic =
            || panic!("Invalid filter value of {value:?} for key: {key_type}");

        // For key types other than array and object,
        // we need to validate the value type.
        match value {
            Metadata::Text(_) => {
                if key_type != TEXT {
                    panic();
                }
            }
            Metadata::Integer(_) => {
                if key_type != INTEGER {
                    panic();
                }
            }
            Metadata::Float(_) => {
                if key_type != FLOAT {
                    panic();
                }
            }
            Metadata::Boolean(_) => {
                if key_type != BOOLEAN {
                    panic();
                }
            }
            // Array and object values has been handled above.
            _ => {}
        }
    }

    /// Validates the filter operator based on the key type.
    fn validate_operator(key_type: &str, operator: &FilterOperator) {
        match operator {
            // Contains operator is only valid for text, array, and object types.
            FilterOperator::Contains => {
                let valid_types = vec![TEXT, ARRAY, OBJECT];
                if !valid_types.contains(&key_type) {
                    panic!("Invalid CONTAINS operator for key: {key_type}");
                }
            }
            // Numeric operators are not valid for text and boolean types.
            FilterOperator::GreaterThan
            | FilterOperator::GreaterThanOrEqual
            | FilterOperator::LessThan
            | FilterOperator::LessThanOrEqual => {
                let invalid_types = vec![TEXT, BOOLEAN];
                if invalid_types.contains(&key_type) {
                    panic!("Invalid numeric operator for key type: {key_type}");
                }
            }
            // Equal and not equal are valid for all types.
            _ => {}
        }
    }
}

impl From<&str> for Filter {
    fn from(filter: &str) -> Self {
        if filter.is_empty() {
            panic!("Filter string cannot be empty.");
        }

        // Split the filter string into EXACTLY 3 parts.
        let parts: Vec<&str> = filter.splitn(3, ' ').collect();
        let parts: Vec<&str> = parts.into_iter().map(|p| p.trim()).collect();

        // Get and validate the filter operator.
        let operator = match parts[1] {
            "=" => FilterOperator::Equal,
            "!=" => FilterOperator::NotEqual,
            ">" => FilterOperator::GreaterThan,
            ">=" => FilterOperator::GreaterThanOrEqual,
            "<" => FilterOperator::LessThan,
            "<=" => FilterOperator::LessThanOrEqual,
            "CONTAINS" => FilterOperator::Contains,
            _ => panic!("Invalid filter operator: {}", parts[1]),
        };

        let key = parts[0].to_string();
        let value = Metadata::from(parts[2]);
        Self::new(&key, &value, &operator)
    }
}

impl From<String> for Filter {
    fn from(filter: String) -> Self {
        Filter::from(filter.as_str())
    }
}
