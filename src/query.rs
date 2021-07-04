use crate::error::Result;
use crate::raw_query as raw;
use serde_json::value::Value as JSONValue;
use std::cmp;
use std::convert;
use std::fmt;
use std::vec;

pub trait EvaluableQuery {
    fn eval(&self, value: &JSONValue) -> Result<bool>;
}

/// JSON filter.
pub struct Query {
    pub(crate) query: Box<dyn EvaluableQueryCondition>,
}

impl convert::From<raw::Query> for Query {
    fn from(v: raw::Query) -> Self {
        Query {
            query: Box::new(QueryCondition::from(v.query)),
        }
    }
}

/// Evaluate the query condition.
pub trait EvaluableQueryCondition {
    /// Report whether a given json value satifies the condition.
    fn eval(&self, value: &JSONValue) -> Result<bool>;
}

/// A top level element of [`Query`].
pub enum QueryCondition {
    Raw(Box<dyn EvaluableQueryPair>),
    /// Match if a given condition denies a given value.
    Not(Box<QueryCondition>),
    /// Match if the all given conditions accept a given value.
    And(vec::Vec<QueryCondition>),
    /// Match if at least one of the given conditions accepts a given value.
    Or(vec::Vec<QueryCondition>),
}

impl convert::From<raw::QueryCondition> for QueryCondition {
    fn from(v: raw::QueryCondition) -> Self {
        match v {
            raw::QueryCondition::Raw { pair } => {
                QueryCondition::Raw(Box::new(QueryPair::from(pair)))
            }
            raw::QueryCondition::Not { pair } => QueryCondition::Not(Box::new(Self::from(*pair))),
            raw::QueryCondition::And { pair } => {
                QueryCondition::And(pair.into_iter().map(Self::from).collect())
            }
            raw::QueryCondition::Or { pair } => {
                QueryCondition::Or(pair.into_iter().map(Self::from).collect())
            }
        }
    }
}

/// Evaluate the pair.
pub trait EvaluableQueryPair {
    /// Report whether a given json value satifies the pair.
    fn eval(&self, value: &JSONValue) -> Result<bool>;
}

/// Query target and condition.
pub struct QueryPair {
    /// JSON pointer, Location of data to be tested by `condition`.
    pub(crate) pointer: String,
    pub(crate) condition: Box<dyn EvaluableCondition>,
}

impl convert::From<raw::QueryPair> for QueryPair {
    fn from(v: raw::QueryPair) -> Self {
        QueryPair {
            pointer: v.pointer,
            condition: Box::new(Condition::from(v.condition)),
        }
    }
}

/// Target value of [`Condition`].
#[derive(Debug)]
pub enum Value {
    /// JSON null.
    Null,
    /// JSON boolean.
    Bool(bool),
    /// JSON number as an integer.
    Int(i32),
    /// JSON number as a floating point.
    Float(f64),
    /// JSON string.
    String(String),
}

impl cmp::PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Float(x), Value::Float(y)) => (x.abs() - y.abs()).abs() <= f64::EPSILON,
            (Value::String(x), Value::String(y)) => x == y,
            _ => false,
        }
    }
}

impl cmp::Eq for Value {}

impl convert::From<raw::Value> for Value {
    fn from(v: raw::Value) -> Self {
        match v {
            raw::Value::Null => Value::Null,
            raw::Value::Bool { value } => Value::Bool(value),
            raw::Value::Number { value } => {
                if value.ceil() - value == 0.0 {
                    Value::Int(value as i32)
                } else {
                    Value::Float(value)
                }
            }
            raw::Value::String { value } => Value::String(value),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Null => write!(f, "Null"),
            Value::Bool(x) => write!(f, "Bool({})", x),
            Value::Int(x) => write!(f, "Int({})", x),
            Value::Float(x) => write!(f, "Float({})", x),
            Value::String(x) => write!(f, "String({})", x),
        }
    }
}

/// Evaluate the condition.
pub trait EvaluableCondition {
    /// Report whether a given json value satifies the condition.
    fn eval(&self, value: &Value) -> Result<bool>;
}

/// Condition part of [`QueryPair`].
#[derive(Debug)]
pub enum Condition {
    /// Match if a given value is equal to `Value`.
    Equal(Value),
    /// Match if a given value is greater than `Value`.
    GreaterThan(Value),
    /// Match if a given value is less than `Value`.
    LessThan(Value),
    /// String matching.
    Match(Value, MatchType),
    /// Match if a given condition denies a given value.
    Not(Box<Condition>),
    /// Match if the all given conditions accept a given value.
    And(vec::Vec<Condition>),
    /// Match if at least one of the given conditions accepts a given value.
    Or(vec::Vec<Condition>),
}

impl convert::From<raw::Condition> for Condition {
    fn from(v: raw::Condition) -> Self {
        match v {
            raw::Condition::Equal { value } => Condition::Equal(Value::from(value)),
            raw::Condition::GreaterThan { value } => Condition::GreaterThan(Value::from(value)),
            raw::Condition::LessThan { value } => Condition::LessThan(Value::from(value)),
            raw::Condition::Match { value, mtype } => {
                Condition::Match(Value::from(value), MatchType::from(mtype))
            }
            raw::Condition::Not { value } => Condition::Not(Box::new(Self::from(*value))),
            raw::Condition::And { value } => {
                Condition::And(value.into_iter().map(Self::from).collect())
            }
            raw::Condition::Or { value } => {
                Condition::Or(value.into_iter().map(Self::from).collect())
            }
        }
    }
}

/// Condition of matching string.
#[derive(Debug)]
pub enum MatchType {
    // Substring.
    Contain,
    // Regular expression.
    Regex,
}

impl convert::From<raw::MatchType> for MatchType {
    fn from(v: raw::MatchType) -> Self {
        match v {
            raw::MatchType::Contain => MatchType::Contain,
            raw::MatchType::Regex => MatchType::Regex,
        }
    }
}
