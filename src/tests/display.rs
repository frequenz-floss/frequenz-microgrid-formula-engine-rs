// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

use crate::expression::Expr;
use crate::parse;

fn shown(formula: &str) -> String {
    parse::<f32>(formula).unwrap().to_string()
}

#[test]
fn display_parenthesises_nested_operators_only() {
    assert_eq!(shown("#1 + #2 * 3"), "#1 + (#2 * 3)");
    assert_eq!(shown("(#1 + #2) * 3"), "(#1 + #2) * 3");
    assert_eq!(shown("#1 - #2 - #3"), "(#1 - #2) - #3");
    assert_eq!(shown("#1 - (#2 - #3)"), "#1 - (#2 - #3)");
    assert_eq!(shown("-(#1 + #2)"), "-(#1 + #2)");
    assert_eq!(shown("-#1"), "-#1");
    assert_eq!(shown("-(-#1)"), "-(-#1)");
}

#[test]
fn display_does_not_round_trip_other_keys_or_negative_constants() {
    let other_key = Expr::<f32, String>::Component("v1".to_string());
    assert_eq!(other_key.to_string(), "#v1");
    assert!(parse::<f32>(&other_key.to_string()).is_err());

    let negative = Expr::<f32>::Value(Some(-2.0));
    assert_eq!(negative.to_string(), "-2");
    assert_eq!(
        parse::<f32>(&negative.to_string()).unwrap(),
        Expr::Neg(Box::new(Expr::Value(Some(2.0))))
    );
}

#[test]
fn display_renders_constants_functions_and_none() {
    assert_eq!(shown("COALESCE(#1, None, 0.5)"), "COALESCE(#1, None, 0.5)");
    assert_eq!(shown("SQRT(AVG(#1, #2))"), "SQRT(AVG(#1, #2))");
    assert_eq!(shown("MIN(#1 + 2, MAX(#3, 4))"), "MIN(#1 + 2, MAX(#3, 4))");
    assert_eq!(shown("3.0"), "3");
}

#[test]
fn display_round_trips_through_parse() {
    let formulas = [
        "#1",
        "None",
        "2.5",
        "0.0000001",
        "-#1",
        "-(-#1)",
        "#1 + #2 * 3 - #4 / 5",
        "(#1 + #2) * (#3 - #4)",
        "#1 - (#2 - #3)",
        "-(#1 + #2) * SQRT(#3)",
        "-MAX(#1, #2)",
        "COALESCE(-#1, 0)",
        "COALESCE(#1, MIN(#2, #3), AVG(#4, #5, 0.0), None)",
        "MIN(0.0, COALESCE(#4 + #3, #2, COALESCE(#4, 0.0) + COALESCE(#3, 0.0))) + MIN(0.0, COALESCE(#6, #5, 0.0)) + MIN(0.0, COALESCE(#7, 0.0))",
        "MAX(0.0, #1 - COALESCE(#2, #3, 0.0) - COALESCE(#5, COALESCE(#7, 0.0) + COALESCE(#6, 0.0))) + COALESCE(MAX(0.0, #2 - #3), 0.0) + COALESCE(MAX(0.0, #5 - #6 - #7), 0.0)",
    ];
    for formula in formulas {
        let expr = parse::<f32>(formula).unwrap();
        let reparsed = parse::<f32>(&expr.to_string()).unwrap();
        assert_eq!(reparsed, expr, "{formula} displayed as {expr}");
    }
}
