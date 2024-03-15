# flake8: noqa F821

from oasysdb.collection import Collection, Record, Config


class Database:
    """The persistent storage of vector collections.

    Args:
    - path: Path to the database file.
    """

    def __init__(self, path: str,) -> None: ...

    def new(path: str) -> Database:
        """Creates a new database at the given path.
        This will reset the database if it exists.

        Args:
        - path: Path to the database file.
        """

    def get_collection(self, name: str) -> Collection:
        """Returns the collection with the given name.

        Args:
        - name: Collection name.
        """

    def save_collection(self, name: str, collection: Collection) -> None:
        """Saves new or update existing collection to the database.

        Args:
        - name: Collection name.
        - collection: Vector collection.
        """

    def delete_collection(self, name: str) -> None:
        """Deletes the collection from the database.

        Args:
        - name: Collection name.
        """

    def len(self) -> int:
        """Returns the number of collections in the database."""

    def is_empty(self) -> bool:
        """Returns True if the database is empty."""
