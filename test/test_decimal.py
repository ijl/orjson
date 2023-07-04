from decimal import Decimal

import orjson


def test_decimal_naive():
    # Given decimal
    d = Decimal("3.1416")

    # When serialize
    json_str = orjson.dumps(d)

    # It must be the expected
    assert json_str == b"3.14160"


