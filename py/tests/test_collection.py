from oasysdb.collection import Config, Record, Collection
from oasysdb.vector import VectorID


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
    dimension = 128
    record = Record.random(dimension=dimension)
    assert len(record.vector) == dimension
    assert isinstance(record.data, int)


def test_generate_many_random_records():
    dimension = 128
    records = Record.many_random(dimension=dimension, len=10)
    assert len(records) == 10
    assert all(len(record.vector) == dimension for record in records)


def test_create_collection():
    config = Config.create_default()
    collection = Collection(config=config)
    assert collection.config.ml == config.ml
    assert collection.is_empty()


def test_create_collection_from_records():
    vector = [0.1, 0.2, 0.3]
    data = "This is an example."
    records = [Record(vector=vector, data=data)]

    config = Config.create_default()
    collection = Collection.from_records(config=config, records=records)

    assert collection.contains(VectorID(0))
    assert collection.len() == len(records)
    assert not collection.is_empty()
