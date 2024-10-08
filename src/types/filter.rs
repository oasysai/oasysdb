use super::*;

/// Joined multiple filters operation with either AND or OR.
///
/// At the moment, OasysDB only supports single-type join operations. This
/// means that we can't use both AND and OR operations in the same filter.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Filters {
    NONE,
    AND(Vec<Filter>),
    OR(Vec<Filter>),
}

impl Filters {
    /// Returns true if the record passes the filters.
    /// - metadata: Record metadata to check against the filters.
    ///
    /// Filters of NONE type will always return true. This is useful when
    /// no filters are provided and we want to include all records.
    pub fn apply(&self, metadata: &HashMap<String, Value>) -> bool {
        match self {
            Filters::NONE => true,
            Filters::AND(filters) => filters.iter().all(|f| f.apply(metadata)),
            Filters::OR(filters) => filters.iter().any(|f| f.apply(metadata)),
        }
    }
}

impl TryFrom<&str> for Filters {
    type Error = Status;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(Filters::NONE);
        }

        const OR: &str = " OR ";
        const AND: &str = " AND ";

        // Check which join operator is used.
        let or_count = value.matches(OR).count();
        let and_count = value.matches(AND).count();

        if or_count > 0 && and_count > 0 {
            let message = "Mixing AND and OR join operators is not supported";
            return Err(Status::invalid_argument(message));
        }

        let join = if or_count > 0 { OR } else { AND };
        let filters = value
            .split(join)
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;

        let filters = match join {
            OR => Filters::OR(filters),
            _ => Filters::AND(filters),
        };

        Ok(filters)
    }
}

/// Record metadata filter.
///
/// Using the filter operator, the record metadata can be compared against
/// a specific value to determine if it should be included in the results.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Filter {
    key: String,
    value: Value,
    operator: Operator,
}

impl Filter {
    fn apply(&self, metadata: &HashMap<String, Value>) -> bool {
        let value = match metadata.get(&self.key) {
            Some(value) => value,
            None => return false,
        };

        match (value, &self.value) {
            (Value::Text(a), Value::Text(b)) => self.filter_text(a, b),
            (Value::Number(a), Value::Number(b)) => self.filter_number(a, b),
            (Value::Boolean(a), Value::Boolean(b)) => self.filter_boolean(a, b),
            _ => false,
        }
    }

    fn filter_text(&self, a: impl AsRef<str>, b: impl AsRef<str>) -> bool {
        let (a, b) = (a.as_ref(), b.as_ref());
        match self.operator {
            Operator::Equal => a == b,
            Operator::NotEqual => a != b,
            Operator::Contains => a.contains(b),
            _ => false,
        }
    }

    fn filter_number(&self, a: &f64, b: &f64) -> bool {
        match self.operator {
            Operator::Equal => a == b,
            Operator::NotEqual => a != b,
            Operator::GreaterThan => a > b,
            Operator::GreaterThanOrEqual => a >= b,
            Operator::LessThan => a < b,
            Operator::LessThanOrEqual => a <= b,
            _ => false,
        }
    }

    fn filter_boolean(&self, a: &bool, b: &bool) -> bool {
        match self.operator {
            Operator::Equal => a == b,
            Operator::NotEqual => a != b,
            _ => false,
        }
    }
}

impl TryFrom<&str> for Filter {
    type Error = Status;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            let message = "Filter string cannot be empty";
            return Err(Status::invalid_argument(message));
        }

        // Split the filter string into EXACTLY 3 parts.
        let parts = value
            .splitn(3, ' ')
            .map(|token| token.trim())
            .collect::<Vec<&str>>();

        let key = parts[0].to_string();
        let operator = Operator::try_from(parts[1])?;
        let value = Value::from(parts[2]);

        let filter = Filter { key, value, operator };
        Ok(filter)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
pub enum Operator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Contains,
}

impl TryFrom<&str> for Operator {
    type Error = Status;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let operator = match value {
            "CONTAINS" => Operator::Contains,
            "=" => Operator::Equal,
            "!=" => Operator::NotEqual,
            ">" => Operator::GreaterThan,
            ">=" => Operator::GreaterThanOrEqual,
            "<" => Operator::LessThan,
            "<=" => Operator::LessThanOrEqual,
            _ => {
                let message = format!("Invalid filter operator: {value}");
                return Err(Status::invalid_argument(message));
            }
        };

        Ok(operator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_filters_from_string() {
        let filters = Filters::try_from("name CONTAINS Ada").unwrap();
        let expected = Filters::AND(vec![Filter {
            key: "name".into(),
            value: "Ada".into(),
            operator: Operator::Contains,
        }]);

        assert_eq!(filters, expected);

        let filters = Filters::try_from("gpa >= 3.0 OR age < 21").unwrap();
        let expected = {
            let filter_gpa = Filter {
                key: "gpa".into(),
                value: Value::Number(3.0),
                operator: Operator::GreaterThanOrEqual,
            };

            let filter_age = Filter {
                key: "age".into(),
                value: Value::Number(21.0),
                operator: Operator::LessThan,
            };

            Filters::OR(vec![filter_gpa, filter_age])
        };

        assert_eq!(filters, expected);
    }

    #[test]
    fn test_filters_apply() -> Result<(), Box<dyn Error>> {
        let data = setup_metadata();

        let filters = Filters::try_from("name CONTAINS Alice")?;
        assert!(filters.apply(&data));

        let filters = Filters::try_from("name = Bob")?;
        assert!(!filters.apply(&data));

        let filters = Filters::try_from("age >= 20 AND gpa < 4.0")?;
        assert!(filters.apply(&data));

        let filters = Filters::try_from("age >= 20 AND gpa < 3.0")?;
        assert!(!filters.apply(&data));

        let filters = Filters::try_from("active = true")?;
        assert!(filters.apply(&data));

        Ok(())
    }

    fn setup_metadata() -> HashMap<String, Value> {
        let keys = vec!["name", "age", "gpa", "active"];
        let values: Vec<Value> = vec![
            "Alice".into(),
            Value::Number(20.0),
            Value::Number(3.5),
            Value::Boolean(true),
        ];

        let mut data = HashMap::new();
        for (key, value) in keys.into_iter().zip(values.into_iter()) {
            data.insert(key.into(), value);
        }

        data
    }
}
