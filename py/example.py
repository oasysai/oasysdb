# flake8: noqa F403

from oasysdb.prelude import *


if __name__ == "__main__":
    # Open the database.
    db = Database("data/example")

    # Create a vector collection.
    config = Config.create_default()
    records = Record.many_random(dimension=128, len=100)
    collection = Collection.from_records(config, records)

    # Optionally, persist the collection to the database.
    db.save_collection("my_collection", collection)

    # Search for the nearest neighbors.
    # Replace with your own query.
    query = Vector.random(128)
    result = collection.search(query, n=5)

    # Print the result.
    print("Nearest neighbors ID: {}".format(result[0].id))
