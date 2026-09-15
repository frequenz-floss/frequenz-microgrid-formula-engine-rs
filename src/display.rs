// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

//! Rendering formulas back to strings.
//!
//! An operand is parenthesised only where precedence or left associativity
//! requires it, so a long operator chain renders flat. The [`Formula`] docs
//! list the cases where the output does not round-trip.

use std::fmt::{self, Display, Formatter};

use crate::formula::{Formula, Function, Op};

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
                let nested = matches!(**inner, Formula::Op { .. } | Formula::Neg(_));
                inner.fmt_wrapped(f, nested)
            }
            Formula::Op { lhs, op, rhs } => {
                lhs.fmt_wrapped(f, lhs.needs_parens(*op, false))?;
                write!(f, " {op} ")?;
                rhs.fmt_wrapped(f, rhs.needs_parens(*op, true))
            }
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
    /// Whether `self`, as an operand of `op`, needs parentheses to parse back
    /// to the same tree. A looser-binding operator always does, and so does
    /// an equally tight one on the right, because operators associate left.
    /// A unary minus binds tightest of all, so it never does.
    fn needs_parens(&self, op: Op, on_right: bool) -> bool {
        match self {
            Formula::Op { op: inner, .. } => {
                precedence(*inner) < precedence(op)
                    || (on_right && precedence(*inner) == precedence(op))
            }
            _ => false,
        }
    }
}

fn precedence(op: Op) -> u8 {
    match op {
        Op::Add | Op::Sub => 1,
        Op::Mul | Op::Div => 2,
    }
}
