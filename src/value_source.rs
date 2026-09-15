// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

//! Where component values come from during evaluation.

use std::collections::HashMap;
use std::hash::Hash;

/// One reading of a component value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reading<T> {
    /// The value is known: present, or known to be missing.
    Known(Option<T>),
    /// The value is not known yet. Nothing that depends on it can be
    /// decided; `COALESCE` waits for it instead of reading past it.
    Unknown,
}

impl<T> Reading<T> {
    /// Applies `f` to a present value, leaving `None` and `Unknown` as is.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Reading<U> {
        self.and_then(|value| Some(f(value)))
    }

    /// Applies `f` to a present value, where `f` returning `None` makes the
    /// reading known-missing. `None` and `Unknown` are left as is.
    pub fn and_then<U>(self, f: impl FnOnce(T) -> Option<U>) -> Reading<U> {
        match self {
            Reading::Known(value) => Reading::Known(value.and_then(f)),
            Reading::Unknown => Reading::Unknown,
        }
    }

    /// Pairs two readings: `Unknown` if either is, otherwise `None` if
    /// either is, otherwise both values.
    pub fn zip<U>(self, other: Reading<U>) -> Reading<(T, U)> {
        match (self, other) {
            (Reading::Known(a), Reading::Known(b)) => Reading::Known(a.zip(b)),
            _ => Reading::Unknown,
        }
    }
}

/// Supplies component values to
/// [`Formula::evaluate`](crate::Formula::evaluate).
///
/// `evaluate` calls [`read`](Self::read) for every component leaf it needs
/// and for no other, so an implementation that records the keys it is asked
/// for learns exactly which components the formula currently depends on.
pub trait ValueSource<T, K = u64> {
    /// Reads the current value of component `key`.
    fn read(&mut self, key: &K) -> Reading<T>;
}

/// A map is a source in which a present key is a known value and a missing
/// key is [`Reading::Unknown`].
impl<K: Eq + Hash, T: Copy> ValueSource<T, K> for HashMap<K, Option<T>> {
    fn read(&mut self, key: &K) -> Reading<T> {
        match self.get(key) {
            Some(value) => Reading::Known(*value),
            None => Reading::Unknown,
        }
    }
}

/// A mutable reference to a source is itself a source, so a `&mut dyn
/// ValueSource<T, K>` (an erased source) can be passed to `evaluate` too.
impl<K, T, S: ValueSource<T, K> + ?Sized> ValueSource<T, K> for &mut S {
    fn read(&mut self, key: &K) -> Reading<T> {
        (**self).read(key)
    }
}
