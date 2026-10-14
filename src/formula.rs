// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::error::FormulaError;
use crate::value_source::{Reading, ValueSource};
use num_traits::Float;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Neg;

/// An expression tree over constants, component references, operators and
/// functions.
///
/// `Constant(None)` is the known-missing constant written as `None` in the
/// grammar: it means the value is decided to be absent, which is different
/// from [`Reading::Unknown`](crate::Reading::Unknown), meaning the value
/// is not known yet.
///
/// Every walk over the tree recurses once per level. A chain of `+` and `-`, or
/// of `*` and `/`, is one node however long it is. So each level of parentheses
/// or function calls adds only a few levels to a parsed formula, and parsing
/// bounds that nesting by [`MAX_NESTING`](crate::MAX_NESTING).
///
/// Each operator or builder call adds at most one level above its deepest
/// operand. When `a` is already a sum, `a + b` and `a - b` add `b` to the chain
/// of `a` instead of wrapping `a`. `a * b` and `a / b` do the same when `a` is
/// a product.
///
/// A formula that wraps the previous result again and again can still grow
/// deep enough to overflow the stack. Examples are `total = x + total` and
/// `total = total * x + y`.
///
/// `Display` round-trips through `str::parse` for `K = u64` keys and finite,
/// non-negative constants: `expr.to_string().parse() == Ok(expr)`.
/// It does not round-trip when:
/// - `K` is not `u64` — parsing always yields `Formula<T, u64>`, so the result
///   is a different type, and a key that does not render as `#` followed by
///   digits does not parse at all.
/// - a constant is `NaN` or infinite — these do not parse back.
/// - a constant is negative — `Constant(Some(-2.0))` renders as `-2`, which
///   re-parses as `Neg(Constant(2.0))`, not `Constant(-2.0)`. And
///   `Neg(Constant(Some(-2.0)))` renders as `--2`, which does not parse.
/// - a hand-built `Sum` starts with a `Sum`, or a `Product` with a `Product` —
///   the two render as one flat chain, which re-parses as a single node.
/// - a hand-built `Sum` or `Product` has an empty `rest` — it renders as its
///   first operand alone, but keeps the parentheses a longer chain would have.
///   A `Product` with `first: #1 + #2` and an empty `rest` renders as
///   `(#1 + #2)`. A `Sum` with `first: #2` and an empty `rest`, used as the
///   right side of `#1 * …`, renders as `#1 * (#2)`.
#[derive(Debug, Clone, PartialEq)]
pub enum Formula<T, K = u64> {
    /// A constant, or `None` for the known-missing constant.
    Constant(Option<T>),
    /// The arithmetic negation of an expression (`-expr`).
    Neg(Box<Formula<T, K>>),
    /// Operands added and subtracted left to right: `first + a - b …`.
    Sum {
        /// The first operand.
        first: Box<Formula<T, K>>,
        /// Each further operand, with the operator that applies it.
        rest: Vec<(SumOp, Formula<T, K>)>,
    },
    /// Operands multiplied and divided left to right: `first * a / b …`.
    Product {
        /// The first operand.
        first: Box<Formula<T, K>>,
        /// Each further operand, with the operator that applies it.
        rest: Vec<(ProductOp, Formula<T, K>)>,
    },
    /// A function call over one or more argument expressions.
    Function {
        /// The function being called.
        function: Function,
        /// The argument expressions.
        args: Vec<Formula<T, K>>,
    },
    /// A reference to a component's value, keyed by `K`.
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
            Formula::Sum { first, rest } => {
                for operand in Self::chain_operands(first, rest) {
                    operand.collect_components(into);
                }
            }
            Formula::Product { first, rest } => {
                for operand in Self::chain_operands(first, rest) {
                    operand.collect_components(into);
                }
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
            Formula::Sum { first, rest } => Formula::Sum {
                first: Box::new(first.map_components_ref(f)),
                rest: Self::map_rest(rest, f),
            },
            Formula::Product { first, rest } => Formula::Product {
                first: Box::new(first.map_components_ref(f)),
                rest: Self::map_rest(rest, f),
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

    /// The operands of a sum or product, `first` included, in order.
    fn chain_operands<'a, O>(
        first: &'a Self,
        rest: &'a [(O, Self)],
    ) -> impl Iterator<Item = &'a Self> {
        std::iter::once(first).chain(rest.iter().map(|(_, operand)| operand))
    }

    /// Re-keys the `rest` operands of a sum or product, keeping the operators.
    fn map_rest<O, K2>(rest: Vec<(O, Self)>, f: &impl Fn(K) -> K2) -> Vec<(O, Formula<T, K2>)> {
        rest.into_iter()
            .map(|(op, operand)| (op, operand.map_components_ref(f)))
            .collect()
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

    /// `self op rhs`, appending to `self` if it is already a sum, so that a
    /// chain stays one node.
    fn sum(self, op: SumOp, rhs: Self) -> Self {
        let (first, mut rest) = match self {
            Formula::Sum { first, rest } => (first, rest),
            lhs => (Box::new(lhs), Vec::new()),
        };
        rest.push((op, rhs));
        Formula::Sum { first, rest }
    }

    /// `self op rhs`, appending to `self` if it is already a product, so that a
    /// chain stays one node.
    fn product(self, op: ProductOp, rhs: Self) -> Self {
        let (first, mut rest) = match self {
            Formula::Product { first, rest } => (first, rest),
            lhs => (Box::new(lhs), Vec::new()),
        };
        rest.push((op, rhs));
        Formula::Product { first, rest }
    }
}

impl<T, K> std::ops::Add for Formula<T, K> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.sum(SumOp::Add, rhs)
    }
}

