// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::traits::NumberLike;
use std::collections::HashSet;
use std::fmt::Debug;
use std::str::FromStr;

use crate::value_source::{Reading, ValueSource};
use crate::{error::FormulaError, expression::Expr, parser};

/// FormulaEngine holds the parsed expression and can evaluate the result
/// based on the provided component values.
#[derive(Debug)]
pub struct FormulaEngine<T> {
    expr: Expr<T>,
    components: HashSet<u64>,
}

impl<T: FromStr + NumberLike> FormulaEngine<T>
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

    /// Evaluates the formula, pulling component values from `source`.
    ///
    /// A `HashMap<u64, Option<T>>` is a source; a key that is absent from
    /// the map reads as [`Reading::Undecided`].
    pub fn evaluate(
        &self,
        source: &mut impl ValueSource<u64, T>,
    ) -> Result<Reading<T>, FormulaError> {
        self.expr.evaluate(source)
    }
}
