# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest

import orjson


class BytesAsStringTests(unittest.TestCase):
    def test_bytes(self):
        obj = b'some_bytes'
        self.assertEqual(b'"some_bytes"', orjson.dumps(obj, option=orjson.OPT_SERIALIZE_BYTES_AS_STRING))
