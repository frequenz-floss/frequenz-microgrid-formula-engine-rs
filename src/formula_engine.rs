// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::traits::NumberLike;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::str::FromStr;

use crate::{error::FormulaError, expression::Expr, parser};

/// FormulaEngine holds the parsed expression and can calculate the result
/// based on the provided component values.
#[derive(Debug)]
pub struct FormulaEngine<T> {
    expr: Expr<T>,
    components: HashSet<u64>,
}

impl<T: FromStr + NumberLike<T> + PartialOrd> FormulaEngine<T>
where
    <T as FromStr>::Err: Debug,
{
    /// Create a new FormulaEngine from a formula string.
    pub fn try_new(s: &str) -> Result<Self, FormulaError> {
        let expr = parser::parse(s)?;

        let components = expr.components();

        Ok(Self { expr, components })
    }

    /// Get the components of the formula.
    pub fn components(&self) -> &HashSet<u64> {
        &self.components
    }

    /// Calculate the result of the formula based on the provided component values.
    pub fn calculate(&self, values: HashMap<u64, Option<T>>) -> Result<Option<T>, FormulaError> {
        self.expr.calculate(&values)
    }
}
