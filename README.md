# frequenz-microgrid-formula-engine-rs

[<img alt="docs.rs" src="https://img.shields.io/docsrs/frequenz-microgrid-formula-engine">](https://docs.rs/frequenz-microgrid-formula-engine)
[<img alt="Crates.io" src="https://img.shields.io/crates/v/frequenz-microgrid-formula-engine">](https://crates.io/crates/frequenz-microgrid-formula-engine)

A library to create formulas over streamed data, primarily used for calculating and processing values within microgrid applications.

## Usage

The `FormulaEngine` can *only* work with _resampled_ component data streams. It has been designed to work with the following libraries:
- [frequenz-resampling-rs](https://github.com/frequenz-floss/frequenz-resampling-rs) - A resampling library, which sends `None` values when data is missing. See [Handling Nulls and Missing Values](#handling-nulls-and-missing-values).
- [frequenz-microgrid-component-graph-rs](https://github.com/frequenz-floss/frequenz-microgrid-component-graph-rs) - A component graph library, for generating formulas.

The `FormulaEngine` can be created from a string formula using the `try_new` method. Formulas can contain component placeholders, represented by `#` followed by a number. To calculate the formula, provide an iterator of `Option` values where:
- `None` represents a missing value
- `Some(value)` represents a value

The result of the calculation will be an `Option` value.

### Example:

```rust
use frequenz_formula_engine::{FormulaEngine, FormulaError};

fn main() -> Result<(), FormulaError> {
    let fe = FormulaEngine::try_new("#0 + #1")?;
    assert_eq!(fe.calculate(&[Some(1.0), Some(2.0)])?, Some(3.0));
    Ok(())
}
```

### Handling Nulls and Missing Values

When dealing with missing data, use `None` to indicate a missing value. If the formula requires a value for a placeholder but it is missing, the result will also be `None` unless a fallback function like `COALESCE` is used.

Example:

```
COALESCE(#1, 0)  // If #1 is None, it will return 0
```

### Error Handling

The FormulaEngine may return errors for invalid formulas, incorrect argument counts, or division by zero. These are returned as a FormulaError. Handle them gracefully for robust applications.

```Rust
match FormulaEngine::try_new("#0 / #1") {
    Ok(fe) => match fe.calculate(&[Some(10.0), Some(0.0)]) {
        Ok(result) => println!("Result: {:?}", result),
        Err(e) => println!("Calculation error: {:?}", e),
    },
    Err(e) => println!("Invalid formula: {:?}", e),
}
```

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

Formulas support these functions that operate on a comma-separated list of expressions:

- `COALESCE(a, b, ...)` — Returns the first non-null value from the list
- `MIN(a, b, ...)` — Returns the smallest value
- `MAX(a, b, ...)` — Returns the largest value

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
frequenz-microgrid-formula-engine = "0.1"
```
