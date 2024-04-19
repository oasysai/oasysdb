# flake8: noqa F821

from typing import Any, List, Dict
from oasysdb.vector import Vector, VectorID


class Config:
    """The configuration for the vector collection.

    Args:
    - ef_construction: Nodes to consider during index construction.
    - ef_search: Nodes to consider during the search.
    - ml: Layer multiplier of the HNSW index.
    - distance: Distance metric function.

    Distance metrics:
    - euclidean
    - dot
    - cosine
    - norm_cosine
    """

    ef_construction: int
    ef_search: int
    ml: float
    distance: str

    def __init__(
        self,
        ef_construction: int,
        ef_search: int,
        ml: float,
        distance: str
    ) -> None: ...

    @staticmethod
    def create_default() -> Config:
        """Returns a default configuration.

        Default values:
        - ef_construction: 40
        - ef_search: 15
        - ml: 0.2885
        - distance: euclidean
        """

    @staticmethod
    def default() -> Config:
        """Returns a default configuration object.
        This is an alias of create_default method and
        shared the same implementation.
        """


class Record:
    """The vector record to store in the collection.

    Args:
    - vector: Vector embedding of float values.
    - data: Metadata of the vector.

    Metadata types:
    - String
    - Number
    - List of metadata types
    - Dictionary of metadata types
    """

    vector: Vector
    data: Any

    def __init__(self, vector: List[float], data: Any) -> None: ...

    @staticmethod
    def random(dimension: int) -> Record:
        """Generates a random record with the given dimension
        with a random integer metadata.

        Args:
        - dimension: Vector dimension.
        """

    @staticmethod
    def many_random(dimension: int, len: int) -> List[Record]:
        """Generates a list of random records.

        Args:
        - dimension: Vector dimension.
        - len: Number of records.
        """


class Collection:
    """The collection of vectors and their metadata."""

    config: Config
    dimension: int
    relevancy: float

    def __init__(self, config: Config) -> None: ...

    @staticmethod
    def from_records(config: Config, records: List[Record]) -> Collection:
        """Build a collection from the given records.

        Args:
        - config: Collection configuration.
        - records: Records used to build the collection.
        """

    @staticmethod
    def build(config: Config, records: List[Record]) -> Collection:
        """Build a collection from the records.
        This is an alias of from_records method and shared
        the same implementation.

        Args:
        - config: Collection configuration.
        - records: Records used to build the collection.
        """

    def insert(self, record: Record) -> VectorID:
        """Inserts a record into the collection.

        Args:
        - record: Record to insert.
        """

    def insert_many(self, records: List[Record]) -> List[VectorID]:
        """Inserts multiple records into the collection.

        Args:
        - records: Records to insert.
        """

    def delete(self, id: VectorID) -> None:
        """Deletes a record from the collection.

        Args:
        - id: Vector ID to delete.
        """

    def get(self, id: VectorID) -> Record:
        """Returns a record from the collection.

        Args:
        - id: Vector ID to fetch.
        """

    def list(self) -> Dict[VectorID, Record]:
        """Returns a dictionary of records in the collection
        in the format of { VectorID: Record }.
        """

    def update(self, id: VectorID, record: Record) -> None:
        """Updates a record in the collection.

        Args:
        - id: Vector ID to update.
        - record: New record.
        """

    def search(self, vector: Vector, n: int) -> List[SearchResult]:
        """Searches for the nearest neighbors to
        the given vector using HNSW indexing algorithm

        Args:
        - vector: Vector to search.
        - n: Number of neighbors to return.
        """

    def true_search(self, vector: Vector, n: int) -> List[SearchResult]:
        """Searches for the nearest neighbors using brute force.

        Args:
        - vector: Vector to search.
        - n: Number of neighbors to return.
        """

    def len(self) -> int:
        """Returns the number of records in the collection."""

    def is_empty(self) -> bool:
        """Returns True if the collection is empty."""

    def contains(self, id: VectorID) -> bool:
        """Returns True if the vector ID is in the collection."""


class SearchResult:
    """The result of a search operation on the collection."""

    id: int
    distance: float
    data: Any
