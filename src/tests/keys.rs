// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

use std::collections::{HashMap, HashSet};

use super::parsed;
use crate::Formula;
use crate::Reading;

#[test]
fn map_components_retags_component_leaves_only() {
    let expr = parsed("COALESCE(#1, 2.5) + #2");
    let mapped = expr.map_components(|id| format!("power:{id}"));
    assert_eq!(
        mapped.components(),
        HashSet::from(["power:1".to_string(), "power:2".to_string()])
    );
    assert_eq!(mapped.to_string(), "COALESCE(#power:1, 2.5) + #power:2");

    let fe = mapped;
    let mut values = HashMap::from([
        ("power:1".to_string(), None),
        ("power:2".to_string(), Some(2.0)),
    ]);
    assert_eq!(fe.evaluate(&mut values).unwrap(), Reading::Known(Some(4.5)));
}

#[test]
fn map_components_keeps_negation() {
    let mapped = parsed("-#1 + COALESCE(-#2, 0)").map_components(|id| id + 10);
    let fe = mapped;
    let mut values = HashMap::from([(11, Some(1.0)), (12, Some(2.0))]);
    assert_eq!(
        fe.evaluate(&mut values).unwrap(),
        Reading::Known(Some(-3.0))
    );
}

#[test]
fn parsed_formula_reports_its_components() {
    assert_eq!(parsed("MIN(#3, #4)").components(), HashSet::from([3, 4]));
}

#[test]
fn default_key_is_u64() {
    let fe: Formula<f32> = crate::parse("#7").unwrap();
    assert_eq!(
        fe.evaluate(&mut HashMap::from([(7, Some(1.0))])).unwrap(),
        Reading::Known(Some(1.0))
    );
}
