# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest

import orjson


class BytesTests(unittest.TestCase):
    def test_bytes_without_opt(self):
        obj = b'{"a":"a","b":1}'
        with self.assertRaises(TypeError):
            orjson.dumps(obj)

    def test_bytes_as_json(self):
        obj = b'{"a":"a","b":1}'
        result = orjson.dumps(obj, option=orjson.OPT_SERIALIZE_BYTES_AS_JSON)
        self.assertEqual(b'{"a":"a","b":1}', result)

    def test_bytes_as_json_in_dict(self):
        value = {"a": "a", "b": 1}
        serialized_value = orjson.dumps(value)

        obj = {"foo": serialized_value}
        matching_obj = {"foo": value}

        self.assertEqual(
            orjson.dumps(matching_obj), orjson.dumps(obj, option=orjson.OPT_SERIALIZE_BYTES_AS_JSON)
        )

    def test_bytes_as_string(self):
        obj = b'some_bytes'
        self.assertEqual(b'"some_bytes"', orjson.dumps(obj, option=orjson.OPT_SERIALIZE_BYTES_AS_STRING))
