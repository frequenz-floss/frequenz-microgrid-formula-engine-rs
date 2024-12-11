# License: MIT
# Copyright © 2024 Frequenz Energy-as-a-Service GmbH

"""Tests to verify that the FormulaEngine can be used from Python."""

from frequenz.microgrid_formula_engine import FormulaEngine


def test_formula_engine() -> None:
    """Test the formula engine."""
    fe = FormulaEngine("1 + 2")
    assert fe.calculate({}) == 3


def test_formula_engine_components() -> None:
    """Test the formula engine components."""
    fe = FormulaEngine("1 + #2")
    assert fe.components() == [2]

    fe.calculate({2: 3})
    assert fe.calculate({2: 3}) == 4
