// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

use std::collections::HashMap;

use super::eval;
use crate::expression::{Expr, Function};
use crate::Reading;

#[test]
fn coalesce_returns_first_some_without_reading_further() {
    // #2 is not in the map, so reading it would yield Undecided.
    assert_eq!(
        eval("COALESCE(#1, #2)", &[(1, Some(1.0))]),
        Reading::Value(Some(1.0))
    );
}

#[test]
fn coalesce_moves_past_none() {
    assert_eq!(
        eval("COALESCE(#1, 0.0)", &[(1, None)]),
        Reading::Value(Some(0.0))
    );
    assert_eq!(
        eval("COALESCE(#1, #2)", &[(1, None), (2, None)]),
        Reading::Value(None)
    );
}

#[test]
fn coalesce_stops_at_undecided() {
    assert_eq!(eval("COALESCE(#1, 0.0)", &[]), Reading::Undecided);
    assert_eq!(
        eval("COALESCE(#1, #2, 0.0)", &[(1, None)]),
        Reading::Undecided
    );
}

#[test]
fn strict_nodes_yield_undecided_over_none() {
    assert_eq!(eval("#1 + #2", &[(1, None)]), Reading::Undecided);
    assert_eq!(eval("MIN(#1, #2)", &[(1, None)]), Reading::Undecided);
    assert_eq!(eval("-#1", &[]), Reading::Undecided);
}

#[test]
fn strict_nodes_yield_none_when_any_operand_is_none() {
    assert_eq!(
        eval("#1 + #2", &[(1, None), (2, Some(1.0))]),
        Reading::Value(None)
    );
    assert_eq!(
        eval("MAX(#1, #2)", &[(1, Some(1.0)), (2, None)]),
        Reading::Value(None)
    );
}

#[test]
fn division_by_zero_is_none() {
    assert_eq!(
        eval("#1 / #2", &[(1, Some(1.0)), (2, Some(0.0))]),
        Reading::Value(None)
    );
    assert_eq!(eval("1 / 0", &[]), Reading::Value(None));
    assert_eq!(eval("0 / 0", &[]), Reading::Value(None));
    assert_eq!(eval("1 / 2", &[]), Reading::Value(Some(0.5)));
}

#[test]
fn zero_argument_function_is_a_structural_error() {
    let expr = Expr::<f32>::Function {
        function: Function::Coalesce,
        args: vec![],
    };
    let mut source = HashMap::<u64, Option<f32>>::new();
    assert!(expr.evaluate(&mut source).is_err());
}
