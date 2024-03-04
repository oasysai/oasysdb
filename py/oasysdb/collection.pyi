# flake8: noqa F821

from typing import Any, List
from oasysdb.vector import VectorID


class Config:
    """The configuration for the vector collection.

    Args:
    - ef_construction (int): Elements to consider during index construction.
    - ef_search (int): Elements to consider during the search.
    - ml (float): Layer multiplier of HNSW index.
    """

    def __init__(
        self,
        ef_construction: int,
        ef_search: int,
        ml: float,
    ) -> None: ...

    @staticmethod
    def create_default() -> Config:
        """Returns a default configuration.

        Default values:
        - ef_construction: 40
        - ef_search: 15
        - ml: 0.3
        """


class Record:
    """The vector record to store in the collection.

    Args:
    - vector (List[float]): Vector embedding of float values.
    - data (Metadata): Metadata of the vector.

    Metadata types:
    - String
    - Number
    - List of metadata types
    - Dictionary of metadata types
    """

    def __init__(
        self,
        vector: List[float],
        data: Any,
    ) -> None: ...


class Collection:
    """The collection of vectors and their metadata."""

    def __init__(
        self,
        config: Config,
    ) -> None: ...

    @staticmethod
    def from_records(
        config: Config,
        records: List[Record],
    ) -> Collection:
        """Creates a collection from the given records."""

    def len(self) -> int:
        """Returns the number of records in the collection."""

    def is_empty(self) -> bool:
        """Returns True if the collection is empty."""

    def contains(self, id: VectorID) -> bool:
        """Returns True if the record ID is in the collection."""
