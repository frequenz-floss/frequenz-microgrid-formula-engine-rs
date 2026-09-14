// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use rand::Rng;
use std::{
    collections::HashMap,
    ops::{Add, Sub},
    vec,
};

use crate::formula::Formula;
use crate::{parse, FormulaError, Reading};

mod evaluate;
mod functions;
mod keys;
mod value_source;

fn max<T>(a: OptionW<T>, b: OptionW<T>) -> OptionW<T>
where
    T: PartialOrd,
{
    OptionW(match (a.inner(), b.inner()) {
        (Some(a), Some(b)) => {
            if a > b {
                Some(a)
            } else {
                Some(b)
            }
        }
        _ => None,
    })
}

fn min<T>(a: OptionW<T>, b: OptionW<T>) -> OptionW<T>
where
    T: PartialOrd,
{
    OptionW(match (a.inner(), b.inner()) {
        (Some(a), Some(b)) => {
            if a < b {
                Some(a)
            } else {
                Some(b)
            }
        }
        _ => None,
    })
}

fn coalesce<T>(list: Vec<OptionW<T>>) -> OptionW<T> {
    list.into_iter()
        .find(|x| x.is_some())
        .unwrap_or(OptionW(None))
}

#[derive(Debug, Clone)]
struct OptionW<T>(Option<T>);

impl<T> OptionW<T> {
    fn inner(self) -> Option<T> {
        self.0
    }

    fn is_some(&self) -> bool {
        self.0.is_some()
    }
}

impl Add for OptionW<f32> {
    type Output = OptionW<f32>;

    fn add(self, other: Self) -> Self::Output {
        OptionW(match (self.inner(), other.inner()) {
            (Some(a), Some(b)) => Some(a + b),
            _ => None,
        })
    }
}

impl Sub for OptionW<f32> {
    type Output = OptionW<f32>;

    fn sub(self, other: Self) -> Self::Output {
        OptionW(match (self.inner(), other.inner()) {
            (Some(a), Some(b)) => Some(a - b),
            _ => None,
        })
    }
}

/// Parses `formula` with `f32` constants and `u64` keys.
fn parsed(formula: &str) -> Formula<f32> {
    parse::<f32>(formula).unwrap()
}

/// Parses `formula` and evaluates it over a map built from `values`.
fn eval(formula: &str, values: &[(u64, Option<f32>)]) -> Reading<f32> {
    let fe = crate::parse::<f32>(formula).unwrap();
    let mut source: HashMap<u64, Option<f32>> = values.iter().copied().collect();
    fe.evaluate(&mut source).unwrap()
}

/// Evaluates `fe` over `values`, panicking if the result is unknown.
fn calc(fe: &Formula<f32>, mut values: HashMap<u64, Option<f32>>) -> Option<f32> {
    match fe.evaluate(&mut values).unwrap() {
        Reading::Known(value) => value,
        Reading::Unknown => panic!("unknown"),
    }
}

#[test]
fn test_none_formula() {
    let fe = crate::parse::<f32>("None").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), None);

    let fe = crate::parse::<f32>("2 + None").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), None);
    let fe = crate::parse::<f32>("#2 + None").unwrap();
    assert_eq!(calc(&fe, HashMap::from([(2, Some(12.))])), None);

    let fe = crate::parse::<f32>("MIN(None, None)").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), None);
    let fe = crate::parse::<f32>("MAX(None, None)").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), None);

    let fe = crate::parse::<f32>("MIN(None, 10, -10)").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), None);
    let fe = crate::parse::<f32>("MAX(None, 10, -10)").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), None);
    let fe = crate::parse::<f32>("COALESCE(None, 10)").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), Some(10.));
    let fe = crate::parse::<f32>("COALESCE(10, None, 12)").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), Some(10.));
    let fe = crate::parse::<f32>("COALESCE(#2, None, 12)").unwrap();
    assert_eq!(calc(&fe, HashMap::from([(2, Some(2.))])), Some(2.));
}

