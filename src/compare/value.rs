use serde_json::value::Value;
use std::cmp::Ordering;

/// Wrap [`Value`] for comparing.
pub(crate) struct PairValue(Value);

impl From<Value> for PairValue {
    fn from(v: Value) -> PairValue {
        PairValue(v)
    }
}

impl PartialEq for PairValue {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Value::Null, Value::Null) => true,
            (Value::Array(_), Value::Array(_)) => true,
            (Value::Object(_), Value::Object(_)) => true,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Number(x), Value::Number(y)) => {
                (x.as_f64().unwrap() - y.as_f64().unwrap()).abs() <= f64::EPSILON
            }
            (Value::String(x), Value::String(y)) => x.cmp(y) == Ordering::Equal,
            (_, _) => false,
        }
    }
}

impl PartialOrd for PairValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            return Some(Ordering::Equal);
        }
        let r = match (&self.0, &other.0) {
            (Value::Null, _) => Ordering::Less,
            (Value::Array(_), Value::Null) => Ordering::Greater,
            (Value::Array(_), _) => Ordering::Less,
            (Value::Object(_), Value::Null | Value::Array(_)) => Ordering::Greater,
            (Value::Object(_), _) => Ordering::Less,
            (Value::Bool(_), Value::Null | Value::Array(_) | Value::Object(_)) => Ordering::Greater,
            (Value::Bool(false), Value::Bool(true)) => Ordering::Less,
            (Value::Bool(true), Value::Bool(false)) => Ordering::Greater,
            (Value::Bool(_), _) => Ordering::Less,
            (
                Value::Number(_),
                Value::Null | Value::Array(_) | Value::Object(_) | Value::Bool(_),
            ) => Ordering::Greater,
            (Value::Number(x), Value::Number(y)) => {
                let a = x.as_f64().unwrap();
                let b = y.as_f64().unwrap();
                if a < b {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            (Value::Number(_), _) => Ordering::Less,
            (Value::String(x), Value::String(y)) => x.cmp(y),
            (Value::String(_), _) => Ordering::Greater,
        };
        Some(r)
    }
}

impl Eq for PairValue {}

impl Ord for PairValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;

    const NULL: Value = Value::Null;

    fn array() -> Value {
        from_str(r#"[null]"#).unwrap()
    }
    fn object() -> Value {
        from_str(r#"{"x":null}"#).unwrap()
    }
    const TRUE: Value = Value::Bool(true);
    const FALSE: Value = Value::Bool(false);
    fn int_value(x: i32) -> Value {
        from_str(&format!("{}", x).to_string()).unwrap()
    }
    fn float_value(x: f64) -> Value {
        from_str(&format!("{}", x).to_string()).unwrap()
    }
    fn string_value(x: &str) -> Value {
        Value::String(x.to_owned())
    }

    struct P(usize, PairValue);

    fn to_indexed_values(values: Vec<Value>) -> Vec<P> {
        let mut v = Vec::new();
        let mut i: usize = 0;
        for x in values {
            v.push(P(i, PairValue(x)));
            i += 1;
        }
        v
    }

    macro_rules! test_sort {
        ($name:ident, $values:expr, $want:expr) => {
            #[test]
            fn $name() {
                let mut v = to_indexed_values($values);
                v.sort_by(|a, b| a.1.cmp(&b.1));
                let got: Vec<usize> = v.iter().map(|x| x.0).collect();
                assert_eq!($want, got);
            }
        };
    }

    test_sort!(
        sort_types,
        vec![
            int_value(1),
            TRUE,
            object(),
            string_value("moon"),
            array(),
            NULL
        ],
        vec![5, 4, 2, 1, 0, 3]
    );
    test_sort!(sort_bools, vec![TRUE, FALSE], vec![1, 0]);
    test_sort!(
        sort_numbers,
        vec![int_value(3), float_value(1.2), int_value(2)],
        vec![1, 2, 0]
    );
    test_sort!(
        sort_strings,
        vec![
            string_value("moon"),
            string_value("harbinger"),
            string_value("sun")
        ],
        vec![1, 0, 2]
    );
}
