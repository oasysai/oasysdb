# flake8: noqa F821

from typing import Any, List


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
