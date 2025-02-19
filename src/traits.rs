// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

//! Traits used in the FormulaEngine.

use std::ops::{Add, Div, Mul, Neg, Sub};

/// Represents types that can be used in formula engines.
pub trait NumberLike<T>:
    Copy + Neg<Output = T> + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>
{
}

/// Implement the NumberLike trait for all types that implement the required traits.
impl<T, U> NumberLike<T> for U where
    U: Copy
        + Neg<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + Div<Output = T>
{
}
