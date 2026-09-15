# Frequenz Microgrid Formula Engine Release Notes

## Summary

Formulas are now public and buildable in code, component keys are generic, and evaluation pulls values through a `ValueSource` with a three-valued `Reading`. `COALESCE` is lazy.

## Upgrading

- `FormulaEngine` is gone: `Formula` implements `FromStr`, so `s.parse::<Formula<f32>>()` replaces `FormulaEngine::try_new(s)`, and the formula evaluates and reports its components directly. `Formula::evaluate(&mut impl ValueSource<T, K>)` replaces `FormulaEngine::calculate(&HashMap<u64, Option<T>>)` and returns a `Reading<T>` instead of an `Option<T>`. A `HashMap<K, Option<T>>` is a `ValueSource`: pass it as `&mut map` and match on `Reading::Known(value)`. A key absent from the map now reads as `Reading::Unknown` instead of being an error.
- `Formula` has a second type parameter for the component key, defaulting to `u64`, so `Formula<f32>` means a formula over `u64` keys.
- The `traits` module and its `NumberLike` trait are removed. Numeric types are bounded by `Real` instead, re-exported from `num_traits::real`, which `f32` and `f64` implement, so a bound written `T: NumberLike<T> + PartialOrd` becomes `T: Real`. `Real` asks for the whole real-number API, so a custom numeric type that implemented `NumberLike` has to implement `num_traits::real::Real` instead, which means depending on `num-traits` 0.2 directly.
- `FormulaError` is a `#[non_exhaustive]` enum with one variant per failure (`InvalidSyntax`, `InvalidConstant`, `ConstantOutOfRange`, `InvalidComponentId`, `NestedTooDeep`, `TooDeep`, `WrongArity` and `InternalError`) instead of a tuple struct around a message, so a `match` on it needs a `_` arm. `Display` messages are not the same as in 0.1.0; code that matched on the message text should match on the variant instead.
- Division by any zero divisor, including `0 / 0`, yields `None`. It used to yield infinity, or NaN for `0 / 0`.
- A numeric constant larger than the number type can hold, such as a 40-digit literal with `f32`, is now a parse error. It used to parse as infinity.
- A formula nested deeper than 128 levels of parentheses or function calls, or deeper than 1024 levels once its operators are counted, is now a parse error. Such a formula used to overflow the stack, either while parsing or while being evaluated, rendered or dropped.

## New Features

- `Formula`, `Op` and `Function` are public. `Op` and `Function` are `#[non_exhaustive]`, so a `match` on them needs a `_` arm. Formulas can be built from the `Formula` variants with the `+ - * /` and unary `-` operators, and `coalesce`, `min`, `max`, `avg`, `sqrt`.
- `Formula::map_components` re-keys every component leaf.
- `Formula` implements `Display`, producing a formula string that re-parses to an equal expression, for `u64` keys and finite, non-negative constants.
- `Reading::Unknown` for values that are not known yet. `COALESCE` stops at an unknown argument instead of reading past it, and reads only as far as the first present value, so a `ValueSource` can observe exactly which components an evaluation needed.
- `Reading::map` and `Reading::and_then` apply a function to a present value, leaving `None` and `Unknown` untouched. `Reading::zip` pairs two readings: `Unknown` if either is, otherwise `None` if either is.
- New function `AVG(...)`: the mean of the arguments that have a value, or `None` when none has.
- New function `SQRT(x)`.
