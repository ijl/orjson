from decimal import Decimal

import orjson


def test_decimal_naive():
    # Given decimal
    d = Decimal("3.14")

    # When serialize
    json_str = orjson.dumps(d)

    # It must be the expected
    assert json_str == b'[3.140]'


def test_float_naive():
    # Given float
    d = 3.14

    # When serialize
    json_str = orjson.dumps(d)

    # It must be the expected
    assert json_str == b'3.14'
