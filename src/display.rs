// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

//! Rendering expressions back to formula strings.
//!
//! Every nested operator is parenthesised, so the output needs no
//! precedence bookkeeping to re-parse. The [`Expr`] docs list the cases
//! where it still does not round-trip.

use std::fmt::{self, Display, Formatter};

use crate::expression::{Expr, Function, Op};

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

/// Renders `self` as a formula string. See [`Expr`] for when the output
/// re-parses to an equal expression.
impl<T: Display, K: Display> Display for Expr<T, K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Value(Some(value)) => write!(f, "{value}"),
            Expr::Value(None) => f.write_str("None"),
            Expr::Component(key) => write!(f, "#{key}"),
            Expr::Neg(inner) => {
                f.write_str("-")?;
                inner.fmt_operand(f)
            }
            Expr::Op { lhs, op, rhs } => {
                lhs.fmt_operand(f)?;
                write!(f, " {op} ")?;
                rhs.fmt_operand(f)
            }
            Expr::Function { function, args } => {
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

impl<T: Display, K: Display> Expr<T, K> {
    /// Writes `self` as the operand of a `Neg` or `Op` node, parenthesised
    /// when it is itself an operator so the output re-parses unchanged.
    fn fmt_operand(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Op { .. } | Expr::Neg(_) => write!(f, "({self})"),
            _ => write!(f, "{self}"),
        }
    }
}
