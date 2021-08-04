# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import json
import unittest

import pytest

import orjson

from .util import read_fixture_str

ASCII_TEST = b"""\
{
  "a": "qwe",
  "b": "qweqwe",
  "c": "qweq",
  "d: "qwe"
}
"""

MULTILINE_EMOJI = """[
    "😊",
    "a"
"""


class JsonDecodeErrorTests(unittest.TestCase):
    def _get_error_infos(self, json_decode_error_exc_info):
        return {
            k: v
            for k, v in json_decode_error_exc_info.value.__dict__.items()
            if k in ("pos", "lineno", "colno")
        }

    def _test(self, data, expected_err_infos):
        with pytest.raises(json.decoder.JSONDecodeError) as json_exc_info:
            json.loads(data)

        with pytest.raises(json.decoder.JSONDecodeError) as orjson_exc_info:
            orjson.loads(data)

        assert (
            self._get_error_infos(json_exc_info)
            == self._get_error_infos(orjson_exc_info)
            == expected_err_infos
        )

    def test_ascii(self):
        self._test(
            ASCII_TEST,
            {"pos": 55, "lineno": 5, "colno": 8},
        )

    def test_latin1(self):
        self._test(
            """["üýþÿ", "a" """,
            {"pos": 13, "lineno": 1, "colno": 14},
        )

    def test_two_byte_str(self):
        self._test(
            """["東京", "a" """,
            {"pos": 11, "lineno": 1, "colno": 12},
        )

    def test_two_byte_bytes(self):
        self._test(
            b'["\xe6\x9d\xb1\xe4\xba\xac", "a" ',
            {"pos": 11, "lineno": 1, "colno": 12},
        )

    def test_four_byte(self):
        self._test(
            MULTILINE_EMOJI,
            {"pos": 19, "lineno": 4, "colno": 1},
        )

    def test_tab(self):
        data = read_fixture_str("fail26.json", "jsonchecker")
        with pytest.raises(json.decoder.JSONDecodeError) as json_exc_info:
            json.loads(data)

        assert self._get_error_infos(json_exc_info) == {
            "pos": 5,
            "lineno": 1,
            "colno": 6,
        }

        with pytest.raises(json.decoder.JSONDecodeError) as orjson_exc_info:
            orjson.loads(data)

        assert self._get_error_infos(orjson_exc_info) == {
            "pos": 6,
            "lineno": 1,
            "colno": 7,
        }
