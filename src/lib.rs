// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

/*!
# frequenz-formula-engine-rs

A library to create formulas over streamed data

## Usage

A [`FormulaEngine`] instance can be created from a [`String`] formula with the [`try_new`][`FormulaEngine::try_new`] method.
Such a formula can contain component placeholders, which are represented by `#`
followed by a number.
To calculate the formula, you need to provide an iterator of Option values,
where
- `None` represents a missing value
- `Some(value)` represents a value.

The iterator must contain as many values as the formula has placeholders.
The result of the calculation is an Option value.

```rust
use frequenz_microgrid_formula_engine::{FormulaEngine, FormulaError};
use std::collections::HashMap;

fn main() -> Result<(), FormulaError> {
    let fe = FormulaEngine::try_new("#0 + #1")?;
    let components = fe.components();
    assert_eq!(components, &[0, 1].into_iter().collect());
    assert_eq!(fe.calculate(HashMap::from([(0, Some(1.)), (1, Some(2.))]))?, Some(3.0));
    Ok(())
}
```
*/

mod error;
mod expression;
mod formula_engine;
mod parser;
pub mod traits;

pub use error::FormulaError;
pub use formula_engine::FormulaEngine;

#[cfg(test)]
mod tests;
