// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use crate::{error::FormulaError, traits::NumberLike};
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

impl<T: FromStr> Expr<T> where <T as FromStr>::Err: Debug {}

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
