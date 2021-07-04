use crate::error;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::convert;
use std::sync::Mutex;

static REGEX_CACHE: Lazy<Mutex<HashMap<String, Regex>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Select string matched with specified pattern.
pub(crate) enum Matcher<'a> {
    Raw(&'a str),
    Regex(&'a str),
}

impl Matcher<'_> {
    /// Return `true` if `value` matched with specified pattern.
    pub(crate) fn test(&self, value: impl convert::Into<String>) -> error::Result<bool> {
        match self {
            Self::Raw(_) => self.test_raw(value),
            Self::Regex(_) => self.test_regex(value),
        }
    }
    fn test_raw(&self, value: impl convert::Into<String>) -> error::Result<bool> {
        if let Matcher::Raw(l) = self {
            Ok(value.into().contains(l))
        } else {
            Err(error::Error::unreachable())
        }
    }
    fn test_regex(&self, value: impl convert::Into<String>) -> error::Result<bool> {
        if let Matcher::Regex(l) = self {
            Self::_test_regex(l, value)
        } else {
            Err(error::Error::unreachable())
        }
    }
    fn _test_regex(pattern: &str, value: impl convert::Into<String>) -> error::Result<bool> {
        let mut l = REGEX_CACHE.lock().unwrap();
        match l.get(pattern) {
            Some(x) => Ok(x.is_match(&value.into())),
            _ => {
                let x = Regex::new(pattern).map_err(|_| {
                    error::Error::new(error::ErrorCode::InvalidRegex(pattern.to_owned()))
                })?;
                let b = x.is_match(&value.into());
                l.insert(pattern.to_owned(), x);
                Ok(b)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_raw {
        ($name:ident, $pattern:expr, $value:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got = Matcher::Raw($pattern).test($value).unwrap();
                assert_eq!($want, got);
            }
        };
    }

    test_raw!(raw_eq, "dwarf", "dwarf", true);
    test_raw!(raw_not, "dwarf", "giant", false);
    test_raw!(raw_contain, "dwarf", "white dwarf", true);

    macro_rules! test_regex {
        ($name:ident, $pattern:expr, $value:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got = Matcher::Regex($pattern).test($value).unwrap();
                assert_eq!($want, got);
            }
        };
    }

    test_regex!(regex_eq, "dwarf", "dwarf", true);
    test_regex!(regex_match1, r"s.*e", "slice", true);
    test_regex!(regex_match2, r"s.*e", "slice ice", true);
    test_regex!(regex_not, r"^dwarf", "brown dwarf", false);
}
