# SPDX-License-Identifier: MPL-2.0
# Copyright ijl (2018-2026)

import datetime

import pytest

import orjson

from .util import SUPPORTS_FROZENDICT


@pytest.mark.skipif(SUPPORTS_FROZENDICT is False, reason="python3.15")
class TestFrozenDict:
    def test_frozendict(self):
        """
        frozendict
        """
        obj = frozendict({"key": "value"})  # type: ignore[name-defined]
        ref = '{"key":"value"}'
        assert orjson.dumps(obj) == ref.encode("utf-8")
        assert orjson.loads(ref) == obj

    def test_frozendict_immutable(self):
        """
        frozendict immmutable
        """
        obj = frozendict({"0": 0})  # type: ignore[name-defined]
        with pytest.raises(TypeError):
            obj["1"] = 1

    def test_frozendict_unicode(self):
        """
        frozendict unicode keys
        """
        obj = frozendict({"🐈": "value"})  # type: ignore[name-defined]
        ref = b'{"\xf0\x9f\x90\x88":"value"}'
        assert orjson.dumps(obj) == ref
        assert orjson.loads(ref) == dict(obj)

    def test_frozendict_large_dict(self):
        """
        frozendict with >512 keys
        """
        obj = frozendict(  # type: ignore[name-defined]
            {f"key_{idx}": [{}, {"a": [{}, {}, {}]}, {}] for idx in range(513)},
        )  # type: ignore
        assert len(obj) == 513
        assert orjson.loads(orjson.dumps(obj)) == obj

    def test_frozendict_invalid_key_dumps(self):
        """
        frozendict invalid key dumps()
        """
        with pytest.raises(orjson.JSONEncodeError):
            orjson.dumps(frozendict({1: "value"}))  # type: ignore[name-defined]
        with pytest.raises(orjson.JSONEncodeError):
            orjson.dumps(frozendict({b"key": "value"}))  # type: ignore[name-defined]

    def test_frozendict_non_str_and_sort_keys(self):
        obj = frozendict(  # type: ignore[name-defined]
            {
                "other": 1,
                datetime.date(1970, 1, 5): 2,
                datetime.date(1970, 1, 3): 3,
            },
        )
        assert (
            orjson.dumps(
                obj,
                option=orjson.OPT_NON_STR_KEYS | orjson.OPT_SORT_KEYS,
            )
            == b'{"1970-01-03":3,"1970-01-05":2,"other":1}'
        )

    def test_frozendict_default(self):
        """
        frozendict default
        """
        ref = {"1": True}

        class Custom:
            def __init__(self):
                self.data = frozendict(ref)  # type: ignore[name-defined]

        def default(obj, /):
            return obj.data

        assert orjson.loads(orjson.dumps(Custom(), default=default)) == ref
