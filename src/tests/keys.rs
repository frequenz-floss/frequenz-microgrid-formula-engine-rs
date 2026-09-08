// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

use std::collections::{HashMap, HashSet};

use crate::{parse, FormulaEngine, Reading};

#[test]
fn map_components_retags_component_leaves_only() {
    let expr = parse::<f32>("COALESCE(#1, 2.5) + #2").unwrap();
    let mapped = expr.map_components(|id| format!("power:{id}"));
    assert_eq!(
        mapped.components(),
        HashSet::from(["power:1".to_string(), "power:2".to_string()])
    );
    assert_eq!(mapped.to_string(), "COALESCE(#power:1, 2.5) + #power:2");

    let fe = FormulaEngine::from_expr(mapped);
    let mut values = HashMap::from([
        ("power:1".to_string(), None),
        ("power:2".to_string(), Some(2.0)),
    ]);
    assert_eq!(fe.evaluate(&mut values).unwrap(), Reading::Value(Some(4.5)));
}

#[test]
fn map_components_keeps_negation() {
    let mapped = parse::<f32>("-#1 + COALESCE(-#2, 0)")
        .unwrap()
        .map_components(|id| id + 10);
    let fe = FormulaEngine::from_expr(mapped);
    let mut values = HashMap::from([(11, Some(1.0)), (12, Some(2.0))]);
    assert_eq!(
        fe.evaluate(&mut values).unwrap(),
        Reading::Value(Some(-3.0))
    );
}

#[test]
fn engine_from_expr_exposes_the_expression_and_components() {
    let fe = FormulaEngine::from_expr(parse::<f32>("MIN(#3, #4)").unwrap());
    assert_eq!(fe.components(), &HashSet::from([3, 4]));
    assert_eq!(fe.expr().components(), HashSet::from([3, 4]));
}

#[test]
fn default_key_is_u64() {
    let fe: FormulaEngine<f32> = FormulaEngine::try_new("#7").unwrap();
    assert_eq!(
        fe.evaluate(&mut HashMap::from([(7, Some(1.0))])).unwrap(),
        Reading::Value(Some(1.0))
    );
}
