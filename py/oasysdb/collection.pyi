# flake8: noqa F821

from typing import Any, List
from oasysdb.vector import VectorID


class Config:
    """The configuration for the vector collection.

    Args:
    - ef_construction (int): Nodes to consider during index construction.
    - ef_search (int): Nodes to consider during the search.
    - ml (float): Layer multiplier of the HNSW index.
    """

    ef_construction: int
    ef_search: int
    ml: float

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

    vector: List[float]
    data: Any

    def __init__(
        self,
        vector: List[float],
        data: Any,
    ) -> None: ...

    @staticmethod
    def random(dimension: int) -> Record:
        """Generates a random record with the given dimension
        with a random integer metadata.

        Args:
        - dimension (int): Vector dimension.
        """

    @staticmethod
    def many_random(dimension: int, len: int) -> List[Record]:
        """Generates a list of random records.

        Args:
        - dimension (int): Vector dimension.
        - len (int): Number of records.
        """


class Collection:
    """The collection of vectors and their metadata."""

    config: Config

    def __init__(
        self,
        config: Config,
    ) -> None: ...

    @staticmethod
    def from_records(
        config: Config,
        records: List[Record],
    ) -> Collection:
        """Creates a collection from the given records.

        Args:
        - config (Config): Collection configuration.
        - records (List[Record]): Records used to build the collection.
        """

    def insert(self, record: Record) -> None:
        """Inserts a record into the collection.

        Args:
        - record (Record): Record to insert.
        """

    def delete(self, id: VectorID) -> None:
        """Deletes a record from the collection.

        Args:
        - id (VectorID): Vector ID to delete.
        """

    def get(self, id: VectorID) -> Record:
        """Returns a record from the collection.

        Args:
        - id (VectorID): Vector ID to fetch.
        """

    def update(self, id: VectorID, record: Record) -> None:
        """Updates a record in the collection.

        Args:
        - id (VectorID): Vector ID to update.
        - record (Record): New record.
        """

    def len(self) -> int:
        """Returns the number of records in the collection."""

    def is_empty(self) -> bool:
        """Returns True if the collection is empty."""

    def contains(self, id: VectorID) -> bool:
        """Returns True if the vector ID is in the collection."""
