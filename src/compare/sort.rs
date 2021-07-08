use crate::compare::value::PairValue;
use crate::raw_sort::Order;
use serde_json::value::Value;

/// JSON pointer and sort order.
struct PairSetting(String, Order);

/// Sort indexes.
pub(crate) struct PairsListSettings(Vec<PairSetting>);

impl PairsListSettings {
    /// Return a new empty [`PairsListSettings`].
    pub(crate) fn new() -> PairsListSettings {
        PairsListSettings(Vec::new())
    }
    /// Add a new sort index.
    pub(crate) fn add(&mut self, pointer: String, order: Order) {
        self.0.push(PairSetting(pointer, order));
    }
    pub(crate) fn builder(self) -> PairsListBuilder {
        PairsListBuilder::from(self)
    }
}

/// An index in the original list and the sort indexes.
struct Pairs {
    index: usize,
    pairs: Vec<PairValue>,
}

/// [`Pairs`] list generator.
pub(crate) struct PairsListBuilder {
    settings: Vec<PairSetting>,
    list: Vec<Pairs>,
}

impl From<PairsListSettings> for PairsListBuilder {
    fn from(v: PairsListSettings) -> PairsListBuilder {
        PairsListBuilder {
            settings: v.0,
            list: Vec::new(),
        }
    }
}

impl PairsListBuilder {
    /// Generate sort indexed value.
    pub(crate) fn add(&mut self, value: Value) {
        let pairs: Vec<PairValue> = self
            .settings
            .iter()
            .map(|s| {
                let x = value
                    .pointer(&s.0)
                    .map_or_else(|| Value::Null, |x| x.clone()); // invalid pointer as null
                PairValue::from(x)
            })
            .collect::<Vec<_>>();
        let index = self.list.len();
        self.list.push(Pairs { index, pairs });
    }
    pub(crate) fn build(self) -> PairsList {
        PairsList::from(self)
    }
}

impl From<PairsListBuilder> for PairsList {
    fn from(v: PairsListBuilder) -> PairsList {
        PairsList {
            list: v.list,
            settings: v.settings,
        }
    }
}

/// Sortable JSON values.
pub(crate) struct PairsList {
    list: Vec<Pairs>,
    settings: Vec<PairSetting>,
}

impl PairsList {
    fn sort_by(&mut self, index: usize) {
        let PairSetting(_, order) = &self.settings[index];
        if matches!(order, Order::Asc) {
            self.list
                .sort_by(|a: &Pairs, b: &Pairs| a.pairs[index].cmp(&b.pairs[index]));
        } else {
            self.list
                .sort_by(|a: &Pairs, b: &Pairs| b.pairs[index].cmp(&a.pairs[index]));
        }
    }
    /// Sort the values sequentially.
    pub(crate) fn sort(&mut self) {
        for i in 0..self.settings.len() {
            self.sort_by(i);
        }
    }
    /// Read the indexes of the values.
    pub(crate) fn indexes(&self) -> Vec<usize> {
        self.list.iter().map(|x| x.index).collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::from_str;

    fn value(x: &str) -> Value {
        from_str(x).unwrap()
    }

    macro_rules! test_sort {
        ($name:ident, $values:expr, $pointers:expr, $want:expr) => {
            #[test]
            fn $name() {
                let mut s = PairsListSettings::new();
                for p in $pointers {
                    let p: &str = p;
                    s.add(p.to_string(), Order::Asc);
                }
                let mut b = s.builder();
                for v in $values {
                    b.add(value(v));
                }
                let mut a = b.build();
                a.sort();
                assert_eq!($want, a.indexes());
            }
        };
    }

    test_sort!(
        no_sort_indexes,
        vec![r#"{"i":1}"#, r#"{"i":0}"#,],
        vec![],
        vec![0, 1]
    );
    test_sort!(
        sort_by_i,
        vec![
            r#"{"i":1,"s":"snow","opt":10}"#,
            r#"{"i":2,"s":"fire"}"#,
            r#"{"i":0,"s":"abyss","opt":100}"#
        ],
        vec!["/i"],
        vec![2, 0, 1]
    );
    test_sort!(
        sort_by_s,
        vec![
            r#"{"i":1,"s":"snow","opt":10}"#,
            r#"{"i":2,"s":"fire"}"#,
            r#"{"i":0,"s":"abyss","opt":100}"#
        ],
        vec!["/s"],
        vec![2, 1, 0]
    );
    test_sort!(
        sort_by_opt,
        vec![
            r#"{"i":1,"s":"snow","opt":10}"#,
            r#"{"i":2,"s":"fire"}"#,
            r#"{"i":0,"s":"abyss","opt":100}"#
        ],
        vec!["/opt"],
        vec![1, 0, 2]
    );
    test_sort!(
        sort_by_ij,
        vec![
            r#"{"i":0,"j":1,"opt":10}"#,
            r#"{"i":1,"j":1}"#,
            r#"{"i":1,"j":0,"opt":100}"#,
            r#"{"i":0,"j":0}"#
        ],
        vec!["/i", "/j"],
        vec![3, 2, 0, 1]
    );
    test_sort!(
        sort_by_ji,
        vec![
            r#"{"i":0,"j":1,"opt":10}"#,
            r#"{"i":1,"j":1}"#,
            r#"{"i":1,"j":0,"opt":100}"#,
            r#"{"i":0,"j":0}"#
        ],
        vec!["/j", "/i"],
        vec![3, 0, 2, 1]
    );
}
