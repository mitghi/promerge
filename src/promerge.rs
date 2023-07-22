use crate::parser;

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
enum Kind {
    Counter,
    Gauge,
    Histogram,
    Summary,
    Untyped,
}
