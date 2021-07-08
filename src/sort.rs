use crate::compare::sort::{PairsListBuilder, PairsListSettings};
use crate::raw_sort::{Order, Sort as RawSort};
use serde_json::value::Value;

/// JSON sorter.
///
/// ```
/// # use jsongrep::raw_sort::Sort as RawSort;
/// # use jsongrep::sort::Sort;
/// # use std::convert::{From, TryFrom};
/// # use serde_json::from_str;
///
/// /// sort by key "i" desc
/// const js: &str = r#"{"sort":[{"p":"/i","ord":"desc"}]}"#;
/// let values: Vec<&str> = vec![
///   r#"{"i":10,"s":"bellatrix"}"#,
///   r#"{"i":5,"s":"cassandra"}"#,
///   r#"{"i":20,"s":"alexander"}"#,
///   r#"{"i":0,"s":"dimitrius"}"#,
/// ];
/// let mut st = RawSort::try_from(js).map(Sort::from).unwrap();
/// for v in values.iter().map(|x| from_str(x).unwrap()) {
///   st.add(v);
/// }
/// assert_eq!(vec![2, 0, 1, 3], st.sorted_indexes());
/// ```
pub struct Sort {
    builder: PairsListBuilder,
}

impl From<RawSort> for Sort {
    fn from(v: RawSort) -> Sort {
        let mut s = PairsListSettings::new();
        for p in v.sort {
            s.add(p.pointer, p.order.unwrap_or(Order::Asc));
        }
        let builder = s.builder();
        Sort { builder }
    }
}

impl Sort {
    /// Add a value to be sorted.
    pub fn add(&mut self, value: Value) {
        self.builder.add(value);
    }
    /// Return the indexes of the sorted values.
    pub fn sorted_indexes(self) -> Vec<usize> {
        let mut list = self.builder.build();
        list.sort();
        list.indexes()
    }
}
