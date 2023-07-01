import orjson
from decimal import Decimal


def test_datetime_naive():
    """
    decimal dumps
    """
    assert (
            orjson.dumps([Decimal("3.14")])
            == b'[3.140]'
    )
