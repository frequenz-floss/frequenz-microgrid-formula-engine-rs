# License: MIT
# Copyright © 2024 Frequenz Energy-as-a-Service GmbH

__all__ = "FormulaEngine"

class FormulaEngine:
    """A class that evaluates formulas from a microgrid."""

    def __init__(self, formula: str):
        """
        Create a new FormulaEngine.

        Args:
            formula: A string with the formula to evaluate.
        """

    def components(self) -> list[int]:
        """
        Return the components of the formula.

        Returns:
            A list with the components of the formula.
        """

    def calculate(self, values: dict[int, float | None]) -> float | None:
        """
        Calculate the formula.

        Args:
            values: A dictionary with the values of the components.

        Returns:
            The result of the formula.
        """
