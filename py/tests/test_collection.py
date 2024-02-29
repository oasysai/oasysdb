from oasysdb.collection import Config


def test_create_config():
    config = Config(ef_construction=40, ef_search=15, ml=0.3)
    default = Config.create_default()
    assert config.ef_construction == default.ef_construction
    assert config.ef_search == default.ef_search
    assert config.ml == default.ml
