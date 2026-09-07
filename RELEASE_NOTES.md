# Frequenz Microgrid Formula Engine Release Notes

## Summary

Expressions are now public and buildable in code, component keys are generic, and evaluation pulls values through a `ValueSource` with a three-valued `Reading`. `COALESCE` is lazy.

## Upgrading

- `FormulaEngine::calculate(&HashMap<u64, Option<T>>)` is replaced by `FormulaEngine::evaluate(&mut impl ValueSource<K, T>)`, which returns a `Reading<T>` instead of an `Option<T>`. A `HashMap<K, Option<T>>` is a `ValueSource`: pass it as `&mut map` and match on `Reading::Value(value)`. A key absent from the map now reads as `Reading::Undecided` instead of being an error.
- `FormulaEngine` has a second type parameter for the component key, defaulting to `u64`, so `FormulaEngine<f32>` keeps meaning what it did.
- `traits::NumberLike<T>` is now `NumberLike`, without a type parameter and with `PartialOrd` as a supertrait, so a bound written `T: NumberLike<T> + PartialOrd` becomes `T: NumberLike`. It is an explicit trait implemented for `f32` and `f64` rather than a blanket impl: a custom numeric type must implement `zero`, `from_usize` and `sqrt`.
- Division by any zero divisor, including `0 / 0`, yields `None`. It used to yield infinity, or NaN for `0 / 0`.

## New Features

- `Expr`, `Op`, `Function` and `parse` are public. The three enums are `#[non_exhaustive]`, so a `match` on them needs a `_` arm. Expressions can be built with `Expr::component`, `Expr::value`, the `+ - * /` and unary `-` operators, and `coalesce`, `min`, `max`, `avg`, `sqrt`.
- `Expr::map_components` re-keys every component leaf.
- `Expr` implements `Display`, producing a formula string that re-parses to an equal expression, for `u64` keys and finite, non-negative constants.
- `FormulaEngine::from_expr` and `FormulaEngine::expr`.
- `Reading::Undecided` for values that are not known yet. `COALESCE` stops at an undecided argument instead of reading past it, and reads only as far as the first present value, so a `ValueSource` can observe exactly which components an evaluation needed.
- New functions `AVG(...)` and `SQRT(x)`.
