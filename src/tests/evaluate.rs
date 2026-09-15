// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

use std::collections::HashMap;

use super::{calc, eval};
use crate::formula::{Formula, Function};
use crate::{FormulaError, Reading};

#[test]
fn coalesce_returns_first_some_without_reading_further() {
    // #2 is not in the map, so reading it would yield Unknown.
    assert_eq!(
        eval("COALESCE(#1, #2)", &[(1, Some(1.0))]),
        Reading::Known(Some(1.0))
    );
}

#[test]
fn coalesce_moves_past_none() {
    assert_eq!(
        eval("COALESCE(#1, 0.0)", &[(1, None)]),
        Reading::Known(Some(0.0))
    );
    assert_eq!(
        eval("COALESCE(#1, #2)", &[(1, None), (2, None)]),
        Reading::Known(None)
    );
}

#[test]
fn coalesce_stops_at_unknown() {
    assert_eq!(eval("COALESCE(#1, 0.0)", &[]), Reading::Unknown);
    assert_eq!(
        eval("COALESCE(#1, #2, 0.0)", &[(1, None)]),
        Reading::Unknown
    );
}

#[test]
fn strict_nodes_yield_unknown_over_none() {
    assert_eq!(eval("#1 + #2", &[(1, None)]), Reading::Unknown);
    assert_eq!(eval("MIN(#1, #2)", &[(1, None)]), Reading::Unknown);
    assert_eq!(eval("-#1", &[]), Reading::Unknown);
}

#[test]
fn strict_nodes_yield_none_when_any_operand_is_none() {
    assert_eq!(
        eval("#1 + #2", &[(1, None), (2, Some(1.0))]),
        Reading::Known(None)
    );
    assert_eq!(
        eval("MAX(#1, #2)", &[(1, Some(1.0)), (2, None)]),
        Reading::Known(None)
    );
}

#[test]
fn division_by_zero_is_none() {
    assert_eq!(
        eval("#1 / #2", &[(1, Some(1.0)), (2, Some(0.0))]),
        Reading::Known(None)
    );
    assert_eq!(eval("1 / 0", &[]), Reading::Known(None));
    assert_eq!(eval("0 / 0", &[]), Reading::Known(None));
    assert_eq!(eval("1 / 2", &[]), Reading::Known(Some(0.5)));
}

#[test]
fn zero_argument_function_is_a_structural_error() {
    let expr = Formula::<f32>::Function {
        function: Function::Coalesce,
        args: vec![],
    };
    let mut source = HashMap::<u64, Option<f32>>::new();
    assert_eq!(
        expr.evaluate(&mut source),
        Err(FormulaError::WrongArity {
            function: Function::Coalesce,
            args: 0
        })
    );
}

#[test]
fn non_finite_readings_are_none() {
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            eval("#1", &[(1, Some(value))]),
            Reading::Known(None),
            "{value}"
        );
    }
}

#[test]
fn non_finite_constants_are_none() {
    // The grammar has no NaN or infinity literal, but a builder can make one.
    for constant in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(
            calc(&Formula::Constant(Some(constant)), HashMap::new()),
            None,
            "{constant}"
        );
    }
}

#[test]
fn non_finite_results_are_none() {
    // `f32` overflows to infinity, which no node ever yields.
    assert_eq!(
        eval("#1 * #2", &[(1, Some(1e30)), (2, Some(1e30))]),
        Reading::Known(None)
    );
}

#[test]
fn coalesce_moves_past_a_non_finite_value() {
    assert_eq!(
        eval("COALESCE(#1, #2)", &[(1, Some(f32::NAN)), (2, Some(1.0))]),
        Reading::Known(Some(1.0))
    );
    assert_eq!(
        eval(
            "COALESCE(#1 * #2, 0.0)",
            &[(1, Some(1e30)), (2, Some(1e30))]
        ),
        Reading::Known(Some(0.0))
    );
}
