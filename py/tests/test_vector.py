from oasysdb.prelude import Vector, VectorID


def test_create_vector():
    value = [0.1, 0.2, 0.3]
    vector = Vector(value)
    assert len(vector) == 3


def test_generate_random_vector():
    dimension = 128
    vector = Vector.random(dimension)
    assert len(vector) == dimension


def test_create_vector_id():
    id = 1
    vector_id = VectorID(id)
    assert vector_id.is_valid()
