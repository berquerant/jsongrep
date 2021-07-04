use crate::error;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::convert;
use std::vec;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Value {
    #[serde(rename = "null")]
    Null,
    #[serde(rename = "bool")]
    Bool { value: bool },
    #[serde(rename = "number")]
    Number { value: f64 },
    #[serde(rename = "string")]
    String { value: String },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Condition {
    #[serde(rename = "eq")]
    Equal { value: Value },
    #[serde(rename = "gt")]
    GreaterThan { value: Value },
    #[serde(rename = "lt")]
    LessThan { value: Value },
    #[serde(rename = "match")]
    Match { value: Value, mtype: MatchType },
    #[serde(rename = "not")]
    Not { value: Box<Condition> },
    #[serde(rename = "and")]
    And { value: vec::Vec<Condition> },
    #[serde(rename = "or")]
    Or { value: vec::Vec<Condition> },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum MatchType {
    #[serde(rename = "contain")]
    Contain,
    #[serde(rename = "regex")]
    Regex,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QueryPair {
    #[serde(rename = "p")]
    pub pointer: String,
    #[serde(rename = "cond")]
    pub condition: Condition,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum QueryCondition {
    #[serde(rename = "raw")]
    Raw { pair: QueryPair },
    #[serde(rename = "not")]
    Not { pair: Box<QueryCondition> },
    #[serde(rename = "and")]
    And { pair: vec::Vec<QueryCondition> },
    #[serde(rename = "or")]
    Or { pair: vec::Vec<QueryCondition> },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Query {
    pub query: QueryCondition,
}

impl convert::TryFrom<&str> for Query {
    type Error = error::Error;
    fn try_from(v: &str) -> Result<Self, Self::Error> {
        from_str(v).map_err(|x| error::Error::new(error::ErrorCode::Json(x)))
    }
}
