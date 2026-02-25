# frequenz-microgrid-formula-engine-rs

[<img alt="docs.rs" src="https://img.shields.io/docsrs/frequenz-microgrid-formula-engine">](https://docs.rs/frequenz-microgrid-formula-engine)
[<img alt="Crates.io" src="https://img.shields.io/crates/v/frequenz-microgrid-formula-engine">](https://crates.io/crates/frequenz-microgrid-formula-engine)

A Rust library for parsing and evaluating mathematical formulas over streamed data with support for missing values. Designed for microgrid energy calculations where sensor data may be intermittently unavailable.

The `FormulaEngine` can *only* work with _resampled_ component data streams. It has been designed to work with the following libraries:
- [frequenz-resampling-rs](https://github.com/frequenz-floss/frequenz-resampling-rs) - A resampling library, which sends `None` values when data is missing. See [Handling Missing Values](#handling-missing-values).
- [frequenz-microgrid-component-graph-rs](https://github.com/frequenz-floss/frequenz-microgrid-component-graph-rs) - A component graph library, for generating formulas.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
frequenz-microgrid-formula-engine = "0.1"
```

## Quick Start

```rust
use frequenz_microgrid_formula_engine::{FormulaEngine, FormulaError};
use std::collections::HashMap;

fn main() -> Result<(), FormulaError> {
    // Create a formula engine from a string expression
    let fe = FormulaEngine::try_new("#0 + #1")?;

    // Provide values for each component placeholder
    let values = HashMap::from([(0, Some(1.0)), (1, Some(2.0))]);

    // Calculate the result
    assert_eq!(fe.calculate(&values)?, Some(3.0));
    Ok(())
}
```

## Formula Grammar

Formulas support arithmetic operations, component placeholders, and built-in functions.

### Components

Component placeholders represent input values and are written as `#` followed by an integer index:

| Syntax | Description |
|--------|-------------|
| `#0`   | First component |
| `#1`   | Second component |
| `#42`  | Component at index 42 |

### Operators

Standard arithmetic operators with conventional precedence (multiplication/division before addition/subtraction):

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `#0 + #1` |
| `-` | Subtraction | `#0 - #1` |
| `*` | Multiplication | `#0 * 2` |
| `/` | Division | `#0 / #1` |
| `-` (unary) | Negation | `-#0` |

### Literals

Numeric literals can be integers or decimals:

```text
1
0.5
3.14159
```

### Parentheses

Use parentheses to override operator precedence:

```text
(#0 + #1) * #2
```

### Functions

Built-in functions for handling multiple values. All functions accept one or more arguments.

| Function | Description | Example |
|----------|-------------|---------|
| `COALESCE(a, b, ...)` | Returns the first non-`None` value | `COALESCE(#0, #1, 0.0)` |
| `MIN(a, b, ...)` | Returns the minimum value (returns `None` if any argument is `None`) | `MIN(#0, #1, #2)` |
| `MAX(a, b, ...)` | Returns the maximum value (returns `None` if any argument is `None`) | `MAX(#0, #1, #2)` |

### Grammar Summary (EBNF)

```ebnf
formula    = expr ;
expr       = atom , { operator , atom } ;
atom       = [ "-" ] , primary ;
primary    = number | component | "(" , expr , ")" | function | "None" ;
number     = digit+ , [ "." , digit+ ] ;
component  = "#" , digit+ ;
function   = ( "COALESCE" | "MIN" | "MAX" ) , "(" , expr , { "," , expr } , ")" ;
operator   = "+" | "-" | "*" | "/" ;
digit      = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
```

## Usage

### Handling Missing Values

The formula engine is designed to work with optional values. When any operand in an arithmetic operation is `None`, the result is `None`.

**Note:** The `calculate()` method requires an entry in the HashMap for every component index (`#0`, `#1`, etc.) used in the formula. Use `None` to represent missing data. If an index is missing from the HashMap entirely, the method returns an error.

```rust
use frequenz_microgrid_formula_engine::FormulaEngine;
use std::collections::HashMap;

let fe = FormulaEngine::try_new("#0 + #1").unwrap();

// When all values are present
let result = fe.calculate(&HashMap::from([(0, Some(1.0)), (1, Some(2.0))])).unwrap();
assert_eq!(result, Some(3.0));

// When a value is missing
let result = fe.calculate(&HashMap::from([(0, Some(1.0)), (1, None)])).unwrap();
assert_eq!(result, None);
```

### Using COALESCE for Fallbacks

`COALESCE` returns the first non-`None` value, useful for providing fallback values:

```rust
use frequenz_microgrid_formula_engine::FormulaEngine;
use std::collections::HashMap;

let fe = FormulaEngine::try_new("COALESCE(#0, #1, 0.0)").unwrap();

// Returns #0 when available
let result = fe.calculate(&HashMap::from([(0, Some(5.0)), (1, Some(3.0))])).unwrap();
assert_eq!(result, Some(5.0));

// Falls back to #1 when #0 is None
let result = fe.calculate(&HashMap::from([(0, None), (1, Some(3.0))])).unwrap();
assert_eq!(result, Some(3.0));

// Falls back to literal 0.0 when both are None
let result = fe.calculate(&HashMap::from([(0, None), (1, None)])).unwrap();
assert_eq!(result, Some(0.0));
```

### Inspecting Components

You can retrieve the set of component indices used in a formula:

```rust
use frequenz_microgrid_formula_engine::FormulaEngine;

let fe = FormulaEngine::<f64>::try_new("MAX(#0, #1) + #2").unwrap();
let components = fe.components();
assert!(components.contains(&0));
assert!(components.contains(&1));
assert!(components.contains(&2));
```

### Complex Formulas

The engine supports nested functions and complex expressions:

```rust
use frequenz_microgrid_formula_engine::FormulaEngine;
use std::collections::HashMap;

// Nested functions with arithmetic
let fe = FormulaEngine::try_new(
    "MAX(0.0, #0 - COALESCE(#1, #2, 0.0))"
).unwrap();

let result = fe.calculate(&HashMap::from([
    (0, Some(10.0)),
    (1, None),
    (2, Some(3.0))
])).unwrap();
assert_eq!(result, Some(7.0));
```

## License

MIT
