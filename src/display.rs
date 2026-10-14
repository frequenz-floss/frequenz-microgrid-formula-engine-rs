// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

//! Rendering formulas back to strings.
//!
//! An operand is parenthesised only where precedence or left associativity
//! requires it, so a sum or product renders as one flat chain. The [`Formula`]
//! docs list the cases where the output does not round-trip.

use std::fmt::{self, Display, Formatter};

use crate::formula::{Formula, Function, Op, ProductOp, SumOp};

impl Display for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
        })
    }
}

impl Display for SumOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SumOp::Add => "+",
            SumOp::Sub => "-",
        })
    }
}

impl Display for ProductOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ProductOp::Mul => "*",
            ProductOp::Div => "/",
        })
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Function::Coalesce => "COALESCE",
            Function::Min => "MIN",
            Function::Max => "MAX",
            Function::Avg => "AVG",
            Function::Sqrt => "SQRT",
        })
    }
}

/// Renders `self` as a formula string. See [`Formula`] for when the output
/// re-parses to an equal expression.
impl<T: Display, K: Display> Display for Formula<T, K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Formula::Constant(Some(value)) => write!(f, "{value}"),
            Formula::Constant(None) => f.write_str("None"),
            Formula::Component(key) => write!(f, "#{key}"),
            Formula::Neg(inner) => {
                f.write_str("-")?;
                self.fmt_operand(f, inner, true)
            }
            Formula::Op { lhs, op, rhs } => {
                self.fmt_operand(f, lhs, false)?;
                write!(f, " {op} ")?;
                self.fmt_operand(f, rhs, true)
            }
            Formula::Sum { first, rest } => self.fmt_chain(f, first, rest),
            Formula::Product { first, rest } => self.fmt_chain(f, first, rest),
            Formula::Function { function, args } => {
                write!(f, "{function}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                f.write_str(")")
            }
        }
    }
}

impl<T: Display, K: Display> Formula<T, K> {
    /// Writes the operands of `self`, a sum or product, with their operators.
    fn fmt_chain<O: Display>(
        &self,
        f: &mut Formatter<'_>,
        first: &Self,
        rest: &[(O, Self)],
    ) -> fmt::Result {
        self.fmt_operand(f, first, false)?;
        for (op, operand) in rest {
            write!(f, " {op} ")?;
            self.fmt_operand(f, operand, true)?;
        }
        Ok(())
    }

    /// Writes `operand` of `self`, in parentheses when it binds less tightly
    /// than `self`, or equally tightly and `on_right` is set. Operators
    /// associate left, so a right operand that binds equally needs parentheses.
    /// A negation inside a negation needs them too, because the grammar takes
    /// only one `-` before an operand.
    fn fmt_operand(&self, f: &mut Formatter<'_>, operand: &Self, on_right: bool) -> fmt::Result {
        let (outer, inner) = (self.precedence(), operand.precedence());
        operand.fmt_wrapped(f, inner < outer || (on_right && inner == outer))
    }

    /// Writes `self`, in parentheses when `parens` is set.
    fn fmt_wrapped(&self, f: &mut Formatter<'_>, parens: bool) -> fmt::Result {
        if parens {
            write!(f, "({self})")
        } else {
            write!(f, "{self}")
        }
    }
}

impl<T, K> Formula<T, K> {
    /// The precedence of `self` as an operand: a higher value binds tighter.
    fn precedence(&self) -> u8 {
        match self {
            Formula::Op {
                op: Op::Add | Op::Sub,
                ..
            } => 1,
            Formula::Op {
                op: Op::Mul | Op::Div,
                ..
            } => 2,
            Formula::Sum { .. } => 1,
            Formula::Product { .. } => 2,
            Formula::Neg(_) => 3,
            Formula::Constant(_) | Formula::Component(_) | Formula::Function { .. } => 4,
        }
    }
}
