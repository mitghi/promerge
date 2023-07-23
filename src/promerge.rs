use std::borrow::Cow;

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use crate::parser;

#[derive(Debug, Clone)]
pub enum Kind {
    Untyped,
    Counter,
    Gauge,
    Histogram,
    Summary,
}

#[derive(Debug, Clone)]
pub struct Context<'a> {
    input: Cow<'a, str>,
    prefix: Option<String>,
    result: Option<Vec<Value<'a>>>,
}

impl<'a> Context<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: std::borrow::Cow::Borrowed(input),
            prefix: None,
            result: None,
        }
    }

    pub fn with_prefix<S: Into<String>>(input: &'a str, prefix: S) -> Self {
        Self {
            input: std::borrow::Cow::Borrowed(input),
            prefix: Some(prefix.into()),
            result: None,
        }
    }

    pub fn run(&mut self) -> Result<String, pest::error::Error<crate::parser::Rule>> {
        let mut result = parser::parse(self.input.as_ref())?;
        let mut output: String = String::new();
        let prefix: String = if let Some(p) = &self.prefix {
            p.to_owned()
        } else {
            "".to_string()
        };
        for v in &mut result {
            v.prefix = Some(prefix.clone());
            output.push_str(v.to_string().as_str());
        }
        Ok(output)
    }
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
    pub prefix: Option<String>,
    pub description: Option<Desc<'a>>,
    pub key: String,
    pub pairs: Vec<Vec<(Cow<'a, str>, Cow<'a, str>)>>,
    pub values: Vec<(Cow<'a, str>, Option<Cow<'a, str>>)>,
    pub sum: Option<Cow<'a, str>>,
    pub count: Option<Cow<'a, str>>,
}

impl<'a> Value<'a> {
    fn to_string(&self) -> String {
        use std::fmt::Write;
        let mut buffer: String = String::new();
        let lenpairs = self.pairs.len();
        let lenvalues = self.values.len();
        let prefix = if let Some(p) = &self.prefix {
            p.as_ref()
        } else {
            ""
        };
        let is_histogram = if let Some(k) = &self.description {
            match k.kind {
                Kind::Histogram => true,
                _ => false,
            }
        } else {
            false
        };
        let key = if let Some(k) = &self.description {
            let nk = k.name.as_ref();
            if nk.is_empty() {
                self.key.as_ref()
            } else {
                nk
            }
        } else {
            self.key.as_ref()
        };

        if lenpairs == 0 && lenvalues > 0 {
            for v in &self.values {
                let lval = v.0.as_ref();
                if let Some(rval) = &v.1 {
                    writeln!(buffer, "{}{} {} {}", &prefix, &key, lval, rval.as_ref()).unwrap();
                } else {
                    writeln!(buffer, "{}{} {}", &prefix, &key, lval).unwrap();
                }
            }
        } else {
            let mut had_tuple = false;
            for (i, p) in self.pairs.iter().enumerate() {
                let mut pbuff: String = String::new();
                let lenp = p.len();
                for (i, tuple) in p.iter().enumerate() {
                    if tuple.0.as_ref().is_empty() {
                        continue;
                    }
                    had_tuple = true;
                    if i == (lenp - 1) {
                        write!(pbuff, "{}=\"{}\"", tuple.0.as_ref(), tuple.1.as_ref()).unwrap();
                    } else {
                        write!(pbuff, "{}=\"{}\",", tuple.0.as_ref(), tuple.1.as_ref()).unwrap();
                    }
                }

                let v = &self.values[i];
                let lval = v.0.as_ref();
                if let Some(rval) = &v.1 {
                    if !had_tuple {
                        writeln!(buffer, "{}{} {} {}", &prefix, &key, lval, rval.as_ref()).unwrap();
                    } else {
                        if is_histogram {
                            writeln!(
                                buffer,
                                "{}{}_bucket{{{}}} {} {}",
                                &prefix,
                                &key,
                                pbuff,
                                lval,
                                rval.as_ref()
                            )
                            .unwrap();
                        } else {
                            writeln!(
                                buffer,
                                "{}{}{{{}}} {} {}",
                                &prefix,
                                &key,
                                pbuff,
                                lval,
                                rval.as_ref()
                            )
                            .unwrap();
                        }
                    }
                } else {
                    if !had_tuple {
                        writeln!(buffer, "{}{} {}", &prefix, &key, lval).unwrap();
                    } else {
                        if is_histogram {
                            writeln!(buffer, "{}{}_bucket{{{}}} {}", &prefix, &key, pbuff, lval)
                                .unwrap();
                        } else {
                            writeln!(buffer, "{}{}{{{}}} {}", &prefix, &key, pbuff, lval).unwrap();
                        }
                    }
                }
            }
        }
        if let Some(sum) = &self.sum {
            writeln!(buffer, "{}{}_sum {}", &prefix, &key, &sum).unwrap();
        }
        if let Some(count) = &self.count {
            writeln!(buffer, "{}{}_count {}", &prefix, &key, &count).unwrap();
        }
        buffer
    }
}

impl<'a> Desc<'a> {
    fn to_string(&self, prefix: &Option<String>) -> String {
        let mut buffer: String = String::new();
        let kind = self.kind.to_string();
        let prefix = if let Some(p) = &prefix { &p } else { "" };
        use std::fmt::Write;
        if let Some(comment) = &self.comment {
            writeln!(buffer, "# {}", comment.as_ref()).unwrap();
        }

        if let Some(help_desc) = &self.help_desc {
            writeln!(
                buffer,
                "# HELP {} {}",
                format!("{}{}", &prefix, self.name.as_ref()),
                help_desc.as_ref()
            )
            .unwrap();
        }
        match &self.kind {
            Kind::Untyped => {}
            _ => {
                writeln!(
                    buffer,
                    "# TYPE {} {}",
                    format!("{}{}", &prefix, self.name.as_ref()),
                    kind
                )
                .unwrap();
            }
        };

        buffer
    }
}

impl<'a> std::fmt::Display for Value<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        if let Some(desc) = &self.description {
            write!(f, "{}", desc.to_string(&self.prefix)).unwrap();
        }
        writeln!(f, "{}", self.to_string()).unwrap();
        Ok(())
    }
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

    fn to_string(&self) -> &str {
        match &self {
            Kind::Counter => "counter",
            Kind::Gauge => "gauge",
            Kind::Histogram => "histogram",
            Kind::Summary => "summary",
            Kind::Untyped => "untyped",
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
            prefix: None,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_context() {
        let input = r#"# Finally a summary, which has a complex representation, too:
# HELP rpc_duration_seconds A summary of the RPC duration in seconds.
# TYPE rpc_duration_seconds summary
rpc_duration_seconds{quantile="0.01"} 3102
rpc_duration_seconds{quantile="0.05"} 3272
rpc_duration_seconds{quantile="0.5"} 4773
rpc_duration_seconds{quantile="0.9"} 9001
rpc_duration_seconds{quantile="0.99"} 76656
rpc_duration_seconds_sum 1.7560473e+07
rpc_duration_seconds_count 2693
"#;
        let mut ctx = Context::with_prefix(&input, "prefix_");
        let output = ctx.run();
        assert_eq!(output.is_err(), false);
        println!("{}", output.unwrap());
    }
}
