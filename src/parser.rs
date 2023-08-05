//! Module containing parser for promerge.

use crate::promerge::{Desc, Kind, Segment, Value};

use crate::*;

#[derive(Parse)]
#[grammar = "./grammar.pest"]
pub struct ExpressionParser;

fn parse_gencom<'i, R>(
    node: &mut pest::iterators::Pairs<'i, R>,
    desc: Option<Desc<'i>>,
) -> Option<Desc<'i>>
where
    R: pest::RuleType,
{
    let comment = node.next().unwrap().as_span().as_str();
    return desc.map_or(
        Some(Desc::with_comment(comment)),
        move |mut v: Desc<'_>| -> Option<Desc<'_>> {
            v.comment = Some(comment.into());
            Some(v)
        },
    );
}

fn parse_helpexpr<'i, R: pest::RuleType>(
    node: pest::iterators::Pairs<'i, R>,
    desc: Option<Desc<'i>>,
) -> Option<Desc<'i>> {
    let result: Vec<&str> = node.map(|v| v.as_span().as_str()).collect();
    return desc.map_or(
        Some(Desc::with_help(result[0], result[1])),
        move |mut v: Desc<'_>| -> Option<Desc<'_>> {
            v.name = result[0].into();
            v.help_desc = Some(result[1].into());
            Some(v)
        },
    );
}

fn parse_typexpr<'i, R: pest::RuleType>(
    node: pest::iterators::Pairs<'i, R>,
    desc: Option<Desc<'i>>,
) -> Option<Desc<'i>> {
    let result: Vec<&str> = node.map(|v| v.as_span().as_str()).collect();
    return desc.map_or(
        Some(Desc::new(result[0], result[1])),
        move |mut v: Desc<'_>| -> Option<Desc<'_>> {
            v.name = result[0].into();
            v.kind = Kind::from(result[1]);
            Some(v)
        },
    );
}

pub(crate) fn parse<'a>(input: &'a str) -> Result<Vec<Value<'_>>, pest::error::Error<Rule>> {
    let pairs = ExpressionParser::parse(Rule::statement, input);
    if pairs.is_err() {
        return Err(pairs.err().unwrap());
    }
    let mut output: Vec<Value<'_>> = Vec::new();
    let root = pairs.unwrap().next().unwrap();
    for token in root.into_inner() {
        match token.as_rule() {
            Rule::block => {
                let inner = token.clone().into_inner();
                let mut desc: Option<Desc> = None;
                let mut node: Option<Value> = None;
                let mut pairs: Vec<&str> = Vec::new();
                for value in inner {
                    match value.as_rule() {
                        Rule::genericomment => {
                            desc = parse_gencom(value.clone().into_inner().by_ref(), desc);
                        }
                        Rule::typexpr => {
                            desc = parse_typexpr(value.clone().into_inner(), desc);
                        }
                        Rule::helpexpr => {
                            desc = parse_helpexpr(value.clone().into_inner(), desc);
                        }
                        Rule::promstmt => {
                            let mut had_pairs = false;
                            let mut nums: [&str; 2] = [""; 2];
                            let mut nidx = 0;
                            let mut should_skip_nums = false;
                            let mut key: &str = "";
                            for v in value.clone().into_inner() {
                                match &v.as_rule() {
                                    Rule::key => {
                                        let name = v.as_span().as_str();
                                        key = &name;
                                        node = node.map_or(
                                            Some(Value::new(name)),
                                            |n: Value<'_>| -> Option<Value<'_>> { Some(n) },
                                        );
                                    }
                                    Rule::NaN | Rule::number | Rule::posInf | Rule::negInf => {
                                        let is_sum = key.ends_with("_sum");
                                        let is_count = key.ends_with("_count");
                                        if is_sum || is_count {
                                            should_skip_nums = true;
                                            let mut n = node.unwrap();
                                            let content = v.as_span().as_str();
                                            if is_sum {
                                                n.sum = n.sum.map_or(
                                                    {
                                                        let mut v = Segment::default();
                                                        v.set_value(content);
                                                        Some(v)
                                                    },
                                                    |mut v: Segment<'_>| {
                                                        v.set_value(content);
                                                        Some(v)
                                                    },
                                                );
                                            }
                                            if is_count {
                                                n.count = n.count.map_or(
                                                    {
                                                        let mut v = Segment::default();
                                                        v.set_value(content);
                                                        Some(v)
                                                    },
                                                    |mut v: Segment<'_>| {
                                                        v.set_value(content);
                                                        Some(v)
                                                    },
                                                );
                                            }
                                            node = Some(n);
                                            continue;
                                        }
                                        nums[nidx] = v.as_span().as_str();
                                        nidx += 1;
                                    }
                                    Rule::pairs => {
                                        let is_sum = key.ends_with("_sum");
                                        let is_count = key.ends_with("_count");
                                        had_pairs = (is_sum || is_count) == false;
                                        for p in v.into_inner() {
                                            let mut inner = p.into_inner();
                                            let key = inner.next().unwrap().as_span().as_str();
                                            let value = inner
                                                .next()
                                                .unwrap()
                                                .into_inner()
                                                .next()
                                                .unwrap()
                                                .as_span()
                                                .as_str();
                                            if had_pairs {
                                                pairs.push(key);
                                                pairs.push(value);
                                            } else {
                                                let mut n = node.unwrap();
                                                if is_sum {
                                                    n.sum = n.sum.map_or(
                                                        {
                                                            let mut v = Segment::default();
                                                            v.push_pairs(&[&key, &value]);
                                                            Some(v)
                                                        },
                                                        |mut v: Segment<'_>| {
                                                            v.push_pairs(&[&key, &value]);
                                                            Some(v)
                                                        },
                                                    );
                                                }
                                                if is_count {
                                                    n.count = n.count.map_or(
                                                        {
                                                            let mut v = Segment::default();
                                                            v.push_pairs(&[&key, &value]);
                                                            Some(v)
                                                        },
                                                        |mut v: Segment<'_>| {
                                                            v.push_pairs(&[&key, &value]);
                                                            Some(v)
                                                        },
                                                    );
                                                }
                                                node = Some(n);
                                            }
                                        }
                                    }
                                    _ => {
                                        todo!("not implemented");
                                    }
                                }
                            }
                            let mut n = node.unwrap();
                            n.description = desc.clone();
                            if !should_skip_nums {
                                n.push_values(&nums);
                            }
                            if !had_pairs && !should_skip_nums {
                                pairs.push("");
                                pairs.push("");
                            }
                            if pairs.len() > 0 {
                                n.push_pairs(&pairs);
                            }
                            pairs.clear();
                            nums[0] = "";
                            nums[1] = "";
                            node = Some(n);
                        }
                        _ => {}
                    }
                }

                output.push(node.unwrap());
            }
            _ => {}
        }
    }
    Ok(output)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let input = r#"# TYPE http_requests_total counter
