// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

use std::collections::HashSet;

use crate::{parse, Expr};

fn c(id: u64) -> Expr<f32> {
    Expr::component(id)
}

fn k(v: f32) -> Expr<f32> {
    Expr::value(Some(v))
}

fn parsed(formula: &str) -> Expr<f32> {
    parse::<f32>(formula).unwrap()
}

#[test]
fn operators_build_the_same_tree_as_the_parser() {
    assert_eq!(c(1) + c(2), parsed("#1 + #2"));
    assert_eq!(c(1) - k(2.0), parsed("#1 - 2"));
    assert_eq!(c(1) * c(2) / c(3), parsed("#1 * #2 / #3"));
    assert_eq!(-c(1), parsed("-#1"));
    assert_eq!(-(c(1) + c(2)), parsed("-(#1 + #2)"));
    assert_eq!((c(1) + c(2)) * k(3.0), parsed("(#1 + #2) * 3"));
    assert_eq!(Expr::<f32>::value(None) + c(1), parsed("None + #1"));
}

#[test]
fn function_builders_flatten_chains_of_the_same_function() {
    assert_eq!(
        c(1).coalesce(c(2)).coalesce(k(0.0)),
        parsed("COALESCE(#1, #2, 0)")
    );
    assert_eq!(c(1).min(c(2)).min(c(3)), parsed("MIN(#1, #2, #3)"));
    assert_eq!(c(1).max(c(2)), parsed("MAX(#1, #2)"));
    assert_eq!(c(1).avg(vec![c(2), c(3)]), parsed("AVG(#1, #2, #3)"));
    assert_eq!(c(1).avg(vec![]), parsed("AVG(#1)"));
    assert_eq!(c(1).sqrt(), parsed("SQRT(#1)"));
}

#[test]
fn avg_does_not_flatten_across_calls() {
    assert_eq!(
        c(1).avg(vec![c(2)]).avg(vec![c(3)]),
        parsed("AVG(AVG(#1, #2), #3)")
    );
}

#[test]
fn function_builders_do_not_flatten_across_functions_or_arguments() {
    assert_eq!(
        c(1).min(c(2)).coalesce(c(3)),
        parsed("COALESCE(MIN(#1, #2), #3)")
    );
    assert_eq!(
        c(1).coalesce(c(2).coalesce(c(3))),
        parsed("COALESCE(#1, COALESCE(#2, #3))")
    );
}

#[test]
fn builders_work_for_any_key_type() {
    let expr = Expr::<f32, String>::component("v1".to_string()) * Expr::component("i1".to_string());
    assert_eq!(
        expr.components(),
        HashSet::from(["v1".to_string(), "i1".to_string()])
    );
    assert_eq!(expr.to_string(), "#v1 * #i1");
}
