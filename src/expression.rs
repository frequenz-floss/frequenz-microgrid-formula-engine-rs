// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::{
    error::FormulaError,
    parser::{Rule, PRATT_PARSER},
    traits::NumberLike,
};
use pest::iterators::Pairs;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};
use std::{ops::Neg, str::FromStr};

#[derive(Debug)]
pub enum Expr<T> {
    Value(Option<T>),
    UnaryMinus(Box<Expr<T>>),
    Op {
        lhs: Box<Expr<T>>,
        op: Op,
        rhs: Box<Expr<T>>,
    },
    Function {
        function: Function,
        args: Vec<Expr<T>>,
    },
    Component(usize),
}

impl<T: FromStr> Expr<T>
where
    <T as FromStr>::Err: Debug,
{
    pub(crate) fn try_new(value: Pairs<Rule>) -> Result<Self, FormulaError> {
        PRATT_PARSER
            .map_primary(|primary| {
                Ok(match primary.as_rule() {
                    Rule::expr => Expr::try_new(primary.into_inner())?,
                    Rule::num => primary
                        .as_str()
                        .parse()
                        .map(|num| Expr::Value(Some(num)))
                        .map_err(|e| FormulaError(format!("Invalid number: {:?}", e)))?,
                    Rule::component => primary
                        .as_str()
                        .replace("#", "")
                        .parse()
                        .map(Expr::Component)
                        .map_err(|e| FormulaError(format!("Invalid component id: {:?}", e)))?,
                    Rule::coalesce => Expr::Function {
                        function: Function::Coalesce,
                        args: primary
                            .into_inner()
                            .map(|x| Expr::try_new(Pairs::single(x)))
                            .collect::<Result<_, _>>()?,
                    },
                    Rule::min => Expr::Function {
                        function: Function::Min,
                        args: primary
                            .into_inner()
                            .map(|x| Expr::try_new(Pairs::single(x)))
                            .collect::<Result<_, _>>()?,
                    },
                    Rule::max => Expr::Function {
                        function: Function::Max,
                        args: primary
                            .into_inner()
                            .map(|x| Expr::try_new(Pairs::single(x)))
                            .collect::<Result<_, _>>()?,
                    },
                    rule => {
                        return Err(FormulaError(format!(
                            "Expr::parse expected atom, found {:?}",
                            rule
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
                    Ok(Expr::Op {
                        lhs: Box::new(lhs),
                        op: match op.as_rule() {
                            Rule::add => Op::Add,
                            Rule::sub => Op::Sub,
                            Rule::mul => Op::Mul,
                            Rule::div => Op::Div,
                            rule => {
                                return Err(FormulaError(format!(
                                    "Expr::parse expected operator, found {:?}",
                                    rule
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
                        Ok(Expr::UnaryMinus(Box::new(rhs)))
                    } else {
                        rhs
                    }
                }
                rule => {
                    return Err(FormulaError(format!(
                        "Expr::parse unexpected prefix rule: {:?}",
                        rule
                    )))
                }
            })
            .map_postfix(|lhs, op| match op.as_rule() {
                Rule::EOI => lhs,
                rule => {
                    return Err(FormulaError(format!(
                        "Expr::parse unexpected postfix rule: {:?}",
                        rule
                    )))
                }
            })
            .parse(value)
    }
}

impl<T: NumberLike<T> + PartialOrd> Expr<T> {
    pub fn calculate(&self, values: &HashMap<usize, Option<T>>) -> Result<Option<T>, FormulaError> {
        Ok(match self {
            Expr::Value(value) => *value,
            Expr::UnaryMinus(expr) => expr.calculate(values)?.map(Neg::neg),
            Expr::Op { lhs, op, rhs } => op.apply(lhs.calculate(values)?, rhs.calculate(values)?),
            Expr::Function { function, args } => function.apply(
                &args
                    .iter()
                    .map(|expr| expr.calculate(values))
                    .collect::<Result<Vec<Option<T>>, FormulaError>>()?,
            ),
            Expr::Component(i) => values
                .get(i)
                .copied()
                .ok_or(FormulaError("Placeholder out of bounds".to_string()))?,
        })
    }

    pub fn components(&self) -> HashSet<usize> {
        match self {
            Expr::Value(_) => HashSet::new(),
            Expr::UnaryMinus(expr) => expr.components(),
            Expr::Op { lhs, rhs, .. } => {
                let mut components = lhs.components();
                components.extend(rhs.components());
                components
            }
            Expr::Function { args, .. } => args
                .iter()
                .map(Expr::components)
                .fold(HashSet::new(), |acc, x| acc.union(&x).copied().collect()),
            Expr::Component(i) => HashSet::from([*i]),
        }
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
    pub fn apply<T: NumberLike<T>>(&self, lhs: Option<T>, rhs: Option<T>) -> Option<T> {
        if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
            Some(match self {
                Op::Add => lhs + rhs,
                Op::Sub => lhs - rhs,
                Op::Mul => lhs * rhs,
                Op::Div => lhs / rhs,
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum Function {
    Coalesce,
    Min,
    Max,
}

impl Function {
    pub fn apply<T: Copy + PartialOrd>(&self, values: &[Option<T>]) -> Option<T> {
        match self {
            Function::Coalesce => values
                .iter()
                .copied()
                .find(Option::is_some)
                .unwrap_or_default(),
            // Option::min defines None as the smallest value, so we need to handle this case separately
            Function::Min => values.iter().copied().fold(None, |acc, x| match (acc, x) {
                (Some(acc), Some(x)) => match acc.partial_cmp(&x) {
                    Some(std::cmp::Ordering::Less) => Some(acc),
                    _ => Some(x),
                },
                (Some(acc), None) => Some(acc),
                (None, Some(x)) => Some(x),
                (None, None) => None,
            }),
            Function::Max => values.iter().copied().fold(None, |acc, x| match (acc, x) {
                (Some(acc), Some(x)) => match acc.partial_cmp(&x) {
                    Some(std::cmp::Ordering::Greater) => Some(acc),
                    _ => Some(x),
                },
                (Some(acc), None) => Some(acc),
                (None, Some(x)) => Some(x),
                (None, None) => None,
            }),
        }
    }
}
