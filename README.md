# frequenz-microgrid-formula-engine-rs
A library to create formulas over streamed data

## Usage

A `FormulaEngine` instance can be created from a String formula with the
`try_new` method.
Such a formula can contain component placeholders, which are represented by `#`
followed by a number.
To calculate the formula, you need to provide an iterator of Option values,
where
- `None` represents a missing value
- `Some(value)` represents a value.

The iterator must contain as many values as the formula has placeholders.
The result of the calculation is an Option value.

```rust
use frequenz_formula_engine::{FormulaEngine, FormulaError};

fn main() -> Result<(), FormulaError> {
    let fe = FormulaEngine::try_new("#0 + #1")?;
    assert_eq!(fe.calculate(&[Some(1.0), Some(2.0)])?, Some(3.0));
    Ok(())
}
```
