# SPDX-License-Identifier: (Apache-2.0 OR MIT)

import unittest
import decimal
import orjson


class DecimalTests(unittest.TestCase):
    def test_dumps_decimal(self):
        obj = decimal.Decimal("12.22")
        result = orjson.dumps([obj], option=orjson.OPT_SERIALIZE_DECIMAL)
        self.assertEqual(result, b'["12.22"]')

    def test_dumps_decimal_as_dict_key(self):
        obj = decimal.Decimal("12.22")
        result = orjson.dumps({obj: 1}, option=orjson.OPT_NON_STR_KEYS | orjson.OPT_SERIALIZE_DECIMAL)
        self.assertEqual(result, b'{"12.22":1}')

    def test_decimal_without_opt__raise_exception(self):
        obj = decimal.Decimal("12.22")

        with self.assertRaises(orjson.JSONEncodeError):
            orjson.dumps(obj)
