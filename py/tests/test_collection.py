from oasysdb.collection import Config, Record, Collection
from oasysdb.vector import VectorID

DIMENSION = 128
LEN = 100


def create_test_collection() -> Collection:
    """Creates a collection with random records for testing."""
    records = Record.many_random(dimension=DIMENSION, len=LEN)
    config = Config.create_default()
    collection = Collection.from_records(config=config, records=records)

    assert collection.len() == len(records)
    return collection


def test_create_config():
    config = Config(ef_construction=40, ef_search=15, ml=0.3)
    default = Config.create_default()

    assert config.ef_construction == default.ef_construction
    assert config.ef_search == default.ef_search
    assert config.ml == default.ml


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
    dimension = 128
    records = Record.many_random(dimension=dimension, len=LEN)
    assert len(records) == LEN


def test_create_collection():
    config = Config.create_default()
    collection = Collection(config=config)

    assert collection.config.ml == config.ml
    assert collection.is_empty()


def test_create_collection_from_records():
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
