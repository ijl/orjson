# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest

import orjson


class BytesTests(unittest.TestCase):
    def test_bytes_without_opt(self):
        obj = b'{"a":"a","b":1}'
        with self.assertRaises(TypeError):
            orjson.dumps(obj)

    def test_bytes(self):
        obj = b'{"a":"a","b":1}'
        result = orjson.dumps(obj, option=orjson.OPT_DIRECT_SERIALIZE_BYTES)
        self.assertEqual(b'{"a":"a","b":1}', result)

    def test_bytes_in_dict(self):
        value = {"a": "a", "b": 1}
        serialized_value = orjson.dumps(value)

        obj = {"foo": serialized_value}
        matching_obj = {"foo": value}

        self.assertEqual(
            orjson.dumps(matching_obj), orjson.dumps(obj, option=orjson.OPT_DIRECT_SERIALIZE_BYTES)
        )
