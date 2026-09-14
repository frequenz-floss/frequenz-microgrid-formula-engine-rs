// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

use super::{eval, parsed};
use crate::formula::Formula;
use crate::Reading;

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
fn min_max_with_nan_keep_the_incoming_value_over_a_nan_accumulator() {
    // `<` and `>` are false whenever either side is NaN, so the fold falls
    // through to its `_ => x` arm: a NaN accumulator is always replaced by
    // the next value, but a NaN arriving as `x` always wins.
    // The grammar has no NaN literal, so build it through a HashMap value.
    assert_eq!(
        eval("MIN(#1, #2)", &[(1, Some(f32::NAN)), (2, Some(1.0))]),
        Reading::Known(Some(1.0))
    );
    assert_eq!(
        eval("MAX(#1, #2)", &[(1, Some(f32::NAN)), (2, Some(1.0))]),
        Reading::Known(Some(1.0))
    );

    match eval("MIN(#1, #2)", &[(1, Some(1.0)), (2, Some(f32::NAN))]) {
        Reading::Known(Some(value)) => assert!(value.is_nan()),
        other => panic!("expected a NaN value, got {other:?}"),
    }
    match eval("MAX(#1, #2)", &[(1, Some(1.0)), (2, Some(f32::NAN))]) {
        Reading::Known(Some(value)) => assert!(value.is_nan()),
        other => panic!("expected a NaN value, got {other:?}"),
    }
}

#[test]
fn function_arity_is_checked_by_the_parser() {
    assert!(crate::parse::<f32>("AVG()").is_err());
}

#[test]
fn neg_parses_and_evaluates() {
    assert!(matches!(parsed("-#1"), Formula::Neg(_)));
    assert_eq!(
        eval("-(#1 + 1)", &[(1, Some(1.0))]),
        Reading::Known(Some(-2.0))
    );
}
