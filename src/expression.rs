// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::value_source::{strict, Reading, ValueSource};
use crate::{error::FormulaError, traits::NumberLike};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Neg;

#[derive(Debug)]
pub enum Expr<T, K = u64> {
    Value(Option<T>),
    Neg(Box<Expr<T, K>>),
    Op {
        lhs: Box<Expr<T, K>>,
        op: Op,
        rhs: Box<Expr<T, K>>,
    },
    Function {
        function: Function,
        args: Vec<Expr<T, K>>,
    },
    Component(K),
}

impl<T, K> Expr<T, K> {
    /// The set of component keys the expression references.
    pub fn components(&self) -> HashSet<K>
    where
        K: Clone + Eq + Hash,
    {
        let mut components = HashSet::new();
        self.collect_components(&mut components);
        components
    }

    fn collect_components(&self, into: &mut HashSet<K>)
    where
        K: Clone + Eq + Hash,
    {
        match self {
            Expr::Value(_) => {}
            Expr::Component(key) => {
                into.insert(key.clone());
            }
            Expr::Neg(expr) => expr.collect_components(into),
            Expr::Op { lhs, rhs, .. } => {
                lhs.collect_components(into);
                rhs.collect_components(into);
            }
            Expr::Function { args, .. } => {
                for arg in args {
                    arg.collect_components(into);
                }
            }
        }
    }

    /// Replaces every component key with `f(key)`, leaving constants and
    /// structure untouched.
    pub fn map_components<K2>(self, f: impl Fn(K) -> K2) -> Expr<T, K2> {
        self.map_components_ref(&f)
    }

    fn map_components_ref<K2>(self, f: &impl Fn(K) -> K2) -> Expr<T, K2> {
        match self {
            Expr::Value(value) => Expr::Value(value),
            Expr::Component(key) => Expr::Component(f(key)),
            Expr::Neg(expr) => Expr::Neg(Box::new(expr.map_components_ref(f))),
            Expr::Op { lhs, op, rhs } => Expr::Op {
                lhs: Box::new(lhs.map_components_ref(f)),
                op,
                rhs: Box::new(rhs.map_components_ref(f)),
            },
            Expr::Function { function, args } => Expr::Function {
                function,
                args: args
                    .into_iter()
                    .map(|arg| arg.map_components_ref(f))
                    .collect(),
            },
        }
    }
}

impl<T: NumberLike, K> Expr<T, K> {
    /// Evaluates the expression, pulling component values from `source`.
    pub(crate) fn evaluate(
        &self,
        source: &mut impl ValueSource<K, T>,
    ) -> Result<Reading<T>, FormulaError> {
        Ok(match self {
            Expr::Value(value) => Reading::Value(*value),
            Expr::Component(id) => source.get(id),
            Expr::Neg(expr) => expr.evaluate(source)?.map(Neg::neg),
            Expr::Op { lhs, op, rhs } => {
                let lhs = lhs.evaluate(source)?;
                let rhs = rhs.evaluate(source)?;
                op.apply(lhs, rhs)
            }
            Expr::Function { function, args } => function.evaluate(args, source)?,
        })
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
    Avg,
    Sqrt,
}

impl Function {
    /// Evaluates a function call. `COALESCE` reads its arguments lazily;
    /// every other function reads all of them first.
    pub(crate) fn evaluate<T: NumberLike, K>(
        &self,
        args: &[Expr<T, K>],
        source: &mut impl ValueSource<K, T>,
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
            Function::Sqrt => {
                if args.len() != 1 {
                    return Err(FormulaError("SQRT takes exactly one argument".to_string()));
                }
                Ok(args[0]
                    .evaluate(source)?
                    .and_then(|value| (value >= T::zero()).then(|| value.sqrt())))
            }
            Function::Avg | Function::Min | Function::Max => {
                let readings = args
                    .iter()
                    .map(|arg| arg.evaluate(source))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(strict(readings).and_then(|values| {
                    let count = T::from_usize(values.len());
                    let reduced = values.into_iter().reduce(|acc, x| match self {
                        Function::Avg => acc + x,
                        Function::Min if acc.partial_cmp(&x) == Some(Ordering::Less) => acc,
                        Function::Max if acc.partial_cmp(&x) == Some(Ordering::Greater) => acc,
                        _ => x,
                    });
                    match self {
                        Function::Avg => reduced.map(|sum| sum / count),
                        _ => reduced,
                    }
                }))
            }
        }
    }
}
