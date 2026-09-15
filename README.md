# frequenz-microgrid-formula-engine-rs

[<img alt="docs.rs" src="https://img.shields.io/docsrs/frequenz-microgrid-formula-engine">](https://docs.rs/frequenz-microgrid-formula-engine)
[<img alt="Crates.io" src="https://img.shields.io/crates/v/frequenz-microgrid-formula-engine">](https://crates.io/crates/frequenz-microgrid-formula-engine)

A library to create formulas over streamed data, primarily used for calculating and processing values within microgrid applications.

## Usage

The engine evaluates formulas over per-component values, one evaluation per
snapshot. It is designed to work with:
- [frequenz-resampling-rs](https://github.com/frequenz-floss/frequenz-resampling-rs) - resampled streams that yield `None` when data is missing.
- [frequenz-microgrid-component-graph-rs](https://github.com/frequenz-floss/frequenz-microgrid-component-graph-rs) - generates formula strings from the component graph.

### Example

```rust
use frequenz_microgrid_formula_engine::{Formula, FormulaError, Reading};
use std::collections::HashMap;

fn main() -> Result<(), FormulaError> {
    let fe: Formula<f32> = "COALESCE(#0, #1 + #2)".parse()?;
    let mut values = HashMap::from([(0, None), (1, Some(2.0)), (2, Some(3.0))]);
    assert_eq!(fe.evaluate(&mut values)?, Reading::Known(Some(5.0)));
    Ok(())
}
```

### Value sources and readings

`evaluate` pulls values through the `ValueSource` trait. Each lookup returns a
`Reading`:

- `Reading::Known(Some(v))` — a value.
- `Reading::Known(None)` — known to be missing, e.g. a resampler with no data.
- `Reading::Unknown` — not known yet, e.g. a component that is not subscribed.

A `HashMap<K, Option<T>>` is a `ValueSource`; a key absent from the map reads
as `Unknown`.

`COALESCE` returns its first present argument, moves past `None`, and stops at
`Unknown`. Every other operator and function reads all of its operands and
yields `Unknown` if any operand is, otherwise `None` if any operand is. The
exception is `AVG`, which averages the operands that have a value and yields
`None` only when none has; an unknown operand still makes it `Unknown`.
A formula never evaluates to a NaN or infinite value: a reading, a hand-built
constant or an intermediate result that is not finite is `None`, so `COALESCE`
and `AVG` move past it like any other missing value.
Because `evaluate` reads exactly the components it needs, a `ValueSource` that
records the keys it is asked for learns which components the formula currently
depends on.

### Building formulas in code

```rust
use frequenz_microgrid_formula_engine::Formula;

let grid = Formula::<f32>::Component(1);
let pv = Formula::<f32>::Component(2);
let net = (grid + pv.coalesce(Formula::Constant(Some(0.0)))) * Formula::Constant(Some(0.5));
assert_eq!(net.to_string(), "(#1 + COALESCE(#2, 0)) * 0.5");
```

`Formula::map_components` replaces every component key, so a parsed formula can
be re-keyed by whatever the caller indexes its values with.

### Errors

Parsing returns a `FormulaError` for a formula that does not parse,
including wrong function arity, a constant too large for the number type,
and a formula deeper than `MAX_NESTING` or `MAX_DEPTH` allows. `evaluate` returns an error only for a
structurally invalid hand-built expression, such as a function with no
arguments. Missing data never errors: division by zero, the square root of a
negative number, and any NaN or infinite value yield `None`.

## Formula Syntax Overview

The formula engine supports simple arithmetic expressions that combine numbers, references to components, mathematical operators, and basic functions.

### Numbers

You can write integer or decimal numbers directly in formulas:

```
42
3.14
0.001
```

### Component References

Microgrid electrical component or sensor references are written as # followed by one or more digits:

```
#1     // Refers to component 1
#42    // Refers to component 42
```

### Operators

The following standard arithmetic operators are supported:

- Addition: `+`
- Subtraction: `-` (also supports unary minus, e.g., `-5`)
- Multiplication: `*`
- Division: `/`

Expressions follow standard precedence rules (multiplication/division before addition/subtraction). Use parentheses to override precedence as needed:

```
#1 + 3 * #2         // Multiplies #2 by 3, then adds #1
(#1 + 3) * #2       // Adds #1 and 3, then multiplies the result by #2
```

### Functions

Formulas support these functions. All but `SQRT` take a comma-separated list of expressions:

- `COALESCE(a, b, ...)` — Returns the first non-null value from the list
- `MIN(a, b, ...)` — Returns the smallest value
- `MAX(a, b, ...)` — Returns the largest value
- `AVG(a, b, ...)` — Returns the arithmetic mean of the arguments that have a value, or null if none has
- `SQRT(a)` — Returns the square root, or null for a negative argument

Examples:

```
COALESCE(#3, 0)         // Returns #3 if it's not null, otherwise 0
MIN(#1, #2, 100)        // Returns the smaller value of component #1 and #2, but at most the value of 100
MAX(#1 + 2, #4 * 5)     // Evaluates both expressions, returns the larger
```

### Whitespace

Whitespace is ignored and can be used freely to improve readability:

```
( #1 + 4 ) * ( #2 - 1 )
```

### More Examples

Here are some additional examples of the formula engine syntax:

```
#1 + 5
-#2 / 3.0
(#1 + #2) * 0.5
COALESCE(#5, #6, 0)
MAX(0, #3 - 100)
```

## Installation
To add the library to your project, include the following in your Cargo.toml:

```toml
[dependencies]
frequenz-microgrid-formula-engine = "0.2"
```
