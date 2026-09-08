// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::value_source::{strict, Reading, ValueSource};
use crate::{error::FormulaError, traits::NumberLike};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::ops::Neg;

#[derive(Debug)]
pub enum Expr<T> {
    Value(Option<T>),
    UnaryMinus(Box<Expr<T>>),
    Op {
        lhs: Box<Expr<T>>,
        op: Op,
        rhs: Box<Expr<T>>,
    },
    Function {
        function: Function,
        args: Vec<Expr<T>>,
    },
    Component(u64),
}

impl<T: NumberLike> Expr<T> {
    /// Evaluates the expression, pulling component values from `source`.
    pub(crate) fn evaluate(
        &self,
        source: &mut impl ValueSource<u64, T>,
    ) -> Result<Reading<T>, FormulaError> {
        Ok(match self {
            Expr::Value(value) => Reading::Value(*value),
            Expr::Component(id) => source.get(id),
            Expr::UnaryMinus(expr) => expr.evaluate(source)?.map(Neg::neg),
            Expr::Op { lhs, op, rhs } => {
                let lhs = lhs.evaluate(source)?;
                let rhs = rhs.evaluate(source)?;
                op.apply(lhs, rhs)
            }
            Expr::Function { function, args } => function.evaluate(args, source)?,
        })
    }

    pub fn components(&self) -> HashSet<u64> {
        match self {
            Expr::Value(_) => HashSet::new(),
            Expr::UnaryMinus(expr) => expr.components(),
            Expr::Op { lhs, rhs, .. } => {
                let mut components = lhs.components();
                components.extend(rhs.components());
                components
            }
            Expr::Function { args, .. } => args
                .iter()
                .map(Expr::components)
                .fold(HashSet::new(), |acc, x| acc.union(&x).copied().collect()),
            Expr::Component(i) => HashSet::from([*i]),
        }
    }
}

#[derive(Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    /// Combines two already-read operands. Both are read before this is
    /// called, so a `None` on one side never hides the other from the
    /// source.
    pub(crate) fn apply<T: NumberLike>(&self, lhs: Reading<T>, rhs: Reading<T>) -> Reading<T> {
        lhs.zip(rhs).and_then(|(l, r)| match self {
            Op::Add => Some(l + r),
            Op::Sub => Some(l - r),
            Op::Mul => Some(l * r),
            Op::Div => (r != T::zero()).then(|| l / r),
        })
    }
}

#[derive(Debug)]
pub enum Function {
    Coalesce,
    Min,
    Max,
}

impl Function {
    /// Evaluates a function call. `COALESCE` reads its arguments lazily;
    /// every other function reads all of them first.
    pub(crate) fn evaluate<T: NumberLike>(
        &self,
        args: &[Expr<T>],
        source: &mut impl ValueSource<u64, T>,
    ) -> Result<Reading<T>, FormulaError> {
        if args.is_empty() {
            return Err(FormulaError(format!(
                "{self:?} requires at least one argument"
            )));
        }
        match self {
            Function::Coalesce => {
                for arg in args {
                    match arg.evaluate(source)? {
                        Reading::Value(None) => continue,
                        decided => return Ok(decided),
                    }
                }
                Ok(Reading::Value(None))
            }
            Function::Min | Function::Max => {
                let readings = args
                    .iter()
                    .map(|arg| arg.evaluate(source))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(strict(readings).and_then(|values| {
                    values.into_iter().reduce(|acc, x| match self {
                        Function::Min if acc.partial_cmp(&x) == Some(Ordering::Less) => acc,
                        Function::Max if acc.partial_cmp(&x) == Some(Ordering::Greater) => acc,
                        _ => x,
                    })
                }))
            }
        }
    }
}
