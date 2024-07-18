#![allow(missing_docs)]

use crate::types::record::*;
use std::collections::HashMap;

/// Joined multiple filters operation with either AND or OR.
///
/// The chosen join will be applied to the filters monotonically.
/// So, it's not possible to mix AND and OR filters in the same operation.
#[derive(Debug, PartialEq)]
pub enum Filters {
    NONE,
    AND(Vec<Filter>),
    OR(Vec<Filter>),
}

impl Filters {
    pub fn apply(&self, data: &HashMap<ColumnName, Option<DataValue>>) -> bool {
        match self {
            Filters::NONE => true,
            Filters::AND(filters) => filters.iter().all(|f| f.apply(data)),
            Filters::OR(filters) => filters.iter().any(|f| f.apply(data)),
        }
    }
}

impl From<&str> for Filters {
    fn from(value: &str) -> Self {
        if value.is_empty() {
            return Filters::NONE;
        }

        const OR: &str = " OR ";
        const AND: &str = " AND ";

        // Check which join operator is used.
        let or_count = value.matches(OR).count();
        let and_count = value.matches(AND).count();

        if or_count > 0 && and_count > 0 {
            panic!("Mixing AND and OR join operators is not supported.");
        }

        let join = if or_count > 0 { OR } else { AND };
        let filters = value.split(join).map(Into::into).collect();
        match join {
            OR => Filters::OR(filters),
            _ => Filters::AND(filters),
        }
    }
}

/// Record metadata filter.
///
/// Using the filter operator, the record metadata can be compared against
/// a specific value to determine if it should be included in the results.
#[derive(Debug, PartialEq)]
pub struct Filter {
    pub column: ColumnName,
    pub value: DataValue,
    pub operator: FilterOperator,
}

impl Filter {
    pub fn apply(&self, data: &HashMap<ColumnName, Option<DataValue>>) -> bool {
        let value = match data.get(&self.column).unwrap_or(&None) {
            Some(value) => value,
            None => return false,
        };

        // This alias helps us cut down lines of code.
        type Type = DataValue;
        match (value, &self.value) {
            (Type::Boolean(a), Type::Boolean(b)) => self.match_boolean(*a, *b),
            (Type::Float(a), Type::Float(b)) => self.match_number(a, b),
            (Type::Integer(a), Type::Integer(b)) => self.match_number(a, b),
            (Type::String(a), Type::String(b)) => self.match_string(a, b),
            _ => false,
        }
    }

    fn match_boolean(&self, a: bool, b: bool) -> bool {
        match self.operator {
            FilterOperator::Equal => a == b,
            FilterOperator::NotEqual => a != b,
            _ => false,
        }
    }

    fn match_number<T: PartialEq + PartialOrd>(&self, a: T, b: T) -> bool {
        match self.operator {
            FilterOperator::Equal => a == b,
            FilterOperator::NotEqual => a != b,
            FilterOperator::GreaterThan => a > b,
            FilterOperator::GreaterThanOrEqual => a >= b,
            FilterOperator::LessThan => a < b,
            FilterOperator::LessThanOrEqual => a <= b,
            _ => false,
        }
    }

    fn match_string(&self, a: &str, b: &str) -> bool {
        match self.operator {
            FilterOperator::Contain => a.contains(b),
            FilterOperator::Equal => a == b,
            FilterOperator::NotEqual => a != b,
            _ => false,
        }
    }
}

impl From<&str> for Filter {
    fn from(value: &str) -> Self {
        if value.is_empty() {
            panic!("Filter string cannot be empty.");
        }

        // Split the filter string into EXACTLY 3 parts.
        let parts: Vec<&str> = value.splitn(3, ' ').collect();
        let parts: Vec<&str> = parts.into_iter().map(|p| p.trim()).collect();

        let column = parts[0].into();
        let operator = FilterOperator::from(parts[1]);
        let value = DataValue::from(parts[2]);
        Filter { column, value, operator }
    }
}

/// Filter operator.
///
/// Some of the operators are only applicable to specific data types.
/// - Contain is only applicable to string data type.
/// - Equal and NotEqual is applicable to all data types.
/// - The rest are applicable to integer and float data types.
#[derive(Debug, PartialEq, Eq)]
pub enum FilterOperator {
    Contain,
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl From<&str> for FilterOperator {
    fn from(value: &str) -> Self {
        match value {
            "CONTAINS" => FilterOperator::Contain,
            "=" => FilterOperator::Equal,
            "!=" => FilterOperator::NotEqual,
            ">" => FilterOperator::GreaterThan,
            ">=" => FilterOperator::GreaterThanOrEqual,
            "<" => FilterOperator::LessThan,
            "<=" => FilterOperator::LessThanOrEqual,
            _ => panic!("Invalid filter operator: {}", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data() -> HashMap<ColumnName, Option<DataValue>> {
        let columns = vec!["name", "age", "gpa", "active"];
        let values: Vec<DataValue> = vec![
            "Alice".into(),
            DataValue::Integer(20),
            DataValue::Float(3.5),
            DataValue::Boolean(true),
        ];

        let mut data = HashMap::new();
        for (column, value) in columns.into_iter().zip(values.into_iter()) {
            data.insert(column.into(), Some(value));
        }

        data
    }

    #[test]
    fn test_filters_from_string() {
        let filters = Filters::from("name CONTAINS Ada");
        let expected = Filters::AND(vec![Filter {
            column: "name".into(),
            value: "Ada".into(),
            operator: FilterOperator::Contain,
        }]);

        assert_eq!(filters, expected);

        let filters = Filters::from("gpa >= 3.0 OR age < 21");
        let expected = {
            let filter_gpa = Filter {
                column: "gpa".into(),
                value: DataValue::Float(3.0),
                operator: FilterOperator::GreaterThanOrEqual,
            };

            let filter_age = Filter {
                column: "age".into(),
                value: DataValue::Integer(21),
                operator: FilterOperator::LessThan,
            };

            Filters::OR(vec![filter_gpa, filter_age])
        };

        assert_eq!(filters, expected);
    }

    #[test]
    fn test_filters_apply() {
        let data = create_test_data();

        let filters = Filters::from("name CONTAINS Alice");
        assert!(filters.apply(&data));

        let filters = Filters::from("name = Bob");
        assert!(!filters.apply(&data));

        let filters = Filters::from("age >= 20 AND gpa < 4.0");
        assert!(filters.apply(&data));

        let filters = Filters::from("age >= 20 AND gpa < 3.0");
        assert!(!filters.apply(&data));

        let filters = Filters::from("active = true");
        assert!(filters.apply(&data));
    }
}
