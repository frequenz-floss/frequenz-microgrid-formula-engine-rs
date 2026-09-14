// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

/*!
# frequenz-formula-engine-rs

A library to create formulas over streamed data

## Usage

A [`Formula`] is parsed from a string with [`FromStr`](std::str::FromStr).
Such a formula can contain component placeholders, which are represented by `#`
followed by a number.

To evaluate the formula, provide a [`ValueSource`]; a `HashMap<u64,
Option<T>>` is one, where `None` is a missing value and an absent key is
an unknown one. The result is a [`Reading`].

```rust
use frequenz_microgrid_formula_engine::{Formula, FormulaError, Reading};
use std::collections::HashMap;

fn main() -> Result<(), FormulaError> {
    let fe: Formula<f32> = "#0 + #1".parse()?;
    assert_eq!(fe.components(), [0, 1].into_iter().collect());
    let mut values = HashMap::from([(0, Some(1.)), (1, Some(2.))]);
    assert_eq!(fe.evaluate(&mut values)?, Reading::Known(Some(3.0)));
    Ok(())
}
```
*/

mod error;
mod formula;
mod parser;
mod value_source;

pub use error::FormulaError;
pub use formula::Formula;
pub use num_traits::real::Real;
#[cfg(test)]
pub(crate) use parser::parse;
pub use parser::{MAX_DEPTH, MAX_NESTING};
pub use value_source::{Reading, ValueSource};

#[cfg(test)]
mod tests;
