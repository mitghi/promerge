use std::borrow::Cow;

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum Kind {
    Untyped,
    Counter,
    Gauge,
    Histogram,
    Summary,
}

#[derive(Debug, Clone)]
pub struct Desc<'a> {
    pub kind: Kind,
    pub name: Cow<'a, str>,
    pub help_desc: Option<Cow<'a, str>>,
    pub comment: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone)]
pub struct Value<'a> {
    pub description: Option<Desc<'a>>,
    pub key: String,
    pub pairs: Vec<Vec<(Cow<'a, str>, Cow<'a, str>)>>,
    pub values: Vec<(Cow<'a, str>, Option<Cow<'a, str>>)>,
    pub sum: Option<Cow<'a, str>>,
    pub count: Option<Cow<'a, str>>,
}

impl Kind {
    pub fn from(input: &str) -> Self {
        match input {
            "counter" => Kind::Counter,
            "gauge" => Kind::Gauge,
            "histogram" => Kind::Histogram,
            "summary" => Kind::Summary,
            "untyped" | _ => Kind::Untyped,
        }
    }
}

impl<'a> Desc<'a> {
    pub fn new(name: &'a str, kind: &str) -> Self {
        Self {
            kind: Kind::from(kind),
            name: name.into(),
            help_desc: None,
            comment: None,
        }
    }

    pub fn with_help(name: &'a str, help: &'a str) -> Self {
        Self {
            kind: Kind::Untyped,
            name: name.into(),
            help_desc: Some(help.into()),
            comment: None,
        }
    }

    pub fn with_comment(comment: &'a str) -> Self {
        Self {
            kind: Kind::Untyped,
            name: "".into(),
            comment: Some(comment.into()),
            help_desc: None,
        }
    }
}

impl<'a> Value<'a> {
    pub fn new<S: Into<String>>(key: S) -> Self {
        Self {
            description: None,
            key: key.into(),
            pairs: Vec::new(),
            values: Vec::new(),
            sum: None,
            count: None,
        }
    }

    pub fn push_values<'b>(&mut self, values: &'b [&'a str; 2]) {
        let a = {
            if values[0].is_empty() {
                None
            } else {
                Some(std::borrow::Cow::Borrowed(values[0]))
            }
        };
        if a.is_none() {
            return;
        }
        let b = {
            if values[1].is_empty() {
                None
            } else {
                Some(std::borrow::Cow::Borrowed(values[1]))
            }
        };
        self.values.push((a.unwrap(), b));
    }

    pub fn push_pairs(&mut self, values: &Vec<&'a str>) {
        let mut result: Vec<(Cow<'a, str>, Cow<'a, str>)> = Vec::with_capacity(values.len());
        for slice in values.chunks_exact(2) {
            result.push((slice[0].into(), slice[1].into()));
        }
        self.pairs.push(result);
    }

    pub fn set_sum(&mut self, sum: &'a str) {
        self.sum = Some(std::borrow::Cow::Borrowed(sum));
    }

    pub fn set_count(&mut self, count: &'a str) {
        self.count = Some(std::borrow::Cow::Borrowed(count));
    }
}