#[test]
fn test_parse_addition() {
    let fe = crate::parse::<f32>("1 + 1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 1. + 1.);
}

#[test]
fn test_parse_multiplication() {
    let fe = crate::parse::<f32>("0.9 * 1.1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 0.9 * 1.1);
}

#[test]
fn test_parse_subtraction() {
    let fe = crate::parse::<f32>("1 - 1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 1. - 1.);
}

#[test]
fn test_parse_division() {
    let fe = crate::parse::<f32>("1 / 1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 1. / 1.);
}

#[test]
fn test_parse_addition_whitespace() {
    let fe = crate::parse::<f32>("1+1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 1. + 1.);
    let fe = crate::parse::<f32>("1+ 1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 1. + 1.);
    let fe = crate::parse::<f32>("1 +1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 1. + 1.);
}

#[test]
fn test_combination() {
    let fe = crate::parse::<f32>("1 + 1 * 2").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 1. + 1. * 2.);
}

#[test]
fn test_combination_mul_add() {
    let fe = crate::parse::<f32>("2 * 1 + 2").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 2. * 1. + 2.);
}

#[test]
fn test_negative_value() {
    let fe = crate::parse::<f32>("-1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), -1.);
}

#[test]
fn test_placeholder() {
    let fe = crate::parse::<f32>("#0").unwrap();
    assert_eq!(calc(&fe, HashMap::from([(0, Some(1.))])).unwrap(), 1.);
}

#[test]
fn test_negative_placeholder() {
    let fe = crate::parse::<f32>("-#0").unwrap();
    assert_eq!(calc(&fe, HashMap::from([(0, Some(1.))])).unwrap(), -1.);
}

#[test]
fn test_missing_placeholder_is_unknown() {
    let fe = crate::parse::<f32>("#1").unwrap();
    assert_eq!(
        fe.evaluate(&mut HashMap::from([(0, Some(1.))])).unwrap(),
        Reading::Unknown
    );
}

#[test]
fn test_placeholder_addition() {
    let fe = crate::parse::<f32>("#0 + #1").unwrap();
    assert_eq!(
        calc(&fe, HashMap::from([(0, Some(1.)), (1, Some(2.))])).unwrap(),
        3.
    );
}

#[test]
fn test_calculating_with_nones() {
    let fe = crate::parse::<f32>("#0 + #1").unwrap();
    assert!(calc(&fe, HashMap::from([(0, Some(1.)), (1, None)])).is_none());
}

#[test]
fn test_function_coalesce() {
    let fe = crate::parse::<f32>("COALESCE(#0, #1,#2)").unwrap();
    assert_eq!(
        calc(
            &fe,
            HashMap::from([(0, None), (1, Some(1.)), (2, Some(2.))])
        )
        .unwrap(),
        1.
    );
    let fe = crate::parse::<f32>("COALESCE(#0)").unwrap();
    assert_eq!(calc(&fe, HashMap::from([(0, None)])), None);
}

#[test]
fn test_function_min() {
    let fe = crate::parse::<f32>("MIN(#0, #1,#2)").unwrap();
    assert_eq!(
        calc(
            &fe,
            HashMap::from([(0, Some(3.)), (1, Some(1.)), (2, Some(2.))])
        )
        .unwrap(),
        1.
    );
    let fe = crate::parse::<f32>("MIN(#1)").unwrap();
    assert_eq!(calc(&fe, HashMap::from([(1, Some(1.))])), Some(1.));
    assert_eq!(calc(&fe, HashMap::from([(1, None)])), None);
}

#[test]
fn test_function_min_none() {
    let fe = crate::parse::<f32>("MIN(#0, #1,#2)").unwrap();
    assert_eq!(
        calc(
            &fe,
            HashMap::from([(0, None), (1, Some(1.)), (2, Some(2.))])
        ),
        None
    );
}

#[test]
fn test_function_max() {
    let fe = crate::parse::<f32>("MAX(#0, #1,#2)").unwrap();
    assert_eq!(
        calc(
            &fe,
            HashMap::from([(0, Some(3.)), (1, Some(1.)), (2, Some(2.))])
        )
        .unwrap(),
        3.
    );
    let fe = crate::parse::<f32>("MAX(#1)").unwrap();
    assert_eq!(calc(&fe, HashMap::from([(1, Some(1.))])), Some(1.));
    assert_eq!(calc(&fe, HashMap::from([(1, None)])), None);
}

#[test]
fn test_function_max_none() {
    let fe = crate::parse::<f32>("MAX(#0, #1,#2)").unwrap();
    assert_eq!(
        calc(&fe, HashMap::from([(0, None), (1, None), (2, Some(2.))])),
        None
    );
}

#[test]
fn test_components_getter_op() {
    let fe = crate::parse::<f32>("#0 + #1").unwrap();
    assert_eq!(fe.components(), vec![0, 1].into_iter().collect());
}

#[test]
fn test_components_getter_neg() {
    let fe = crate::parse::<f32>("#0 + (-#1)").unwrap();
    assert_eq!(fe.components(), vec![0, 1].into_iter().collect());
}

#[test]
fn test_components_getter_function() {
    let fe = crate::parse::<f32>("-MAX(#0, #1)").unwrap();
    assert_eq!(fe.components(), vec![0, 1].into_iter().collect());
}

#[test]
fn test_components_getter_function_function() {
    let fe = crate::parse::<f32>("MAX(#0, COALESCE(#1, #2))").unwrap();
    assert_eq!(fe.components(), vec![0, 1, 2].into_iter().collect());
}

fn test_large_microgrid_formula(components: HashMap<u64, Option<f32>>) {
    let formula_result = calc(
        &crate::parse(concat!(
            "MIN(0.0, COALESCE(#4 + #3, #2, COALESCE(#4, 0.0) + COALESCE(#3, 0.0))) + ",
            "MIN(0.0, COALESCE(#6, #5, 0.0)) + ",
            "MIN(0.0, COALESCE(#7, 0.0))"
        ))
        .unwrap(),
        components.clone(),
    );

    let expected_result = min(
        OptionW(Some(0.0)),
        coalesce(vec![
            OptionW(*components.get(&4).unwrap()) + OptionW(*components.get(&3).unwrap()),
            OptionW(*components.get(&2).unwrap()),
            coalesce(vec![
                OptionW(*components.get(&4).unwrap()),
                OptionW(Some(0.0)),
            ]) + coalesce(vec![
                OptionW(*components.get(&3).unwrap()),
                OptionW(Some(0.0)),
            ]),
        ]),
    ) + min(
        OptionW(Some(0.0)),
        coalesce(vec![
            OptionW(*components.get(&6).unwrap()),
            OptionW(*components.get(&5).unwrap()),
            OptionW(Some(0.0)),
        ]),
    ) + min(
        OptionW(Some(0.0)),
        coalesce(vec![
            OptionW(*components.get(&7).unwrap()),
            OptionW(Some(0.0)),
        ]),
    );

    assert_eq!(formula_result, expected_result.inner());
}

#[test]
fn test_large_microgrid_formula_fuzz() {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let mut components = HashMap::new();
        for i in 2..8 {
            let value = if rng.gen_bool(0.5) {
                Some(0.5 - rng.gen::<f32>())
            } else {
                None
            };
            components.insert(i, value);
        }
        test_large_microgrid_formula(components);
    }
}

fn test_large_microgrid_formula_2(components: HashMap<u64, Option<f32>>) {
    let formula_result = calc(
        &crate::parse(concat!(
            "MAX(0.0, #1 - COALESCE(#2, #3, 0.0) - ",
            "COALESCE(#5, COALESCE(#7, 0.0) + COALESCE(#6, 0.0))) + ",
            "COALESCE(MAX(0.0, #2 - #3), 0.0) + COALESCE(MAX(0.0, #5 - #6 - #7), 0.0)",
        ))
        .unwrap(),
        components.clone(),
    );

    let expected_result = max(
        OptionW(Some(0.0)),
        OptionW(*components.get(&1).unwrap())
            - coalesce(vec![
                OptionW(*components.get(&2).unwrap()),
                OptionW(*components.get(&3).unwrap()),
                OptionW(Some(0.0)),
            ])
            - coalesce(vec![
                OptionW(*components.get(&5).unwrap()),
                coalesce(vec![
                    OptionW(*components.get(&7).unwrap()),
                    OptionW(Some(0.0)),
                ]) + coalesce(vec![
                    OptionW(*components.get(&6).unwrap()),
                    OptionW(Some(0.0)),
                ]),
            ]),
    ) + coalesce(vec![
        max(
            OptionW(Some(0.0)),
            OptionW(*components.get(&2).unwrap()) - OptionW(*components.get(&3).unwrap()),
        ),
        OptionW(Some(0.0)),
    ]) + coalesce(vec![
        max(
            OptionW(Some(0.0)),
            OptionW(*components.get(&5).unwrap())
                - OptionW(*components.get(&6).unwrap())
                - OptionW(*components.get(&7).unwrap()),
        ),
        OptionW(Some(0.0)),
    ]);

    assert_eq!(formula_result, expected_result.inner());
}

#[test]
fn test_large_microgrid_formula_2_fuzz() {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let mut components = HashMap::new();
        for i in 1..8 {
            let value = if rng.gen_bool(0.5) {
                Some(0.5 - rng.gen::<f32>())
            } else {
                None
            };
            components.insert(i, value);
        }
        test_large_microgrid_formula_2(components);
    }
}

#[test]
fn test_constant_out_of_range_is_a_parse_error() {
    let big = "1".repeat(40);
    let err = crate::parse::<f32>(&big).unwrap_err();
    assert_eq!(err, FormulaError::NumberOutOfRange(big.clone()));
    assert_eq!(err.to_string(), format!("Number out of range: {big}"));
    assert!(crate::parse::<f64>(&big).is_ok());
    let huge = "9".repeat(400);
    assert!(crate::parse::<f64>(&huge).is_err());
    assert!(crate::parse::<f32>(&f32::MAX.to_string()).is_ok());
    assert!(crate::parse::<f64>(&f64::MAX.to_string()).is_ok());
}

#[test]
fn test_formulas_deeper_than_the_limits_are_a_parse_error() {
    let nested = |n: usize| format!("{}#1{}", "(".repeat(n), ")".repeat(n));
    assert!(crate::parse::<f32>(&nested(crate::MAX_NESTING)).is_ok());
    let err = crate::parse::<f32>(&nested(crate::MAX_NESTING + 1)).unwrap_err();
    assert_eq!(err, FormulaError::NestedTooDeep { limit: 128 });
    assert_eq!(err.to_string(), "Formula nests deeper than 128 levels");
    let coalesced = |n: usize| format!("{}#1{}", "COALESCE(".repeat(n), ", 0)".repeat(n));
    assert!(crate::parse::<f32>(&coalesced(crate::MAX_NESTING + 1)).is_err());

    // Every operator counts as a level, whichever it is.
    let chain = |n: usize| {
        let ops = ["+", "-", "*", "/"];
        (1..n).fold("#1".to_string(), |s, i| format!("{s} {} #1", ops[i % 4]))
    };
    assert!(crate::parse::<f32>(&chain(crate::MAX_DEPTH)).is_ok());
    let err = crate::parse::<f32>(&chain(crate::MAX_DEPTH + 1)).unwrap_err();
    assert_eq!(err, FormulaError::TooDeep { limit: 1024 });
    assert_eq!(err.to_string(), "Formula is deeper than 1024 levels");

    // Siblings do not nest: closing a group must give its level back, and
    // a deep group followed by a shallow one is still too deep.
    let siblings = vec!["COALESCE(#1, 0)"; 200].join(" + ");
    assert!(crate::parse::<f32>(&siblings).is_ok());
    let deep_then_shallow = format!("{} + (#1)", nested(crate::MAX_NESTING + 1));
    assert!(crate::parse::<f32>(&deep_then_shallow).is_err());
}

#[test]
fn test_invalid_literals_are_named_in_the_error() {
    let err = crate::parse::<f32>("#99999999999999999999").unwrap_err();
    assert_eq!(
        err,
        FormulaError::InvalidComponentId("#99999999999999999999".to_string())
    );
    assert_eq!(
        err.to_string(),
        "Invalid component id: #99999999999999999999"
    );
    let err = crate::parse::<f32>("1.2.3").unwrap_err();
    assert_eq!(err, FormulaError::InvalidNumber("1.2.3".to_string()));
    assert_eq!(err.to_string(), "Invalid number: 1.2.3");
}
