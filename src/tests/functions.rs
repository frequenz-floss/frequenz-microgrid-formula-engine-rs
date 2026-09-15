// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

use std::collections::HashMap;

use super::{eval, parsed};
use crate::formula::{Formula, Function};
use crate::{FormulaError, Reading};

#[test]
fn avg_of_values() {
    assert_eq!(
        eval("AVG(#1, #2, 3)", &[(1, Some(1.0)), (2, Some(2.0))]),
        Reading::Known(Some(2.0))
    );
    assert_eq!(
        eval("AVG(#1)", &[(1, Some(4.0))]),
        Reading::Known(Some(4.0))
    );
}

#[test]
fn avg_skips_missing_values_but_not_unknown_ones() {
    assert_eq!(
        eval(
            "AVG(#1, #2, #3)",
            &[(1, Some(1.0)), (2, None), (3, Some(3.0))]
        ),
        Reading::Known(Some(2.0))
    );
    assert_eq!(
        eval("AVG(#1, #2)", &[(1, None), (2, None)]),
        Reading::Known(None)
    );
    assert_eq!(eval("AVG(#1, #2)", &[(1, Some(1.0))]), Reading::Unknown);
    assert_eq!(eval("AVG(#1, #2)", &[(1, None)]), Reading::Unknown);
}

#[test]
fn sqrt_of_values() {
    assert_eq!(eval("SQRT(9)", &[]), Reading::Known(Some(3.0)));
    assert_eq!(
        eval("SQRT(#1 * #1 + #2 * #2)", &[(1, Some(3.0)), (2, Some(4.0))]),
        Reading::Known(Some(5.0))
    );
    assert_eq!(eval("SQRT(0)", &[]), Reading::Known(Some(0.0)));
    assert_eq!(eval("SQRT(-1)", &[]), Reading::Known(None));
    assert_eq!(
        eval("SQRT(#1)", &[(1, Some(f32::NAN))]),
        Reading::Known(None)
    );
    assert_eq!(eval("SQRT(#1)", &[(1, None)]), Reading::Known(None));
    assert_eq!(eval("SQRT(#1)", &[]), Reading::Unknown);
}

#[test]
fn min_max_are_none_when_an_argument_is_not_finite() {
    // A non-finite reading is `None` at the leaf, and MIN and MAX are
    // strict, so a NaN argument yields `None` in either position. The
    // grammar has no NaN literal, so build it through a HashMap value.
    for formula in ["MIN(#1, #2)", "MAX(#1, #2)", "MIN(#2, #1)", "MAX(#2, #1)"] {
        assert_eq!(
            eval(formula, &[(1, Some(f32::NAN)), (2, Some(1.0))]),
            Reading::Known(None),
            "{formula}"
        );
    }
}

#[test]
fn avg_skips_non_finite_arguments() {
    assert_eq!(
        eval(
            "AVG(#1, #2, #3)",
            &[(1, Some(f32::NAN)), (2, Some(1.0)), (3, Some(3.0))]
        ),
        Reading::Known(Some(2.0))
    );
}

#[test]
fn function_arity_is_checked_by_the_parser() {
    assert!(crate::parse::<f32>("SQRT(1, 2)").is_err());
    assert!(crate::parse::<f32>("AVG()").is_err());
    assert!(crate::parse::<f32>("SQRT()").is_err());
}

#[test]
fn sqrt_with_two_args_is_a_structural_error() {
    let expr = Formula::<f32>::Function {
        function: Function::Sqrt,
        args: vec![Formula::Constant(Some(1.0)), Formula::Constant(Some(2.0))],
    };
    assert_eq!(
        expr.evaluate(&mut HashMap::<u64, Option<f32>>::new()),
        Err(FormulaError::WrongArity {
            function: Function::Sqrt,
            args: 2
        })
    );
}

#[test]
fn neg_parses_and_evaluates() {
    assert!(matches!(parsed("-#1"), Formula::Neg(_)));
    assert_eq!(
        eval("-(#1 + 1)", &[(1, Some(1.0))]),
        Reading::Known(Some(-2.0))
    );
}
