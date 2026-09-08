// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use rand::Rng;
use std::{
    collections::HashMap,
    ops::{Add, Sub},
    vec,
};

use crate::formula_engine::FormulaEngine;
use crate::Reading;

mod evaluate;
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

/// Parses `formula` and evaluates it over a map built from `values`.
fn eval(formula: &str, values: &[(u64, Option<f32>)]) -> Reading<f32> {
    let fe = FormulaEngine::<f32>::try_new(formula).unwrap();
    let mut source: HashMap<u64, Option<f32>> = values.iter().copied().collect();
    fe.evaluate(&mut source).unwrap()
}

/// Evaluates `fe` over `values`, panicking if the result is undecided.
fn calc(fe: &FormulaEngine<f32>, mut values: HashMap<u64, Option<f32>>) -> Option<f32> {
    match fe.evaluate(&mut values).unwrap() {
        Reading::Value(value) => value,
        Reading::Undecided => panic!("undecided"),
    }
}

#[test]
fn test_none_formula() {
    let fe = FormulaEngine::<f32>::try_new("None").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), None);

    let fe = FormulaEngine::<f32>::try_new("2 + None").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), None);
    let fe = FormulaEngine::<f32>::try_new("#2 + None").unwrap();
    assert_eq!(calc(&fe, HashMap::from([(2, Some(12.))])), None);

    let fe = FormulaEngine::<f32>::try_new("MIN(None, None)").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), None);
    let fe = FormulaEngine::<f32>::try_new("MAX(None, None)").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), None);

    let fe = FormulaEngine::<f32>::try_new("MIN(None, 10, -10)").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), None);
    let fe = FormulaEngine::<f32>::try_new("MAX(None, 10, -10)").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), None);
    let fe = FormulaEngine::<f32>::try_new("COALESCE(None, 10)").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), Some(10.));
    let fe = FormulaEngine::<f32>::try_new("COALESCE(10, None, 12)").unwrap();
    assert_eq!(calc(&fe, HashMap::new()), Some(10.));
    let fe = FormulaEngine::<f32>::try_new("COALESCE(#2, None, 12)").unwrap();
    assert_eq!(calc(&fe, HashMap::from([(2, Some(2.))])), Some(2.));
}

