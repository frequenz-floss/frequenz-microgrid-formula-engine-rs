// License: MIT
// Copyright © 2026 Frequenz Energy-as-a-Service GmbH

//! Where component values come from during evaluation.

use std::collections::HashMap;
use std::hash::Hash;

/// One reading of a component value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reading<T> {
    /// The value is known: present, or known to be missing.
    Value(Option<T>),
    /// The value is not known yet. Nothing that depends on it can be
    /// decided; `COALESCE` waits for it instead of reading past it.
    Undecided,
}

impl<T> Reading<T> {
    /// Applies `f` to a present value, leaving `None` and `Undecided` as is.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Reading<U> {
        self.and_then(|value| Some(f(value)))
    }

    /// Applies `f` to a present value, where `f` returning `None` makes the
    /// reading known-missing. `None` and `Undecided` are left as is.
    pub fn and_then<U>(self, f: impl FnOnce(T) -> Option<U>) -> Reading<U> {
        match self {
            Reading::Value(value) => Reading::Value(value.and_then(f)),
            Reading::Undecided => Reading::Undecided,
        }
    }

    /// Pairs two readings: `Undecided` if either is, otherwise `None` if
    /// either is, otherwise both values.
    pub fn zip<U>(self, other: Reading<U>) -> Reading<(T, U)> {
        match (self, other) {
            (Reading::Value(a), Reading::Value(b)) => Reading::Value(a.zip(b)),
            _ => Reading::Undecided,
        }
    }
}

/// Supplies component values to
/// [`FormulaEngine::evaluate`](crate::FormulaEngine::evaluate).
///
/// `evaluate` calls [`get`](Self::get) for every component leaf it needs
/// and for no other, so an implementation that records the keys it is asked
/// for learns exactly which components the formula currently depends on.
pub trait ValueSource<K, T> {
    /// Reads the current value of component `key`.
    fn get(&mut self, key: &K) -> Reading<T>;
}

/// A map is a source in which a present key is a known value and a missing
/// key is [`Reading::Undecided`].
impl<K: Eq + Hash, T: Copy> ValueSource<K, T> for HashMap<K, Option<T>> {
    fn get(&mut self, key: &K) -> Reading<T> {
        match HashMap::get(self, key) {
            Some(value) => Reading::Value(*value),
            None => Reading::Undecided,
        }
    }
}

/// A mutable reference to a source is itself a source, so a `&mut dyn
/// ValueSource<K, T>` (an erased source) can be passed to `evaluate` too.
impl<K, T, S: ValueSource<K, T> + ?Sized> ValueSource<K, T> for &mut S {
    fn get(&mut self, key: &K) -> Reading<T> {
        (**self).get(key)
    }
}

/// Combines the readings of a strict node's operands, all of which have
/// already been read: `Undecided` if any is, otherwise `None` if any is,
/// otherwise all the values in order.
// Only used by tests until Task 3 wires it into evaluation; remove this
// allow once that lands.
#[allow(dead_code)]
pub(crate) fn strict<T>(readings: impl IntoIterator<Item = Reading<T>>) -> Reading<Vec<T>> {
    readings
        .into_iter()
        .fold(Reading::Value(Some(Vec::new())), |values, reading| {
            values.zip(reading).map(|(mut values, value)| {
                values.push(value);
                values
            })
        })
}
