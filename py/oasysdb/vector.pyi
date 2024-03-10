# flake8: noqa F821

from typing import List


class Vector:
    """The vector embedding containing float values."""

    def __init__(self, vector: List[float]) -> None: ...

    def len(self) -> int:
        """Returns the length of the vector."""

    def is_empty(self) -> bool:
        """Returns True if the vector is empty."""

    def to_list(self) -> List[float]:
        """Returns the vector as a list of floats."""

    @staticmethod
    def random(dimension: int) -> Vector:
        """Generates a random vector of the given dimension."""


class VectorID:
    """Identifier of the vector in the collection."""

    def __init__(self, id: int) -> None: ...

    def is_valid(self) -> bool:
        """Returns True if the vector ID is valid."""
