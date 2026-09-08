// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

//! The demand contract: `evaluate` reads every component it needs under
//! the lazy-COALESCE semantics and no other.

use std::collections::{HashMap, HashSet};

use crate::{FormulaEngine, Reading, ValueSource};

struct Recording {
    values: HashMap<u64, Option<f32>>,
    read: Vec<u64>,
}

impl ValueSource<u64, f32> for Recording {
    fn get(&mut self, key: &u64) -> Reading<f32> {
        self.read.push(*key);
        ValueSource::get(&mut self.values, key)
    }
}

/// Evaluates `formula` over `values` and returns the result alongside the
/// set of keys read.
fn reads(formula: &str, values: &[(u64, Option<f32>)]) -> (Reading<f32>, HashSet<u64>) {
    let fe = FormulaEngine::<f32>::try_new(formula).unwrap();
    let mut source = Recording {
        values: values.iter().copied().collect(),
        read: vec![],
    };
    let result = fe.evaluate(&mut source).unwrap();
    (result, source.read.into_iter().collect())
}

const SMALL_SITE: &str =
    "COALESCE(#1007, COALESCE(#1003, 0.0) + COALESCE(#1002, 0.0) + COALESCE(#1001, 0.0))";

#[test]
fn coalesce_reads_only_up_to_the_first_value() {
    assert_eq!(
        reads("COALESCE(#1, #2)", &[(1, Some(1.0))]).1,
        HashSet::from([1])
    );
    assert_eq!(
        reads("COALESCE(#1, #2)", &[(1, None), (2, Some(1.0))]).1,
        HashSet::from([1, 2])
    );
}

#[test]
fn coalesce_reads_only_up_to_the_first_undecided() {
    assert_eq!(reads("COALESCE(#1, #2)", &[]).1, HashSet::from([1]));
    assert_eq!(
        reads("COALESCE(#1, #2, #3)", &[(1, None)]).1,
        HashSet::from([1, 2])
    );
}

#[test]
fn strict_nodes_read_every_operand() {
    assert_eq!(reads("#1 + #2", &[(1, None)]).1, HashSet::from([1, 2]));
    assert_eq!(reads("#1 + #2", &[]).1, HashSet::from([1, 2]));
    assert_eq!(
        reads("MIN(#1, #2, #3)", &[(1, None)]).1,
        HashSet::from([1, 2, 3])
    );
    assert_eq!(reads("AVG(#1, #2)", &[(2, None)]).1, HashSet::from([1, 2]));
    assert_eq!(reads("SQRT(#1 * #2)", &[]).1, HashSet::from([1, 2]));
}

#[test]
fn small_site_formula_demands_only_the_primary_while_it_delivers() {
    assert_eq!(
        reads(SMALL_SITE, &[(1007, Some(5.0))]),
        (Reading::Value(Some(5.0)), HashSet::from([1007]))
    );
    assert_eq!(
        reads(SMALL_SITE, &[]),
        (Reading::Undecided, HashSet::from([1007]))
    );
}

#[test]
fn small_site_formula_demands_all_fallbacks_once_the_primary_is_none() {
    // The sum is strict, so all three fallbacks are read in one evaluation
    // whatever they read. Here the first fallback is itself undecided, so
    // the whole sum -- and thus the formula -- is undecided.
    assert_eq!(
        reads(SMALL_SITE, &[(1007, None)]),
        (Reading::Undecided, HashSet::from([1007, 1003, 1002, 1001]))
    );
    assert_eq!(
        reads(
            SMALL_SITE,
            &[
                (1007, None),
                (1003, Some(1.0)),
                (1002, None),
                (1001, Some(2.0))
            ]
        ),
        (
            Reading::Value(Some(3.0)),
            HashSet::from([1007, 1003, 1002, 1001])
        )
    );
}

#[test]
fn a_component_referenced_twice_may_be_read_twice() {
    let fe = FormulaEngine::<f32>::try_new("#1 * #1").unwrap();
    let mut source = Recording {
        values: HashMap::from([(1, Some(2.0))]),
        read: vec![],
    };
    assert_eq!(fe.evaluate(&mut source).unwrap(), Reading::Value(Some(4.0)));
    assert_eq!(source.read, vec![1, 1]);
}
