import orjson
import pytest


class TestGenerator:
    def test_generator(self):
        def generator():
            yield 1
            yield 2
            yield 3

        assert list(generator()) == [1, 2, 3]
        g = generator()
        assert orjson.dumps(g, option=orjson.OPT_SERIALIZE_GENERATOR) == b'[1,2,3]'

    def test_consumed_generator(self):
        def generator():
            yield 1
            yield 2
            yield 3

        g = generator()
        # Consume
        assert list(g) == [1, 2, 3]
        # Should be empty
        assert orjson.dumps(g, option=orjson.OPT_SERIALIZE_GENERATOR) == b'[]'

    def test_generator_recursive(self):
        def generator():
            yield 1
            yield 2
            yield 3
            yield from generator()

        with pytest.raises(orjson.JSONEncodeError):
            orjson.dumps(generator(), option=orjson.OPT_SERIALIZE_GENERATOR)

    def test_other_types(self):
        def generator():
            yield [1, 2, 3]
            yield {1, 2, 3}
            yield {'1': 2, '3': 4}
            yield (1, 2, 3)
            yield 1
            yield 2.0
            yield '3'
            yield True
            yield None

        assert orjson.dumps(generator(), option=orjson.OPT_SERIALIZE_GENERATOR | orjson.OPT_SERIALIZE_SET) == b'[[1,2,3],[1,2,3],{"1":2,"3":4},[1,2,3],1,2.0,"3",true,null]'

    def test_error_handling(self):
        def generator():
            yield 1
            yield 2
            raise ValueError('error')

        with pytest.raises(orjson.JSONEncodeError):
            orjson.dumps(generator(), option=orjson.OPT_SERIALIZE_GENERATOR)

    def test_error_cause(self):
        def generator():
            yield 1
            yield 2
            yield 1 / 0

        with pytest.raises(orjson.JSONEncodeError):
            try:
                orjson.dumps(generator(), option=orjson.OPT_SERIALIZE_GENERATOR)
            except orjson.JSONEncodeError as e:
                assert isinstance(e.__cause__, ZeroDivisionError)
                raise

    def test_error_cause_recursive(self):
        def generator():
            yield 1
            yield 2
            yield from generator()

        with pytest.raises(orjson.JSONEncodeError):
            try:
                orjson.dumps(generator(), option=orjson.OPT_SERIALIZE_GENERATOR)
            except orjson.JSONEncodeError as e:
                print(e.__cause__)
                assert isinstance(e.__cause__, RecursionError)
                raise

    def test_default(self):
        class SomeClass:
            def __init__(self, value):
                self.value = value

        def default(obj):
            if isinstance(obj, SomeClass):
                return obj.value
            raise TypeError

        def generator():
            yield 1
            yield 2
            yield 3
            yield SomeClass(4)
            yield SomeClass(5)

        assert orjson.dumps(generator(), option=orjson.OPT_SERIALIZE_GENERATOR, default=default) == b'[1,2,3,4,5]'

    def test_default_error(self):
        def default(obj):
            raise TypeError

        def generator():
            yield 1
            yield 2
            yield 3
            yield lambda x: x
            yield 2

        with pytest.raises(orjson.JSONEncodeError):
            orjson.dumps(generator(), option=orjson.OPT_SERIALIZE_GENERATOR, default=default)

    def test_default_error_cause(self):
        def default(obj):
            raise TypeError

        def generator():
            yield 1
            yield 2
            yield 3
            yield 1 / 0
            yield 2

        with pytest.raises(orjson.JSONEncodeError):
            try:
                orjson.dumps(generator(), option=orjson.OPT_SERIALIZE_GENERATOR, default=default)
            except orjson.JSONEncodeError as e:
                assert isinstance(e.__cause__, ZeroDivisionError)
                raise
