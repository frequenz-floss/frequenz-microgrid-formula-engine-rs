// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::error::FormulaError;
use crate::value_source::{Reading, ValueSource};
use num_traits::real::Real;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Neg;

#[derive(Debug)]
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
    Avg,
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
