# SPDX-License-Identifier: (Apache-2.0 OR MIT)


import json

import pytest

import orjson


class TestUltraJSON:
    def test_doubleLongIssue(self):
        sut = {"a": -4342969734183514}
        encoded = orjson.dumps(sut)
        decoded = orjson.loads(encoded)
        assert sut == decoded
        encoded = orjson.dumps(sut)
        decoded = orjson.loads(encoded)
        assert sut == decoded

    def test_doubleLongDecimalIssue(self):
        sut = {"a": -12345678901234.56789012}
        encoded = orjson.dumps(sut)
        decoded = orjson.loads(encoded)
        assert sut == decoded
        encoded = orjson.dumps(sut)
        decoded = orjson.loads(encoded)
        assert sut == decoded

    def test_encodeDecodeLongDecimal(self):
        sut = {"a": -528656961.4399388}
        encoded = orjson.dumps(sut)
        orjson.loads(encoded)

    def test_decimalDecodeTest(self):
        sut = {"a": 4.56}
        encoded = orjson.dumps(sut)
        decoded = orjson.loads(encoded)
        pytest.approx(sut["a"], decoded["a"])

    def test_encodeDictWithUnicodeKeys(self):
        val = {
            "key1": "value1",
            "key1": "value1",
            "key1": "value1",
            "key1": "value1",
            "key1": "value1",
            "key1": "value1",
        }
        orjson.dumps(val)

        val = {
            "بن": "value1",
            "بن": "value1",
            "بن": "value1",
            "بن": "value1",
            "بن": "value1",
            "بن": "value1",
            "بن": "value1",
        }
        orjson.dumps(val)

    def test_encodeArrayOfNestedArrays(self):
        val = [[[[]]]] * 20
        output = orjson.dumps(val)
        assert val == orjson.loads(output)
        assert val == orjson.loads(output)

    def test_encodeArrayOfDoubles(self):
        val = [31337.31337, 31337.31337, 31337.31337, 31337.31337] * 10
        output = orjson.dumps(val)
        assert val == orjson.loads(output)
        assert val == orjson.loads(output)

    def test_encodeStringConversion2(self):
        val = "A string \\ / \b \f \n \r \t"
        output = orjson.dumps(val)
        assert val == orjson.loads(output)
        assert output == b'"A string \\\\ / \\b \\f \\n \\r \\t"'
        assert val == orjson.loads(output)

    def test_decodeUnicodeConversion(self):
        pass

    def test_encodeUnicodeConversion1(self):
        val = "Räksmörgås اسامة بن محمد بن عوض بن لادن"
        enc = orjson.dumps(val)
        dec = orjson.loads(enc)
        assert enc == orjson.dumps(val)
        assert dec == orjson.loads(enc)

    def test_encodeControlEscaping(self):
        val = "\x19"
        enc = orjson.dumps(val)
        dec = orjson.loads(enc)
        assert val == dec
        assert enc == orjson.dumps(val)

    def test_encodeUnicodeConversion2(self):
        val = "\xe6\x97\xa5\xd1\x88"
        enc = orjson.dumps(val)
        dec = orjson.loads(enc)
        assert enc == orjson.dumps(val)
        assert dec == orjson.loads(enc)

    def test_encodeUnicodeSurrogatePair(self):
        val = "\xf0\x90\x8d\x86"
        enc = orjson.dumps(val)
        dec = orjson.loads(enc)

        assert enc == orjson.dumps(val)
        assert dec == orjson.loads(enc)

    def test_encodeUnicode4BytesUTF8(self):
        val = "\xf0\x91\x80\xb0TRAILINGNORMAL"
        enc = orjson.dumps(val)
        dec = orjson.loads(enc)

        assert enc == orjson.dumps(val)
        assert dec == orjson.loads(enc)

    def test_encodeUnicode4BytesUTF8Highest(self):
        val = "\xf3\xbf\xbf\xbfTRAILINGNORMAL"
        enc = orjson.dumps(val)
        dec = orjson.loads(enc)

        assert enc == orjson.dumps(val)
        assert dec == orjson.loads(enc)

    def testEncodeUnicodeBMP(self):
        s = "\U0001f42e\U0001f42e\U0001F42D\U0001F42D"  # 🐮🐮🐭🐭
        orjson.dumps(s)
        json.dumps(s)

        assert json.loads(json.dumps(s)) == s
        assert orjson.loads(orjson.dumps(s)) == s

    def testEncodeSymbols(self):
        s = "\u273f\u2661\u273f"  # ✿♡✿
        encoded = orjson.dumps(s)
        encoded_json = json.dumps(s)

        decoded = orjson.loads(encoded)
        assert s == decoded

        encoded = orjson.dumps(s)

        # json outputs an unicode object
        encoded_json = json.dumps(s, ensure_ascii=False)
        assert encoded == encoded_json.encode("utf-8")
        decoded = orjson.loads(encoded)
        assert s == decoded

    def test_encodeArrayInArray(self):
        val = [[[[]]]]
        output = orjson.dumps(val)

        assert val == orjson.loads(output)
        assert output == orjson.dumps(val)
        assert val == orjson.loads(output)

    def test_encodeIntConversion(self):
        val = 31337
        output = orjson.dumps(val)
        assert val == orjson.loads(output)
        assert output == orjson.dumps(val)
        assert val == orjson.loads(output)

    def test_encodeIntNegConversion(self):
        val = -31337
        output = orjson.dumps(val)
        assert val == orjson.loads(output)
        assert output == orjson.dumps(val)
        assert val == orjson.loads(output)

    def test_encodeLongNegConversion(self):
        val = -9223372036854775808
        output = orjson.dumps(val)

        orjson.loads(output)
        orjson.loads(output)

        assert val == orjson.loads(output)
        assert output == orjson.dumps(val)
        assert val == orjson.loads(output)

    def test_encodeListConversion(self):
        val = [1, 2, 3, 4]
        output = orjson.dumps(val)
        assert val == orjson.loads(output)
        assert val == orjson.loads(output)

    def test_encodeDictConversion(self):
        val = {"k1": 1, "k2": 2, "k3": 3, "k4": 4}
        output = orjson.dumps(val)
        assert val == orjson.loads(output)
        assert val == orjson.loads(output)
        assert val == orjson.loads(output)

    def test_encodeNoneConversion(self):
        val = None
        output = orjson.dumps(val)
        assert val == orjson.loads(output)
        assert output == orjson.dumps(val)
        assert val == orjson.loads(output)

    def test_encodeTrueConversion(self):
        val = True
        output = orjson.dumps(val)
        assert val == orjson.loads(output)
        assert output == orjson.dumps(val)
        assert val == orjson.loads(output)

    def test_encodeFalseConversion(self):
        val = False
        output = orjson.dumps(val)
        assert val == orjson.loads(output)
        assert output == orjson.dumps(val)
        assert val == orjson.loads(output)

    def test_encodeToUTF8(self):
        val = b"\xe6\x97\xa5\xd1\x88"
        val = val.decode("utf-8")
        enc = orjson.dumps(val)
        dec = orjson.loads(enc)
        assert enc == orjson.dumps(val)
        assert dec == orjson.loads(enc)

    def test_decodeFromUnicode(self):
        val = '{"obj": 31337}'
        dec1 = orjson.loads(val)
        dec2 = orjson.loads(str(val))
        assert dec1 == dec2

    def test_decodeJibberish(self):
        val = "fdsa sda v9sa fdsa"
        pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeBrokenArrayStart(self):
        val = "["
        pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeBrokenObjectStart(self):
        val = "{"
        pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeBrokenArrayEnd(self):
        val = "]"
        pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeBrokenObjectEnd(self):
        val = "}"
        pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeObjectDepthTooBig(self):
        val = "{" * (1024 * 1024)
        pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeStringUnterminated(self):
        val = '"TESTING'
        pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeStringUntermEscapeSequence(self):
        val = '"TESTING\\"'
        pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeStringBadEscape(self):
        val = '"TESTING\\"'
        pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeTrueBroken(self):
        val = "tru"
        pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeFalseBroken(self):
        val = "fa"
        pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeNullBroken(self):
        val = "n"
        pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeBrokenDictKeyTypeLeakTest(self):
        val = '{{1337:""}}'
        for _ in range(1000):
            pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeBrokenDictLeakTest(self):
        val = '{{"key":"}'
        for _ in range(1000):
            pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeBrokenListLeakTest(self):
        val = "[[[true"
        for _ in range(1000):
            pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeDictWithNoKey(self):
        val = "{{{{31337}}}}"
        pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeDictWithNoColonOrValue(self):
        val = '{{{{"key"}}}}'
        pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeDictWithNoValue(self):
        val = '{{{{"key":}}}}'
        pytest.raises(orjson.JSONDecodeError, orjson.loads, val)

    def test_decodeNumericIntPos(self):
        val = "31337"
        assert 31337 == orjson.loads(val)

    def test_decodeNumericIntNeg(self):
        assert -31337 == orjson.loads("-31337")

    def test_encodeNullCharacter(self):
        val = "31337 \x00 1337"
        output = orjson.dumps(val)
        assert val == orjson.loads(output)
        assert output == orjson.dumps(val)
        assert val == orjson.loads(output)

        val = "\x00"
        output = orjson.dumps(val)
        assert val == orjson.loads(output)
        assert output == orjson.dumps(val)
        assert val == orjson.loads(output)

        assert b'"  \\u0000\\r\\n "' == orjson.dumps("  \u0000\r\n ")

    def test_decodeNullCharacter(self):
        val = '"31337 \\u0000 31337"'
        assert orjson.loads(val) == json.loads(val)

    def test_decodeEscape(self):
        base = "\u00e5".encode()
        quote = b'"'
        val = quote + base + quote
        assert json.loads(val) == orjson.loads(val)

    def test_decodeBigEscape(self):
        for _ in range(10):
            base = "\u00e5".encode()
            quote = b'"'
            val = quote + (base * 1024 * 1024 * 2) + quote
            assert json.loads(val) == orjson.loads(val)
