//! Module containing parser for promerge.

use std::cell::RefCell;
use std::rc::Rc;

use pest::iterators::Pair;

use crate::*;

#[derive(Parse)]
#[grammar = "./grammar.pest"]
pub struct ExpressionParser;

pub(crate) fn parse<'a>(input: &'a str) -> Result<(), pest::error::Error<Rule>> {
    todo!("not implemented error");
}
