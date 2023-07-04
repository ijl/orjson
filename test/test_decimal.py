from decimal import Decimal

import orjson


def test_decimal_naive_encode():
    # Given decimal
    d = Decimal("3.1416")

    # When encode
    encoded = orjson.dumps(d)

    # It must be the expected
    assert encoded == b"3.14160"

def test_decimal_naive_decode():
    # Given and encoded decimal
    encoded = b"3.14160"

    # When decode
    d = orjson.loads(encoded)

    # It must be the expected
    assert d ==  Decimal("3.1416")


def test_decimal_in_array():
    # Given array with decimal
    d = [Decimal("3.1416")]

    # When serialize
    json_str = orjson.dumps(d)

    # It must be the expected
    assert json_str == b"[3.14160]"