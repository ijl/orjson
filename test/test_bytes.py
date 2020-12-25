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
        # barray = eval(str(result)[2:-1])
        # print("".join([chr(ch) for ch in barray]))
        self.assertEqual(result, b'{"a":"a","b":1}')
