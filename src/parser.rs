// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use pest::{iterators::Pairs, pratt_parser::PrattParser, Parser};
use pest_derive::Parser;
use std::fmt::Debug;
use std::str::FromStr;

use crate::formula::{Formula, Function, Op};
use crate::traits::NumberLike;
use crate::FormulaError;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct FormulaParser;

lazy_static::lazy_static! {
    pub static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        PrattParser::new()
            .op(Op::infix(add, Left) | Op::infix(sub, Left))
            .op(Op::infix(mul, Left) | Op::infix(div, Left))
            .op(Op::prefix(unary_minus))
            .op(Op::postfix(Rule::EOI))
    };
}

pub(crate) fn parse<T>(formula: &str) -> Result<Formula<T>, FormulaError>
where
    T: FromStr + NumberLike<T>,
    <T as FromStr>::Err: Debug,
{
    let pairs = FormulaParser::parse(Rule::formula, formula)?;
    parse_to_formula(pairs)
}

/// Parse a formula string into an expression tree.
impl<T> FromStr for Formula<T>
where
    T: FromStr + NumberLike<T>,
    <T as FromStr>::Err: Debug,
{
    type Err = FormulaError;

    fn from_str(formula: &str) -> Result<Self, Self::Err> {
        parse(formula)
    }
}

fn parse_to_formula<T>(pairs: Pairs<Rule>) -> Result<Formula<T>, FormulaError>
where
    T: FromStr + NumberLike<T>,
    <T as FromStr>::Err: Debug,
{
    PRATT_PARSER
        .map_primary(|primary| {
            Ok(match primary.as_rule() {
                Rule::none => Formula::Value(None),
                Rule::expr => parse_to_formula(primary.into_inner())?,
                Rule::num => primary
                    .as_str()
                    .parse()
                    .map(|num| Formula::Value(Some(num)))
                    .map_err(|e| FormulaError(format!("Invalid number: {e:?}")))?,
                Rule::component => primary
                    .as_str()
                    .replace("#", "")
                    .parse()
                    .map(Formula::Component)
                    .map_err(|e| FormulaError(format!("Invalid component id: {e:?}")))?,
                Rule::coalesce => Formula::Function {
                    function: Function::Coalesce,
                    args: primary
                        .into_inner()
                        .map(|x| parse_to_formula(Pairs::single(x)))
                        .collect::<Result<_, _>>()?,
                },
                Rule::min => Formula::Function {
                    function: Function::Min,
                    args: primary
                        .into_inner()
                        .map(|x| parse_to_formula(Pairs::single(x)))
                        .collect::<Result<_, _>>()?,
                },
                Rule::max => Formula::Function {
                    function: Function::Max,
                    args: primary
                        .into_inner()
                        .map(|x| parse_to_formula(Pairs::single(x)))
                        .collect::<Result<_, _>>()?,
                },
                rule => {
                    return Err(FormulaError(format!(
                        "parse: expected atom, found {rule:?}"
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
                            return Err(FormulaError(format!(
                                "parse: expected operator, found {rule:?}"
                            )))
                        }
                    },
                    rhs: Box::new(rhs),
                })
            } else {
                Err(FormulaError("Internal error".to_string()))
            }
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::unary_minus => {
                if let Ok(rhs) = rhs {
                    Ok(Formula::UnaryMinus(Box::new(rhs)))
                } else {
                    rhs
                }
            }
            rule => Err(FormulaError(format!(
                "parse: unexpected prefix rule: {rule:?}"
            ))),
        })
        .map_postfix(|lhs, op| match op.as_rule() {
            Rule::EOI => lhs,
            rule => Err(FormulaError(format!(
                "parse: unexpected postfix rule: {rule:?}"
            ))),
        })
        .parse(pairs)
}
