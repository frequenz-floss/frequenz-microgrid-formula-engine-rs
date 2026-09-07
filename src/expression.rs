// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::value_source::{strict, Reading, ValueSource};
use crate::{error::FormulaError, traits::NumberLike};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Neg;

/// `Display` round-trips through [`parse`](crate::parse) for `K = u64` keys
/// and finite, non-negative constants: `parse(&expr.to_string()) == expr`.
/// It does not round-trip when:
/// - `K` is not `u64` — other keys render as `#<key>`, which `parse` rejects.
/// - a constant is `NaN` or infinite — these do not parse back.
/// - a constant is negative — `Value(Some(-2.0))` renders as `-2`, which
///   re-parses as `Neg(Value(2.0))`, not `Value(-2.0)`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
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

    /// A constant, `None` for a missing value.
    pub fn value(value: Option<T>) -> Self {
        Expr::Value(value)
    }

    /// A reference to the component with the given key.
    pub fn component(key: K) -> Self {
        Expr::Component(key)
    }

    /// `COALESCE(self, other)`, extending `self` if it is already a coalesce.
    pub fn coalesce(self, other: Self) -> Self {
        self.push_or_wrap(Function::Coalesce, other)
    }

    /// `MIN(self, other)`, extending `self` if it is already a min.
    pub fn min(self, other: Self) -> Self {
        self.push_or_wrap(Function::Min, other)
    }

    /// `MAX(self, other)`, extending `self` if it is already a max.
    pub fn max(self, other: Self) -> Self {
        self.push_or_wrap(Function::Max, other)
    }

    /// `AVG(self, others...)`.
    pub fn avg(self, others: impl IntoIterator<Item = Self>) -> Self {
        Expr::Function {
            function: Function::Avg,
            args: std::iter::once(self).chain(others).collect(),
        }
    }

    /// `SQRT(self)`.
    pub fn sqrt(self) -> Self {
        Expr::Function {
            function: Function::Sqrt,
            args: vec![self],
        }
    }

    fn push_or_wrap(self, function: Function, other: Self) -> Self {
        match self {
            Expr::Function {
                function: existing,
                mut args,
            } if existing == function => {
                args.push(other);
                Expr::Function { function, args }
            }
            first => Expr::Function {
                function,
                args: vec![first, other],
            },
        }
    }

    fn binary(self, op: Op, rhs: Self) -> Self {
        Expr::Op {
            lhs: Box::new(self),
            op,
            rhs: Box::new(rhs),
        }
    }
}

impl<T, K> std::ops::Add for Expr<T, K> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.binary(Op::Add, rhs)
    }
}

impl<T, K> std::ops::Sub for Expr<T, K> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.binary(Op::Sub, rhs)
    }
}

impl<T, K> std::ops::Mul for Expr<T, K> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        self.binary(Op::Mul, rhs)
    }
}

impl<T, K> std::ops::Div for Expr<T, K> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        self.binary(Op::Div, rhs)
    }
}

impl<T, K> std::ops::Neg for Expr<T, K> {
    type Output = Self;

    fn neg(self) -> Self {
        Expr::Neg(Box::new(self))
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

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
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

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
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
                "{self} requires at least one argument"
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
