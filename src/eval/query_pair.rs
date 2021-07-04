use crate::error::{Error, ErrorCode, Result};
use crate::query::{EvaluableQueryPair, QueryPair, Value};
use serde_json::value::Value as JSONValue;

impl QueryPair {
    fn to_value(pointer: &str, v: &JSONValue) -> Result<Value> {
        let p = v.pointer(pointer).ok_or_else(|| {
            Error::new(ErrorCode::InvalidPointer {
                pointer: pointer.to_owned(),
                value: format!("{}", v),
            })
        })?;
        match p {
            JSONValue::Null => Ok(Value::Null),
            JSONValue::Bool(x) => Ok(Value::Bool(*x)),
            JSONValue::Number(x) => {
                if x.is_i64() {
                    Ok(Value::Int(x.as_i64().unwrap() as i32))
                } else {
                    Ok(Value::Float(x.as_f64().unwrap()))
                }
            }
            JSONValue::String(x) => Ok(Value::String(x.as_str().to_string())),
            _ => Err(Error::new(ErrorCode::InvalidTarget {
                pointer: pointer.to_owned(),
                value: format!("{}", v),
            })),
        }
    }
}

impl EvaluableQueryPair for QueryPair {
    fn eval(&self, value: &JSONValue) -> Result<bool> {
        let v = Self::to_value(&self.pointer, value)?;
        self.condition.eval(&v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;

    const SAMPLE: &str = r#"
{
    "n": null,
    "d": {
        "i": 1,
        "f": 1.2,
        "a": [
            "one",
            "two",
            "three"
        ]
    }
}"#;

    macro_rules! test_to_value {
        ($name:ident, $pointer:literal, $want:expr) => {
            #[test]
            fn $name() {
                let j = from_str(SAMPLE).unwrap();
                let got = QueryPair::to_value($pointer, &j).unwrap();
                assert_eq!($want, got);
            }
        };
    }

    test_to_value!(to_value_null, "/n", Value::Null);
    test_to_value!(to_value_int, "/d/i", Value::Int(1));
    test_to_value!(to_value_float, "/d/f", Value::Float(1.2));
    test_to_value!(to_value_str, "/d/a/1", Value::String("two".to_owned()));

    macro_rules! test_to_value_fail {
        ($name:ident, $pointer:literal) => {
            #[test]
            #[should_panic]
            fn $name() {
                let j = from_str(SAMPLE).unwrap();
                QueryPair::to_value($pointer, &j).unwrap();
            }
        };
    }

    test_to_value_fail!(to_value_fail_out_of_bounds, "/X");
    test_to_value_fail!(to_value_fail_array, "/d/a");
    test_to_value_fail!(to_value_fail_object, "/d");
}
