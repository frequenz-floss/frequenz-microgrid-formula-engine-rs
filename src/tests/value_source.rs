// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

use std::collections::HashMap;

use crate::value_source::{strict, Reading, ValueSource};
use crate::FormulaEngine;

#[test]
fn hashmap_source_reads_present_keys_and_undecided_for_missing() {
    let mut map: HashMap<u64, Option<f32>> = HashMap::from([(1, Some(2.0)), (2, None)]);
    assert_eq!(ValueSource::get(&mut map, &1), Reading::Value(Some(2.0)));
    assert_eq!(ValueSource::get(&mut map, &2), Reading::Value(None));
    assert_eq!(ValueSource::get(&mut map, &3), Reading::Undecided);
}

#[test]
fn evaluate_works_through_an_erased_source() {
    let mut map: HashMap<u64, Option<f32>> = HashMap::from([(1, Some(2.0))]);
    let mut erased: &mut dyn ValueSource<u64, f32> = &mut map;

    let fe = FormulaEngine::<f32>::try_new("#1 + 1").unwrap();
    assert_eq!(fe.evaluate(&mut erased).unwrap(), Reading::Value(Some(3.0)));
}

#[test]
fn reading_map_touches_only_present_values() {
    assert_eq!(
        Reading::Value(Some(2)).map(|v| v * 2),
        Reading::Value(Some(4))
    );
    assert_eq!(
        Reading::<i32>::Value(None).map(|v| v * 2),
        Reading::Value(None)
    );
    assert_eq!(Reading::<i32>::Undecided.map(|v| v * 2), Reading::Undecided);
}

#[test]
fn reading_and_then_can_turn_a_value_into_none() {
    assert_eq!(
        Reading::Value(Some(2)).and_then(|v| Some(v * 2)),
        Reading::Value(Some(4))
    );
    assert_eq!(
        Reading::Value(Some(2)).and_then(|_| None::<i32>),
        Reading::Value(None)
    );
    assert_eq!(
        Reading::<i32>::Value(None).and_then(|v| Some(v * 2)),
        Reading::Value(None)
    );
    assert_eq!(
        Reading::<i32>::Undecided.and_then(|_| None::<i32>),
        Reading::Undecided
    );
}

#[test]
fn reading_zip_prefers_undecided_over_none_over_values() {
    assert_eq!(
        Reading::Value(Some(1)).zip(Reading::Value(Some(2))),
        Reading::Value(Some((1, 2)))
    );
    assert_eq!(
        Reading::Value(Some(1)).zip(Reading::<i32>::Value(None)),
        Reading::Value(None)
    );
    assert_eq!(
        Reading::<i32>::Value(None).zip(Reading::<i32>::Undecided),
        Reading::Undecided
    );
    assert_eq!(
        Reading::<i32>::Undecided.zip(Reading::Value(Some(1))),
        Reading::Undecided
    );
}

#[test]
fn strict_prefers_undecided_over_none_over_values() {
    assert_eq!(
        strict([Reading::Value(Some(1)), Reading::Value(Some(2))]),
        Reading::Value(Some(vec![1, 2]))
    );
    assert_eq!(
        strict([Reading::Value(Some(1)), Reading::Value(None)]),
        Reading::Value(None)
    );
    assert_eq!(
        strict([
            Reading::Value(None),
            Reading::Undecided,
            Reading::Value(Some(1))
        ]),
        Reading::Undecided
    );
    assert_eq!(
        strict(Vec::<Reading<i32>>::new()),
        Reading::Value(Some(vec![]))
    );
}