#[test]
fn test_parse_addition() {
    let fe = FormulaEngine::<f32>::try_new("1 + 1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 1. + 1.);
}

#[test]
fn test_parse_multiplication() {
    let fe = FormulaEngine::<f32>::try_new("0.9 * 1.1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 0.9 * 1.1);
}

#[test]
fn test_parse_subtraction() {
    let fe = FormulaEngine::<f32>::try_new("1 - 1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 1. - 1.);
}

#[test]
fn test_parse_division() {
    let fe = FormulaEngine::<f32>::try_new("1 / 1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 1. / 1.);
}

#[test]
fn test_parse_addition_whitespace() {
    let fe = FormulaEngine::<f32>::try_new("1+1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 1. + 1.);
    let fe = FormulaEngine::<f32>::try_new("1+ 1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 1. + 1.);
    let fe = FormulaEngine::<f32>::try_new("1 +1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 1. + 1.);
}

#[test]
fn test_combination() {
    let fe = FormulaEngine::<f32>::try_new("1 + 1 * 2").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 1. + 1. * 2.);
}

#[test]
fn test_combination_mul_add() {
    let fe = FormulaEngine::<f32>::try_new("2 * 1 + 2").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), 2. * 1. + 2.);
}

#[test]
fn test_negative_value() {
    let fe = FormulaEngine::<f32>::try_new("-1").unwrap();
    assert_eq!(calc(&fe, HashMap::new()).unwrap(), -1.);
}

#[test]
fn test_placeholder() {
    let fe = FormulaEngine::<f32>::try_new("#0").unwrap();
    assert_eq!(calc(&fe, HashMap::from([(0, Some(1.))])).unwrap(), 1.);
}

#[test]
fn test_negative_placeholder() {
    let fe = FormulaEngine::<f32>::try_new("-#0").unwrap();
    assert_eq!(calc(&fe, HashMap::from([(0, Some(1.))])).unwrap(), -1.);
}

#[test]
fn test_missing_placeholder_is_undecided() {
    let fe = FormulaEngine::<f32>::try_new("#1").unwrap();
    assert_eq!(
        fe.evaluate(&mut HashMap::from([(0, Some(1.))])).unwrap(),
        Reading::Undecided
    );
}

#[test]
fn test_placeholder_addition() {
    let fe = FormulaEngine::<f32>::try_new("#0 + #1").unwrap();
    assert_eq!(
        calc(&fe, HashMap::from([(0, Some(1.)), (1, Some(2.))])).unwrap(),
        3.
    );
}

#[test]
fn test_calculating_with_nones() {
    let fe = FormulaEngine::<f32>::try_new("#0 + #1").unwrap();
    assert!(calc(&fe, HashMap::from([(0, Some(1.)), (1, None)])).is_none());
}

#[test]
fn test_function_coalesce() {
    let fe = FormulaEngine::<f32>::try_new("COALESCE(#0, #1,#2)").unwrap();
    assert_eq!(
        calc(
            &fe,
            HashMap::from([(0, None), (1, Some(1.)), (2, Some(2.))])
        )
        .unwrap(),
        1.
    );
    let fe = FormulaEngine::<f32>::try_new("COALESCE(#0)").unwrap();
    assert_eq!(calc(&fe, HashMap::from([(0, None)])), None);
}

#[test]
fn test_function_min() {
    let fe = FormulaEngine::<f32>::try_new("MIN(#0, #1,#2)").unwrap();
    assert_eq!(
        calc(
            &fe,
            HashMap::from([(0, Some(3.)), (1, Some(1.)), (2, Some(2.))])
        )
        .unwrap(),
        1.
    );
    let fe = FormulaEngine::<f32>::try_new("MIN(#1)").unwrap();
    assert_eq!(calc(&fe, HashMap::from([(1, Some(1.))])), Some(1.));
    assert_eq!(calc(&fe, HashMap::from([(1, None)])), None);
}

#[test]
fn test_function_min_none() {
    let fe = FormulaEngine::<f32>::try_new("MIN(#0, #1,#2)").unwrap();
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
    let fe = FormulaEngine::<f32>::try_new("MAX(#0, #1,#2)").unwrap();
    assert_eq!(
        calc(
            &fe,
            HashMap::from([(0, Some(3.)), (1, Some(1.)), (2, Some(2.))])
        )
        .unwrap(),
        3.
    );
    let fe = FormulaEngine::<f32>::try_new("MAX(#1)").unwrap();
    assert_eq!(calc(&fe, HashMap::from([(1, Some(1.))])), Some(1.));
    assert_eq!(calc(&fe, HashMap::from([(1, None)])), None);
}

#[test]
fn test_function_max_none() {
    let fe = FormulaEngine::<f32>::try_new("MAX(#0, #1,#2)").unwrap();
    assert_eq!(
        calc(&fe, HashMap::from([(0, None), (1, None), (2, Some(2.))])),
        None
    );
}

#[test]
fn test_components_getter_op() {
    let fe = FormulaEngine::<f32>::try_new("#0 + #1").unwrap();
    assert_eq!(fe.components(), &vec![0, 1].into_iter().collect());
}

#[test]
fn test_components_getter_neg() {
    let fe = FormulaEngine::<f32>::try_new("#0 + (-#1)").unwrap();
    assert_eq!(fe.components(), &vec![0, 1].into_iter().collect());
}

#[test]
fn test_components_getter_function() {
    let fe = FormulaEngine::<f32>::try_new("-MAX(#0, #1)").unwrap();
    assert_eq!(fe.components(), &vec![0, 1].into_iter().collect());
}

#[test]
fn test_components_getter_function_function() {
    let fe = FormulaEngine::<f32>::try_new("MAX(#0, COALESCE(#1, #2))").unwrap();
    assert_eq!(fe.components(), &vec![0, 1, 2].into_iter().collect());
}

fn test_large_microgrid_formula(components: HashMap<u64, Option<f32>>) {
    let formula_result = calc(
        &FormulaEngine::try_new(concat!(
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
        &FormulaEngine::try_new(concat!(
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
fn test_number_like_f32_and_f64() {
    use crate::traits::NumberLike;
    assert_eq!(<f32 as NumberLike>::zero(), 0.0);
    assert_eq!(<f32 as NumberLike>::from_usize(3), 3.0);
    assert_eq!(NumberLike::sqrt(9.0_f32), 3.0);
    assert_eq!(<f64 as NumberLike>::zero(), 0.0);
    assert_eq!(<f64 as NumberLike>::from_usize(3), 3.0);
    assert_eq!(NumberLike::sqrt(9.0_f64), 3.0);
}
