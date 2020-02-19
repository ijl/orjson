# SPDX-License-Identifier: (Apache-2.0 OR MIT)


import json
import math
import unittest

import orjson


class UltraJSONTests(unittest.TestCase):
    def test_doubleLongIssue(self):
        sut = {"a": -4342969734183514}
        encoded = orjson.dumps(sut)
        decoded = orjson.loads(encoded)
        self.assertEqual(sut, decoded)
        encoded = orjson.dumps(sut)
        decoded = orjson.loads(encoded)
        self.assertEqual(sut, decoded)

    def test_doubleLongDecimalIssue(self):
        sut = {"a": -12345678901234.56789012}
        encoded = orjson.dumps(sut)
        decoded = orjson.loads(encoded)
        self.assertEqual(sut, decoded)
        encoded = orjson.dumps(sut)
        decoded = orjson.loads(encoded)
        self.assertEqual(sut, decoded)

    def test_encodeDecodeLongDecimal(self):
        sut = {"a": -528656961.4399388}
        encoded = orjson.dumps(sut)
        orjson.loads(encoded)

    def test_decimalDecodeTest(self):
        sut = {"a": 4.56}
        encoded = orjson.dumps(sut)
        decoded = orjson.loads(encoded)
        self.assertAlmostEqual(sut["a"], decoded["a"])

    def test_encodeDictWithUnicodeKeys(self):
        input = {
            "key1": "value1",
            "key1": "value1",
            "key1": "value1",
            "key1": "value1",
            "key1": "value1",
            "key1": "value1",
        }
        orjson.dumps(input)

        input = {
            "بن": "value1",
            "بن": "value1",
            "بن": "value1",
            "بن": "value1",
            "بن": "value1",
            "بن": "value1",
            "بن": "value1",
        }
        orjson.dumps(input)

    def test_encodeDoubleConversion(self):
        input = math.pi
        output = orjson.dumps(input)
        self.assertEqual(round(input, 5), round(orjson.loads(output), 5))
        self.assertEqual(round(input, 5), round(orjson.loads(output), 5))

    def test_encodeDoubleNegConversion(self):
        input = -math.pi
        output = orjson.dumps(input)

        self.assertEqual(round(input, 5), round(orjson.loads(output), 5))
        self.assertEqual(round(input, 5), round(orjson.loads(output), 5))

    def test_encodeArrayOfNestedArrays(self):
        input = [[[[]]]] * 20
        output = orjson.dumps(input)
        self.assertEqual(input, orjson.loads(output))
        self.assertEqual(input, orjson.loads(output))

    def test_encodeArrayOfDoubles(self):
        input = [31337.31337, 31337.31337, 31337.31337, 31337.31337] * 10
        output = orjson.dumps(input)
        self.assertEqual(input, orjson.loads(output))
        self.assertEqual(input, orjson.loads(output))

    def test_encodeStringConversion2(self):
        input = "A string \\ / \b \f \n \r \t"
        output = orjson.dumps(input)
        self.assertEqual(input, orjson.loads(output))
        self.assertEqual(output, b'"A string \\\\ / \\b \\f \\n \\r \\t"')
        self.assertEqual(input, orjson.loads(output))

    def test_decodeUnicodeConversion(self):
        pass

    def test_encodeUnicodeConversion1(self):
        input = "Räksmörgås اسامة بن محمد بن عوض بن لادن"
        enc = orjson.dumps(input)
        dec = orjson.loads(enc)
        self.assertEqual(enc, orjson.dumps(input))
        self.assertEqual(dec, orjson.loads(enc))

    def test_encodeControlEscaping(self):
        input = "\x19"
        enc = orjson.dumps(input)
        dec = orjson.loads(enc)
        self.assertEqual(input, dec)
        self.assertEqual(enc, orjson.dumps(input))

    def test_encodeUnicodeConversion2(self):
        input = "\xe6\x97\xa5\xd1\x88"
        enc = orjson.dumps(input)
        dec = orjson.loads(enc)
        self.assertEqual(enc, orjson.dumps(input))
        self.assertEqual(dec, orjson.loads(enc))

    def test_encodeUnicodeSurrogatePair(self):
        input = "\xf0\x90\x8d\x86"
        enc = orjson.dumps(input)
        dec = orjson.loads(enc)

        self.assertEqual(enc, orjson.dumps(input))
        self.assertEqual(dec, orjson.loads(enc))

    def test_encodeUnicode4BytesUTF8(self):
        input = "\xf0\x91\x80\xb0TRAILINGNORMAL"
        enc = orjson.dumps(input)
        dec = orjson.loads(enc)

        self.assertEqual(enc, orjson.dumps(input))
        self.assertEqual(dec, orjson.loads(enc))

    def test_encodeUnicode4BytesUTF8Highest(self):
        input = "\xf3\xbf\xbf\xbfTRAILINGNORMAL"
        enc = orjson.dumps(input)
        dec = orjson.loads(enc)

        self.assertEqual(enc, orjson.dumps(input))
        self.assertEqual(dec, orjson.loads(enc))

    # Characters outside of Basic Multilingual Plane(larger than
    # 16 bits) are represented as \UXXXXXXXX in python but should be encoded
    # as \uXXXX\uXXXX in orjson.
    def testEncodeUnicodeBMP(self):
        s = "\U0001f42e\U0001f42e\U0001F42D\U0001F42D"  # 🐮🐮🐭🐭
        orjson.dumps(s)
        json.dumps(s)

        self.assertEqual(json.loads(json.dumps(s)), s)
        self.assertEqual(orjson.loads(orjson.dumps(s)), s)

    def testEncodeSymbols(self):
        s = "\u273f\u2661\u273f"  # ✿♡✿
        encoded = orjson.dumps(s)
        encoded_json = json.dumps(s)

        decoded = orjson.loads(encoded)
        self.assertEqual(s, decoded)

        encoded = orjson.dumps(s)

        # json outputs an unicode object
        encoded_json = json.dumps(s, ensure_ascii=False)
        self.assertEqual(encoded, encoded_json.encode("utf-8"))
        decoded = orjson.loads(encoded)
        self.assertEqual(s, decoded)

    def test_encodeArrayInArray(self):
        input = [[[[]]]]
        output = orjson.dumps(input)

        self.assertEqual(input, orjson.loads(output))
        self.assertEqual(output, orjson.dumps(input))
        self.assertEqual(input, orjson.loads(output))

    def test_encodeIntConversion(self):
        input = 31337
        output = orjson.dumps(input)
        self.assertEqual(input, orjson.loads(output))
        self.assertEqual(output, orjson.dumps(input))
        self.assertEqual(input, orjson.loads(output))

    def test_encodeIntNegConversion(self):
        input = -31337
        output = orjson.dumps(input)
        self.assertEqual(input, orjson.loads(output))
        self.assertEqual(output, orjson.dumps(input))
        self.assertEqual(input, orjson.loads(output))

    def test_encodeLongNegConversion(self):
        input = -9223372036854775808
        output = orjson.dumps(input)

        orjson.loads(output)
        orjson.loads(output)

        self.assertEqual(input, orjson.loads(output))
        self.assertEqual(output, orjson.dumps(input))
        self.assertEqual(input, orjson.loads(output))

    def test_encodeListConversion(self):
        input = [1, 2, 3, 4]
        output = orjson.dumps(input)
        self.assertEqual(input, orjson.loads(output))
        self.assertEqual(input, orjson.loads(output))

    def test_encodeDictConversion(self):
        input = {"k1": 1, "k2": 2, "k3": 3, "k4": 4}
        output = orjson.dumps(input)
        self.assertEqual(input, orjson.loads(output))
        self.assertEqual(input, orjson.loads(output))
        self.assertEqual(input, orjson.loads(output))

    def test_encodeNoneConversion(self):
        input = None
        output = orjson.dumps(input)
        self.assertEqual(input, orjson.loads(output))
        self.assertEqual(output, orjson.dumps(input))
        self.assertEqual(input, orjson.loads(output))

    def test_encodeTrueConversion(self):
        input = True
        output = orjson.dumps(input)
        self.assertEqual(input, orjson.loads(output))
        self.assertEqual(output, orjson.dumps(input))
        self.assertEqual(input, orjson.loads(output))

    def test_encodeFalseConversion(self):
        input = False
        output = orjson.dumps(input)
        self.assertEqual(input, orjson.loads(output))
        self.assertEqual(output, orjson.dumps(input))
        self.assertEqual(input, orjson.loads(output))

    def test_encodeToUTF8(self):
        input = b"\xe6\x97\xa5\xd1\x88"
        input = input.decode("utf-8")
        enc = orjson.dumps(input)
        dec = orjson.loads(enc)
        self.assertEqual(enc, orjson.dumps(input))
        self.assertEqual(dec, orjson.loads(enc))

    def test_decodeFromUnicode(self):
        input = '{"obj": 31337}'
        dec1 = orjson.loads(input)
        dec2 = orjson.loads(str(input))
        self.assertEqual(dec1, dec2)

    def test_decodeJibberish(self):
        input = "fdsa sda v9sa fdsa"
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeBrokenArrayStart(self):
        input = "["
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeBrokenObjectStart(self):
        input = "{"
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeBrokenArrayEnd(self):
        input = "]"
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeBrokenObjectEnd(self):
        input = "}"
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeObjectDepthTooBig(self):
        input = "{" * (1024 * 1024)
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeStringUnterminated(self):
        input = '"TESTING'
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeStringUntermEscapeSequence(self):
        input = '"TESTING\\"'
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeStringBadEscape(self):
        input = '"TESTING\\"'
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeTrueBroken(self):
        input = "tru"
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeFalseBroken(self):
        input = "fa"
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeNullBroken(self):
        input = "n"
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeBrokenDictKeyTypeLeakTest(self):
        input = '{{1337:""}}'
        for _ in range(1000):
            self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeBrokenDictLeakTest(self):
        input = '{{"key":"}'
        for _ in range(1000):
            self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeBrokenListLeakTest(self):
        input = "[[[true"
        for _ in range(1000):
            self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeDictWithNoKey(self):
        input = "{{{{31337}}}}"
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeDictWithNoColonOrValue(self):
        input = '{{{{"key"}}}}'
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeDictWithNoValue(self):
        input = '{{{{"key":}}}}'
        self.assertRaises(orjson.JSONDecodeError, orjson.loads, input)

    def test_decodeNumericIntPos(self):
        input = "31337"
        self.assertEqual(31337, orjson.loads(input))

    def test_decodeNumericIntNeg(self):
        input = "-31337"
        self.assertEqual(-31337, orjson.loads(input))

    def test_encodeNullCharacter(self):
        input = "31337 \x00 1337"
        output = orjson.dumps(input)
        self.assertEqual(input, orjson.loads(output))
        self.assertEqual(output, orjson.dumps(input))
        self.assertEqual(input, orjson.loads(output))

        input = "\x00"
        output = orjson.dumps(input)
        self.assertEqual(input, orjson.loads(output))
        self.assertEqual(output, orjson.dumps(input))
        self.assertEqual(input, orjson.loads(output))

        self.assertEqual(b'"  \\u0000\\r\\n "', orjson.dumps("  \u0000\r\n "))

    def test_decodeNullCharacter(self):
        input = '"31337 \\u0000 31337"'
        self.assertEqual(orjson.loads(input), json.loads(input))

    def test_decodeEscape(self):
        base = "\u00e5".encode()
        quote = b'"'
        input = quote + base + quote
        self.assertEqual(json.loads(input), orjson.loads(input))

    def test_decodeBigEscape(self):
        for _ in range(10):
            base = "\u00e5".encode()
            quote = b'"'
            input = quote + (base * 1024 * 1024 * 2) + quote
            self.assertEqual(json.loads(input), orjson.loads(input))
