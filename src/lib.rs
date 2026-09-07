// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

/*!
# frequenz-formula-engine-rs

A library to create formulas over streamed data

## Usage

A [`FormulaEngine`] instance can be created from a [`String`] formula with the [`try_new`][`FormulaEngine::try_new`] method.
Such a formula can contain component placeholders, which are represented by `#`
followed by a number.

To evaluate the formula, provide a [`ValueSource`]; a `HashMap<u64,
Option<T>>` is one, where `None` is a missing value and an absent key is
an undecided one. The result is a [`Reading`].

```rust
use frequenz_microgrid_formula_engine::{FormulaEngine, FormulaError, Reading};
use std::collections::HashMap;

fn main() -> Result<(), FormulaError> {
    let fe = FormulaEngine::<f32>::try_new("#0 + #1")?;
    assert_eq!(fe.components(), &[0, 1].into_iter().collect());
    let mut values = HashMap::from([(0, Some(1.)), (1, Some(2.))]);
    assert_eq!(fe.evaluate(&mut values)?, Reading::Value(Some(3.0)));
    Ok(())
}
```
*/

mod display;
mod error;
mod expression;
mod formula_engine;
mod parser;
pub mod traits;
mod value_source;

pub use error::FormulaError;
pub use formula_engine::FormulaEngine;
pub use parser::parse;
pub use value_source::{Reading, ValueSource};

#[cfg(test)]
mod tests;
