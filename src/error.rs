// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::formula::Function;
use crate::parser::Rule;

/// An error parsing a formula string or evaluating a structurally invalid
/// formula, such as a hand-built function call with the wrong number of
/// arguments. Missing or `None` data never produces a `FormulaError`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum FormulaError {
    /// The formula does not match the grammar; the message comes from the
    /// parser.
    Syntax(String),
    /// A numeric constant the number type cannot parse.
    InvalidNumber(String),
    /// A component id that does not fit `u64`.
    InvalidComponentId(String),
    /// A function call with a number of arguments it does not take.
    Arity {
        /// The function that was called.
        function: Function,
        /// The number of arguments it was given.
        args: usize,
    },
    /// A parser invariant did not hold, which is a bug in this crate.
    Internal(String),
}

impl Display for FormulaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FormulaError::Syntax(message) => f.write_str(message),
            FormulaError::InvalidNumber(literal) => write!(f, "Invalid number: {literal}"),
            FormulaError::InvalidComponentId(literal) => {
                write!(f, "Invalid component id: {literal}")
            }
            FormulaError::Arity { function, args: 0 } => {
                write!(f, "{function:?} requires at least one argument")
            }
            FormulaError::Arity { function, .. } => {
                write!(f, "{function:?} takes exactly one argument")
            }
            FormulaError::Internal(message) => write!(f, "internal parser error: {message}"),
        }
    }
}

impl Error for FormulaError {}

impl From<pest::error::Error<Rule>> for FormulaError {
    fn from(err: pest::error::Error<Rule>) -> Self {
        FormulaError::Syntax(err.to_string())
    }
}
