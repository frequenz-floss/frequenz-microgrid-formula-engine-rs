// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

#![deny(missing_docs)]

/*!
# frequenz-microgrid-formula-engine

A synchronous evaluator for formulas over microgrid component values.

## Expressions

An [`Expr`] is built either by parsing a string with [`parse`], where
component placeholders are written `#` followed by a number, or in code
with the builder methods and `std::ops` operators. Component keys are
`u64` when parsed; [`Expr::map_components`] retags them with any key type.

## Evaluation

A [`FormulaEngine`] wraps an expression and evaluates it against values
pulled from a [`ValueSource`]. Each lookup is a [`Reading`]: a known
value, a known-missing `None`, or `Undecided` when the value is not known
yet. `COALESCE` returns the first present value, moves past `None`, and
stops at `Undecided`; every other node reads all its operands and yields
`Undecided` if any is, else `None` if any is.

A `HashMap<K, Option<T>>` is a `ValueSource` in which an absent key reads
as `Undecided`.

```rust
use frequenz_microgrid_formula_engine::{FormulaEngine, FormulaError, Reading};
use std::collections::HashMap;

fn main() -> Result<(), FormulaError> {
    let fe = FormulaEngine::<f32>::try_new("COALESCE(#0, #1 + #2)")?;
    assert_eq!(fe.components(), &[0, 1, 2].into_iter().collect());

    let mut values = HashMap::from([(0, Some(1.))]);
    assert_eq!(fe.evaluate(&mut values)?, Reading::Value(Some(1.0)));

    let mut values = HashMap::from([(0, None), (1, Some(2.)), (2, Some(3.))]);
    assert_eq!(fe.evaluate(&mut values)?, Reading::Value(Some(5.0)));

    let mut values = HashMap::from([(0, None)]);
    assert_eq!(fe.evaluate(&mut values)?, Reading::Undecided);
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
pub use expression::{Expr, Function, Op};
pub use formula_engine::FormulaEngine;
pub use parser::parse;
pub use traits::NumberLike;
pub use value_source::{Reading, ValueSource};

#[cfg(test)]
mod tests;
