use crate::error::{Error, ErrorCode, Result};
use crate::eval::matcher::Matcher;
use crate::query::{Condition, EvaluableCondition, MatchType, Value};
use crate::util;
use std::cmp;

impl EvaluableCondition for Condition {
    fn eval(&self, value: &Value) -> Result<bool> {
        match self {
            Condition::Equal(_) => self.equal(value),
            Condition::GreaterThan(_) => self.greater_than(value),
            Condition::LessThan(_) => self.less_than(value),
            Condition::Not(_) => self.not(value),
            Condition::And(_) => self.and(value),
            Condition::Or(_) => self.or(value),
            Condition::Match(_, _) => self.test(value),
        }
    }
}

impl Condition {
    fn type_name<T>(v: T) -> String {
        util::type_name(v).to_string()
    }
    fn test(&self, r: &Value) -> Result<bool> {
        if let Condition::Match(l, t) = self {
            match (l, t, r) {
                (Value::String(x), _, Value::String(y)) => match t {
                    MatchType::Contain => Matcher::Raw(x).test(y),
                    MatchType::Regex => Matcher::Regex(x).test(y),
                },
                _ => Err(Error::new(ErrorCode::MatcherTypeMismatch {
                    matcher_type: format!("{:?}", t),
                    matcher_value: format!("{}", l),
                    target: format!("{}", r),
                    by: Self::type_name(self),
                })),
            }
        } else {
            Err(Error::unreachable())
        }
    }
    fn equal(&self, r: &Value) -> Result<bool> {
        if let Condition::Equal(l) = self {
            match (l, r) {
                (Value::Null, Value::Null) => Ok(true),
                (Value::Bool(x), Value::Bool(y)) => Ok(*x == *y),
                (Value::Int(x), Value::Int(y)) => Ok(*x == *y),
                (Value::Float(x), Value::Float(y)) => {
                    Ok(((*x).abs() - (*y).abs()).abs() <= f64::EPSILON)
                }
                (Value::String(x), Value::String(y)) => Ok(*x == *y),
                _ => Err(Error::new(ErrorCode::TypeMismatch {
                    want: Self::type_name(l),
                    got: format!("{}", r),
                    by: Self::type_name(self),
                })),
            }
        } else {
            Err(Error::unreachable())
        }
    }
    fn greater_than(&self, r: &Value) -> Result<bool> {
        if let Condition::GreaterThan(l) = self {
            match (l, r) {
                (Value::Bool(x), Value::Bool(y)) => Ok(!(*x) & *y),
                (Value::Int(x), Value::Int(y)) => Ok(*x < *y),
                (Value::Float(x), Value::Float(y)) => Ok(*x < *y),
                (Value::String(x), Value::String(y)) => Ok(x.cmp(y) == cmp::Ordering::Less),
                _ => Err(Error::new(ErrorCode::TypeMismatch {
                    want: Self::type_name(l),
                    got: format!("{}", r),
                    by: Self::type_name(self),
                })),
            }
        } else {
            Err(Error::unreachable())
        }
    }
    fn less_than(&self, r: &Value) -> Result<bool> {
        if let Condition::LessThan(l) = self {
            match (l, r) {
                (Value::Bool(x), Value::Bool(y)) => Ok(*x & !(*y)),
                (Value::Int(x), Value::Int(y)) => Ok(*x > *y),
                (Value::Float(x), Value::Float(y)) => Ok(*x > *y),
                (Value::String(x), Value::String(y)) => Ok(x.cmp(y) == cmp::Ordering::Greater),
                _ => Err(Error::new(ErrorCode::TypeMismatch {
                    want: Self::type_name(l),
                    got: format!("{}", r),
                    by: Self::type_name(self),
                })),
            }
        } else {
            Err(Error::unreachable())
        }
    }
    fn not(&self, r: &Value) -> Result<bool> {
        if let Condition::Not(l) = self {
            l.eval(r).map(|x| !x)
        } else {
            Err(Error::unreachable())
        }
    }
    fn and(&self, r: &Value) -> Result<bool> {
        if let Condition::And(l) = self {
            if l.is_empty() {
                return Err(Error::new(ErrorCode::NoChildren {
                    by: Self::type_name(self),
                }));
            }
            for v in l {
                let x = v.eval(r);
                match x {
                    Err(_) | Ok(false) => return x,
                    _ => continue,
                }
            }
            Ok(true)
        } else {
            Err(Error::unreachable())
        }
    }
    fn or(&self, r: &Value) -> Result<bool> {
        if let Condition::Or(l) = self {
            if l.is_empty() {
                return Err(Error::new(ErrorCode::NoChildren {
                    by: Self::type_name(self),
                }));
            }
            for v in l {
                let x = v.eval(r);
                match x {
                    Err(_) | Ok(true) => return x,
                    _ => continue,
                }
            }
            Ok(false)
        } else {
            Err(Error::unreachable())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! test_or {
        ($name:ident, $left:expr, $right:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got = Condition::Or($left).or(&$right).unwrap();
                assert_eq!($want, got);
            }
        };
    }

    macro_rules! test_or_panic {
        ($name:ident, $left:expr, $right:expr) => {
            #[test]
            #[should_panic]
            fn $name() {
                Condition::Or($left).or(&$right).unwrap();
            }
        };
    }

    test_or_panic!(or_no_children, vec![], Value::Null);
    test_or_panic!(
        or_one_type_diff,
        vec![Condition::Equal(Value::Bool(true))],
        Value::Null
    );
    test_or_panic!(
        or_two_type_diff,
        vec![
            Condition::Equal(Value::Bool(true)),
            Condition::Equal(Value::Null)
        ],
        Value::Null
    );

    test_or!(
        or_one,
        vec![Condition::Equal(Value::Null)],
        Value::Null,
        true
    );
    test_or!(
        or_one_not,
        vec![Condition::Equal(Value::Bool(true))],
        Value::Bool(false),
        false
    );
    test_or!(
        or_one_two,
        vec![
            Condition::Equal(Value::Bool(false)),
            Condition::Equal(Value::Bool(true))
        ],
        Value::Bool(true),
        true
    );
    test_or!(
        or_one_two_not,
        vec![
            Condition::Equal(Value::Bool(false)),
            Condition::Equal(Value::Bool(false))
        ],
        Value::Bool(true),
        false
    );

    macro_rules! test_and {
        ($name:ident, $left:expr, $right:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got = Condition::And($left).and(&$right).unwrap();
                assert_eq!($want, got);
            }
        };
    }