# HELP http_requests_total The total number of HTTP requests.
http_requests_total{method="post",code="200"} 1027 1395066363000
http_requests_total{method="post",code="400"}    3 1395066363000

# Escaping in label values:
msdos_file_access_time_seconds{path="C:\\DIR\\FILE.TXT",error="Cannot find file:\n\"FILE.TXT\""} 1.458255915e9

# Minimalistic line:
metric_without_timestamp_and_labels 12.47

# A weird metric from before the epoch:
something_weird{problem="division by zero"} +Inf -3982045

# A histogram, which has a pretty complex representation in the text format:
# HELP http_request_duration_seconds A histogram of the request duration.
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_bucket{le="0.05"} 24054
http_request_duration_seconds_bucket{le="0.1"} 33444
http_request_duration_seconds_bucket{le="0.2"} 100392
http_request_duration_seconds_bucket{le="0.5"} 129389
http_request_duration_seconds_bucket{le="1"} 133988
http_request_duration_seconds_bucket{le="+Inf"} 144320
http_request_duration_seconds_sum 53423
http_request_duration_seconds_count 144320

# Finally a summary, which has a complex representation, too:
# HELP rpc_duration_seconds A summary of the RPC duration in seconds.
# TYPE rpc_duration_seconds summary
rpc_duration_seconds{quantile="0.01"} 3102
rpc_duration_seconds{quantile="0.05"} 3272
rpc_duration_seconds{quantile="0.5"} 4773
rpc_duration_seconds{quantile="0.9"} 9001
rpc_duration_seconds{quantile="0.99"} 76656
rpc_duration_seconds_sum{some="value"} 1.7560473e+07
rpc_duration_seconds_count{another="value2"} 2693
"#;

        /*
        parsing this input generates the following AST:

            - statement
              - block
                - typexpr
                  - typekey > key: "http_requests_total"
                  - typeval > countertype: "counter"
                - helpexpr
                  - helpkey > key: "http_requests_total"
                  - commentval: "The total number of HTTP requests."
                - promstmt
                  - key: "http_requests_total"
                  - pairs
                    - pair
                      - ident: "method"
                      - string > inner: "post"
                    - pair
                      - ident: "code"
                      - string > inner: "200"
                  - number: "1027"
                  - number: "1395066363000"
                - promstmt
                  - key: "http_requests_total"
                  - pairs
                    - pair
                      - ident: "method"
                      - string > inner: "post"
                    - pair
                      - ident: "code"
                      - string > inner: "400"
                  - number: "3"
                  - number: "1395066363000"
              - block
                - genericomment > commentval: "Escaping in label values:"
                - promstmt
                  - key: "msdos_file_access_time_seconds"
                  - pairs
                    - pair
                      - ident: "path"
                      - string > inner: "C:\\\\DIR\\\\FILE.TXT"
                    - pair
                      - ident: "error"
                      - string > inner: "Cannot find file:\\n\\\"FILE.TXT\\\""
                  - number: "1.458255915e9"
              - block
                - genericomment > commentval: "Minimalistic line:"
                - promstmt
                  - key: "metric_without_timestamp_and_labels"
                  - number: "12.47"
              - block
                - genericomment > commentval: "A weird metric from before the epoch:"
                - promstmt
                  - key: "something_weird"
                  - pairs > pair
                    - ident: "problem"
                    - string > inner: "division by zero"
                  - posInf: "+Inf"
                  - number: "-3982045"
              - block
                - genericomment > commentval: "A histogram, which has a pretty complex representation in the text format:"
                - helpexpr
                  - helpkey > key: "http_request_duration_seconds"
                  - commentval: "A histogram of the request duration."
                - typexpr
                  - typekey > key: "http_request_duration_seconds"
                  - typeval > histogramtype: "histogram"
                - promstmt
                  - key: "http_request_duration_seconds_bucket"
                  - pairs > pair
                    - ident: "le"
                    - string > inner: "0.05"
                  - number: "24054"
                - promstmt
                  - key: "http_request_duration_seconds_bucket"
                  - pairs > pair
                    - ident: "le"
                    - string > inner: "0.1"
                  - number: "33444"
                - promstmt
                  - key: "http_request_duration_seconds_bucket"
                  - pairs > pair
                    - ident: "le"
                    - string > inner: "0.2"
                  - number: "100392"
                - promstmt
                  - key: "http_request_duration_seconds_bucket"
                  - pairs > pair
                    - ident: "le"
                    - string > inner: "0.5"
                  - number: "129389"
                - promstmt
                  - key: "http_request_duration_seconds_bucket"
                  - pairs > pair
                    - ident: "le"
                    - string > inner: "1"
                  - number: "133988"
                - promstmt
                  - key: "http_request_duration_seconds_bucket"
                  - pairs > pair
                    - ident: "le"
                    - string > inner: "+Inf"
                  - number: "144320"
                - promstmt
                  - key: "http_request_duration_seconds_sum"
                  - number: "53423"
                - promstmt
                  - key: "http_request_duration_seconds_count"
                  - number: "144320"
              - block
                - genericomment > commentval: "Finally a summary, which has a complex representation, too:"
                - helpexpr
                  - helpkey > key: "rpc_duration_seconds"
                  - commentval: "A summary of the RPC duration in seconds."
                - typexpr
                  - typekey > key: "rpc_duration_seconds"
                  - typeval > summarytype: "summary"
                - promstmt
                  - key: "rpc_duration_seconds"
                  - pairs > pair
                    - ident: "quantile"
                    - string > inner: "0.01"
                  - number: "3102"
                - promstmt
                  - key: "rpc_duration_seconds"
                  - pairs > pair
                    - ident: "quantile"
                    - string > inner: "0.05"
                  - number: "3272"
                - promstmt
                  - key: "rpc_duration_seconds"
                  - pairs > pair
                    - ident: "quantile"
                    - string > inner: "0.5"
                  - number: "4773"
                - promstmt
                  - key: "rpc_duration_seconds"
                  - pairs > pair
                    - ident: "quantile"
                    - string > inner: "0.9"
                  - number: "9001"
                - promstmt
                  - key: "rpc_duration_seconds"
                  - pairs > pair
                    - ident: "quantile"
                    - string > inner: "0.99"
                  - number: "76656"
                - promstmt
                  - key: "rpc_duration_seconds_sum"
                  - number: "1.7560473e+07"
                - promstmt
                  - key: "rpc_duration_seconds_count"
                  - number: "2693"
              - EOI: ""
             */

        let result = parse(&input);
        if let Ok(ref v) = result {
            for d in v {
                println!("{}", format!("{d}"));
            }
        }
        assert_eq!(result.is_ok(), true);
    }
}
