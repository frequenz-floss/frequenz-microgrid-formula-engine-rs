// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::{error::FormulaError, traits::NumberLike};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};
use std::{ops::Neg, str::FromStr};

#[derive(Debug)]
pub enum Formula<T> {
    Constant(Option<T>),
    UnaryMinus(Box<Formula<T>>),
    Op {
        lhs: Box<Formula<T>>,
        op: Op,
        rhs: Box<Formula<T>>,
    },
    Function {
        function: Function,
        args: Vec<Formula<T>>,
    },
    Component(u64),
}

impl<T: FromStr> Formula<T> where <T as FromStr>::Err: Debug {}

impl<T: NumberLike<T> + PartialOrd> Formula<T> {
    pub fn calculate(&self, values: &HashMap<u64, Option<T>>) -> Result<Option<T>, FormulaError> {
        Ok(match self {
            Formula::Constant(value) => *value,
            Formula::UnaryMinus(expr) => expr.calculate(values)?.map(Neg::neg),
            Formula::Op { lhs, op, rhs } => {
                op.apply(lhs.calculate(values)?, rhs.calculate(values)?)
            }
            Formula::Function { function, args } => function.apply(
                &args
                    .iter()
                    .map(|expr| expr.calculate(values))
                    .collect::<Result<Vec<Option<T>>, FormulaError>>()?,
            ),
            Formula::Component(i) => values
                .get(i)
                .copied()
                .ok_or(FormulaError("Placeholder out of bounds".to_string()))?,
        })
    }

    pub fn components(&self) -> HashSet<u64> {
        match self {
            Formula::Constant(_) => HashSet::new(),
            Formula::UnaryMinus(expr) => expr.components(),
            Formula::Op { lhs, rhs, .. } => {
                let mut components = lhs.components();
                components.extend(rhs.components());
                components
            }
            Formula::Function { args, .. } => args
                .iter()
                .map(Formula::components)
                .fold(HashSet::new(), |acc, x| acc.union(&x).copied().collect()),
            Formula::Component(i) => HashSet::from([*i]),
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
            // If any of the values is `None`, return `None` for Min/Max.
            Function::Min => values
                .iter()
                .copied()
                .reduce(|acc, x| match (acc, x) {
                    (Some(acc), Some(x)) => match acc.partial_cmp(&x) {
                        Some(std::cmp::Ordering::Less) => Some(acc),
                        _ => Some(x),
                    },
                    _ => None,
                })
                .unwrap_or_default(),
            Function::Max => values
                .iter()
                .copied()
                .reduce(|acc, x| match (acc, x) {
                    (Some(acc), Some(x)) => match acc.partial_cmp(&x) {
                        Some(std::cmp::Ordering::Greater) => Some(acc),
                        _ => Some(x),
                    },
                    _ => None,
                })
                .unwrap_or_default(),
        }
    }
}