impl<T, K> std::ops::Sub for Formula<T, K> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.sum(SumOp::Sub, rhs)
    }
}

impl<T, K> std::ops::Mul for Formula<T, K> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        self.product(ProductOp::Mul, rhs)
    }
}

impl<T, K> std::ops::Div for Formula<T, K> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        self.product(ProductOp::Div, rhs)
    }
}

impl<T, K> std::ops::Neg for Formula<T, K> {
    type Output = Self;

    fn neg(self) -> Self {
        Formula::Neg(Box::new(self))
    }
}

impl<T: Float, K> Formula<T, K> {
    /// Evaluates the formula, pulling component values from `source`.
    ///
    /// The result is never NaN or infinite: a reading, a hand-built constant
    /// or an intermediate result that is not finite is `None`, so `COALESCE`
    /// and `AVG` move past it like any other missing value.
    ///
    /// A `HashMap` from keys to `Option<T>` is a source; a key absent from
    /// the map reads as [`Reading::Unknown`].
    pub fn evaluate(
        &self,
        source: &mut impl ValueSource<T, K>,
    ) -> Result<Reading<T>, FormulaError> {
        let reading = match self {
            Formula::Constant(value) => Reading::Known(*value),
            Formula::Component(id) => source.read(id),
            Formula::Neg(expr) => expr.evaluate(source)?.map(Neg::neg),
            Formula::Sum { first, rest } => {
                Self::evaluate_chain(first, rest, source, SumOp::apply)?
            }
            Formula::Product { first, rest } => {
                Self::evaluate_chain(first, rest, source, ProductOp::apply)?
            }
            Formula::Function { function, args } => function.evaluate(args, source)?,
        };
        Ok(reading.and_then(|value| value.is_finite().then_some(value)))
    }

    /// Evaluates a sum or product left to right. Every operand is read before
    /// it is combined, so a `None` never hides a later operand from the source.
    ///
    /// The running value is not checked for finiteness; `evaluate` checks the
    /// chain's result. Every operand comes from `evaluate`, so its value, when
    /// it has one, is finite. An infinite or NaN running value stays infinite
    /// or NaN under `+ - * /` with a finite operand.
    fn evaluate_chain<O: Copy>(
        first: &Self,
        rest: &[(O, Self)],
        source: &mut impl ValueSource<T, K>,
        apply: impl Fn(O, T, T) -> T,
    ) -> Result<Reading<T>, FormulaError> {
        let mut acc = first.evaluate(source)?;
        for (op, operand) in rest {
            let next = operand.evaluate(source)?;
            acc = acc.zip(next).map(|(lhs, rhs)| apply(*op, lhs, rhs));
        }
        Ok(acc)
    }
}

/// The operator that applies an operand of a [`Formula::Sum`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SumOp {
    /// Addition.
    Add,
    /// Subtraction.
    Sub,
}

/// The operator that applies an operand of a [`Formula::Product`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProductOp {
    /// Multiplication.
    Mul,
    /// Division. Division by zero gives infinity or NaN, which evaluates to
    /// `None`.
    Div,
}

impl SumOp {
    fn apply<T: Float>(self, lhs: T, rhs: T) -> T {
        match self {
            SumOp::Add => lhs + rhs,
            SumOp::Sub => lhs - rhs,
        }
    }
}

impl ProductOp {
    fn apply<T: Float>(self, lhs: T, rhs: T) -> T {
        match self {
            ProductOp::Mul => lhs * rhs,
            ProductOp::Div => lhs / rhs,
        }
    }
}

/// A function call in an expression.
///
/// All functions take one or more arguments, except [`Function::Sqrt`],
/// which takes exactly one.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Function {
    /// Returns the first argument with a value: an argument that is
    /// known-missing is skipped, and an unknown argument stops evaluation
    /// (see [`crate`] for the full semantics).
    Coalesce,
    /// The smallest of its arguments.
    Min,
    /// The largest of its arguments.
    Max,
    /// The arithmetic mean of the arguments that have a value. Known-missing
    /// arguments are skipped; when none has a value the result is `None`.
    /// An unknown argument makes the result unknown.
    Avg,
    /// The square root of its single argument. A negative argument gives
    /// NaN, which evaluates to `None`.
    Sqrt,
}

impl Function {
    /// Evaluates a function call. `COALESCE` reads its arguments lazily;
    /// every other function reads all of them first.
    pub(crate) fn evaluate<T: Float, K>(
        &self,
        args: &[Formula<T, K>],
        source: &mut impl ValueSource<T, K>,
    ) -> Result<Reading<T>, FormulaError> {
        if args.is_empty() {
            return Err(FormulaError::WrongArity {
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
                    return Err(FormulaError::WrongArity {
                        function: *self,
                        args: args.len(),
                    });
                }
                Ok(args[0].evaluate(source)?.map(Float::sqrt))
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
