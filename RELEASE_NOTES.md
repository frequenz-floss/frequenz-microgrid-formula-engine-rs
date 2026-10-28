# Frequenz Microgrid Formula Engine Release Notes

## Summary

A chain of `+` and `-`, or of `*` and `/`, is now one node in the formula tree. Long chains no longer nest deep enough to overflow the stack. The parser no longer limits how many operators a formula has.

## Upgrading

- `Formula::Op` is replaced by `Formula::Sum` and `Formula::Product`. Each holds its first operand in `first` and a list of `(operator, operand)` pairs in `rest`, applied left to right. For example, `#1 + #2 - #3` is one `Sum` with `first: #1` and `rest: [(Add, #2), (Sub, #3)]`.
  - `Op` is split into `SumOp` (`Add`, `Sub`) and `ProductOp` (`Mul`, `Div`). Both implement `Display` and are `#[non_exhaustive]`, like `Op`, so a `match` on them needs a `_` arm.
  - Code that builds formulas with the `+ - * /` operators or by parsing needs no change. Code that builds `Formula::Op { lhs, op, rhs }` directly can write `*lhs + *rhs` (or `-`, `*`, `/`) instead. Code that walks the tree and recurses into `lhs` and `rhs` has to visit `first` and then each operand in `rest`.
  - `a + b` and `a - b` add `b` to the end of `a` when `a` is already a sum. `a * b` and `a / b` do the same when `a` is already a product. A `Sum` whose first operand is a `Sum`, or a `Product` whose first operand is a `Product`, can only be built by writing the variant directly.
- `MAX_DEPTH` and `FormulaError::TooDeep` are removed. A chain of `+` and `-`, or of `*` and `/`, is now one node, so the number of operators no longer needs a limit. Remove any use of `MAX_DEPTH` and any `match` arm for `TooDeep`. `MAX_NESTING` is now the only limit on the shape of a parsed formula.
- The crate now uses Rust edition 2024, so building it needs Rust 1.85 or newer.

## Bug Fixes

- A formula with more than about 1000 operators, such as the consumer formula of a large site, now parses. It used to fail with `TooDeep`.
- Adding many formulas one after another, as in `total = total + x` or `total = total - x`, no longer makes the tree one level deeper at each step. The same goes for `total = total * x`. So evaluating, rendering, cloning or dropping the result no longer overflows the stack just because many formulas were added.
  - The tree still grows when a step wraps the running result in a new sum or product. `total = x + total` adds one level per step. `total = total * x + y` adds two, a product and then a sum.
