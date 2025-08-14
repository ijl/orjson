# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import datetime

import orjson


class TestEnsureAsciiOutput:
    def test_all_ascii(self):
        obj = {"a": "b", "c": {"d": True}, "e": [1, 2]}
        assert (
            orjson.dumps(obj, option=orjson.OPT_ENSURE_ASCII)
            == b'{"a":"b","c":{"d":true},"e":[1,2]}'
        )

    def test_equivalent_emoji_key(self):
        obj = {"🤨": "b", "c": {"d": True}, "e": [1, 2]}
        assert (
            orjson.dumps(obj, option=orjson.OPT_ENSURE_ASCII)
            == b'{"\\ud83e\\udd28":"b","c":{"d":true},"e":[1,2]}'
        )

    def test_equivalent_emoji_value(self):
        obj = {"a": "b", "🤨": {"d": True}, "e": [1, 2]}
        assert (
            orjson.dumps(obj, option=orjson.OPT_ENSURE_ASCII)
            == b'{"a":"b","\\ud83e\\udd28":{"d":true},"e":[1,2]}'
        )

    def test_equivalent_non_emoji_value(self):
        """
        Test a lower code-point character that is not an emoji.

        Characters with code points in range 0x0000 to 0xFFFF are handled
        differently than characters beyond.
        """
        obj = {"ni_hao": "你_好"}
        assert (
            orjson.dumps(obj, option=orjson.OPT_ENSURE_ASCII)
            == b'{"ni_hao":"\\u4f60_\\u597d"}'
        )

    def test_round_trip(self):
        """
        Test that the output can be loaded and dumped again.
        """
        obj = {"ni_hao": "你_好", "emoji": "🍉"}
        dumped = orjson.dumps(obj, option=orjson.OPT_ENSURE_ASCII)
        assert dumped == b'{"ni_hao":"\\u4f60_\\u597d","emoji":"\\ud83c\\udf49"}'
        loaded = orjson.loads(dumped)
        assert loaded == obj
        assert orjson.dumps(loaded, option=orjson.OPT_ENSURE_ASCII) == dumped

    def test_sort(self):
        obj = {"b": 1, "a": 2, "c🤨": "d🔥"}
        assert (
            orjson.dumps(obj, option=orjson.OPT_ENSURE_ASCII | orjson.OPT_SORT_KEYS)
            == b'{"a":2,"b":1,"c\\ud83e\\udd28":"d\\ud83d\\udd25"}'
        )

    def test_non_str(self):
        obj = {1: 1, "a🔥": 2}
        assert (
            orjson.dumps(obj, option=orjson.OPT_ENSURE_ASCII | orjson.OPT_NON_STR_KEYS)
            == b'{"1":1,"a\\ud83d\\udd25":2}'
        )

    def test_options(self):
        obj = {
            1: 1,
            "b": True,
            "a": datetime.datetime(1970, 1, 1),
            "d🔥": "e🤨",
        }
        assert (
            orjson.dumps(
                obj,
                option=orjson.OPT_ENSURE_ASCII
                | orjson.OPT_SORT_KEYS
                | orjson.OPT_NON_STR_KEYS
                | orjson.OPT_NAIVE_UTC,
            )
            == b'{"1":1,"a":"1970-01-01T00:00:00+00:00","b":true,"d\\ud83d\\udd25":"e\\ud83e\\udd28"}'
        )

    def test_empty(self):
        obj = [{}, [[[]]], {"key": []}]
        ref = b'[{},[[[]]],{"key":[]}]'
        assert orjson.dumps(obj, option=orjson.OPT_ENSURE_ASCII) == ref
