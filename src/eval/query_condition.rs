use crate::error::{Error, ErrorCode, Result};
use crate::query::{EvaluableQueryCondition, QueryCondition};
use crate::util;
use serde_json::value::Value as JSONValue;

impl EvaluableQueryCondition for QueryCondition {
    fn eval(&self, value: &JSONValue) -> Result<bool> {
        match self {
            QueryCondition::Raw(x) => x.eval(value),
            QueryCondition::Not(x) => x.eval(value).map(|b| !b),
            QueryCondition::And(x) => {
                if x.is_empty() {
                    return Err(Error::new(ErrorCode::NoChildren {
                        by: util::type_name(self).to_owned(),
                    }));
                }
                for v in x {
                    let r = v.eval(value);
                    match r {
                        Err(_) | Ok(false) => return r,
                        _ => continue,
                    }
                }
                Ok(true)
            }
            QueryCondition::Or(x) => {
                if x.is_empty() {
                    return Err(Error::new(ErrorCode::NoChildren {
                        by: util::type_name(self).to_owned(),
                    }));
                }
                for v in x {
                    let r = v.eval(value);
                    match r {
                        Err(_) | Ok(true) => return r,
                        _ => continue,
                    }
                }
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::EvaluableQueryPair;

    const NULL: JSONValue = JSONValue::Null;

    struct MockQueryPair(bool);
    impl EvaluableQueryPair for MockQueryPair {
        fn eval(&self, _: &JSONValue) -> Result<bool> {
            Ok(self.0)
        }
    }

    macro_rules! test_or {
        ($name:ident, $v:expr, $want:expr) => {
            #[test]
            fn $name() {
                assert_eq!(QueryCondition::Or($v).eval(&NULL).unwrap(), $want);
            }
        };
    }

    #[test]
    #[should_panic]
    fn or_no_children() {
        QueryCondition::Or(vec![]).eval(&NULL).unwrap();
    }
    test_or!(
        or_one,
        vec![QueryCondition::Raw(Box::new(MockQueryPair(true)))],
        true
    );
    test_or!(
        or_one_not,
        vec![QueryCondition::Raw(Box::new(MockQueryPair(false)))],
        false
    );
    test_or!(
        or_two,
        vec![
            QueryCondition::Raw(Box::new(MockQueryPair(false))),
            QueryCondition::Raw(Box::new(MockQueryPair(true)))
        ],
        true
    );
    test_or!(
        or_two_not,
        vec![
            QueryCondition::Raw(Box::new(MockQueryPair(false))),
            QueryCondition::Raw(Box::new(MockQueryPair(false)))
        ],
        false
    );

    macro_rules! test_and {
        ($name:ident, $v:expr, $want:expr) => {
            #[test]
            fn $name() {
                assert_eq!(QueryCondition::And($v).eval(&NULL).unwrap(), $want);
            }
        };
    }

    #[test]
    #[should_panic]
    fn and_no_children() {
        QueryCondition::And(vec![]).eval(&NULL).unwrap();
    }
    test_and!(
        and_one,
        vec![QueryCondition::Raw(Box::new(MockQueryPair(true)))],
        true
    );
    test_and!(
        and_one_not,
        vec![QueryCondition::Raw(Box::new(MockQueryPair(false)))],
        false
    );
    test_and!(
        and_two,
        vec![
            QueryCondition::Raw(Box::new(MockQueryPair(true))),
            QueryCondition::Raw(Box::new(MockQueryPair(true)))
        ],
        true
    );
    test_and!(
        and_two_not,
        vec![
            QueryCondition::Raw(Box::new(MockQueryPair(true))),
            QueryCondition::Raw(Box::new(MockQueryPair(false)))
        ],
        false
    );

    #[test]
    fn raw_true() {
        assert!(QueryCondition::Raw(Box::new(MockQueryPair(true)))
            .eval(&NULL)
            .unwrap());
    }
    #[test]
    fn raw_false() {
        assert!(!QueryCondition::Raw(Box::new(MockQueryPair(false)))
            .eval(&NULL)
            .unwrap());
    }
    #[test]
    fn not_true() {
        assert!(
            !QueryCondition::Not(Box::new(QueryCondition::Raw(Box::new(MockQueryPair(true)))))
                .eval(&NULL)
                .unwrap()
        );
    }
    #[test]
    fn not_false() {
        assert!(
            QueryCondition::Not(Box::new(QueryCondition::Raw(Box::new(MockQueryPair(
                false
            )))))
            .eval(&NULL)
            .unwrap()
        );
    }
}
