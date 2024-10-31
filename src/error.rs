// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::parser::Rule;
use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct FormulaError(pub String);

impl Display for FormulaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for FormulaError {}

impl From<pest::error::Error<Rule>> for FormulaError {
    fn from(err: pest::error::Error<Rule>) -> Self {
        FormulaError(format!("{}", err))
    }
}

impl From<std::num::ParseFloatError> for FormulaError {
    fn from(err: std::num::ParseFloatError) -> Self {
        FormulaError(format!("{}", err))
    }
}
