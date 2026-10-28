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

use crate::formula::{Formula, Function};
use crate::FormulaError;
use num_traits::Float;

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

/// The deepest nesting of parentheses and function calls parsing accepts. It
/// also bounds the depth of the parsed tree. A chain of `+` and `-`, or of `*`
/// and `/`, is one node however long it is, so each level of nesting adds only
/// a few levels to the tree.
pub const MAX_NESTING: usize = 128;

pub(crate) fn parse<T>(formula: &str) -> Result<Formula<T>, FormulaError>
where
    T: FromStr + Float,
{
    if nesting(formula) > MAX_NESTING {
        return Err(FormulaError::NestedTooDeep { limit: MAX_NESTING });
    }
    let pairs = FormulaParser::parse(Rule::formula, formula)?;
    parse_to_formula(pairs)
}

/// Parses a formula string into a formula with `u64` component keys. A
/// constant larger than `T` can hold is an error, and so is a formula nested
/// deeper than [`MAX_NESTING`] levels.
impl<T> FromStr for Formula<T>
where
    T: FromStr + Float,
{
    type Err = FormulaError;

    fn from_str(formula: &str) -> Result<Self, Self::Err> {
        parse(formula)
    }
}

/// The deepest run of unclosed `(` in `formula`. Every function call and
/// parenthesised group opens a `(`, so this bounds the depth of the parsed tree
/// and of every recursive walk over it.
fn nesting(formula: &str) -> usize {
    let mut depth = 0usize;
    let mut deepest = 0usize;
    for c in formula.chars() {
        match c {
            '(' => {
                depth += 1;
                deepest = deepest.max(depth);
            }
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    deepest
}

/// Builds `function` over the argument expressions inside a function rule.
fn function_call<T>(function: Function, call: Pair<Rule>) -> Result<Formula<T>, FormulaError>
where
    T: FromStr + Float,
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
    T: FromStr + Float,
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
                        .map_err(|_| FormulaError::InvalidConstant(primary.as_str().to_string()))?;
                    if !num.is_finite() {
                        return Err(FormulaError::ConstantOutOfRange(
                            primary.as_str().to_string(),
                        ));
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
                    return Err(FormulaError::InternalError(format!(
                        "expected atom, found {rule:?}"
                    )))
                }
            })
        })
        .map_infix(|lhs, op, rhs| {
            let (lhs, rhs) = (lhs?, rhs?);
            Ok(match op.as_rule() {
                Rule::add => lhs + rhs,
                Rule::sub => lhs - rhs,
                Rule::mul => lhs * rhs,
                Rule::div => lhs / rhs,
                rule => {
                    return Err(FormulaError::InternalError(format!(
                        "expected operator, found {rule:?}"
                    )))
                }
            })
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::unary_minus => Ok(-rhs?),
            rule => Err(FormulaError::InternalError(format!(
                "unexpected prefix rule: {rule:?}"
            ))),
        })
        .map_postfix(|lhs, op| match op.as_rule() {
            Rule::EOI => lhs,
            rule => Err(FormulaError::InternalError(format!(
                "unexpected postfix rule: {rule:?}"
            ))),
        })
        .parse(pairs)
}
