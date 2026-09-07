// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

use rand::Rng;
use std::{
    collections::HashMap,
    ops::{Add, Sub},
    vec,
};

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

#[test]
fn test_none_formula() {
    let fe = crate::parse::<f32>("None").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap(), None);

    let fe = crate::parse::<f32>("2 + None").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap(), None);
    let fe = crate::parse::<f32>("#2 + None").unwrap();
    assert_eq!(
        fe.calculate(&HashMap::from([(2, Some(12.))])).unwrap(),
        None
    );

    let fe = crate::parse::<f32>("MIN(None, None)").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap(), None);
    let fe = crate::parse::<f32>("MAX(None, None)").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap(), None);

    let fe = crate::parse::<f32>("MIN(None, 10, -10)").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap(), None);
    let fe = crate::parse::<f32>("MAX(None, 10, -10)").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap(), None);
    let fe = crate::parse::<f32>("COALESCE(None, 10)").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap(), Some(10.));
    let fe = crate::parse::<f32>("COALESCE(10, None, 12)").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap(), Some(10.));
    let fe = crate::parse::<f32>("COALESCE(#2, None, 12)").unwrap();
    assert_eq!(
        fe.calculate(&HashMap::from([(2, Some(2.))])).unwrap(),
        Some(2.)
    );
}

#[test]
fn test_parse_addition() {
    let fe = crate::parse::<f32>("1 + 1").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap().unwrap(), 1. + 1.);
}

#[test]
fn test_parse_multiplication() {
    let fe = crate::parse::<f32>("0.9 * 1.1").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap().unwrap(), 0.9 * 1.1);
}

#[test]
fn test_parse_subtraction() {
    let fe = crate::parse::<f32>("1 - 1").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap().unwrap(), 1. - 1.);
}

#[test]
fn test_parse_division() {
    let fe = crate::parse::<f32>("1 / 1").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap().unwrap(), 1. / 1.);
}

#[test]
fn test_parse_addition_whitespace() {
    let fe = crate::parse::<f32>("1+1").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap().unwrap(), 1. + 1.);
    let fe = crate::parse::<f32>("1+ 1").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap().unwrap(), 1. + 1.);
    let fe = crate::parse::<f32>("1 +1").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap().unwrap(), 1. + 1.);
}

#[test]
fn test_combination() {
    let fe = crate::parse::<f32>("1 + 1 * 2").unwrap();
    assert_eq!(
        fe.calculate(&HashMap::new()).unwrap().unwrap(),
        1. + 1. * 2.
    );
}

#[test]
fn test_combination_mul_add() {
    let fe = crate::parse::<f32>("2 * 1 + 2").unwrap();
    assert_eq!(
        fe.calculate(&HashMap::new()).unwrap().unwrap(),
        2. * 1. + 2.
    );
}

#[test]
fn test_negative_value() {
    let fe = crate::parse::<f32>("-1").unwrap();
    assert_eq!(fe.calculate(&HashMap::new()).unwrap().unwrap(), -1.);
}

#[test]
fn test_placeholder() {
    let fe = crate::parse::<f32>("#0").unwrap();
    assert_eq!(
        fe.calculate(&HashMap::from([(0, Some(1.))]))
            .unwrap()
            .unwrap(),
        1.
    );
}

#[test]
fn test_negative_placeholder() {
    let fe = crate::parse::<f32>("-#0").unwrap();
    assert_eq!(
        fe.calculate(&HashMap::from([(0, Some(1.))]))
            .unwrap()
            .unwrap(),
        -1.
    );
}

#[test]
fn test_invalid_placeholder() {
    let fe = crate::parse::<f32>("#1").unwrap();
    assert!(fe.calculate(&HashMap::from([(0, Some(1.))])).is_err());
}

#[test]
fn test_placeholder_addition() {
    let fe = crate::parse::<f32>("#0 + #1").unwrap();
    assert_eq!(
        fe.calculate(&HashMap::from([(0, Some(1.)), (1, Some(2.))]))
            .unwrap()
            .unwrap(),
        3.
    );
}

#[test]
fn test_calculating_with_nones() {
    let fe = crate::parse::<f32>("#0 + #1").unwrap();
    assert!(fe
        .calculate(&HashMap::from([(0, Some(1.)), (1, None)]))
        .unwrap()
        .is_none());
}

#[test]
fn test_function_coalesce() {
    let fe = crate::parse::<f32>("COALESCE(#0, #1,#2)").unwrap();
    assert_eq!(
        fe.calculate(&HashMap::from([(0, None), (1, Some(1.)), (2, Some(2.))]))
            .unwrap()
            .unwrap(),
        1.
    );
    let fe = crate::parse::<f32>("COALESCE(#0)").unwrap();
    assert_eq!(fe.calculate(&HashMap::from([(0, None)])).unwrap(), None);
}

#[test]
fn test_function_min() {
    let fe = crate::parse::<f32>("MIN(#0, #1,#2)").unwrap();
    assert_eq!(
        fe.calculate(&HashMap::from([
            (0, Some(3.)),
            (1, Some(1.)),
            (2, Some(2.))
        ]))
        .unwrap()
        .unwrap(),
        1.
    );
    let fe = crate::parse::<f32>("MIN(#1)").unwrap();
    assert_eq!(
        fe.calculate(&HashMap::from([(1, Some(1.)),])).unwrap(),
        Some(1.)
    );
    assert_eq!(fe.calculate(&HashMap::from([(1, None),])).unwrap(), None);
}

#[test]
fn test_function_min_none() {
    let fe = crate::parse::<f32>("MIN(#0, #1,#2)").unwrap();
    assert_eq!(
        fe.calculate(&HashMap::from([(0, None), (1, Some(1.)), (2, Some(2.))]))
            .unwrap(),
        None
    );
}

#[test]
fn test_function_max() {
    let fe = crate::parse::<f32>("MAX(#0, #1,#2)").unwrap();
    assert_eq!(
        fe.calculate(&HashMap::from([
            (0, Some(3.)),
            (1, Some(1.)),
            (2, Some(2.))
        ]))
        .unwrap()
        .unwrap(),
        3.
    );
    let fe = crate::parse::<f32>("MAX(#1)").unwrap();
    assert_eq!(
        fe.calculate(&HashMap::from([(1, Some(1.)),])).unwrap(),
        Some(1.)
    );
    assert_eq!(fe.calculate(&HashMap::from([(1, None),])).unwrap(), None);
}

#[test]
fn test_function_max_none() {
    let fe = crate::parse::<f32>("MAX(#0, #1,#2)").unwrap();
    assert_eq!(
        fe.calculate(&HashMap::from([(0, None), (1, None), (2, Some(2.))]))
            .unwrap(),
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
    let formula_result = crate::parse(concat!(
        "MIN(0.0, COALESCE(#4 + #3, #2, COALESCE(#4, 0.0) + COALESCE(#3, 0.0))) + ",
        "MIN(0.0, COALESCE(#6, #5, 0.0)) + ",
        "MIN(0.0, COALESCE(#7, 0.0))"
    ))
    .unwrap()
    .calculate(&components)
    .unwrap();

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
    let formula_result = crate::parse(concat!(
        "MAX(0.0, #1 - COALESCE(#2, #3, 0.0) - ",
        "COALESCE(#5, COALESCE(#7, 0.0) + COALESCE(#6, 0.0))) + ",
        "COALESCE(MAX(0.0, #2 - #3), 0.0) + COALESCE(MAX(0.0, #5 - #6 - #7), 0.0)",
    ))
    .unwrap()
    .calculate(&components)
    .unwrap();

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
