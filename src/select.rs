use crate::error::{Error, ErrorCode, Result};
use crate::query;
use serde_json::from_str;
use serde_json::value::Value;

/// JSON filter.
///
/// ```
/// # use jsongrep::raw_query;
/// # use jsongrep::query;
/// # use jsongrep::select;
/// # use std::convert::{TryFrom, From};
///
/// const jq: &str = r#"{"query":{"type":"raw","pair":{"p":"/s","cond":{"type":"match","mtype":"regex","value":{"type":"string","value":"[sS]irius"}}}}}"#;
/// let sel = raw_query::Query::try_from(jq).map(|x| {
///   let q = query::Query::from(x);
///   select::Query::new(Box::new(q))
/// }).unwrap();
///
/// /// matched
/// const sirius: &str = r#"{"s":"Sirius at the starry night in the winter"}"#;
/// assert!(sel.select(sirius).is_ok());
/// /// unmatched
/// const spica: &str = r#"{"s":"Spica on the earth"}"#;
/// assert!(sel.select(spica).err().unwrap().is_filtered());
/// ```
pub struct Query {
    q: Box<dyn query::EvaluableQuery>,
}

impl Query {
    /// Create a new Query.
    pub fn new(q: Box<dyn query::EvaluableQuery>) -> Query {
        Query { q }
    }
    /// Create a new Query without condition.
    /// It accepts any json.
    pub fn all() -> Query {
        struct OkQuery;
        impl query::EvaluableQuery for OkQuery {
            fn eval(&self, _: &Value) -> Result<bool> {
                Ok(true)
            }
        }
        let q = Box::new(OkQuery);
        Query { q }
    }
    /// Filter a given json.
    /// Return `Ok` if a given json meet the condition.
    /// Return `Err` with `ErrorCode::FilteredByQuery` if a given json does not meet the condition.
    pub fn select(&self, v: &str) -> Result<Value> {
        let v = from_str(v).map_err(|x| Error::new(ErrorCode::Json(x)))?;
        match self.q.eval(&v) {
            Ok(true) => Ok(v),
            Ok(false) => Err(Error::new(ErrorCode::FilteredByQuery)),
            Err(x) => Err(x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockOkEvaluableQuery(bool);
    impl query::EvaluableQuery for MockOkEvaluableQuery {
        fn eval(&self, _: &Value) -> Result<bool> {
            Ok(self.0)
        }
    }

    #[test]
    #[should_panic]
    fn test_query_err_json() {
        let q = MockOkEvaluableQuery(true);
        Query::new(Box::new(q)).select("white").unwrap();
    }
    #[test]
    fn test_query_err_filtered() {
        let q = MockOkEvaluableQuery(false);
        let got = Query::new(Box::new(q)).select(r#"{"x":1}"#);
        assert!(got.err().unwrap().is_filtered());
    }
    #[test]
    fn test_query_ok() {
        let q = MockOkEvaluableQuery(true);
        let got = Query::new(Box::new(q)).select(r#"{"x":1}"#).unwrap();
        let want: Value = from_str(r#"{"x":1}"#).unwrap();
        assert_eq!(want, got);
    }
}
