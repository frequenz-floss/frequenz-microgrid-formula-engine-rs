// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::traits::NumberLike;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::str::FromStr;

use crate::value_source::{Reading, ValueSource};
use crate::{error::FormulaError, expression::Expr, parser};

/// FormulaEngine holds a parsed or built expression and evaluates it
/// against component values pulled from a [`ValueSource`].
#[derive(Debug)]
pub struct FormulaEngine<T, K = u64> {
    expr: Expr<T, K>,
    components: HashSet<K>,
}

impl<T: FromStr + NumberLike> FormulaEngine<T, u64>
where
    <T as FromStr>::Err: Debug,
{
    /// Parses a formula string into a FormulaEngine.
    pub fn try_new(s: &str) -> Result<Self, FormulaError> {
        Ok(Self::from_expr(parser::parse(s)?))
    }
}

impl<T, K> FormulaEngine<T, K> {
    /// The expression being evaluated.
    pub fn expr(&self) -> &Expr<T, K> {
        &self.expr
    }

    /// The components the expression references.
    pub fn components(&self) -> &HashSet<K> {
        &self.components
    }
}

impl<T, K: Clone + Eq + Hash> FormulaEngine<T, K> {
    /// Builds a FormulaEngine from an expression.
    pub fn from_expr(expr: Expr<T, K>) -> Self {
        let components = expr.components();
        Self { expr, components }
    }
}

impl<T: NumberLike, K> FormulaEngine<T, K> {
    /// Evaluates the formula, pulling component values from `source`.
    ///
    /// A `HashMap<K, Option<T>>` is a source; a key that is absent from
    /// the map reads as [`Reading::Undecided`].
    pub fn evaluate(
        &self,
        source: &mut impl ValueSource<K, T>,
    ) -> Result<Reading<T>, FormulaError> {
        self.expr.evaluate(source)
    }
}
