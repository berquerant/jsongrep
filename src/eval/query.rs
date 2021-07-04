use crate::error::Result;
use crate::query::{EvaluableQuery, Query};
use serde_json::value::Value as JSONValue;

impl EvaluableQuery for Query {
    fn eval(&self, value: &JSONValue) -> Result<bool> {
        self.query.eval(value)
    }
}
