// License: MIT
// Copyright © 2024 {{project_author}}

use std::collections::HashMap;

use pyo3::prelude::*;

use crate::FormulaEngine;

#[pyclass(name = "FormulaEngine")]
struct FormulaEngineF32 {
    inner: FormulaEngine<f32>,
}

#[pymethods]
impl FormulaEngineF32 {
    #[new]
    fn new(formula: &str) -> PyResult<Self> {
        Ok(Self {
            inner: FormulaEngine::try_new(formula)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{:?}", e)))?,
        })
    }

    fn components(&self) -> Vec<usize> {
        self.inner.components().iter().copied().collect()
    }

    fn calculate(&self, values: HashMap<usize, Option<f32>>) -> PyResult<Option<f32>> {
        self.inner
            .calculate(values)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{:?}", e)))
    }
}

#[pymodule]
fn _rust_backend(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FormulaEngineF32>()?;
    Ok(())
}
