// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

use std::collections::HashMap;

use super::eval;
use crate::expression::{Expr, Function};
use crate::{FormulaEngine, Reading};

#[test]
fn avg_of_values() {
    assert_eq!(
        eval("AVG(#1, #2, 3)", &[(1, Some(1.0)), (2, Some(2.0))]),
        Reading::Value(Some(2.0))
    );
    assert_eq!(
        eval("AVG(#1)", &[(1, Some(4.0))]),
        Reading::Value(Some(4.0))
    );
}

#[test]
fn avg_is_strict() {
    assert_eq!(
        eval("AVG(#1, #2)", &[(1, Some(1.0)), (2, None)]),
        Reading::Value(None)
    );
    assert_eq!(eval("AVG(#1, #2)", &[(1, None)]), Reading::Undecided);
}

#[test]
fn sqrt_of_values() {
    assert_eq!(eval("SQRT(9)", &[]), Reading::Value(Some(3.0)));
    assert_eq!(
        eval("SQRT(#1 * #1 + #2 * #2)", &[(1, Some(3.0)), (2, Some(4.0))]),
        Reading::Value(Some(5.0))
    );
    assert_eq!(eval("SQRT(0)", &[]), Reading::Value(Some(0.0)));
    assert_eq!(eval("SQRT(-1)", &[]), Reading::Value(None));
    assert_eq!(eval("SQRT(#1)", &[(1, None)]), Reading::Value(None));
    assert_eq!(eval("SQRT(#1)", &[]), Reading::Undecided);
}

#[test]
fn min_max_with_nan_keep_the_incoming_value_over_a_nan_accumulator() {
    // The fold's `partial_cmp` is `None` whenever either side is NaN, which
    // falls through to the `_ => x` arm: a NaN accumulator is always
    // replaced by the next value, but a NaN arriving as `x` always wins.
    // The grammar has no NaN literal, so build it through a HashMap value.
    assert_eq!(
        eval("MIN(#1, #2)", &[(1, Some(f32::NAN)), (2, Some(1.0))]),
        Reading::Value(Some(1.0))
    );
    assert_eq!(
        eval("MAX(#1, #2)", &[(1, Some(f32::NAN)), (2, Some(1.0))]),
        Reading::Value(Some(1.0))
    );

    match eval("MIN(#1, #2)", &[(1, Some(1.0)), (2, Some(f32::NAN))]) {
        Reading::Value(Some(value)) => assert!(value.is_nan()),
        other => panic!("expected a NaN value, got {other:?}"),
    }
    match eval("MAX(#1, #2)", &[(1, Some(1.0)), (2, Some(f32::NAN))]) {
        Reading::Value(Some(value)) => assert!(value.is_nan()),
        other => panic!("expected a NaN value, got {other:?}"),
    }
}

#[test]
fn function_arity_is_checked_by_the_parser() {
    assert!(FormulaEngine::<f32>::try_new("SQRT(1, 2)").is_err());
    assert!(FormulaEngine::<f32>::try_new("AVG()").is_err());
    assert!(FormulaEngine::<f32>::try_new("SQRT()").is_err());
}

#[test]
fn sqrt_with_two_args_is_a_structural_error() {
    let expr = Expr::<f32>::Function {
        function: Function::Sqrt,
        args: vec![Expr::Value(Some(1.0)), Expr::Value(Some(2.0))],
    };
    assert!(expr
        .evaluate(&mut HashMap::<u64, Option<f32>>::new())
        .is_err());
}

#[test]
fn neg_parses_and_evaluates() {
    assert!(matches!(crate::parse::<f32>("-#1").unwrap(), Expr::Neg(_)));
    assert_eq!(
        eval("-(#1 + 1)", &[(1, Some(1.0))]),
        Reading::Value(Some(-2.0))
    );
}
