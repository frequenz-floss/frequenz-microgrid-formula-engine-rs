// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

//! Traits used in the FormulaEngine.

use std::ops::{Add, Div, Mul, Neg, Sub};

/// Numeric types a formula can be evaluated over.
///
/// Implemented for `f32` and `f64`.
pub trait NumberLike:
    Copy
    + PartialOrd
    + Neg<Output = Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
{
    /// The additive identity, used for the division-by-zero check.
    fn zero() -> Self;

    /// Converts a count to a number, used as the `AVG` divisor.
    fn from_usize(n: usize) -> Self;

    /// The square root, used by `SQRT`.
    fn sqrt(self) -> Self;
}

impl NumberLike for f32 {
    fn zero() -> Self {
        0.0
    }

    fn from_usize(n: usize) -> Self {
        n as f32
    }

    fn sqrt(self) -> Self {
        f32::sqrt(self)
    }
}

impl NumberLike for f64 {
    fn zero() -> Self {
        0.0
    }

    fn from_usize(n: usize) -> Self {
        n as f64
    }

    fn sqrt(self) -> Self {
        f64::sqrt(self)
    }
}
