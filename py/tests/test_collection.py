from oasysdb.collection import Config, Record


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
