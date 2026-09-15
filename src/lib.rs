// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

#![deny(missing_docs)]

/*!
# frequenz-microgrid-formula-engine

A synchronous evaluator for formulas over microgrid component values.

## Formulas

A [`Formula`] is built either by parsing a string ([`FromStr`](std::str::FromStr)), where
component placeholders are written `#` followed by a number, or in code
with the builder methods and `std::ops` operators. Component keys are
`u64` when parsed; [`Formula::map_components`] retags them with any key type.

## Evaluation

A [`Formula`] evaluates against values pulled from a [`ValueSource`].
Each lookup is a [`Reading`]: a known value, a known-missing `None`, or
`Unknown` when the value is not known yet. `COALESCE` returns the first
present value, moves past `None`, and stops at `Unknown`; every other
node reads all its operands and yields `Unknown` if any is, else `None`
if any is. The exception is `AVG`, which averages the operands that have
a value and yields `None` only when none has; an unknown operand still
makes it `Unknown`.

A `HashMap<K, Option<T>>` is a `ValueSource` in which an absent key reads
as `Unknown`.

```rust
use frequenz_microgrid_formula_engine::{Formula, FormulaError, Reading};
use std::collections::HashMap;

fn main() -> Result<(), FormulaError> {
    let fe: Formula<f32> = "COALESCE(#0, #1 + #2)".parse()?;

    let mut values = HashMap::from([(0, Some(1.))]);
    assert_eq!(fe.evaluate(&mut values)?, Reading::Known(Some(1.0)));

    let mut values = HashMap::from([(0, None), (1, Some(2.)), (2, Some(3.))]);
    assert_eq!(fe.evaluate(&mut values)?, Reading::Known(Some(5.0)));

    let mut values = HashMap::from([(0, None)]);
    assert_eq!(fe.evaluate(&mut values)?, Reading::Unknown);
    Ok(())
}
```
*/

mod display;
mod error;
mod formula;
mod parser;
mod value_source;

pub use error::FormulaError;
pub use formula::{Formula, Function, Op};
pub use num_traits::Float;
#[cfg(test)]
pub(crate) use parser::parse;
pub use parser::{MAX_DEPTH, MAX_NESTING};
pub use value_source::{Reading, ValueSource};

#[cfg(test)]
mod tests;
