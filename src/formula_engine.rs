// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::{
    ops::{Add, Div, Mul, Neg, Sub},
    str::FromStr,
};

use pest::{iterators::Pairs, Parser};

use crate::{
    error::FormulaError,
    expression::Expr,
    parser::{FormulaParser, Rule},
};

/// FormulaEngine holds the parsed expression and can calculate the result
/// based on the provided component values.
#[derive(Debug)]
pub struct FormulaEngine<T> {
    expr: Expr<T>,
    components: HashSet<usize>,
}

impl<
        'a,
        T: FromStr
            + Copy
            + Neg<Output = T>
            + Add<Output = T>
            + Sub<Output = T>
            + Mul<Output = T>
            + Div<Output = T>
            + PartialOrd,
    > FormulaEngine<T>
where
    Expr<T>: TryFrom<Pairs<'a, Rule>>,
    <T as FromStr>::Err: Debug,
    FormulaError: From<<Expr<T> as TryFrom<Pairs<'a, Rule>>>::Error>,
{
    /// Create a new FormulaEngine from a formula string.
    pub fn try_new(s: &'a str) -> Result<Self, FormulaError> {
        let pairs = FormulaParser::parse(Rule::formula, s)?;
        let expr = Expr::try_from(pairs)?;
        let components = expr.components();

        Ok(Self { expr, components })
    }

    /// Get the components of the formula.
    pub fn components(&self) -> &HashSet<usize> {
        &self.components
    }

    /// Calculate the result of the formula based on the provided component values.
    pub fn calculate(&self, values: HashMap<usize, Option<T>>) -> Result<Option<T>, FormulaError> {
        self.expr.calculate(&values)
    }
}
