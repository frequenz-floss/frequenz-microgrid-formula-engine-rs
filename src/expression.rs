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

impl<T: FromStr + Debug> TryFrom<Pairs<'_, Rule>> for Expr<T>
where
    <T as FromStr>::Err: Debug,
{
    type Error = FormulaError;

    fn try_from(value: Pairs<Rule>) -> Result<Self, Self::Error> {
        Ok(PRATT_PARSER
            .map_primary(|primary| match primary.as_rule() {
                Rule::expr => {
                    Expr::try_from(primary.into_inner()).unwrap_or_else(|_| Expr::Value(None))
                }
                Rule::num => Expr::Value(primary.as_str().parse().ok()),
                Rule::component => primary
                    .as_str()
                    .replace("#", "")
                    .parse()
                    .map(Expr::Component)
                    .unwrap_or_else(|_| Expr::Value(None)),
                Rule::coalesce => Expr::Function {
                    function: Function::Coalesce,
                    args: primary
                        .into_inner()
                        .map(|x| {
                            Expr::try_from(Pairs::single(x)).unwrap_or_else(|_| Expr::Value(None))
                        })
                        .collect(),
                },
                Rule::min => Expr::Function {
                    function: Function::Min,
                    args: primary
                        .into_inner()
                        .map(|x| {
                            Expr::try_from(Pairs::single(x)).unwrap_or_else(|_| Expr::Value(None))
                        })
                        .collect(),
                },
                Rule::max => Expr::Function {
                    function: Function::Max,
                    args: primary
                        .into_inner()
                        .map(|x| {
                            Expr::try_from(Pairs::single(x)).unwrap_or_else(|_| Expr::Value(None))
                        })
                        .collect(),
                },
                rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
            })
            .map_infix(|lhs, op, rhs| Expr::Op {
                lhs: Box::new(lhs),
                op: match op.as_rule() {
                    Rule::add => Op::Add,
                    Rule::sub => Op::Sub,
                    Rule::mul => Op::Mul,
                    Rule::div => Op::Div,
                    rule => unreachable!("Expr::parse expected operator, found {:?}", rule),
                },
                rhs: Box::new(rhs),
            })
            .map_prefix(|op, rhs| match op.as_rule() {
                Rule::unary_minus => Expr::UnaryMinus(Box::new(rhs)),
                _ => unreachable!(),
            })
            .map_postfix(|lhs, op| match op.as_rule() {
                Rule::EOI => lhs,
                _ => unreachable!(),
            })
            .parse(value))
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
