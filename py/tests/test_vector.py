from oasysdb.vector import Vector


def test_create():
    value = [0.1, 0.2, 0.3]
    vector = Vector(value)
    assert len(vector) == 3


def test_generate_random():
    dimension = 128
    vector = Vector.random(dimension)
    assert len(vector) == dimension
