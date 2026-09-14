// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use pest::{
    iterators::{Pair, Pairs},
    pratt_parser::PrattParser,
    Parser,
};
use pest_derive::Parser;
use std::str::FromStr;
use std::sync::LazyLock;

use crate::formula::{Formula, Function, Op};
use crate::FormulaError;
use num_traits::real::Real;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct FormulaParser;

static PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    use pest::pratt_parser::{Assoc::*, Op};
    use Rule::*;

    PrattParser::new()
        .op(Op::infix(add, Left) | Op::infix(sub, Left))
        .op(Op::infix(mul, Left) | Op::infix(div, Left))
        .op(Op::prefix(unary_minus))
        .op(Op::postfix(Rule::EOI))
});

pub(crate) fn parse<T>(formula: &str) -> Result<Formula<T>, FormulaError>
where
    T: FromStr + Real,
{
    let pairs = FormulaParser::parse(Rule::formula, formula)?;
    parse_to_formula(pairs)
}

/// Parses a formula string into a formula with `u64` component keys.
/// A constant larger than `T` can hold is an error.
impl<T> FromStr for Formula<T>
where
    T: FromStr + Real,
{
    type Err = FormulaError;

    fn from_str(formula: &str) -> Result<Self, Self::Err> {
        parse(formula)
    }
}

/// Builds `function` over the argument expressions inside a function rule.
fn function_call<T>(function: Function, call: Pair<Rule>) -> Result<Formula<T>, FormulaError>
where
    T: FromStr + Real,
{
    Ok(Formula::Function {
        function,
        args: call
            .into_inner()
            .map(|arg| parse_to_formula(Pairs::single(arg)))
            .collect::<Result<_, _>>()?,
    })
}

fn parse_to_formula<T>(pairs: Pairs<Rule>) -> Result<Formula<T>, FormulaError>
where
    T: FromStr + Real,
{
    PRATT_PARSER
        .map_primary(|primary| {
            Ok(match primary.as_rule() {
                Rule::none => Formula::Constant(None),
                Rule::expr => parse_to_formula(primary.into_inner())?,
                Rule::num => {
                    let num: T = primary
                        .as_str()
                        .parse()
                        .map_err(|_| FormulaError::InvalidNumber(primary.as_str().to_string()))?;
                    if num > T::max_value() {
                        return Err(FormulaError::NumberOutOfRange(primary.as_str().to_string()));
                    }
                    Formula::Constant(Some(num))
                }
                Rule::component => primary
                    .as_str()
                    .replace("#", "")
                    .parse()
                    .map(Formula::Component)
                    .map_err(|_| FormulaError::InvalidComponentId(primary.as_str().to_string()))?,
                Rule::coalesce => function_call(Function::Coalesce, primary)?,
                Rule::min => function_call(Function::Min, primary)?,
                Rule::max => function_call(Function::Max, primary)?,
                Rule::avg => function_call(Function::Avg, primary)?,
                Rule::sqrt => function_call(Function::Sqrt, primary)?,
                rule => {
                    return Err(FormulaError::Internal(format!(
                        "expected atom, found {rule:?}"
                    )))
                }
            })
        })
        .map_infix(|lhs, op, rhs| {
            if lhs.is_err() {
                lhs
            } else if rhs.is_err() {
                rhs
            } else if let (Ok(lhs), Ok(rhs)) = (lhs, rhs) {
                Ok(Formula::Op {
                    lhs: Box::new(lhs),
                    op: match op.as_rule() {
                        Rule::add => Op::Add,
                        Rule::sub => Op::Sub,
                        Rule::mul => Op::Mul,
                        Rule::div => Op::Div,
                        rule => {
                            return Err(FormulaError::Internal(format!(
                                "expected operator, found {rule:?}"
                            )))
                        }
                    },
                    rhs: Box::new(rhs),
                })
            } else {
                Err(FormulaError::Internal("internal error".to_string()))
            }
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::unary_minus => {
                if let Ok(rhs) = rhs {
                    Ok(Formula::Neg(Box::new(rhs)))
                } else {
                    rhs
                }
            }
            rule => Err(FormulaError::Internal(format!(
                "unexpected prefix rule: {rule:?}"
            ))),
        })
        .map_postfix(|lhs, op| match op.as_rule() {
            Rule::EOI => lhs,
            rule => Err(FormulaError::Internal(format!(
                "unexpected postfix rule: {rule:?}"
            ))),
        })
        .parse(pairs)
}
