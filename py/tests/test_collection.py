from oasysdb.prelude import Config, Record, Collection, Vector, VectorID

DIMENSION = 128
LEN = 100


def create_test_collection() -> Collection:
    """Creates a collection with random records for testing."""
    records = Record.many_random(dimension=DIMENSION, len=LEN)
    config = Config.default()
    collection = Collection.from_records(config=config, records=records)

    assert collection.len() == len(records)
    return collection


def test_create_config():
    default = Config.create_default()

    # Create config based on the default.
    config = Config(
        ef_construction=40,
        ef_search=15,
        ml=0.2885,
        distance="euclidean"
    )

    assert config.ef_construction == default.ef_construction
    assert config.ef_search == default.ef_search
    assert config.ml == default.ml
    assert config.distance == default.distance


def test_create_record():
    vector = [0.1, 0.2, 0.3]
    data = {"text": "This is an example."}
    record = Record(vector=vector, data=data)

    assert len(record.vector) == len(vector)
    assert record.data == data


def test_generate_random_record():
    record = Record.random(dimension=DIMENSION)
    assert len(record.vector) == DIMENSION
    assert isinstance(record.data, int)


def test_generate_many_random_records():
    records = Record.many_random(dimension=DIMENSION, len=LEN)
    assert len(records) == LEN


def test_create_collection():
    config = Config.create_default()
    collection = Collection(config=config)

    assert collection.config.ml == config.ml
    assert collection.is_empty()


def test_build_collection():
    collection = create_test_collection()
    assert collection.contains(VectorID(0))
    assert not collection.is_empty()


def test_insert_record():
    collection = create_test_collection()
    record = Record.random(dimension=128)
    collection.insert(record)

    assert collection.len() == LEN + 1
    assert collection.contains(VectorID(LEN))


def test_insert_record_invalid_dimension():
    collection = create_test_collection()
    record = Record.random(dimension=100)

    # Insert should raise an exception because the
    # vector dimension is invalid.
    try:
        collection.insert(record)
        assert False
    except Exception as e:
        assert "invalid vector dimension" in str(e).lower()

    assert collection.len() == LEN


def test_insert_many_records():
    collection = create_test_collection()
    records = Record.many_random(dimension=DIMENSION, len=LEN)
    collection.insert_many(records)

    assert collection.len() == 2 * LEN
    assert all(collection.contains(VectorID(i)) for i in range(LEN, 2 * LEN))


def test_delete_record():
    collection = create_test_collection()

    id = VectorID(0)
    collection.delete(id)

    assert not collection.contains(id)
    assert collection.len() == LEN - 1


def test_get_record():
    collection = create_test_collection()

    id = VectorID(0)
    record = collection.get(id)

    assert record is not None
    assert record.data is not None


def test_update_record():
    collection = create_test_collection()

    id = VectorID(0)
    record = Record.random(dimension=128)
    collection.update(id, record)

    assert collection.contains(id)
    assert collection.get(id).data == record.data


def test_search_record():
    collection = create_test_collection()
    collection.relevancy = 4.5

    vector = Vector.random(dimension=DIMENSION)
    n = 10

    # Search for approximate neighbors and true neighbors.
    results = collection.search(vector, n=n)
    true_results = collection.true_search(vector, n=n)

    assert len(results) == n
    assert len(true_results) == n

    # Make sure the first result of the approximate search
    # is somewhere in the true results.
    assert results[0].id in [true.id for true in true_results]

    # Check if the result distances are within the relevancy.
    assert results[-1].distance <= collection.relevancy
    assert true_results[-1].distance <= collection.relevancy


def test_set_dimension():
    config = Config.create_default()
    collection = Collection(config=config)

    # Set the collection dimension to 100.
    collection.dimension = 100

    # When inserting a record with a different dimension,
    # the collection should raise an exception.
    try:
        record = Record.random(dimension=128)
        collection.insert(record)
        assert False
    except Exception as e:
        assert "invalid vector dimension" in str(e).lower()


def test_list_records():
    collection = create_test_collection()
    records = collection.list()

    assert len(records) == collection.len()
    assert all(isinstance(k, VectorID) for k in records.keys())
    assert all(isinstance(v, Record) for v in records.values())


def test_collection_distance_euclidean():
    config = Config.default()
    collection = Collection(config=config)

    # Insert records.
    k = 5
    records = Record.many_random(dimension=DIMENSION, len=k)
    collection.insert_many(records)

    # Search for the record.
    query = Vector.random(dimension=DIMENSION)
    results = collection.search(query, n=k)

    # Sort result based on distance ascending.
    sort = sorted(results, key=lambda x: x.distance)

    for i in range(k):
        assert results[i].distance == sort[i].distance


def test_collection_distance_cosine():
    config = Config.create_default()
    config.distance = "cosine"
    collection = Collection(config=config)

    # Insert records.
    k = 5
    records = Record.many_random(dimension=DIMENSION, len=k)
    collection.insert_many(records)

    # Search for the record.
    query = Vector.random(dimension=DIMENSION)
    results = collection.search(query, n=k)
    true_results = collection.true_search(query, n=k)

    # Sort result based on distance descending.
    sort = sorted(results, key=lambda x: x.distance, reverse=True)

    for i in range(k):
        assert results[i].distance == true_results[i].distance
        assert results[i].distance == sort[i].distance