    macro_rules! test_and_panic {
        ($name:ident, $left:expr, $right:expr) => {
            #[test]
            #[should_panic]
            fn $name() {
                Condition::And($left).and(&$right).unwrap();
            }
        };
    }

    test_and_panic!(and_no_children, vec![], Value::Null);
    test_and_panic!(
        and_one_type_diff,
        vec![Condition::Equal(Value::Null)],
        Value::Bool(true)
    );
    test_and_panic!(
        and_two_type_diff,
        vec![
            Condition::Equal(Value::Bool(true)),
            Condition::Equal(Value::Null)
        ],
        Value::Bool(true)
    );

    test_and!(
        and_one,
        vec![Condition::Equal(Value::Null)],
        Value::Null,
        true
    );
    test_and!(
        and_one_not,
        vec![Condition::Equal(Value::Bool(true))],
        Value::Bool(false),
        false
    );
    test_and!(
        and_two,
        vec![Condition::Equal(Value::Null), Condition::Equal(Value::Null)],
        Value::Null,
        true
    );
    test_and!(
        and_two_not,
        vec![
            Condition::Equal(Value::Bool(true)),
            Condition::Equal(Value::Bool(false))
        ],
        Value::Bool(true),
        false
    );

    macro_rules! test_not {
        ($name:ident, $left:expr, $right:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got = Condition::Not(Box::new($left)).not(&$right).unwrap();
                assert_eq!($want, got);
            }
        };
    }

    #[test]
    #[should_panic]
    fn not_type_diff() {
        Condition::Not(Box::new(Condition::Equal(Value::Bool(true))))
            .not(&Value::Null)
            .unwrap();
    }

    test_not!(not_true, Condition::Equal(Value::Null), Value::Null, false);
    test_not!(
        not_false,
        Condition::Equal(Value::Bool(false)),
        Value::Bool(true),
        true
    );

    macro_rules! test_less_than {
        ($name:ident, $left:expr, $right:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got = Condition::LessThan($left).less_than(&$right).unwrap();
                assert_eq!($want, got);
            }
        };
    }

    #[test]
    #[should_panic]
    fn lt_type_diff() {
        Condition::LessThan(Value::Null)
            .less_than(&Value::Bool(true))
            .unwrap();
    }

    test_less_than!(lt_bool_not, Value::Bool(true), Value::Bool(false), true);
    test_less_than!(lt_bool, Value::Bool(false), Value::Bool(true), false);
    test_less_than!(lt_int_not, Value::Int(1), Value::Int(0), true);
    test_less_than!(lt_int, Value::Int(1), Value::Int(2), false);
    test_less_than!(lt_float_not, Value::Float(1.1), Value::Float(1.0), true);
    test_less_than!(lt_float, Value::Float(1.1), Value::Float(1.2), false);
    test_less_than!(
        lt_string_not,
        Value::String("nebula".to_owned()),
        Value::String("galaxy".to_owned()),
        true
    );
    test_less_than!(
        lt_string,
        Value::String("nebula".to_owned()),
        Value::String("quasar".to_owned()),
        false
    );

    macro_rules! test_greater_than {
        ($name:ident, $left:expr, $right:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got = Condition::GreaterThan($left).greater_than(&$right).unwrap();
                assert_eq!($want, got);
            }
        };
    }

    #[test]
    #[should_panic]
    fn gt_type_diff() {
        Condition::GreaterThan(Value::Null)
            .greater_than(&Value::Bool(true))
            .unwrap();
    }

    test_greater_than!(gt_bool, Value::Bool(true), Value::Bool(false), false);
    test_greater_than!(gt_bool_not, Value::Bool(false), Value::Bool(true), true);
    test_greater_than!(gt_int, Value::Int(1), Value::Int(0), false);
    test_greater_than!(gt_int_not, Value::Int(1), Value::Int(2), true);
    test_greater_than!(gt_float, Value::Float(1.1), Value::Float(1.0), false);
    test_greater_than!(gt_float_not, Value::Float(1.1), Value::Float(1.2), true);
    test_greater_than!(
        gt_string,
        Value::String("nebula".to_owned()),
        Value::String("galaxy".to_owned()),
        false
    );
    test_greater_than!(
        gt_string_not,
        Value::String("nebula".to_owned()),
        Value::String("quasar".to_owned()),
        true
    );

    macro_rules! test_equal {
        ($name:ident, $left:expr, $right:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got = Condition::Equal($left).equal(&$right).unwrap();
                assert_eq!($want, got);
            }
        };
    }

    #[test]
    #[should_panic]
    fn eq_type_diff() {
        Condition::Equal(Value::Null)
            .equal(&Value::Bool(true))
            .unwrap();
    }

    test_equal!(eq_null, Value::Null, Value::Null, true);
    test_equal!(eq_bool, Value::Bool(true), Value::Bool(true), true);
    test_equal!(eq_bool_diff, Value::Bool(true), Value::Bool(false), false);
    test_equal!(eq_int, Value::Int(1), Value::Int(1), true);
    test_equal!(eq_int_diff, Value::Int(1), Value::Int(2), false);
    test_equal!(eq_float, Value::Float(1.0), Value::Float(1.0), true);
    test_equal!(eq_float_diff, Value::Float(1.0), Value::Float(1.1), false);
    test_equal!(
        eq_string,
        Value::String("black".to_owned()),
        Value::String("black".to_owned()),
        true
    );
    test_equal!(
        eq_string_diff,
        Value::String("black".to_owned()),
        Value::String("white".to_owned()),
        false
    );
}
