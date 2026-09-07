// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::error::FormulaError;
use crate::value_source::{Reading, ValueSource};
use num_traits::real::Real;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Neg;

/// `Display` round-trips through `str::parse` for `K = u64` keys and finite,
/// non-negative constants: `expr.to_string().parse() == Ok(expr)`.
/// It does not round-trip when:
/// - `K` is not `u64` — parsing always yields `Formula<T, u64>`, so the result
///   is a different type, and a key that does not render as `#` followed by
///   digits does not parse at all.
/// - a constant is `NaN` or infinite — these do not parse back.
/// - a constant is negative — `Constant(Some(-2.0))` renders as `-2`, which
///   re-parses as `Neg(Constant(2.0))`, not `Constant(-2.0)`.
#[derive(Debug, Clone, PartialEq)]
pub enum Formula<T, K = u64> {
    Constant(Option<T>),
    Neg(Box<Formula<T, K>>),
    Op {
        lhs: Box<Formula<T, K>>,
        op: Op,
        rhs: Box<Formula<T, K>>,
    },
    Function {
        function: Function,
        args: Vec<Formula<T, K>>,
    },
    Component(K),
}

impl<T, K> Formula<T, K> {
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
            Formula::Constant(_) => {}
            Formula::Component(key) => {
                into.insert(key.clone());
            }
            Formula::Neg(expr) => expr.collect_components(into),
            Formula::Op { lhs, rhs, .. } => {
                lhs.collect_components(into);
                rhs.collect_components(into);
            }
            Formula::Function { args, .. } => {
                for arg in args {
                    arg.collect_components(into);
                }
            }
        }
    }

    /// Replaces every component key with `f(key)`, leaving constants and
    /// structure untouched.
    pub fn map_components<K2>(self, f: impl Fn(K) -> K2) -> Formula<T, K2> {
        self.map_components_ref(&f)
    }

    fn map_components_ref<K2>(self, f: &impl Fn(K) -> K2) -> Formula<T, K2> {
        match self {
            Formula::Constant(value) => Formula::Constant(value),
            Formula::Component(key) => Formula::Component(f(key)),
            Formula::Neg(expr) => Formula::Neg(Box::new(expr.map_components_ref(f))),
            Formula::Op { lhs, op, rhs } => Formula::Op {
                lhs: Box::new(lhs.map_components_ref(f)),
                op,
                rhs: Box::new(rhs.map_components_ref(f)),
            },
            Formula::Function { function, args } => Formula::Function {
                function,
                args: args
                    .into_iter()
                    .map(|arg| arg.map_components_ref(f))
                    .collect(),
            },
        }
    }

    /// `COALESCE(self, other)`, merging either side that is already a coalesce.
    pub fn coalesce(self, other: Self) -> Self {
        self.merge_or_wrap(Function::Coalesce, other)
    }

    /// `MIN(self, other)`, merging either side that is already a min.
    pub fn min(self, other: Self) -> Self {
        self.merge_or_wrap(Function::Min, other)
    }

    /// `MAX(self, other)`, merging either side that is already a max.
    pub fn max(self, other: Self) -> Self {
        self.merge_or_wrap(Function::Max, other)
    }

    /// `AVG(self, others...)`.
    pub fn avg(self, others: impl IntoIterator<Item = Self>) -> Self {
        Formula::Function {
            function: Function::Avg,
            args: std::iter::once(self).chain(others).collect(),
        }
    }

    /// `SQRT(self)`.
    pub fn sqrt(self) -> Self {
        Formula::Function {
            function: Function::Sqrt,
            args: vec![self],
        }
    }

    /// Builds `function(self, other)`, merging the arguments of either side
    /// that is already a call of `function`, so a chain has one shape
    /// however it was built.
    fn merge_or_wrap(self, function: Function, other: Self) -> Self {
        let mut args = self.args_of(function);
        args.extend(other.args_of(function));
        Formula::Function { function, args }
    }

    /// The arguments of `self` if it is a call of `function`, else `self`
    /// alone.
    fn args_of(self, function: Function) -> Vec<Self> {
        match self {
            Formula::Function {
                function: existing,
                args,
            } if existing == function => args,
            other => vec![other],
        }
    }

    fn binary(self, op: Op, rhs: Self) -> Self {
        Formula::Op {
            lhs: Box::new(self),
            op,
            rhs: Box::new(rhs),
        }
    }
}

impl<T, K> std::ops::Add for Formula<T, K> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.binary(Op::Add, rhs)
    }
}

impl<T, K> std::ops::Sub for Formula<T, K> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.binary(Op::Sub, rhs)
    }
}

impl<T, K> std::ops::Mul for Formula<T, K> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        self.binary(Op::Mul, rhs)
    }
}

impl<T, K> std::ops::Div for Formula<T, K> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        self.binary(Op::Div, rhs)
    }
}

impl<T, K> std::ops::Neg for Formula<T, K> {
    type Output = Self;

    fn neg(self) -> Self {
        Formula::Neg(Box::new(self))
    }
}

impl<T: Real, K> Formula<T, K> {
    /// Evaluates the formula, pulling component values from `source`.
    ///
    /// A `HashMap` from keys to `Option<T>` is a source; a key absent from
    /// the map reads as [`Reading::Unknown`].
    pub fn evaluate(
        &self,
        source: &mut impl ValueSource<T, K>,
    ) -> Result<Reading<T>, FormulaError> {
        Ok(match self {
            Formula::Constant(value) => Reading::Known(*value),
            Formula::Component(id) => source.read(id),
            Formula::Neg(expr) => expr.evaluate(source)?.map(Neg::neg),
            Formula::Op { lhs, op, rhs } => {
                let lhs = lhs.evaluate(source)?;
                let rhs = rhs.evaluate(source)?;
                op.apply(lhs, rhs)
            }
            Formula::Function { function, args } => function.evaluate(args, source)?,
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
    pub(crate) fn apply<T: Real>(&self, lhs: Reading<T>, rhs: Reading<T>) -> Reading<T> {
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
    pub(crate) fn evaluate<T: Real, K>(
        &self,
        args: &[Formula<T, K>],
        source: &mut impl ValueSource<T, K>,
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
            Function::Sqrt => {
                if args.len() != 1 {
                    return Err(FormulaError::Arity {
                        function: *self,
                        args: args.len(),
                    });
                }
                Ok(args[0]
                    .evaluate(source)?
                    .and_then(|value| (value >= T::zero()).then(|| value.sqrt())))
            }
            Function::Avg => {
                let mut sum = T::zero();
                let mut count = T::zero();
                let mut unknown = false;
                for arg in args {
                    match arg.evaluate(source)? {
                        Reading::Known(Some(value)) => {
                            sum = sum + value;
                            count = count + T::one();
                        }
                        Reading::Known(None) => {}
                        Reading::Unknown => unknown = true,
                    }
                }
                Ok(if unknown {
                    Reading::Unknown
                } else if count == T::zero() {
                    Reading::Known(None)
                } else {
                    Reading::Known(Some(sum / count))
                })
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
