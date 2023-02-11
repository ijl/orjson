# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import orjson
import pytest


class TestSet:
    def test_serialize_set_option(self):
        with pytest.raises(orjson.JSONEncodeError):
            orjson.dumps({1, 2, 3})
        with pytest.raises(orjson.JSONEncodeError):
            orjson.dumps(set())
        with pytest.raises(orjson.JSONEncodeError):
            orjson.dumps(frozenset())

    @pytest.mark.parametrize("n", range(0, 100, 5))
    def test_serialize_set(self, n: int):
        s = set(range(n))
        fs = frozenset(range(n))
        assert orjson.dumps(s, option=orjson.OPT_SERIALIZE_SET) == orjson.dumps(list(s))
        assert orjson.dumps(fs, option=orjson.OPT_SERIALIZE_SET) == orjson.dumps(list(fs))

    def test_serialize_set_empty(self):
        assert orjson.dumps(set(), option=orjson.OPT_SERIALIZE_SET) == b"[]"
        assert orjson.dumps(frozenset(), option=orjson.OPT_SERIALIZE_SET) == b"[]"

    @pytest.mark.parametrize("n", range(0, 100, 5))
    def test_roundtrip_set(self, n: int):
        s = set(range(n))
        fs = frozenset(range(n))
        assert set(orjson.loads(orjson.dumps(s, option=orjson.OPT_SERIALIZE_SET))) == s
        assert frozenset(orjson.loads(orjson.dumps(fs, option=orjson.OPT_SERIALIZE_SET))) == fs

    def test_roundtrip_set_empty(self):
        assert orjson.loads(orjson.dumps(set(), option=orjson.OPT_SERIALIZE_SET)) == []
        assert orjson.loads(orjson.dumps(frozenset(), option=orjson.OPT_SERIALIZE_SET)) == []

    def test_nested(self):
        assert orjson.dumps([[{1}, {2}], {3}], option=orjson.OPT_SERIALIZE_SET) == b'[[[1],[2]],[3]]'
        fs = frozenset([frozenset([1, 2]), frozenset([3, 4])])
        # order is not guaranteed
        assert orjson.dumps(fs, option=orjson.OPT_SERIALIZE_SET) == b'[[1,2],[3,4]]' or orjson.dumps(fs, option=orjson.OPT_SERIALIZE_SET) == b'[[3,4],[1,2]]'

    def test_subclassing(self):
        class MySet(set):
            pass

        assert orjson.dumps(MySet([1, 2, 3]), option=orjson.OPT_SERIALIZE_SET) == b'[1,2,3]'

    def test_subclassing_frozenset(self):
        class MyFrozenSet(frozenset):
            pass

        assert orjson.dumps(MyFrozenSet([1, 2, 3]), option=orjson.OPT_SERIALIZE_SET) == b'[1,2,3]'

    def test_subclassing_new_iter_error(self):
        class MySet(set):
            def __iter__(self):
                raise RuntimeError("Oh no")

        with pytest.raises(orjson.JSONEncodeError):
            try:
                orjson.dumps(MySet([1, 2, 3]), option=orjson.OPT_SERIALIZE_SET)
            except orjson.JSONEncodeError as e:
                assert isinstance(e.__cause__, RuntimeError)
                raise

    def test_subclassing_error_while_iterating(self):
        class MySet(set):
            def __iter__(self):
                return (1 / 0 for _ in range(10))

        with pytest.raises(orjson.JSONEncodeError):
            try:
                orjson.dumps(MySet([1, 2, 3]), option=orjson.OPT_SERIALIZE_SET)
            except orjson.JSONEncodeError as e:
                assert isinstance(e.__cause__, ZeroDivisionError)
                raise

    def test_subclassing_new_iter_error_frozenset(self):
        class MyFrozenSet(frozenset):
            def __iter__(self):
                raise RuntimeError("Oh no")

        with pytest.raises(orjson.JSONEncodeError):
            try:
                orjson.dumps(MyFrozenSet([1, 2, 3]), option=orjson.OPT_SERIALIZE_SET)
            except orjson.JSONEncodeError as e:
                assert isinstance(e.__cause__, RuntimeError)
                raise

    def test_subclassing_error_while_iterating_frozenset(self):
        class MyFrozenSet(frozenset):
            def __iter__(self):
                return (1 / 0 for _ in range(10))

        with pytest.raises(orjson.JSONEncodeError):
            try:
                orjson.dumps(MyFrozenSet([1, 2, 3]), option=orjson.OPT_SERIALIZE_SET)
            except orjson.JSONEncodeError as e:
                assert isinstance(e.__cause__, ZeroDivisionError)
                raise
