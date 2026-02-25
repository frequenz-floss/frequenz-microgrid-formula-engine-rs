// License: MIT
// Copyright © 2024 Frequenz Energy-as-a-Service GmbH

#![doc = include_str!("../README.md")]

mod error;
mod expression;
mod formula_engine;
mod parser;
pub mod traits;

pub use error::FormulaError;
pub use formula_engine::FormulaEngine;

#[cfg(test)]
mod tests;
