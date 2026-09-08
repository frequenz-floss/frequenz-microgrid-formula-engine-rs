// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::error::FormulaError;
use crate::value_source::{Reading, ValueSource};
use num_traits::real::Real;
use std::collections::HashSet;
use std::ops::Neg;

#[derive(Debug)]
pub enum Formula<T> {
    Constant(Option<T>),
    UnaryMinus(Box<Formula<T>>),
    Op {
        lhs: Box<Formula<T>>,
        op: Op,
        rhs: Box<Formula<T>>,
    },
    Function {
        function: Function,
        args: Vec<Formula<T>>,
    },
    Component(u64),
}

impl<T: Real> Formula<T> {
    /// Evaluates the formula, pulling component values from `source`.
    ///
    /// A `HashMap` from keys to `Option<T>` is a source; a key absent from
    /// the map reads as [`Reading::Unknown`].
    pub fn evaluate(
        &self,
        source: &mut impl ValueSource<T, u64>,
    ) -> Result<Reading<T>, FormulaError> {
        Ok(match self {
            Formula::Constant(value) => Reading::Known(*value),
            Formula::Component(id) => source.read(id),
            Formula::UnaryMinus(expr) => expr.evaluate(source)?.map(Neg::neg),
            Formula::Op { lhs, op, rhs } => {
                let lhs = lhs.evaluate(source)?;
                let rhs = rhs.evaluate(source)?;
                op.apply(lhs, rhs)
            }
            Formula::Function { function, args } => function.evaluate(args, source)?,
        })
    }

    pub fn components(&self) -> HashSet<u64> {
        match self {
            Formula::Constant(_) => HashSet::new(),
            Formula::UnaryMinus(expr) => expr.components(),
            Formula::Op { lhs, rhs, .. } => {
                let mut components = lhs.components();
                components.extend(rhs.components());
                components
            }
            Formula::Function { args, .. } => args
                .iter()
                .map(Formula::components)
                .fold(HashSet::new(), |acc, x| acc.union(&x).copied().collect()),
            Formula::Component(i) => HashSet::from([*i]),
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
    pub(crate) fn apply<T: Real>(&self, lhs: Reading<T>, rhs: Reading<T>) -> Reading<T> {
        lhs.zip(rhs).and_then(|(l, r)| match self {
            Op::Add => Some(l + r),
            Op::Sub => Some(l - r),
            Op::Mul => Some(l * r),
            Op::Div => (r != T::zero()).then(|| l / r),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Function {
    Coalesce,
    Min,
    Max,
}

impl Function {
    /// Evaluates a function call. `COALESCE` reads its arguments lazily;
    /// every other function reads all of them first.
    pub(crate) fn evaluate<T: Real>(
        &self,
        args: &[Formula<T>],
        source: &mut impl ValueSource<T, u64>,
    ) -> Result<Reading<T>, FormulaError> {
        if args.is_empty() {
            return Err(FormulaError::Arity {
                function: *self,
                args: args.len(),
            });
        }
        match self {
            Function::Coalesce => {
                for arg in args {
                    match arg.evaluate(source)? {
                        Reading::Known(None) => continue,
                        decided => return Ok(decided),
                    }
                }
                Ok(Reading::Known(None))
            }
            Function::Min | Function::Max => {
                let mut acc = args[0].evaluate(source)?;
                for arg in &args[1..] {
                    let x = arg.evaluate(source)?;
                    acc = acc.zip(x).map(|(acc, x)| match self {
                        Function::Min if acc < x => acc,
                        Function::Max if acc > x => acc,
                        _ => x,
                    });
                }
                Ok(acc)
            }
        }
    }
}
