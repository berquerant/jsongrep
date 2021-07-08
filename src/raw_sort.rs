use crate::error;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::convert;
use std::vec;

#[derive(Debug, Deserialize, Serialize)]
pub enum Order {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SortPair {
    #[serde(rename = "p")]
    pub pointer: String,
    #[serde(rename = "ord")]
    pub order: Option<Order>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sort {
    pub sort: vec::Vec<SortPair>,
}

impl convert::TryFrom<&str> for Sort {
    type Error = error::Error;
    fn try_from(v: &str) -> Result<Self, Self::Error> {
        from_str(v).map_err(|x| error::Error::new(error::ErrorCode::Json(x)))
    }
}
