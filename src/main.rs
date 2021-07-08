use jsongrep::error::{Error, ErrorCode, Result};
use jsongrep::query::Query;
use jsongrep::raw_query::Query as RawQuery;
use jsongrep::raw_sort::Sort as RawSort;
use jsongrep::select::Query as Selector;
use jsongrep::sort::Sort;
use serde_json::from_str;
use std::convert::TryFrom;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args().validate().unwrap();
    let q = opt.get_selector().unwrap();
    let mut s = opt.get_sort().unwrap();
    let use_sort = s.is_some();
    let mut lines: Vec<String> = Vec::new();
    let stdin = io::stdin();
    for (n, l) in stdin.lock().lines().enumerate() {
        let line = l.unwrap();
        match q.select(&line) {
            Ok(_) => {
                if use_sort {
                    match s.as_mut() {
                        Some(x) => {
                            let v = from_str(&line).unwrap();
                            x.add(v);
                            lines.push(line);
                        }
                        _ => unreachable!(),
                    }
                } else {
                    println!("{}", line);
                }
            }
            Err(e) if !e.is_filtered() => eprintln!("line {}: {}", n + 1, e),
            _ => continue,
        }
    }
    if !lines.is_empty() {
        s.unwrap().sorted_indexes().iter().for_each(|i| {
            println!("{}", lines[*i]);
        });
    }
}

#[derive(Debug, Clone, StructOpt)]
/// Grep json.
///
/// Grep json from stdin by query.
#[structopt(name = "jsongrep")]
struct Opt {
    /// Specify query on command line.
    ///
    /// Grep `/s` value by regex `[sS]irius`
    ///
    ///
    /// {
    ///   "query": {
    ///     "type": "raw",
    ///     "pair": {
    ///       "p": "/s",
    ///       "cond": {
    ///         "type": "match",
    ///         "mtype": "regex",
    ///         "value": {
    ///           "type": "string",
    ///           "value": "[sS]irius"
    ///         }
    ///       }
    ///     }
    ///   }
    /// }
    ///
    /// It accepts json like {"s":"sirius"}, {"s":"Sirius"},
    /// writes the json to stdout.
    ///
    /// It does not accept string like {"s":"spica"}, {"s":null}, {"a":"sirius"}, and not json string.
    /// If the schema is ok then no output.
    /// Otherwise error is written to stderr.
    ///
    /// See [`jsongrep::select::Query`].
    #[structopt(short = "r", long = "raw_query")]
    raw_query: Option<String>,
    /// Specify query by file.
    #[structopt(short = "q", long = "query_file")]
    query: Option<PathBuf>,
    /// Specify sort on command line.
    ///
    /// Sort `/i` value desc
    ///
    /// {
    ///   "sort": [
    ///      {
    ///        "p": "/i",
    ///        "ord": "desc"
    ///      }
    ///   ]
    /// }
    ///
    /// Basic sort order: null < array < object < bool < number < string.
    /// Ignore inner values of array and object.
    ///
    /// If a pointed value does not exist, the row is sorted as null.
    #[structopt(short = "k", long = "raw_sort")]
    raw_sort: Option<String>,
    /// Specify sort by file.
    #[structopt(short = "s", long = "sort")]
    sort: Option<PathBuf>,
}

impl Opt {
    fn validate(&self) -> Result<Self> {
        match (&self.raw_query, &self.query) {
            (Some(_), Some(_)) => Err(Error::new(ErrorCode::InvalidOption(
                "query and raw_query are exclusive".to_owned(),
            ))),
            _ => Ok(self.clone()),
        }
    }
    fn get_raw_sort(&self) -> Option<Result<RawSort>> {
        let k = self
            .raw_sort
            .as_ref()
            .map(|x| RawSort::try_from(&x as &str));
        let s = self.sort.as_ref().map(|x| {
            let mut f = File::open(x).map_err(|x| Error::new(ErrorCode::Io(x)))?;
            let mut buf = String::new();
            f.read_to_string(&mut buf)
                .map_err(|x| Error::new(ErrorCode::Io(x)))?;
            RawSort::try_from(&buf as &str)
        });
        k.xor(s)
    }
    fn get_sort(&self) -> Result<Option<Sort>> {
        self.get_raw_sort().map(|x| x.map(Sort::from)).transpose()
    }
    fn get_raw_query(&self) -> Option<Result<RawQuery>> {
        let r = self
            .raw_query
            .as_ref()
            .map(|x| RawQuery::try_from(&x as &str));
        let q = self.query.as_ref().map(|x| {
            let mut f = File::open(x).map_err(|x| Error::new(ErrorCode::Io(x)))?;
            let mut buf = String::new();
            f.read_to_string(&mut buf)
                .map_err(|x| Error::new(ErrorCode::Io(x)))?;
            RawQuery::try_from(&buf as &str)
        });
        r.xor(q)
    }
    fn get_query(&self) -> Option<Result<Query>> {
        self.get_raw_query().map(|x| x.map(Query::from))
    }
    fn get_selector(&self) -> Result<Selector> {
        match self.get_query() {
            Some(Ok(q)) => Ok(Selector::new(Box::new(q))),
            Some(Err(x)) => Err(x),
            None => Ok(Selector::all()),
        }
    }
}
